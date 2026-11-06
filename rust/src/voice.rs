//! Голосовое воспроизведение локальных файлов через songbird.
//!
//! Почему так:
//! - Декодирование идёт symphonia (фичи включены в Cargo.toml); ffmpeg для
//!   mp3/ogg/wav/flac не нужен, процесс на песню не плодится.
//! - Драйвер songbird не отдаёт `is_playing`, поэтому истина о треке это
//!   `TrackHandle::get_info().playing`.
//! - После конца трека, ошибки или `/stop` вызывается `Songbird::remove`:
//!   `leave` оставляет `Call` с драйвером и задачами, `remove` освобождает их.
//!   Это закрывает утечку Python-версии, где клиент жил до перезапуска.
//! - У каждой гильдии есть номер поколения. Наблюдатель старого трека видит,
//!   что поколение сменилось, и не выходит из канала, который уже занял новый
//!   трек (иначе `/play` поверх играющего трека выкидывал бота из канала).

use std::collections::HashMap;
use std::num::NonZeroU64;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use poise::serenity_prelude as serenity;
use songbird::tracks::TrackHandle;
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tracing::{debug, warn};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Как часто проверяем, доиграл ли трек.
const TRACK_POLL: Duration = Duration::from_secs(1);
/// `get_info` не имеет таймаута внутри songbird: у запаркованного микшера
/// команда не читается вообще, и future висит вечно. Поэтому свой предел.
const TRACK_INFO_TIMEOUT: Duration = Duration::from_secs(5);
/// Сколько подряд неизвестных состояний терпим, прежде чем считать трек мёртвым.
const MAX_UNKNOWN: u8 = 2;

/// Наблюдаемое состояние трека.
enum Observation {
    Playing,
    Done,
    Unknown,
}

async fn observe(handle: &TrackHandle) -> Observation {
    match timeout(TRACK_INFO_TIMEOUT, handle.get_info()).await {
        // `Err(Finished)` это нормальное состояние после конца трека.
        Ok(Ok(info)) if info.playing.is_done() => {
            // Ошибка декодирования иначе пропадала бы молча: Python печатал
            // её в stdout через `after`-колбэк.
            if let songbird::tracks::PlayMode::Errored(e) = &info.playing {
                warn!("songbird: трек завершился ошибкой: {e}");
            }
            Observation::Done
        }
        Ok(Ok(_)) => Observation::Playing,
        Ok(Err(_)) => Observation::Done,
        Err(_) => Observation::Unknown,
    }
}

pub struct Voice {
    /// Зарегистрирован на клиенте serenity при старте (`.register_songbird_with`).
    pub manager: Arc<songbird::Songbird>,
    handles: Arc<RwLock<HashMap<u64, TrackHandle>>>,
    /// Последнее поколение трека на гильдию; растёт монотонно.
    generations: Arc<RwLock<HashMap<u64, u64>>>,
    /// Операции над гильдией (вход, стоп, уборка) сериализуются: иначе
    /// наблюдатель старого трека мог снять только что установленное
    /// соединение нового.
    ops: Arc<RwLock<HashMap<u64, Arc<Mutex<()>>>>>,
}

fn sb_guild(guild: serenity::GuildId) -> Result<songbird::id::GuildId, BoxError> {
    let id = NonZeroU64::new(guild.get()).ok_or("нулевой guild id")?;
    Ok(songbird::id::GuildId(id))
}

fn sb_channel(channel: serenity::ChannelId) -> Result<songbird::id::ChannelId, BoxError> {
    let id = NonZeroU64::new(channel.get()).ok_or("нулевой channel id")?;
    Ok(songbird::id::ChannelId(id))
}

