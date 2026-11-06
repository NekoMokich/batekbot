//! batekbot: Rust-версия Discord-бота (serenity 0.12, poise 0.7, songbird 0.6).
//!
//! Порядок запуска: окружение и токен, корень репозитория, попытка подключить
//! базу (без неё бот работает), чтение кэша YouTube с диска, клиент, затем
//! фоновая регистрация слеш-команд и обновление кэша. Сеть на старте не нужна,
//! ready не ждёт yt-dlp.

mod commands;
mod db;
mod keywords;
mod library;
mod memes;
mod paths;
mod telegram;
mod voice;
mod youtube;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use poise::serenity_prelude as serenity;
use songbird::SerenityInit;
use tracing::{error, info, warn};

use crate::db::Db;

pub struct Data {
    /// `None`, если PostgreSQL недоступен на старте: команды без базы
    /// (music, help, grandmobile, batekfetch) продолжают работать.
    pub db: Option<Db>,
    pub yt: Arc<youtube::YoutubeCache>,
    pub voice: Arc<voice::Voice>,
    pub root: Arc<PathBuf>,
    pub library: Arc<library::Library>,
    pub memes: Arc<memes::Memes>,
    /// Общая таблица слов, но свои лимиты: Discord ключует по каналу.
    pub keywords: keywords::KeywordLimiter,
    /// Включён ли обработчик ключевых слов. Определяется по флагам
    /// приложения: если Message Content Intent не включён в портале, handler
    /// выключен и `MESSAGE_CONTENT` не запрашивается.
    pub keyword_reactions: bool,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let started = Instant::now();
    dotenvy::dotenv().ok();
    let filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let root = tokio::task::spawn_blocking(paths::resolve_root).await?;
    info!("корень репозитория: {}", root.display());

    let token = match std::env::var("main").or_else(|_| std::env::var("test")) {
        Ok(token) if !token.trim().is_empty() => token,
        _ => anyhow::bail!("нет токена: задайте переменные окружения `main` или `test`"),
    };

    let db = match database_url() {
        Ok(url) => connect_db(&url).await,
        Err(e) => {
            warn!("БД: {e}; работаем без базы, запись пользователей отключена");
            None
        }
    };

    let cache_path = root.join("yt_cache.json");
    let yt = Arc::new(youtube::YoutubeCache::load(cache_path).await);
    youtube::spawn_refresh(yt.clone());

    let songbird_manager = songbird::Songbird::serenity();
    let voice = Arc::new(voice::Voice::new(songbird_manager.clone()));
    let library = Arc::new(library::Library::new(root.join("music")));

    // Мемы читаются один раз: каталоги меняются только пересборкой образа.
    let memes = Arc::new(memes::Memes::scan(&root));

    // MESSAGE_CONTENT привилегированный: если запросить его без галочки в
    // Developer Portal, serenity завершится. Галочка отражается в флагах
    // приложения, поэтому спрашиваем Discord до создания клиента. Сбой
    // проверки не мешает запуску: просто работаем без реакций.
    let keyword_reactions = match serenity::Http::new(&token)
        .get_current_application_info()
        .await
    {
        Ok(info) => {
            let enabled = info.flags.is_some_and(can_request_message_content);
            if enabled {
                info!(
                    "Discord: реакции на ключевые слова включены: \
                     Message Content Intent активен"
                );
            } else {
                info!(
                    "Discord: ключевые слова в Discord выключены, включите \
                     Message Content Intent в Developer Portal"
                );
            }
            enabled
        }
        Err(e) => {
            warn!(
                "Discord: ключевые слова в Discord выключены, включите Message Content \
                 Intent в Developer Portal (проверка не удалась: {e})"
            );
            false
        }
    };

    // Telegram живёт в той же задаче, но своей надзираемой петлёй. Пустой
    // токен всего лишь выключает его, Discord продолжает работать.
    let telegram_token = std::env::var("TELEGRAM_TOKEN").unwrap_or_default();
    let telegram_data = telegram::TgData::new(
        telegram_token,
        yt.clone(),
        memes.clone(),
        db.clone(),
        Arc::new(root.clone()),
    );
    telegram::spawn(telegram_data).await?;

