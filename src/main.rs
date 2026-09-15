mod config;
mod font;
mod images;
mod picker;
mod theme;
mod thumbnails;
mod wallpaper;

use config::Config;
use std::process::{exit, Command};

fn check_deps() {
    // "awww" replaces "magick"/"rofi" from the original dependency check --
    // this binary now does the picker + thumbnailing itself.
    for cmd in ["awww"] {
        let found = Command::new("which")
            .arg(cmd)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !found {
            eprintln!("Error: '{cmd}' is not installed.");
            exit(1);
        }
    }
}

fn notify(summary: &str, body: &str) {
    let _ = Command::new("notify-send")
        .args(["-u", "low", summary, body])
        .status();
}

fn main() {
    check_deps();
    let cfg = Config::load();

    if !cfg.wallpaper_dir.is_dir() {
        eprintln!(
            "Error: wallpaper directory '{}' does not exist.",
            cfg.wallpaper_dir.display()
        );
        exit(1);
    }

    let all_images = images::list_images(&cfg).unwrap_or_else(|e| {
        eprintln!("Error reading wallpaper directory: {e}");
        exit(1);
    });
    if all_images.is_empty() {
        eprintln!(
            "No images found in {} (supported: {:?})",
            cfg.wallpaper_dir.display(),
            config::IMAGE_EXTS
        );
        exit(1);
    }

    let originals = images::filter_originals(all_images);
    if originals.is_empty() {
        notify(
            "No wallpapers found",
            &format!(
                "No original (non-blurred) images in {}",
                cfg.wallpaper_dir.display()
            ),
        );
        exit(1);
    }

    // Build/refresh the thumbnail cache (image-rs, in-process, parallel via
    // rayon) before opening the picker -- same ordering as the bash script.
    if let Err(e) = thumbnails::ensure_thumbnails(&cfg, &originals) {
        eprintln!("Error generating thumbnails: {e}");
        exit(1);
    }

    // Let the user pick. Cancel (Escape / close) exits quietly, same as the
    // script's `exit 0` on an empty rofi choice.
    let Some(selected_img) = picker::pick(&cfg, &originals) else {
        exit(0);
    };

    let blurred_img = images::blurred_variant(&selected_img);

    // Apply both namespaces -- unchanged backend behavior.
    if let Err(e) = wallpaper::apply(&cfg, None, &selected_img) {
        eprintln!("Error applying default wallpaper: {e}");
        exit(1);
    }

    if images::full_path(&cfg, &blurred_img).is_file() {
        if let Err(e) = wallpaper::apply(&cfg, Some("overview"), &blurred_img) {
            eprintln!("Error applying overview wallpaper: {e}");
        }
    } else {
        notify(
            "Blurred variant missing",
            &format!("Could not find {blurred_img} — overview namespace unchanged."),
        );
    }

    notify(
        "Wallpaper changed",
        &format!("Default: {selected_img}\nOverview: {blurred_img}"),
    );
    thumbnails::play_sound(&cfg);
}
