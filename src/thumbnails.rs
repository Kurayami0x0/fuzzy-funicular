use crate::config::Config;
use crate::images::{full_path, thumb_path};
use image::imageops::FilterType;
use rayon::prelude::*;
use std::fs;
use std::process::Command;

/// Returns true if `src` is missing a thumbnail or the thumbnail is older
/// than the source file -- same staleness check as the bash script's
/// `[[ ! -f "$thumb" || "$src" -nt "$thumb" ]]`.
fn needs_thumb(src: &std::path::Path, thumb: &std::path::Path) -> bool {
    let src_mtime = fs::metadata(src).and_then(|m| m.modified()).ok();
    let thumb_meta = fs::metadata(thumb).and_then(|m| m.modified());

    match (src_mtime, thumb_meta) {
        (Some(src_t), Ok(thumb_t)) => src_t > thumb_t,
        _ => true, // no thumb yet, or couldn't stat -> (re)generate
    }
}

/// Cover-crop a source image to `size`x`size` and write it into the cache,
/// replacing the `magick -thumbnail "${SIZE}^" -gravity Center -extent
/// "$SIZE"` call. `resize_to_fill` scales so the image *covers* the target
/// box then center-crops the overflow -- the same operation.
fn generate_one(src: &std::path::Path, thumb: &std::path::Path, size: u32) -> Result<(), String> {
    let img = image::open(src).map_err(|e| format!("{}: {e}", src.display()))?;
    let cropped = img.resize_to_fill(size, size, FilterType::Lanczos3);
    cropped
        .save(thumb)
        .map_err(|e| format!("{}: {e}", thumb.display()))
}

/// Build/refresh the thumbnail cache in parallel via rayon (replaces the
/// script's manual `&`-backgrounded magick jobs capped at $MAX_JOBS -- rayon's
/// global pool already caps concurrency at the core count). Also prunes
/// thumbnails whose source wallpaper no longer exists. Cache directory,
/// filenames, and dimensions are unchanged so the old bash script and this
/// tool can share the same cache.
pub fn ensure_thumbnails(cfg: &Config, images: &[String]) -> std::io::Result<()> {
    fs::create_dir_all(&cfg.thumb_dir)?;

    let pending: Vec<&String> = images
        .iter()
        .filter(|img| needs_thumb(&full_path(cfg, img), &thumb_path(cfg, img)))
        .collect();

    if !pending.is_empty() {
        let _ = Command::new("notify-send")
            .args(["-u", "low", "Wallpaper picker", "Generating thumbnails…"])
            .status();
        play_sound(cfg);
    }

    pending.par_iter().for_each(|img| {
        let src = full_path(cfg, img);
        let thumb = thumb_path(cfg, img);
        if let Err(e) = generate_one(&src, &thumb, cfg.thumb_size) {
            eprintln!("thumbnail generation failed for {img}: {e}");
        }
    });

    prune_stale(cfg, images)?;
    Ok(())
}

/// Remove cached thumbnails for wallpapers that were deleted from
/// `wallpaper_dir`, same as the script's prune loop.
fn prune_stale(cfg: &Config, images: &[String]) -> std::io::Result<()> {
    if !cfg.thumb_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(&cfg.thumb_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !images.contains(&name) {
            let _ = fs::remove_file(entry.path());
        }
    }
    Ok(())
}

pub fn play_sound(cfg: &Config) {
    if cfg.sound.is_file() {
        let _ = Command::new("pw-play").arg(&cfg.sound).status();
    }
}
