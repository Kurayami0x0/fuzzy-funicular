# awww-walls-sctk

A native picker UI for `awww`, replacing `awww-walls-rofi.sh`'s
rofi + ImageMagick front end with a `smithay-client-toolkit` layer-shell
surface and in-process `image-rs` thumbnailing.

**Scope, on purpose:** only the picker UI changes. The backend is
untouched:

- Still shells out to the `awww` CLI with the same flags
  (`awww img -t center --transition-fps 60 ...` for the default namespace,
  `awww img --transition-fps 60 -n overview ...` for the blurred one).
- Still expects `mountain.jpg` / `mountain-b.jpg` naming for the
  default/blurred pair.
- Still reads from `$HOME/Pictures/Wallpapers` and caches thumbnails in
  `$XDG_CACHE_HOME/awww-walls-thumbs` (or `~/.cache/awww-walls-thumbs`) —
  same directory and filenames the bash script used, so the cache is
  shared and both tools can run side by side during the transition.

## Controls

- Type to search/filter by filename. Backspace deletes a character;
  Escape clears the search first, and only cancels the picker on a second
  press with an empty query.
- Click a thumbnail to apply it.
- Left/Right arrow keys move a keyboard selection cursor, shown by the
  same sliding highlight ring mouse hover uses; Enter applies whatever's
  currently highlighted. Moving the mouse re-syncs the keyboard cursor to
  whatever you're hovering. Holding an arrow key repeats after an initial
  delay, same as held keys anywhere else in the desktop.
- Mouse wheel / touchpad two-finger scroll pans the ribbon when there are
  more wallpapers than fit in the window. Scrolling coasts briefly after
  you stop instead of snapping dead. Arrow-key navigation auto-scrolls to
  keep the selection in view, with the same easing as everything else —
  no teleporting.
- Escape (with an empty search) or closing the surface cancels.
- Opening and picking/cancelling both animate — a cubic-bezier
  scale+fade "pop", not a flat opacity fade — and the picker doesn't
  return a result until the closing animation finishes.

## Theming

Drop a `~/.config/awww-walls-sctk/theme.conf` file to override the
defaults. Plain `key = value` lines; `[section]` lines are purely
cosmetic grouping (ignored, not real nesting); `#` for comments. Missing
file or missing keys fall back to the built-in look (Catppuccin Mocha,
same base sizing as the original rofi theme) on a per-field basis. A bad
line just warns to stderr and keeps that one field's default -- it won't
refuse to start over a typo.

```ini
# ~/.config/awww-walls-sctk/theme.conf

[layout]
window_width      = 1179
window_height     = 370   # icon-row height; the search bar adds on top of this
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
open_close_scale_from = 0.92  # panel scales up/down from this factor (1.0 = no scale, fade only)
open_close_curve       = 0.42, 0.0, 0.58, 1.0  # cubic-bezier(x1,y1,x2,y2), CSS convention -- default is "ease-in-out"
scroll_smoothing        = 14  # 1/sec, how fast scroll position chases input (higher = snappier)
highlight_smoothing     = 20  # 1/sec, how fast the ring slides between cells
momentum_half_life_ms   = 120 # touchpad/wheel coast decay
wheel_step               = 60 # px scrolled per mouse-wheel notch
```

`open_close_curve` accepts any four comma-separated numbers as CSS-style
`cubic-bezier(x1, y1, x2, y2)` control points. Some familiar presets, if
you want to swap the default "ease-in-out" for something else:

| name | curve |
|---|---|
| ease-in-out (default) | `0.42, 0.0, 0.58, 1.0` |
| ease | `0.25, 0.1, 0.25, 1.0` |
| ease-in | `0.42, 0.0, 1.0, 1.0` |
| ease-out | `0.0, 0.0, 0.58, 1.0` |
| linear | `0.0, 0.0, 1.0, 1.0` |

No file? No `--theme` flag, no rebuild required — it's read fresh every
time you launch the picker.

## What changed from the bash script

| | bash script | this |
|---|---|---|
| Picker UI | rofi (dmenu, icon mode) | native layer-shell surface (`Layer::Overlay`) |
| Thumbnail generation | `magick` subprocess per image, manually parallelized with a job cap | `image` crate in-process, parallelized with `rayon` |
| Cache format/location | `~/.cache/awww-walls-thumbs/<filename>`, cover-cropped to 350x350 | identical |
| Wallpaper apply | `awww img ...` | identical |
| Filtering | rofi's own fuzzy matching | a built-in search bar (see below) |
| Theming | rofi `.rasi` theme file | `theme.conf` (see above) |
| Selection indicator | rofi's own listview selection styling | a single sliding ring, eased between cells, plus dimming non-selected thumbnails |
| Open/close | instant | cubic-bezier scale+fade animation |

### Search bar

