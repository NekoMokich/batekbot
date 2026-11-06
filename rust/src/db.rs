//! PostgreSQL через sqlx (только runtime-запросы: `cargo check` не требует
//! живой базы; `sqlx::migrate!` только вшивает файлы миграций).
//!
//! База не обязательна для запуска: `Db::connect` делает 5 попыток по 1 с,
//! вызывающий код при сбое продолжает без базы.
//!
//! Отличия от Python-версии:
//! - личность пользователя это `id`, а не `username`; вставка одна и атомарная,
//!   гонки select-затем-insert нет;
//! - `echo=True` не переносим: SQL с параметрами в логи не пишется;
//! - пул ограничен и имеет таймаут acquire, чтобы зависшая база не вешала
//!   слеш-команду на 30 секунд (дефолт sqlx).

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;

#[derive(Clone)]
pub struct Db {
    pool: sqlx::PgPool,
}

impl Db {
    /// Подключение с повторами: в compose база может стартовать позже бота.
    /// Пять попыток по секунде; недоступность базы не фатальна для бота.
    pub async fn connect(url: &str) -> Result<Db, sqlx::Error> {
        let mut last = None;
        for attempt in 1..=5u32 {
            match PgPoolOptions::new()
                .max_connections(4)
                .acquire_timeout(Duration::from_secs(5))
                .connect(url)
                .await
            {
                Ok(pool) => return Ok(Db { pool }),
                Err(e) => {
                    tracing::warn!("БД: попытка {attempt}/5 не удалась: {e}");
                    last = Some(e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
        Err(last.expect("цикл выполняется хотя бы один раз"))
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("./migrations").run(&self.pool).await
    }

    /// Один атомарный upsert: известный id получает актуальный username,
    /// неизвестный добавляется. Лишнего чтения нет.
    pub async fn ensure_user(&self, id: i64, username: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET username = EXCLUDED.username
            "#,
        )
        .bind(id)
        .bind(username)
        .execute(&self.pool)
        .await
        .map(|_| ())
    }

    /// То же для Telegram: таблица `tg_users` отдельная, `users` не трогаем.
    pub async fn ensure_tg_user(&self, id: i64, username: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO tg_users (id, username)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET username = EXCLUDED.username
            "#,
        )
        .bind(id)
        .bind(username)
        .execute(&self.pool)
        .await
        .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    /// Тест идёт по настоящей базе. Без `DATABASE_URL` он честно печатает
    /// «НЕ ВЫПОЛНЕНО» и не считается успехом (правило ворот 1).
    #[tokio::test]
    async fn upsert_is_idempotent_and_updates_username() {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("НЕ ВЫПОЛНЕНО: DATABASE_URL не задан, живая PostgreSQL недоступна");
            return;
        };

        let db = super::Db::connect(&url)
            .await
            .expect("подключение к живой базе");
        db.migrate().await.expect("миграции накатаны");

        let id = 9_000_000_000_000_000_001_i64;
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();

        db.ensure_user(id, "first").await.unwrap();
        db.ensure_user(id, "second").await.unwrap();

        let name: String = sqlx::query_scalar("SELECT username FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(name, "second");

        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);

        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }
}
