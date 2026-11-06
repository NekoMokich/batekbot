-- Parity with the Alembic migration `f0625e9d3718_create_users_table`
-- (migrations/versions/*.py): users table, exactly as alembic created it.
CREATE TABLE IF NOT EXISTS users (
    id BIGINT NOT NULL,
    username VARCHAR(100) NOT NULL,
    CONSTRAINT users_pkey PRIMARY KEY (id)
);