A search bar sits above the thumbnail ribbon. Typing filters wallpapers
whose filename contains the query (case-insensitive substring match);
the ribbon, scroll bounds, and keyboard/mouse selection all operate on
the filtered list, so navigation only ever sees what's currently visible.
Text is rendered with a small built-in 5x7 bitmap font (`src/font.rs`) —
deliberately not a font-rasterization dependency hunting for a system
font file at runtime, since the character set a filename search needs is
small and fixed (letters, digits, and `- _ .`). Anything outside that set
is simply skipped when drawing, rather than erroring.

## Build

Requires a compositor with `wlr-layer-shell` (niri has it) and the
`xkbcommon` dev headers (`libxkbcommon-dev` on Debian/Ubuntu,
`libxkbcommon` on Arch — you'll already have this from your niri setup).

```sh
cargo build --release
install -Dm755 target/release/awww-walls-sctk ~/.local/bin/awww-walls-sctk
```

**Use `--release`.** The rendering is a straightforward per-pixel
software blit into a `wl_shm` buffer; in a debug build that loop is slow
enough to visibly stutter/tear during scrolling, and the animation
timing math assumes roughly frame-rate-paced ticks, so a slow debug build
will also throw off how the easing/momentum feels.

Then swap it in wherever `awww-walls-rofi.sh` was bound (keybind,
`niri.kdl` spawn-at-startup entry, etc).

## Architecture notes

### Render pacing (why the early versions flickered/tore)

Every redraw goes through `Picker::draw()`, which is the *only* place
that attaches a buffer, requests the next frame callback, and commits.
`frame_pending` guarantees at most one buffer is ever in flight at a
time. Input handlers never draw directly — they update state and call
`kick()`, which draws immediately only if nothing's already outstanding;
otherwise the change waits for the next frame callback, where `advance()`
picks it up.

The scene (search bar, icons, highlight ring) is rendered once per frame
into a full-size `content` buffer at natural scale/alpha in
`render_content()`; `draw()` then composites that into the actual `wl_shm`
buffer, applying the open/close scale+alpha transform as a single pass
(nearest-neighbor resample about the panel center) only when it's not at
rest (scale ≈ 1.0 and alpha ≈ 1.0 skips the pass entirely and just
copies — no needless per-pixel work once idle or fully open).

### Motion model

Nothing snaps. Scroll position and the highlight ring position are both
`Eased` values (`src/picker.rs`): a logical `target` set instantly by
input, and a rendered `value` that chases the target every frame via
exponential smoothing (`scroll_smoothing`/`highlight_smoothing` in the
theme control the chase rate). Touchpad/wheel momentum works by
continuing to nudge the scroll *target* after input stops, with the
nudge amount decaying over `momentum_half_life_ms` — the rendered
position then just naturally trails the moving target. Keyboard-driven
auto-scroll (`scroll_into_view`) goes through the exact same
target/easing mechanism, which is what makes arrow-key navigation glide
instead of jump.

Opening and closing run a proper CSS-style cubic-bezier ease
(`cubic_bezier()`, solved via a few Newton's-method iterations the same
way browsers evaluate `cubic-bezier()`) over two things at once: the
panel's alpha, and a scale-from-center factor (`open_close_scale_from`
to `1.0`) for a "pop" rather than a flat fade. Selecting or cancelling
doesn't resolve `pick()`'s return value immediately — it starts the
closing animation (`begin_close`) and only sets the real result once
that finishes, so the caller never sees a value until the surface has
actually finished animating out. Input is ignored once closing has
begun.

### Panel rounding

The whole panel (search bar + icon area) is corner-rounded to
`window_corner_radius`, independently of the per-icon `corner_radius` —
rounded by making those corner pixels transparent in `render_content()`,
the same `corner_mask()` technique the icons and highlight ring already
use, just applied to the full canvas bounds instead of a single icon's.

### Version sensitivity

`smithay-client-toolkit`'s `CompositorHandler`/`KeyboardHandler` trait
shapes have shifted slightly across 0.19.x patch releases (an extra
`surface_enter`/`surface_leave` pair, an extra `layout: u32` param on
`update_modifiers`). If `cargo build` complains about a mismatched trait
impl, the compiler error tells you exactly what's expected — match it, or
pin `smithay-client-toolkit = "=0.19.x"` to the version you first got it
building against so a later `cargo update` doesn't reopen this.

### No GPU path

Rendering is CPU/software into `wl_shm` buffers — no EGL/Vulkan. For a
picker that only opens briefly this is plenty fast in release mode. If
you want it GPU-backed later, that's an isolated swap of `Picker::draw()`
and the blit/text functions; the public shape (`pick(cfg, images) ->
Option<String>`) wouldn't need to change.

- `check_deps()` in `main.rs` only checks for `awww` now — `rofi`,
  `magick`, and `find`/`nproc` are gone from the dependency list.