    let data_yt = yt.clone();
    let data_voice = voice.clone();
    let data_library = library.clone();
    let data_root = root.clone();
    let data_db = db.clone();
    let data_memes = memes.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all(),
            event_handler: on_event,
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        })
        .setup(move |ctx, ready, _framework| {
            Box::pin(async move {
                // Регистрация не блокирует установку user data: сбой сети или
                // 429 иначе оставлял poise без данных, а процесс живым.
                // `Command` не `Clone`, поэтому список строится ещё раз тем же
                // `commands::all()`, что передан в `FrameworkOptions`.
                let http = ctx.http.clone();
                let registration_commands = commands::all();
                tokio::spawn(async move {
                    register_commands_with_retry(http, registration_commands).await;
                });
                info!(
                    "Бот {} запущен, ready за {:?}; слеш-команды регистрируются в фоне",
                    ready.user.name,
                    started.elapsed()
                );
                Ok(Data {
                    db: data_db,
                    yt: data_yt,
                    voice: data_voice,
                    root: Arc::new(data_root),
                    library: data_library,
                    memes: data_memes,
                    keywords: keywords::KeywordLimiter::new(),
                    keyword_reactions,
                })
            })
        })
        .build();

    // Слеш-командам нужны только guilds и voice_states. members не
    // запрашивается. message_content привилегированный и добавляется только
    // вместе с флагом реакций на ключевые слова.
    let mut intents =
        serenity::GatewayIntents::GUILDS | serenity::GatewayIntents::GUILD_VOICE_STATES;
    if keyword_reactions {
        intents |= serenity::GatewayIntents::MESSAGE_CONTENT;
    }

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird_with(songbird_manager)
        .await
        .context("не удалось создать клиент serenity")?;

    // Клон нужен, потому что ветка select не может трогать `client`, пока
    // его future держит заимствование.
    let shard_manager = client.shard_manager.clone();

    tokio::select! {
        result = client.start() => {
            result.context("клиент serenity завершился с ошибкой")?;
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl-C: выходим из голосовых каналов");
            voice.leave_all().await;
            // `leave` и `remove` только ставят op 4 в очередь шарда. Без паузы
            // процесс завершится раньше отправки, и Discord оставит бота в
            // канале до таймаута.
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            shard_manager.shutdown_all().await;
        }
    }

    Ok(())
}

/// Может ли бот запросить MESSAGE_CONTENT. Discord выставляет один из флагов,
/// когда intent включён в Developer Portal; без него запрос завершает клиент.
fn can_request_message_content(flags: serenity::ApplicationFlags) -> bool {
    flags.contains(serenity::ApplicationFlags::GATEWAY_MESSAGE_CONTENT)
        || flags.contains(serenity::ApplicationFlags::GATEWAY_MESSAGE_CONTENT_LIMITED)
}

/// Общий вход событий serenity. Реакции на слова идут отдельной ветвью и
/// никогда не возвращают ошибку наружу: сбой одного сообщения не должен
/// влиять на остальные события.
fn on_event<'a>(
    framework: poise::FrameworkContext<'a, Data, Error>,
    event: &'a serenity::FullEvent,
) -> poise::BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move {
        if let serenity::FullEvent::Message { new_message } = event {
            handle_discord_keyword(framework, new_message).await;
        }
        Ok(())
    })
}

