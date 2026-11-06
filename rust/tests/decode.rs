//! Проверка на реальных файлах: symphonia из нашего дерева зависимостей
//! действительно декодирует музыку из `music/`. Если фичи кодеков в
//! Cargo.toml отвалятся, songbird будет молчать, а этот тест упадёт.
//!
//! Файлы настоящие, лежат в репозитории; синтетических фикстур нет.

use std::path::PathBuf;

use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

fn music_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("music")
}

fn decoded_frames(path: &std::path::Path) -> anyhow::Result<usize> {
    let file = std::fs::File::open(path)?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe().format(
        &hint,
        stream,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or_else(|| anyhow::anyhow!("нет дорожки"))?;
    let mut decoder =
        symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    let mut frames = 0usize;
    for _ in 0..32 {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(_) => break,
        };
        frames += decoder.decode(&packet)?.frames();
    }
    Ok(frames)
}

#[test]
fn decodes_every_mp3_in_music_dir() {
    let dir = music_dir();
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("НЕ ВЫПОЛНЕНО: нет каталога {}: {e}", dir.display());
            return;
        }
    };

    let mut checked = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("mp3") {
            continue;
        }
        let frames = decoded_frames(&path)
            .unwrap_or_else(|e| panic!("не декодируется {}: {e}", path.display()));
        assert!(frames > 0, "ноль сэмплов из {}", path.display());
        checked += 1;
    }

    assert!(checked > 0, "в {} нет mp3-файлов", dir.display());
}
