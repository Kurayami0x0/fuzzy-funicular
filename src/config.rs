use std::path::PathBuf;

/// Mirrors the constants at the top of `awww-walls-rofi.sh`.
/// Backend behavior (awww daemon, namespaces, thumbnail cache location/format)
/// is kept identical on purpose -- only the picker UI is being replaced.
pub struct Config {
    pub wallpaper_dir: PathBuf,
    pub thumb_dir: PathBuf,
    pub thumb_size: u32,
    pub sound: PathBuf,
}

impl Config {
    pub fn load() -> Self {
        let home = std::env::var("HOME").expect("HOME not set");
        let xdg_cache = std::env::var("XDG_CACHE_HOME").unwrap_or_else(|_| format!("{home}/.cache"));

        Config {
            wallpaper_dir: PathBuf::from(format!("{home}/Pictures/Wallpapers")),
            // Same directory the shell script used -- both tools can share
            // the cache interchangeably during the transition.
            thumb_dir: PathBuf::from(format!("{xdg_cache}/awww-walls-thumbs")),
            thumb_size: 350,
            sound: PathBuf::from(format!(
                "{home}/.local/share/sounds/modern-minimal-ui/stereo/dialog-information.oga"
            )),
        }
    }
}

pub const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp"];
