//! Список локальной музыки: кэш по mtime и проверка имени файла.
//!
//! Зачем отдельный модуль:
//! - автокомплит дёргал `os.listdir` на каждое нажатие клавиши (Python);
//!   здесь каталог читается один раз на изменение mtime;
//! - `/play` принимал произвольную строку, включая `/tmp/x.mp3` и `../../`,
//!   и открывал её (path traversal). Здесь имя принимается только если оно
//!   совпадает с реальным файлом из каталога `music/`.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tokio::sync::RwLock;

/// Расширения, которые умеет symphonia через songbird.
pub const AUDIO_EXTENSIONS: [&str; 3] = [".mp3", ".wav", ".ogg"];

type CacheEntry = Option<(Option<SystemTime>, Vec<String>)>;

pub struct Library {
    dir: PathBuf,
    cache: RwLock<CacheEntry>,
}

impl Library {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            cache: RwLock::new(None),
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Существует ли каталог; если нет, создаёт его. `true` означает «создан».
    pub async fn ensure_dir(&self) -> std::io::Result<bool> {
        if tokio::fs::try_exists(&self.dir).await.unwrap_or(false) {
            return Ok(false);
        }
        tokio::fs::create_dir_all(&self.dir).await?;
        Ok(true)
    }

    /// Отсортированный список аудиофайлов; кэш сбрасывается при смене mtime.
    pub async fn list(&self) -> Vec<String> {
        let mtime = tokio::fs::metadata(&self.dir)
            .await
            .ok()
            .and_then(|m| m.modified().ok());

        if let Some((cached_mtime, files)) = self.cache.read().await.clone() {
            if cached_mtime == mtime {
                return files;
            }
        }

        let mut files = Vec::new();
        if let Ok(mut rd) = tokio::fs::read_dir(&self.dir).await {
            while let Ok(Some(entry)) = rd.next_entry().await {
                let Ok(file_type) = entry.file_type().await else {
                    continue;
                };
                if !file_type.is_file() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().into_owned();
                if is_audio_file(&name) {
                    files.push(name);
                }
            }
        }
        files.sort();

        *self.cache.write().await = Some((mtime, files.clone()));
        files
    }

    /// Приводит аргумент команды к реальному имени файла в каталоге.
    ///
    /// Отклоняет пустую строку, `/`, `\`, `..`, `.` и всё, что не совпадает с
    /// файлом из списка. Если расширения нет, дописывается `.mp3` (поведение
    /// Python-версии, но с проверкой по списку).
    pub async fn resolve(&self, raw: &str) -> Option<String> {
        let candidate = normalize_name(raw)?;
        let files = self.list().await;
        if files.iter().any(|f| f == &candidate) {
            Some(candidate)
        } else {
            None
        }
    }
}

/// Чистое правило имени: без каталогов и переходов вверх.
pub fn normalize_name(raw: &str) -> Option<String> {
    if raw.is_empty() || raw == "." || raw == ".." {
        return None;
    }
    if raw.contains('/') || raw.contains('\\') || raw.contains('\0') {
        return None;
    }
    if Path::new(raw).file_name()?.to_str()? != raw {
        return None;
    }
    if is_audio_file(raw) {
        Some(raw.to_string())
    } else {
        Some(format!("{raw}.mp3"))
    }
}

/// Сравнение расширения без учёта регистра: `song.MP3` не должен получить
/// второй `.mp3` (дефект Python-версии).
pub fn is_audio_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    AUDIO_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_traversal_and_directories() {
        for bad in [
            "",
            ".",
            "..",
            "../x.mp3",
            "/tmp/x.mp3",
            "a/b.mp3",
            "a\\b.mp3",
            "x\0.mp3",
        ] {
            assert_eq!(normalize_name(bad), None, "должно быть отклонено: {bad:?}");
        }
    }

    #[test]
    fn appends_mp3_once_and_ignores_case() {
        assert_eq!(normalize_name("track"), Some("track.mp3".to_string()));
        assert_eq!(normalize_name("track.MP3"), Some("track.MP3".to_string()));
        assert_eq!(normalize_name("track.ogg"), Some("track.ogg".to_string()));
    }

    #[tokio::test]
    async fn lists_only_audio_and_resolves_known_file() {
        let dir = std::env::temp_dir().join(format!("batek-lib-{}", std::process::id()));
        let _ = tokio::fs::remove_dir_all(&dir).await;
        tokio::fs::create_dir_all(&dir).await.unwrap();
        tokio::fs::write(dir.join("a.mp3"), b"x").await.unwrap();
        tokio::fs::write(dir.join("b.wav"), b"x").await.unwrap();
        tokio::fs::write(dir.join("c.txt"), b"x").await.unwrap();

        let lib = Library::new(dir.clone());
        assert_eq!(
            lib.list().await,
            vec!["a.mp3".to_string(), "b.wav".to_string()]
        );
        assert_eq!(lib.resolve("a.mp3").await, Some("a.mp3".to_string()));
        assert_eq!(lib.resolve("a").await, Some("a.mp3".to_string()));
        assert_eq!(lib.resolve("../a.mp3").await, None);
        assert_eq!(lib.resolve("nope.mp3").await, None);

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn skips_directories_with_audio_extension() {
        let dir = std::env::temp_dir().join(format!("batek-lib-dir-{}", std::process::id()));
        let _ = tokio::fs::remove_dir_all(&dir).await;
        tokio::fs::create_dir_all(&dir).await.unwrap();
        tokio::fs::write(dir.join("a.mp3"), b"x").await.unwrap();
        tokio::fs::create_dir(dir.join("dir.mp3")).await.unwrap();

        let lib = Library::new(dir.clone());
        let files = lib.list().await;
        assert_eq!(files, vec!["a.mp3".to_string()]);
        assert!(
            !files.iter().any(|f| f == "dir.mp3"),
            "каталог dir.mp3 не должен попадать в список"
        );

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}
