//! Статические ответы, mp4-вложения из `video/` и `/batekfetch`.
//!
//! Тексты дословно из `cogs/roflcal.py`. Исправлено:
//! - вложение больше не уходит первым ответом: сначала defer, потом followup,
//!   иначе файл на 7.87 МБ не успевал в трёхсекундное окно Discord;
//! - размер проверяется до чтения и до отправки;
//! - `/batekfetch` эфемерный (в Python системная информация уходила публично),
//!   и скан пакетного менеджера удалён: его результат нигде не выводился;
//! - `random.randint(0, 10)` с разными компараторами заменён на честные 50/50.

use poise::serenity_prelude as serenity;
use rand::Rng;

use crate::{Context, Error};

/// Лимит вложения для бота без бустов, байт.
pub const DISCORD_UPLOAD_LIMIT: u64 = 8 * 1024 * 1024;

/// Текст грэндмобайла дословно как в Python. Публичный, потому что тот же
/// текст отправляет Telegram по ключевому слову: строки не расходятся.
pub fn grandmobile_text() -> &'static str {
    "гранд мобайл это игра с открытым миром где можно стать кем угодно \
     хоть блогером полицейским или боссом мафии. А скачать ее можно в апп \
     строке или гугл плей или по ссылке в шапке профиля, а по промокоду \
     \"БАТЕК\" вы получите крутые бонусы"
}

/// грэндмобайл
#[poise::command(slash_command)]
pub async fn grandmobile(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(grandmobile_text()).await?;
    Ok(())
}

/// слєш хелп
#[poise::command(slash_command, rename = "help")]
pub async fn help_command(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("аидитынахуй").await?;
    Ok(())
}

async fn send_video_file(ctx: Context<'_>, filename: &str) -> Result<(), Error> {
    ctx.defer().await?;

    let path = (*ctx.data().root).join("video").join(filename);
    let metadata = match tokio::fs::metadata(&path).await {
        Ok(metadata) => metadata,
        Err(_) => {
            ctx.say(format!("❌ Файл '**{filename}**' не найден."))
                .await?;
            return Ok(());
        }
    };

    if metadata.len() > DISCORD_UPLOAD_LIMIT {
        ctx.say("❌ Файл слишком большой для загрузки в Discord.")
            .await?;
        return Ok(());
    }

    let bytes = match tokio::fs::read(&path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            ctx.say(format!("❌ Ошибка при чтении файла: {e}")).await?;
            return Ok(());
        }
    };

    let attachment = serenity::CreateAttachment::bytes(bytes, filename);
    ctx.send(poise::CreateReply::default().attachment(attachment))
        .await?;
    Ok(())
}

/// АКАКФИЛЬМНАНОУТБУКЕВКЛЮЧИТЬ
#[poise::command(slash_command)]
pub async fn bomjara(ctx: Context<'_>) -> Result<(), Error> {
    if rand::rng().random_bool(0.5) {
        send_video_file(ctx, "bomjara1.mp4").await
    } else {
        send_video_file(ctx, "bomjaro2.mp4").await
    }
}

/// не смешная хуйня про 320 пошути же даун ебучий
#[poise::command(slash_command)]
pub async fn roflvideo(ctx: Context<'_>) -> Result<(), Error> {
    if rand::rng().random_bool(0.5) {
        send_video_file(ctx, "makuos.mp4").await
    } else {
        send_video_file(ctx, "shefos.mp4").await
    }
}

/// купянскнаш
#[poise::command(slash_command)]
pub async fn kupyansk(ctx: Context<'_>) -> Result<(), Error> {
    send_video_file(ctx, "kupyanskdis.mp4").await
}

