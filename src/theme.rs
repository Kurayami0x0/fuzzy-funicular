//! Tiny hand-rolled config parser for the picker's look and feel --
//! deliberately not pulling in `serde`/`toml` for a few dozen `key = value`
//! lines. Format:
//!
//!     [colors]
//!     background = "#1e1e2eeb"
//!
//! `[section]` lines are purely cosmetic (ignored) -- there's no nesting,
//! they're just there so the file reads clearly. One assignment per line,
//! `#` starts a comment, blank lines ignored. Unknown keys are ignored
//! (forward-compatible); a bad value on a known key prints a warning and
//! keeps that field at its default rather than aborting the whole file.

use std::fs;
use std::path::PathBuf;

/// `[r, g, b, a]`. `picker.rs` converts to the BGRA byte order the Argb8888
/// shm buffer wants at draw time -- the config file itself stays in the
/// order people actually think in ("#rrggbb" / "#rrggbbaa").
#[derive(Debug, Clone, Copy)]
pub struct Rgba(pub [u8; 4]);

fn parse_hex_color(s: &str) -> Option<Rgba> {
    let s = s.trim().trim_matches('"').trim_start_matches('#');
    let byte = |i: usize| u8::from_str_radix(s.get(i * 2..i * 2 + 2)?, 16).ok();
    match s.len() {
        6 => Some(Rgba([byte(0)?, byte(1)?, byte(2)?, 0xff])),
        8 => Some(Rgba([byte(0)?, byte(1)?, byte(2)?, byte(3)?])),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    // -- layout --
    pub window_width: i32,
    pub window_height: i32,
    pub icon_size: i32,
    pub spacing: i32,
    pub padding: i32,
    pub corner_radius: i32,
    /// Corner rounding for the panel itself, separate from the per-icon
    /// `corner_radius`. Keep this <= `padding` (and <= the search bar
    /// height) or the rounded corner can clip into icon/text content.
    pub window_corner_radius: i32,
    pub search_bar_height: i32,

    // -- colors --
    pub background: Rgba,
    pub search_bar_background: Rgba,
    pub search_text_color: Rgba,
    pub highlight_border: Rgba,
    pub highlight_border_width: i32,
    /// 0-255. Non-highlighted thumbnails are drawn at this alpha (out of
    /// 255) so the highlighted one pops without needing a heavier border.
    pub dim_opacity: u8,

    // -- animation --
    pub open_duration_ms: f64,
    pub close_duration_ms: f64,
    /// The panel scales up from this factor (of its full size) while
    /// opening, and back down to it while closing -- a subtle "pop"
    /// instead of a flat fade. 1.0 disables the scale effect entirely.
    pub open_close_scale_from: f64,
    /// Cubic-bezier control points (x1, y1, x2, y2) for the open/close
    /// easing curve -- same convention as CSS `cubic-bezier()`. Default is
    /// the standard "ease-in-out" curve.
    pub open_close_curve: (f64, f64, f64, f64),
    /// How fast the rendered scroll position chases the logical scroll
    /// target, in 1/sec -- higher is snappier, lower is floatier.
    pub scroll_smoothing: f64,
    /// Same idea, for the sliding highlight ring.
    pub highlight_smoothing: f64,
    /// Momentum decay after touchpad/wheel input stops.
    pub momentum_half_life_ms: f64,
    /// Pixels scrolled per discrete mouse-wheel "click". Touchpad
    /// scrolling uses its own reported delta directly and ignores this.
    pub wheel_step: i32,
}

impl Default for Theme {
    fn default() -> Self {
        // Same look as the rofi theme strings in the original bash script,
        // plus new defaults for the bits that only exist here.
        Theme {
            window_width: 1179,
            window_height: 370,
            icon_size: 350,
            spacing: 15,
            padding: 10,
            corner_radius: 12,
            window_corner_radius: 20,
            search_bar_height: 44,

            background: Rgba([0x1e, 0x1e, 0x2e, 0xeb]),          // Catppuccin Mocha base
            search_bar_background: Rgba([0x18, 0x18, 0x25, 0xeb]), // slightly darker than the panel
            search_text_color: Rgba([0xcd, 0xd6, 0xf4, 0xff]),   // Catppuccin Mocha text
            highlight_border: Rgba([0xf9, 0xe2, 0xaf, 0xff]),     // Catppuccin Mocha yellow
            highlight_border_width: 3,
            dim_opacity: 190,

            open_duration_ms: 200.0,
            close_duration_ms: 160.0,
            open_close_scale_from: 0.92,
            open_close_curve: (0.42, 0.0, 0.58, 1.0), // CSS "ease-in-out"
            scroll_smoothing: 14.0,
            highlight_smoothing: 20.0,
            momentum_half_life_ms: 120.0,
            wheel_step: 60,
        }
    }
}

impl Theme {
    /// Total rendered surface height, including the search bar sitting
    /// above the icon row. `window_height` alone stays the icon-row height
    /// for backward-compat with existing layout math.
    pub fn surface_height(&self) -> i32 {
        self.window_height + self.search_bar_height
    }

    /// Y offset where the icon row starts, below the search bar.
    pub fn icon_area_y0(&self) -> i32 {
        self.search_bar_height + self.padding
    }

    fn config_path() -> Option<PathBuf> {
        let base = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .or_else(|| std::env::var("HOME").ok().map(|h| format!("{h}/.config")))?;
        Some(PathBuf::from(base).join("awww-walls-sctk/theme.conf"))
    }

    /// Load `~/.config/awww-walls-sctk/theme.conf`, falling back to the
    /// built-in defaults for anything missing, unparsable, or if the file
    /// itself doesn't exist.
    pub fn load() -> Self {
        let mut theme = Theme::default();
        let Some(path) = Self::config_path() else {
            return theme;
        };
        let Ok(text) = fs::read_to_string(&path) else {
            return theme;
        };

        for (lineno, raw_line) in text.lines().enumerate() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                continue; // cosmetic section header, no real nesting
            }
            let Some((key, value)) = line.split_once('=') else {
                eprintln!("warning: {}:{}: expected `key = value`, skipping", path.display(), lineno + 1);
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            let where_ = format!("{}:{}", path.display(), lineno + 1);

            let int_field = |v: &str| -> Option<i32> {
                v.parse().ok().or_else(|| {
                    eprintln!("warning: {where_}: '{v}' is not a valid integer for {key}");
                    None
                })
            };
            let float_field = |v: &str| -> Option<f64> {
                v.parse().ok().or_else(|| {
                    eprintln!("warning: {where_}: '{v}' is not a valid number for {key}");
                    None
                })
            };
            let byte_field = |v: &str| -> Option<u8> {
                v.parse::<i32>().ok().map(|n| n.clamp(0, 255) as u8).or_else(|| {
                    eprintln!("warning: {where_}: '{v}' is not a valid 0-255 value for {key}");
                    None
                })
            };
            let color_field = |v: &str| -> Option<Rgba> {
                parse_hex_color(v).or_else(|| {
                    eprintln!(
                        "warning: {where_}: '{v}' is not a valid color for {key} (expected \"#rrggbb\" or \"#rrggbbaa\")"
                    );
                    None
                })
            };
            let curve_field = |v: &str| -> Option<(f64, f64, f64, f64)> {
                let parts: Vec<&str> = v.split(',').map(|s| s.trim()).collect();
                if parts.len() != 4 {
                    eprintln!("warning: {where_}: '{v}' is not 4 comma-separated numbers for {key}");
                    return None;
                }
                let nums: Option<Vec<f64>> = parts.iter().map(|p| p.parse::<f64>().ok()).collect();
                match nums {
                    Some(n) => Some((n[0], n[1], n[2], n[3])),
                    None => {
                        eprintln!("warning: {where_}: '{v}' contains a non-numeric value for {key}");
                        None
                    }
                }
            };

            match key {
                "window_width" => { if let Some(v) = int_field(value) { theme.window_width = v.max(1); } }
                "window_height" => { if let Some(v) = int_field(value) { theme.window_height = v.max(1); } }
                "icon_size" => { if let Some(v) = int_field(value) { theme.icon_size = v.max(1); } }
                "spacing" => { if let Some(v) = int_field(value) { theme.spacing = v.max(0); } }
                "padding" => { if let Some(v) = int_field(value) { theme.padding = v.max(0); } }
                "corner_radius" => { if let Some(v) = int_field(value) { theme.corner_radius = v.max(0); } }
                "window_corner_radius" => { if let Some(v) = int_field(value) { theme.window_corner_radius = v.max(0); } }
                "search_bar_height" => { if let Some(v) = int_field(value) { theme.search_bar_height = v.max(0); } }
                "highlight_border_width" => { if let Some(v) = int_field(value) { theme.highlight_border_width = v.max(0); } }
                "dim_opacity" => { if let Some(v) = byte_field(value) { theme.dim_opacity = v; } }
                "open_duration_ms" => { if let Some(v) = float_field(value) { theme.open_duration_ms = v.max(0.0); } }
                "close_duration_ms" => { if let Some(v) = float_field(value) { theme.close_duration_ms = v.max(0.0); } }
                "open_close_scale_from" => { if let Some(v) = float_field(value) { theme.open_close_scale_from = v.clamp(0.1, 1.0); } }
                "open_close_curve" => { if let Some(v) = curve_field(value) { theme.open_close_curve = v; } }
                "scroll_smoothing" => { if let Some(v) = float_field(value) { theme.scroll_smoothing = v.max(0.1); } }
                "highlight_smoothing" => { if let Some(v) = float_field(value) { theme.highlight_smoothing = v.max(0.1); } }
                "momentum_half_life_ms" => { if let Some(v) = float_field(value) { theme.momentum_half_life_ms = v.max(1.0); } }
                "wheel_step" => { if let Some(v) = int_field(value) { theme.wheel_step = v; } }
                "background" => { if let Some(v) = color_field(value) { theme.background = v; } }
                "search_bar_background" => { if let Some(v) = color_field(value) { theme.search_bar_background = v; } }
                "search_text_color" => { if let Some(v) = color_field(value) { theme.search_text_color = v; } }
                "highlight_border" => { if let Some(v) = color_field(value) { theme.highlight_border = v; } }
                other => eprintln!("warning: {where_}: unknown key '{other}', ignoring"),
            }
        }

        // Corner radius can't exceed half the icon -- clamp defensively so
        // a too-large value doesn't produce an inverted/garbage mask.
        theme.corner_radius = theme.corner_radius.min(theme.icon_size / 2);
        theme.window_corner_radius = theme.window_corner_radius.min(theme.window_width / 2).min(theme.surface_height() / 2);
        theme
    }
}
