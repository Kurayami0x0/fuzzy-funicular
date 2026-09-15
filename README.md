# ⚠️ AI-Generated Project Disclaimer

> **This project was entirely generated with AI assistance.** It is an experimental proof-of-concept and is not intended for production use. The author does not plan to maintain or update it regularly. Use at your own risk. If you encounter issues, contributions are welcome, but don't expect official support.

---

# awww-walls-sctk

A native Wayland wallpaper picker UI for [`awww`](https://github.com/awww-cli/awww), replacing the original `rofi` + ImageMagick frontend with a smooth, animated layer-shell surface powered by `smithay-client-toolkit` and in-process `image-rs` thumbnailing.

## ✨ Features

- **Native Wayland UI** – Layer-shell overlay surface (`Layer::Overlay`) with smooth animations
- **Smooth Animations** – Cubic-bezier scale+fade "pop" on open/close (configurable)
- **Fluid Scrolling** – Momentum-based scrolling with exponential smoothing for mouse wheel, touchpad, and keyboard navigation
- **Sliding Highlight Ring** – Animated selection indicator that glides between thumbnails
- **Search Bar** – Real-time filename filtering with case-insensitive substring matching
- **Parallel Thumbnail Generation** – In-process `image` crate with `rayon` for fast, parallel thumbnail caching
- **Theming Support** – Customizable via `~/.config/awww-walls-sctk/theme.conf`
- **Shared Cache** – Uses the same thumbnail cache as the original bash script, so both tools can coexist

## 📦 Requirements

- A Wayland compositor with `wlr-layer-shell` support (e.g., **niri**, Sway, Hyprland)
- `awww` CLI installed and configured
- `libxkbcommon-dev` (Debian/Ubuntu) or `libxkbcommon` (Arch)
- Rust toolchain

## 🛠️ Build

```sh
cargo build --release
install -Dm755 target/release/awww-walls-sctk ~/.local/bin/awww-walls-sctk
```

> **Important:** Always use `--release`. Debug builds may stutter during scrolling due to unoptimized per-pixel software rendering into `wl_shm` buffers.

## 🎮 Controls

| Input | Action |
|-------|--------|
| **Type** | Filter wallpapers by filename (case-insensitive) |
| **Backspace** | Delete last character in search |
| **Escape** | Clear search (first press), cancel picker (second press) |
| **Click thumbnail** | Apply selected wallpaper |
| **← / → arrows** | Move selection cursor |
| **Enter** | Apply highlighted wallpaper |
| **Mouse wheel / Touchpad scroll** | Pan through thumbnails |
| **Close window** | Cancel without changes |

### Navigation Details

- Keyboard arrow keys auto-scroll to keep selection visible
- Holding an arrow key repeats after an initial delay (400ms, then every 40ms)
- Scrolling has momentum that decays over time (configurable half-life)
- Mouse hover re-syncs the keyboard selection cursor
- All motion eases smoothly—nothing snaps

## 🎨 Theming

Create `~/.config/awww-walls-sctk/theme.conf` to customize appearance and behavior:

```ini
# ~/.config/awww-walls-sctk/theme.conf

[layout]
window_width      = 1179
window_height     = 370   # icon-row height; search bar adds on top
icon_size         = 350
spacing           = 15
padding           = 10
corner_radius     = 12    # per-icon corner rounding
window_corner_radius = 20 # whole-panel corner rounding
search_bar_height = 44

[colors]
background             = "#1e1e2eeb"  # RGBA hex, icon-area background
search_bar_background   = "#181825eb" # RGBA hex, search bar's own background
search_text_color       = "#cdd6f4"    # search text color
highlight_border        = "#f9e2af"    # RGB or RGBA hex, selection ring
highlight_border_width  = 3            # px
dim_opacity              = 190         # 0-255, alpha of non-selected thumbnails

[animation]
open_duration_ms      = 200   # open animation length
close_duration_ms     = 160   # close animation length
open_close_scale_from = 0.92  # panel scales from this factor (1.0 = no scale, fade only)
open_close_curve       = 0.42, 0.0, 0.58, 1.0  # cubic-bezier(x1,y1,x2,y2)
scroll_smoothing        = 14  # 1/sec, how fast scroll position chases input
highlight_smoothing     = 20  # 1/sec, how fast the ring slides between cells
momentum_half_life_ms   = 120 # touchpad/wheel coast decay
wheel_step               = 60 # px scrolled per mouse-wheel notch
```

### Cubic-Bezier Presets

The `open_close_curve` accepts CSS-style `cubic-bezier(x1, y1, x2, y2)` values:

| Name | Curve |
|------|-------|
| ease-in-out (default) | `0.42, 0.0, 0.58, 1.0` |
| ease | `0.25, 0.1, 0.25, 1.0` |
| ease-in | `0.42, 0.0, 1.0, 1.0` |
| ease-out | `0.0, 0.0, 0.58, 1.0` |
| linear | `0.0, 0.0, 1.0, 1.0` |

Missing file or invalid keys fall back to built-in defaults (Catppuccin Mocha colors). Invalid lines warn to stderr but won't prevent startup.

## 🔧 Configuration

Default paths (mirroring the original bash script):

| Setting | Default |
|---------|---------|
| Wallpaper directory | `$HOME/Pictures/Wallpapers` |
| Thumbnail cache | `$XDG_CACHE_HOME/awww-walls-thumbs` (or `~/.cache/awww-walls-thumbs`) |
| Thumbnail size | 350×350 pixels |
| Sound effect | `$HOME/.local/share/sounds/modern-minimal-ui/stereo/dialog-information.oga` |

Supported image formats: **JPEG, PNG, GIF, BMP, WebP**

## 🔄 Backend Compatibility

This tool **only replaces the picker UI**. The backend remains unchanged:

- Shells out to `awww img` with the same flags as the original script
- Expects `mountain.jpg` / `mountain-b.jpg` naming for default/blurred pairs
- Applies to both `default` and `overview` namespaces
- Shares thumbnail cache with the original `awww-walls-rofi.sh` script

## 🆚 Comparison with Original Bash Script

| Feature | Bash Script (rofi) | This (sctk) |
|---------|-------------------|-------------|
| Picker UI | rofi (dmenu, icon mode) | Native layer-shell surface |
| Thumbnails | `magick` subprocess | `image` crate in-process + `rayon` |
| Cache location | `~/.cache/awww-walls-thumbs/<filename>` | Identical |
| Apply command | `awww img ...` | Identical |
| Filtering | rofi's fuzzy matching | Built-in search bar |
| Theming | rofi `.rasi` file | `theme.conf` |
| Selection indicator | rofi listview styling | Sliding highlight ring + dimming |
| Open/close | Instant | Cubic-bezier scale+fade animation |

## 🏗️ Architecture Notes

### Render Pacing

All drawing goes through `Picker::draw()`, which:
1. Attaches a `wl_shm` buffer
2. Requests a frame callback
3. Commits the surface

The `frame_pending` flag ensures at most one buffer is in flight at a time. Input handlers call `kick()`, which draws immediately only if nothing is pending—otherwise changes wait for the next frame callback.

### Motion Model

Nothing snaps. Both scroll position and the highlight ring use `Eased` values:
- **Target**: Set instantly by input
- **Value**: Chases target via exponential smoothing each frame

Momentum works by continuing to nudge the *target* after input stops, with decay controlled by `momentum_half_life_ms`.

### No GPU Path

Rendering is CPU/software into `wl_shm` buffers—no EGL/Vulkan. For a picker that opens briefly, this is plenty fast in release mode.

### Version Sensitivity

`smithay-client-toolkit` trait shapes may shift across patch releases. If you get trait mismatch errors:
- Match the expected signature from the compiler error, or
- Pin the version: `smithay-client-toolkit = "=0.19.x"`

## 📁 Project Structure

```
src/
├── main.rs        # Entry point, dependency checks, orchestration
├── config.rs      # Configuration paths and constants
├── font.rs        # Built-in 5x7 bitmap font for search bar
├── images.rs      # Wallpaper listing and path utilities
├── picker.rs      # Layer-shell picker UI implementation
├── theme.rs       # Theme config parser and defaults
├── thumbnails.rs  # Thumbnail generation and caching
└── wallpaper.rs   # awww CLI invocation
```

## 📝 License

This project is provided as-is without warranty. Feel free to fork, modify, or experiment with it.