async fn handle_discord_keyword(
    framework: poise::FrameworkContext<'_, Data, Error>,
    msg: &serenity::Message,
) {
    let data = framework.user_data;
    if !data.keyword_reactions {
        return;
    }
    // Только гильдии, только люди и только не команды: у вебхуков и ботов
    // нет автора-пользователя, на чьё сообщение стоит отвечать.
    if msg.guild_id.is_none() || msg.author.bot || msg.webhook_id.is_some() {
        return;
    }
    if msg.content.trim_start().starts_with('/') {
        return;
    }
    let Some((phrase, action)) = keywords::match_keyword(&msg.content) else {
        return;
    };

    // Файл проверяется до отправки и до лимита: пропавший мем не занимает
    // окно и не превращается в ответ об ошибке.
    let meme = match action {
        keywords::KeywordAction::Grandmobile => None,
        keywords::KeywordAction::Meme(needle) => match data.memes.find(needle) {
            Some(meme) => {
                if tokio::fs::metadata(&meme.path).await.is_err() {
                    warn!(
                        "Discord: ключевое слово '{phrase}': файл мема '{}' не найден",
                        meme.name
                    );
                    return;
                }
                Some(meme)
            }
            None => {
                warn!("Discord: ключевое слово '{phrase}': мем '{needle}' не найден");
                return;
            }
        },
        keywords::KeywordAction::RandomMeme(needles) => {
            let mut candidates = keywords::random_meme_candidates(&data.memes, needles);
            let mut existing = Vec::with_capacity(candidates.len());
            for meme in candidates.drain(..) {
                if tokio::fs::metadata(&meme.path).await.is_ok() {
                    existing.push(meme);
                } else {
                    warn!(
                        "Discord: ключевое слово '{phrase}': файл мема '{}' не найден",
                        meme.name
                    );
                }
            }
            let Some(meme) = keywords::pick_random_meme(&mut existing) else {
                warn!("Discord: ключевое слово '{phrase}': нет доступных мемов");
                return;
            };
            Some(meme)
        }
    };

    let channel = msg.channel_id;
    let needed = if meme.is_some() {
        serenity::Permissions::SEND_MESSAGES | serenity::Permissions::ATTACH_FILES
    } else {
        serenity::Permissions::SEND_MESSAGES
    };
    if let Some(guild_id) = msg.guild_id {
        if can_send_in_channel(framework.serenity_context, guild_id, channel, needed) == Some(false)
        {
            warn!(
                "Discord: ключевое слово '{phrase}': нет прав писать или прикреплять \
                 файлы в канале {channel}"
            );
            return;
        }
    }

    if !data.keywords.allow(channel.get() as i64, phrase).await {
        return;
    }

    let http = &framework.serenity_context.http;
    match meme {
        Some(meme) => {
            if let Err(e) = send_keyword_file(http, channel, meme).await {
                warn!("Discord: ключевое слово '{phrase}': файл не отправлен: {e}");
            }
        }
        None => {
            let message =
                serenity::CreateMessage::new().content(commands::misc::grandmobile_text());
            if let Err(e) = channel.send_message(http, message).await {
                warn!("Discord: ключевое слово '{phrase}': текст не отправлен: {e}");
            }
        }
    }
}

/// `None` значит, что гильдия или участник ещё не в кэше: тогда отправка
/// попробует сама и залогирует ошибку, вместо ложного отказа.
fn can_send_in_channel(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    channel_id: serenity::ChannelId,
    needed: serenity::Permissions,
) -> Option<bool> {
    let guild = ctx.cache.guild(guild_id)?;
    let channel = guild.channels.get(&channel_id)?;
    let member = guild.members.get(&ctx.cache.current_user().id)?;
    Some(guild.user_permissions_in(channel, member).contains(needed))
}

async fn send_keyword_file(
    http: &serenity::Http,
    channel: serenity::ChannelId,
    meme: &memes::Meme,
) -> Result<(), Error> {
    let metadata = tokio::fs::metadata(&meme.path).await?;
    if metadata.len() > commands::misc::DISCORD_UPLOAD_LIMIT {
        warn!("Discord: файл {} больше 8 МиБ, не отправляю", meme.name);
        return Ok(());
    }

    let bytes = tokio::fs::read(&meme.path).await?;
    let filename = meme
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("meme");
    let attachment = serenity::CreateAttachment::bytes(bytes, filename);
    channel
        .send_message(http, serenity::CreateMessage::new().add_file(attachment))
        .await?;
    Ok(())
}

