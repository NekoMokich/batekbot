-- Пользователи Telegram. Отдельная таблица: `users` хранит Discord id и
-- менять её схему нельзя (жёсткий контракт AGENTS.md). Telegram id тоже
-- BIGINT и тоже уникален, username не обязан быть уникальным.
CREATE TABLE IF NOT EXISTS tg_users (
    id BIGINT PRIMARY KEY,
    username TEXT NOT NULL
);
