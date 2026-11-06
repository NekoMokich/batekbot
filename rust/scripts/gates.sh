#!/usr/bin/env bash
# Ворота 1: автоматические проверки. Запуск из любого каталога.
# Живая PostgreSQL проверяется только если задан DATABASE_URL; иначе тест
# печатает «НЕ ВЫПОЛНЕНО» и это не считается успехом.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "== fmt"
cargo fmt --check

echo "== clippy"
cargo clippy --all-targets -- -D warnings

echo "== tests"
cargo test -- --nocapture

if [[ -z "${DATABASE_URL:-}" ]]; then
    echo "НЕ ВЫПОЛНЕНО: DATABASE_URL не задан, живая PostgreSQL не проверялась"
fi
