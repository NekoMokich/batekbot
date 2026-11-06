//! `/play` и `/stop`: локальные файлы из `music/` через songbird.
//!
//! Тексты ответов дословно из `cogs/music.py`, порядок проверок сохранён,
//! но меняется то, что было дефектом: файл проверяется до входа в канал
//! (в Python бот сначала подключался, потом обнаруживал, что играть нечего),
//! имя файла сверяется со списком каталога, а `/stop` действительно
//! освобождает голос.

use rand::seq::IndexedRandom;

use crate::library::normalize_name;
use crate::{Context, Error};

/// Отправляет «файл не найден» без разбора упоминаний в имени файла.
async fn send_play_not_found(ctx: Context<'_>, name: &str) -> Result<(), Error> {
    use poise::serenity_prelude::CreateAllowedMentions;
    use poise::CreateReply;

    ctx.send(
        CreateReply::default()
            .content(format!("❌ Файл '**{name}**' не найден."))
            .allowed_mentions(
                CreateAllowedMentions::default()
                    .all_users(false)
                    .all_roles(false)
                    .everyone(false),
            ),
    )
    .await?;
    Ok(())
}

async fn music_autocomplete(
    ctx: Context<'_>,
    partial: &str,
) -> poise::serenity_prelude::CreateAutocompleteResponse {
    use poise::serenity_prelude::CreateAutocompleteResponse;

    let needle = partial.to_lowercase();
    let mut response = CreateAutocompleteResponse::new();
    for name in ctx
        .data()
        .library
        .list()
        .await
        .into_iter()
        .filter(|f| f.to_lowercase().contains(&needle))
        .filter(|f| f.chars().count() <= 100)
        .take(25)
    {
        // Discord ограничивает имя и значение 100 символами. Имя длиннее
        // лимита пропущено выше: обрезанное значение уже не совпадает с
        // реальным файлом и всё равно дало бы ошибку.
        let short: String = name.chars().take(100).collect();
        response = response.add_string_choice(&short, &name);
    }
    response
}

fn random_choice<T>(list: &[T]) -> Option<&T> {
    list.choose(&mut rand::rng())
}

/// Я НАФАРМИЛ КРИПТЫ КАК СКУФ
#[poise::command(slash_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Имя файла из папки music"]
    #[autocomplete = "music_autocomplete"]
    filename: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    // 1. Пользователь должен быть в голосовом канале. Подключения здесь нет.
    let channel = ctx.guild().and_then(|g| {
        g.voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
    });
    let (Some(channel), Some(guild)) = (channel, ctx.guild_id()) else {
        ctx.say("❌ Вы должны находиться в голосовом канале!")
            .await?;
        return Ok(());
    };

    // 2. Каталог музыки.
    if ctx.data().library.ensure_dir().await? {
        ctx.say("📁 Папка 'music' не найдена. Я создал её, закинь туда музыку!")
            .await?;
        return Ok(());
    }

    // 3. Имя файла: только существующий файл из каталога.
    let resolved = match filename.as_deref() {
        Some(raw) => match ctx.data().library.resolve(raw).await {
            Some(name) => name,
            None => {
                let candidate = normalize_name(raw).unwrap_or_else(|| raw.to_string());
                send_play_not_found(ctx, &candidate).await?;
                return Ok(());
            }
        },
        None => {
            let files = ctx.data().library.list().await;
            match random_choice(&files).cloned() {
                Some(name) => name,
                None => {
                    ctx.say("🪹 В папке 'music' нет аудиофайлов.").await?;
                    return Ok(());
                }
            }
        }
    };

    // 4. Только теперь подключаемся и играем. Файл мог исчезнуть между
    //    листингом и этим моментом: открываем его, чтобы убедиться, что он
    //    действительно читается.
    let path = ctx.data().library.dir().join(&resolved);
    if tokio::fs::File::open(&path).await.is_err() {
        send_play_not_found(ctx, &resolved).await?;
        return Ok(());
    }

    match ctx.data().voice.play_file(guild, channel, &path).await {
        Ok(()) => {
            ctx.say(format!("🎶 Сейчас играет: **{resolved}**")).await?;
        }
        Err(e) => {
            ctx.say(format!("❌ Ошибка при воспроизведении: {e}"))
                .await?;
        }
    }
    Ok(())
}

/// ВЫКЛЮЧИ ЕГО НАХУЙ
#[poise::command(slash_command, guild_only)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let Some(guild) = ctx.guild_id() else {
        ctx.say("❌ Сейчас ничего не играет.").await?;
        return Ok(());
    };

    if ctx.data().voice.stop(guild).await {
        ctx.say("🛑 Музыка остановлена.").await?;
    } else {
        ctx.say("❌ Сейчас ничего не играет.").await?;
    }
    Ok(())
}
