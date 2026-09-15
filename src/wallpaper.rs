use crate::config::Config;
use crate::images::full_path;
use std::process::Command;

/// Apply `image_file` to an awww namespace. Pass `None` for the default
/// (unnamed) namespace. This is a direct port of `apply_wallpaper()` from the
/// bash script -- the awww daemons themselves are untouched, this just shells
/// out to the same CLI with the same flags.
pub fn apply(cfg: &Config, namespace: Option<&str>, image_file: &str) -> Result<(), String> {
    let full = full_path(cfg, image_file);
    if !full.is_file() {
        return Err(format!("image not found: {}", full.display()));
    }

    let mut cmd = Command::new("awww");
    cmd.arg("img");
    match namespace {
        None => {
            cmd.args(["-t", "center", "--transition-fps", "60"]);
        }
        Some(ns) => {
            cmd.args(["--transition-fps", "60", "-n", ns]);
        }
    }
    cmd.arg(&full);

    let status = cmd
        .status()
        .map_err(|e| format!("failed to run awww: {e}"))?;
    if !status.success() {
        return Err(format!("awww exited with {status}"));
    }
    Ok(())
}
