use crate::config::{Config, IMAGE_EXTS};
use std::fs;
use std::path::PathBuf;

/// List wallpaper filenames (not full paths) directly inside `wallpaper_dir`,
/// filtered to supported extensions, sorted -- same as `get_images()` in the
/// original script. Non-recursive (maxdepth 1), same as `find -maxdepth 1`.
pub fn list_images(cfg: &Config) -> std::io::Result<Vec<String>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(&cfg.wallpaper_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy().to_string();
        let ext_matches = name
            .rsplit_once('.')
            .map(|(_, ext)| IMAGE_EXTS.iter().any(|e| e.eq_ignore_ascii_case(ext)))
            .unwrap_or(false);
        if ext_matches {
            out.push(name);
        }
    }
    out.sort();
    Ok(out)
}

/// Drop pre-blurred variants (`*-b.ext`) from a listing -- only originals are
/// offered in the picker, same as the `filtered_images` step in the script.
pub fn filter_originals(images: Vec<String>) -> Vec<String> {
    images
        .into_iter()
        .filter(|img| {
            let stem = img.rsplit_once('.').map(|(s, _)| s).unwrap_or(img.as_str());
            !stem.ends_with("-b")
        })
        .collect()
}

/// Derive the blurred counterpart filename for a selected image
/// (`mountain.jpg` -> `mountain-b.jpg`), same rule as the script.
pub fn blurred_variant(image: &str) -> String {
    match image.rsplit_once('.') {
        Some((stem, ext)) => format!("{stem}-b.{ext}"),
        None => format!("{image}-b"),
    }
}

pub fn full_path(cfg: &Config, filename: &str) -> PathBuf {
    cfg.wallpaper_dir.join(filename)
}

pub fn thumb_path(cfg: &Config, filename: &str) -> PathBuf {
    cfg.thumb_dir.join(filename)
}