/// Пять повторов регистрации с экспоненциальной задержкой 5, 10, 20, 40, 80 с.
/// Пока регистрация не прошла, команды в Discord недоступны: это логируется,
/// но не мешает боту отвечать на события.
async fn register_commands_with_retry(
    http: Arc<serenity::Http>,
    commands: Vec<poise::Command<Data, Error>>,
) {
    const RETRIES: u32 = 5;
    for attempt in 1..=RETRIES + 1 {
        match poise::builtins::register_globally(&http, &commands).await {
            Ok(()) => {
                info!("слеш-команды зарегистрированы (попытка {attempt})");
                return;
            }
            Err(e) if attempt > RETRIES => {
                error!(
                    "регистрация слеш-команд не удалась после {RETRIES} повторов: {e}; \
                     команды недоступны до перезапуска"
                );
                return;
            }
            Err(e) => {
                let delay = Duration::from_secs(5u64 << (attempt - 1));
                warn!(
                    "регистрация слеш-команд: попытка {attempt} не удалась: {e}; \
                     повтор через {} с, команды пока недоступны",
                    delay.as_secs()
                );
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Строка подключения из окружения. Отсутствие переменных не фатально:
/// музыка, помощь и фетч работают без базы.
fn database_url() -> anyhow::Result<String> {
    Ok(format!(
        "postgres://{}:{}@{}:{}/{}",
        std::env::var("DB_USER").context("нет DB_USER")?,
        std::env::var("DB_PASSWORD").context("нет DB_PASSWORD")?,
        std::env::var("DB_HOST").context("нет DB_HOST")?,
        std::env::var("DB_PORT").context("нет DB_PORT")?,
        std::env::var("DB_NAME").context("нет DB_NAME")?,
    ))
}

/// Подключение и миграции не роняют запуск: при сбое логируем и идём без базы.
async fn connect_db(url: &str) -> Option<Db> {
    match Db::connect(url).await {
        Ok(db) => match db.migrate().await {
            Ok(()) => {
                info!("БД готова");
                Some(db)
            }
            Err(e) => {
                warn!("БД: миграции не применились, работаем без базы: {e}");
                None
            }
        },
        Err(e) => {
            warn!("БД: PostgreSQL недоступен, работаем без базы: {e}");
            None
        }
    }
}

/// Один обработчик на все ошибки команд: пользователь получает эфемерный
/// текст, журнал получает команду и причину. В Python обработчика не было,
/// поэтому любая ошибка выглядела как «приложение не ответило».
async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { ctx, error, .. } => {
            let command = ctx.command().qualified_name.clone();
            error!("команда /{command} упала: {error}");
            let text = "❌ Внутренняя ошибка команды, подробности в журнале.";
            let reply = poise::CreateReply::default().content(text).ephemeral(true);
            let _ = ctx.send(reply).await;
        }
        poise::FrameworkError::Setup { error, .. } => {
            error!("ошибка запуска: {error}");
            // Без user data poise вешает каждое событие в ожидании; живой
            // процесс не даёт restart: always перезапустить контейнер.
            std::process::exit(1);
        }
        other => {
            if let Err(e) = poise::builtins::on_error(other).await {
                error!("необработанная ошибка: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_content_flag_decides_intent() {
        use poise::serenity_prelude::ApplicationFlags;

        assert!(can_request_message_content(
            ApplicationFlags::GATEWAY_MESSAGE_CONTENT
        ));
        assert!(can_request_message_content(
            ApplicationFlags::GATEWAY_MESSAGE_CONTENT_LIMITED
        ));
        assert!(!can_request_message_content(ApplicationFlags::empty()));
        assert!(!can_request_message_content(
            ApplicationFlags::APPLICATION_AUTO_MODERATION_RULE_CREATE_BADGE
        ));
    }
}