impl Voice {
    pub fn new(manager: Arc<songbird::Songbird>) -> Self {
        Self {
            manager,
            handles: Arc::new(RwLock::new(HashMap::new())),
            generations: Arc::new(RwLock::new(HashMap::new())),
            ops: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Замок операций гильдии; создаётся один раз на гильдию.
    async fn op_lock(&self, guild: u64) -> Arc<Mutex<()>> {
        if let Some(lock) = self.ops.read().await.get(&guild) {
            return lock.clone();
        }
        let mut ops = self.ops.write().await;
        ops.entry(guild)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    /// Входит в канал (или переходит в него) и запускает ровно один трек.
    /// Предыдущий трек в этой гильдии останавливается, как `voice_client.stop()`
    /// перед `play()` в Python-версии.
    pub async fn play_file(
        &self,
        guild: serenity::GuildId,
        channel: serenity::ChannelId,
        file: &Path,
    ) -> Result<(), BoxError> {
        let guild_id = sb_guild(guild)?;
        let channel_id = sb_channel(channel)?;

        let op = self.op_lock(guild.get()).await;
        let _guard = op.lock_owned().await;

        // Поколение объявляется ДО входа в канал: наблюдатель прошлого трека
        // может проснуться в любой момент, включая время сетевого join, и не
        // должен принять себя за хозяина канала.
        let generation = {
            let mut generations = self.generations.write().await;
            let next = generations.get(&guild.get()).copied().unwrap_or(0) + 1;
            generations.insert(guild.get(), next);
            next
        };

        // `join` сам переводит бота в другой канал (songbird `should_actually_join`),
        // поэтому ручной `leave` перед входом не нужен: он только добавлял
        // лишний обмен с гейтвеем и гонку с наблюдателем трека.
        let call = match self.manager.join(guild_id, channel_id).await {
            Ok(call) => call,
            Err(e) => {
                self.cleanup(guild).await;
                return Err(e.into());
            }
        };

        if call.lock().await.current_channel() != Some(channel_id) {
            self.cleanup(guild).await;
            return Err("не удалось войти в голосовой канал".into());
        }

        if let Some(previous) = self.handles.write().await.remove(&guild.get()) {
            let _ = previous.stop();
        }

        // `Input` требует владения путём на всё время жизни трека, поэтому
        // копия обязательна (clippy не видит требование `'static`).
        #[allow(clippy::unnecessary_to_owned)]
        let input: songbird::input::Input = songbird::input::File::new(file.to_owned()).into();
        let handle = call.lock().await.play_only_input(input);
        self.handles
            .write()
            .await
            .insert(guild.get(), handle.clone());

        let manager = self.manager.clone();
        let handles = self.handles.clone();
        let generations = self.generations.clone();
        let ops = self.ops.clone();
        tokio::spawn(async move {
            let mut unknown = 0u8;
            loop {
                tokio::time::sleep(TRACK_POLL).await;
                match observe(&handle).await {
                    Observation::Playing => unknown = 0,
                    Observation::Done => break,
                    // Микшер запаркован и не отвечает: дальше ждать нечего.
                    Observation::Unknown => {
                        unknown += 1;
                        if unknown >= MAX_UNKNOWN {
                            break;
                        }
                    }
                }
            }

            // Проверка поколения и уборка идут под замком операций гильдии:
            // иначе между проверкой и `remove` мог начаться новый трек.
            let op = {
                let existing = ops.read().await.get(&guild.get()).cloned();
                match existing {
                    Some(op) => op,
                    None => {
                        let mut map = ops.write().await;
                        map.entry(guild.get())
                            .or_insert_with(|| Arc::new(Mutex::new(())))
                            .clone()
                    }
                }
            };
            let _guard = op.lock_owned().await;

            let is_current =
                generations.read().await.get(&guild.get()).copied() == Some(generation);
            if !is_current {
                return;
            }

            {
                let mut map = handles.write().await;
                match map.get(&guild.get()) {
                    Some(current) if current.uuid() == handle.uuid() => {
                        map.remove(&guild.get());
                    }
                    _ => return,
                }
            }

            if let Err(e) = manager.remove(guild_id).await {
                debug!("songbird: не удалось освободить канал {}: {e}", guild.get());
            }
        });

        Ok(())
    }

    /// Останавливает трек, если он играет, и в любом случае освобождает
    /// канал. Возвращает `false`, когда играть нечего.
    pub async fn stop(&self, guild: serenity::GuildId) -> bool {
        let guild_raw = guild.get();

        let op = self.op_lock(guild_raw).await;
        let _guard = op.lock_owned().await;

        // Наблюдатель трека больше не должен считать себя хозяином канала.
        {
            let mut generations = self.generations.write().await;
            let next = generations.get(&guild_raw).copied().unwrap_or(0) + 1;
            generations.insert(guild_raw, next);
        }

        let handle = self.handles.write().await.remove(&guild_raw);
        let playing = match &handle {
            Some(handle) => match observe(handle).await {
                Observation::Playing => true,
                Observation::Done => false,
                // Неизвестное состояние считаем игравшим: пользователь просил
                // остановить, и канал всё равно освобождается.
                Observation::Unknown => true,
            },
            None => false,
        };

        if let Some(handle) = handle {
            let _ = handle.stop();
        }
        self.cleanup(guild).await;
        playing
    }

    /// Выход из всех каналов при выключении процесса.
    pub async fn leave_all(&self) {
        let guilds: Vec<songbird::id::GuildId> =
            self.manager.iter().map(|(guild, _)| guild).collect();
        self.handles.write().await.clear();
        self.ops.write().await.clear();
        for guild_id in guilds {
            if let Err(e) = self.manager.remove(guild_id).await {
                warn!("songbird: не удалось освободить канал: {e}");
            }
        }
    }

    /// Освобождает `Call` и его задачи. Ошибка означает только «не подключены».
    async fn cleanup(&self, guild: serenity::GuildId) {
        let Ok(guild_id) = sb_guild(guild) else {
            return;
        };
        if let Err(e) = self.manager.remove(guild_id).await {
            debug!("songbird: освобождение канала {}: {e}", guild.get());
        }
    }
}
