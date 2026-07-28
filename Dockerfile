FROM python:3.11-alpine

# Установка системных зависимостей (ffmpeg для видео/аудио, сборщики для C-библиотек)
RUN apk add --no-cache \
    ffmpeg \
    gcc \
    musl-dev \
    libffi-dev

# Скопировать бинарник uv из официального образа
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

WORKDIR /app

# Копируем манифесты зависимостей
COPY pyproject.toml uv.lock ./

# Устанавливаем зависимости с помощью uv
RUN uv sync --frozen --no-dev

# Копируем исходный код проекта
COPY . .

# При старте контейнера автоматически накатятся миграции Alembic и запустится бот
CMD ["sh", "-c", "uv run alembic upgrade head && uv run python main.py"]