/// ЭЭЭФЕТЧИ
#[poise::command(slash_command, guild_only)]
pub async fn batekfetch(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let info = tokio::task::spawn_blocking(fetch_info).await?;
    ctx.send(
        poise::CreateReply::default()
            .content(format_fetch(&info))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Данные для `/batekfetch`, снятые один раз в блокирующем потоке.
pub struct FetchInfo {
    pub os: String,
    pub host: String,
    pub cpu: String,
    pub uptime: String,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
}

const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

/// Публичная, потому что тем же блоком пользуется Telegram: формат не
/// дублируется, а данные снимаются в `spawn_blocking` на обеих сторонах.
pub fn fetch_info() -> FetchInfo {
    use sysinfo::{Disks, System};

    let sys = System::new_all();
    let os = System::name()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Unknown OS".into());
    let host = System::host_name()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Unknown HOST".into());
    let cpu = sys
        .cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown CPU".into());

    let disks = Disks::new_with_refreshed_list();
    let (disk_total, disk_used) = match disks
        .list()
        .iter()
        .find(|d| d.mount_point() == std::path::Path::new("/"))
    {
        Some(d) => {
            let total = d.total_space();
            let used = total.saturating_sub(d.available_space());
            (total as f64 / GIB, used as f64 / GIB)
        }
        None => (0.0, 0.0),
    };

    FetchInfo {
        os,
        host,
        cpu,
        uptime: format_uptime(System::uptime()),
        disk_used_gb: disk_used,
        disk_total_gb: disk_total,
        ram_used_gb: sys.used_memory() as f64 / GIB,
        ram_total_gb: sys.total_memory() as f64 / GIB,
    }
}

/// Формат как в Python: минуты, часы, дни.
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    if days == 0 && hours == 0 {
        format!("{minutes}m")
    } else if days == 0 {
        format!("{hours}h, {minutes}m")
    } else {
        format!("{days}d, {hours}h, {minutes}m")
    }
}

pub fn format_fetch(info: &FetchInfo) -> String {
    format!(
        "```text\n\
         \x20_________    \n\
         |______  /    OS:      {}\n\
         \x20     / /     HOST:    {}\n\
         \x20    / /      CPU:     {}\n\
         \x20   / /       UPTIME:  {}\n\
         \x20  / /        DISK:    {:.2} GB / {:.2} GB\n\
         \x20 / /_____    RAM:     {:.2} GB / {:.2} GB\n\
         \x20/________|   \n\
         ```",
        info.os,
        info.host,
        info.cpu,
        info.uptime,
        info.disk_used_gb,
        info.disk_total_gb,
        info.ram_used_gb,
        info.ram_total_gb,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uptime_matches_python_format() {
        assert_eq!(format_uptime(0), "0m");
        assert_eq!(format_uptime(59), "0m");
        assert_eq!(format_uptime(3_600), "1h, 0m");
        assert_eq!(format_uptime(90_061), "1d, 1h, 1m");
    }

    #[test]
    fn fetch_block_has_all_fields_and_gib_numbers() {
        let info = FetchInfo {
            os: "Debian".into(),
            host: "host".into(),
            cpu: "CPU".into(),
            uptime: "1h, 2m".into(),
            disk_used_gb: 10.0,
            disk_total_gb: 100.0,
            ram_used_gb: 1.5,
            ram_total_gb: 8.0,
        };
        let text = format_fetch(&info);
        for needle in ["OS:", "HOST:", "CPU:", "UPTIME:", "DISK:", "RAM:"] {
            assert!(text.contains(needle), "нет поля {needle}");
        }
        assert!(text.contains("10.00 GB / 100.00 GB"));
        // Первая строка лого начинается с одного пробела, как в Python
        // (roflcal.py:88); континуация строк Rust съедает отступ, поэтому
        // пробел записан как \x20.
        assert!(text.contains("\n _________    \n"), "первая строка лого");
        assert!(!text.contains("\n_________"), "лого без ведущего пробела");
        assert!(text.contains("```text\n _________    \n|______  /    OS:"));
        assert!(text.contains("1.50 GB / 8.00 GB"));
    }
}
