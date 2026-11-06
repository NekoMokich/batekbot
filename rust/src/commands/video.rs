//! `/timur`, `/batek`, `/victorovich`: запись пользователя и случайная ссылка
//! из кэша (`youtube.rs`).
//!
//! Что изменено относительно Python-версии:
//! - запись в БД не блокирует ответ и не может уронить команду: один upsert
//!   по `id` выполняется в фоне, ошибка только логируется (в Python ошибка
//!   базы или дубль username роняли команду);
//! - если базы нет, upsert пропускается, предупреждение пишется один раз:
//!   выбор видео от базы не зависит;
//! - нажатия кнопок ограничены автором команды, чужие получают эфемерный
//!   отказ: в Python кнопки мог нажимать кто угодно.

use std::time::Duration;

use poise::serenity_prelude as serenity;
use rand::seq::IndexedRandom;
use rand::Rng;
use tracing::warn;

use crate::{Context, Error};

/// Python: `randint(1,100) > 10` отправляет кнопки, то есть ровно 10% на шутку.
fn should_aiditinaxuy() -> bool {
    rand::rng().random_range(1..=100) <= 10
}

fn remember_user(ctx: Context<'_>) {
    let Some(db) = ctx.data().db.clone() else {
        static DB_ABSENT_LOGGED: std::sync::Once = std::sync::Once::new();
        DB_ABSENT_LOGGED.call_once(|| {
            warn!("БД недоступна: запись пользователей пропускается");
        });
        return;
    };
    let id = ctx.author().id.get() as i64;
    let username = ctx.author().name.clone();
    tokio::spawn(async move {
        if let Err(e) = db.ensure_user(id, &username).await {
            warn!("БД: не удалось записать пользователя {id}: {e}");
        }
    });
}

async fn video_choice(
    ctx: Context<'_>,
    key_shorts: &'static str,
    key_videos: &'static str,
) -> Result<(), Error> {
    remember_user(ctx);

    if should_aiditinaxuy() {
        ctx.say("аидитынахуй").await?;
        return Ok(());
    }

    let owner = ctx.author().id;
    let uid = ctx.id();
    let id_shorts = format!("{uid}:sh");
    let id_videos = format!("{uid}:vd");

    let reply = poise::CreateReply::default()
        .content("че смотреть будем")
        .components(vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(id_shorts.clone())
                .label("📱 Случайный Шортс")
                .style(serenity::ButtonStyle::Primary),
            serenity::CreateButton::new(id_videos.clone())
                .label("📺 Обычное видео")
                .style(serenity::ButtonStyle::Success),
        ])]);
    ctx.send(reply).await?;

    let yt = ctx.data().yt.clone();
    let serenity_ctx = ctx.serenity_context();

    loop {
        let want_shorts = id_shorts.clone();
        let want_videos = id_videos.clone();

        let Some(interaction) = serenity::ComponentInteractionCollector::new(serenity_ctx)
            .filter(move |mci| {
                mci.data.custom_id == want_shorts || mci.data.custom_id == want_videos
            })
            .timeout(Duration::from_secs(180))
            .await
        else {
            break; // три минуты без нажатий, как дефолтный View в Python
        };

        if interaction.user.id != owner {
            let reply = serenity::CreateInteractionResponseMessage::new()
                .content("это не твои кнопки")
                .ephemeral(true);
            interaction
                .create_response(ctx, serenity::CreateInteractionResponse::Message(reply))
                .await?;
            continue;
        }

        let (key, header, empty) = if interaction.data.custom_id == id_shorts {
            (key_shorts, "Вот твой шортс:", "Шортсов пока нет :(")
        } else {
            (
                key_videos,
                "Вот обычный видос:",
                "Обычных видео пока нет :(",
            )
        };

        let ids = yt.get(key).await;
        let chosen = ids.choose(&mut rand::rng()).map(ToOwned::to_owned);

        let message = match chosen {
            Some(id) => serenity::CreateInteractionResponseMessage::new()
                .content(format!("{header}\nhttps://youtu.be/{id}")),
            None => serenity::CreateInteractionResponseMessage::new()
                .content(empty)
                .ephemeral(true),
        };

        interaction
            .create_response(ctx, serenity::CreateInteractionResponse::Message(message))
            .await?;
    }
    Ok(())
}

/// ЗАЧЕМ Я ПОМЕНЯЛ ЕЕ НА ЭТО НА ЧЕРНОБЕЛОЕ
#[poise::command(slash_command)]
pub async fn timur(ctx: Context<'_>) -> Result<(), Error> {
    video_choice(ctx, "timur:shorts", "timur:videos").await
}

/// белые розы
#[poise::command(slash_command)]
pub async fn batek(ctx: Context<'_>) -> Result<(), Error> {
    video_choice(ctx, "batek:shorts", "batek:videos").await
}

/// В СТАРОМ ПАРКЕ ПАХНЕТ ХВОЙНОЙ ТИШИНОЙ
#[poise::command(slash_command)]
pub async fn victorovich(ctx: Context<'_>) -> Result<(), Error> {
    video_choice(ctx, "victorovich:shorts", "victorovich:videos").await
}
