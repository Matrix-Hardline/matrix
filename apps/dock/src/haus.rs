//! R216 — the dock in the HOUSE (matrixkit-shell + matrixkit-scene, no
//! iced). Started with `matrix-dock --haus` (the iced dock stays the
//! default until the owner switched after an on-device test).
//!
//! One model, two ways to stand at the edge:
//!
//! * **Panes** (no panel pinned — laptop, Surface, PC): every tile is its
//!   own `matrix-chip` layer surface of the house fleet, so the compositor
//!   lays real glass under each one (R174). The main surface is an
//!   invisible, input-transparent strip that only carries the exclusive
//!   zone (and the catch edge while folded away).
//! * **Strip** (a panel is pinned with `--output NAME` — always on a duo,
//!   where the dock belongs on `dock-output`/DSI-1): the house fleet
//!   cannot follow an output yet (its panes open on whatever panel has
//!   focus), so the shelf is ONE surface on the pinned panel and the
//!   tiles wear the painted chip body (the `tile_chrome` fallback R175
//!   keeps for surfaces without compositor glass). A bonus of the one
//!   surface: the house delivers long-press and right-click there.
//!
//! `--panes` / `--strip` force a form (for the probe once the fleet can
//! follow an output).
//!
//! What lives here: the tile row (pins first, then running apps; «Apps»
//! leads on a duo), tap = focus / cycle windows / start a resting pin /
//! put the front app on the shelf (duo), long-press and right-click =
//! the external tile menu (`matrix-context dock …`), the tap answer
//! (haptic tick + lit edge, a calm pulse while an app starts), the
//! running dot and window lines (desktop), the shelf widgets as chips,
//! the OSD morph (Dynamic Dock), cinema mode with its subtitle strip,
//! auto-hide with its catch edge, the exclusive zone (`--reserve`), and
//! live reload of pins, widgets and palette. Window changes flow from the
//! compositor's event stream on a thread (Loom), never on the loop.

use matrixkit_image::ImageAtlas;
use matrixkit_scene::scene as fam;
use matrixkit_scene::symbol;
use matrixkit_shell::shapes::{Cmd, Shape, Style};
use matrixkit_shell::{self as shell, Loom, Refresh, Scene, SceneBuilder, Source};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{Keep, Rect};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};

use crate::{maker_of, Widget};

/// The side air the shelf keeps to the screen edges (the bar's 10 px).
const SIDE: i32 = 10;
/// The gap between two shelf elements (iced: mk::spacing::XS).
const GAP: i32 = mk::spacing::XS as i32;
/// The shelf floats this far above the screen edge.
const LIFT: i32 = 10;
/// Height of one widget element (iced: ELEMENT_HEIGHT).
const ELEMENT_H: u32 = 52;
/// Gap between the two widget rows of the iced shelf (sets the
/// desktop tile height: two rows plus their gap).
const ROW_GAP: u32 = mk::spacing::XS as u32;
/// Headroom on top of a measured text width (mkw::TEXT_SAFETY).
const TEXT_SAFETY: f32 = 10.0;
/// The running dot under a desktop tile.
const DOT: f32 = 4.0;
/// How long the shelf keeps an OSD chip after a key press.
const OSD_DWELL: Duration = Duration::from_millis(1500);
const OSD_W: u32 = 280;
/// Folded (auto-hide), the dock is an invisible catch strip this tall.
const CATCH: u32 = 16;
/// Auto-hide folds the dock after this many still seconds.
const IDLE_HIDE_S: f32 = 3.0;
/// A press that wanders this far is a swipe, not a tap.
const SLOP: f64 = 12.0;
/// A starting pin pulses at most this long while its window is on the way.
const BOUNCE_MAX: Duration = Duration::from_secs(8);

/// The tile height of this device: snug on a duo, level with two widget
/// rows elsewhere (R172k).
fn tile_height() -> u32 {
    mk::device::shelf().tile_height.unwrap_or(2 * ELEMENT_H + ROW_GAP)
}

// ------------------------------------------------------------ the model

/// One open window as the shelf needs it.
#[derive(Clone, Debug, PartialEq)]
pub struct Win {
    id: u64,
    app_id: String,
    title: String,
    focus: bool,
    /// Round 169: lies on the shelf (minimized).
    shelved: bool,
}

/// One shelf tile: a pin and/or a running app.
#[derive(Clone, Debug, PartialEq)]
struct ShelfTile {
    app_id: String,
    windows: Vec<Win>,
    pinned: bool,
    focus: bool,
}

impl ShelfTile {
    fn running(&self) -> bool {
        !self.windows.is_empty()
    }

    /// Tap target: the focused app cycles, else its oldest window.
    fn target(&self) -> Option<u64> {
        if self.focus && self.windows.len() > 1 {
            let idx = self.windows.iter().position(|w| w.focus)?;
            return Some(self.windows[(idx + 1) % self.windows.len()].id);
        }
        self.windows.first().map(|w| w.id)
    }
}

/// Pins first (in pin order), then running apps by seniority — the
/// iced `tiles_read`, as a pure function of pins and windows.
fn shelf_tiles(pins: &[String], windows: &[Win]) -> Vec<ShelfTile> {
    let mut by_app: HashMap<String, Vec<Win>> = HashMap::new();
    for w in windows {
        by_app.entry(w.app_id.clone()).or_default().push(w.clone());
    }
    let mut tiles = Vec::new();
    for pin in pins {
        let mut windows = by_app.remove(pin).unwrap_or_default();
        windows.sort_by_key(|w| w.id);
        let focus = windows.iter().any(|w| w.focus);
        tiles.push(ShelfTile { app_id: pin.clone(), windows, pinned: true, focus });
    }
    let mut running: Vec<ShelfTile> = by_app
        .into_iter()
        .map(|(app_id, mut windows)| {
            windows.sort_by_key(|w| w.id);
            let focus = windows.iter().any(|w| w.focus);
            ShelfTile { app_id, windows, pinned: false, focus }
        })
        .collect();
    running.sort_by_key(|t| t.windows[0].id);
    tiles.extend(running);
    tiles
}

/// The caption lines of a tile (R174j): one per open window, arrow
/// first; a title that only repeats the app's name says nothing.
fn window_lines(t: &ShelfTile, label: &str) -> Vec<String> {
    let mut lines: Vec<String> = t
        .windows
        .iter()
        .filter(|w| {
            let title = w.title.trim();
            !title.is_empty() && title != label && title != format!("Matrix {label}")
        })
        .take(3)
        .map(|w| format!("→ {}", w.title.chars().take(18).collect::<String>()))
        .collect();
    if t.windows.len() > 3 {
        lines.push(format!("→ … {} weitere", t.windows.len() - 3));
    }
    lines
}

/// All open windows from the compositor; None = no answer (keep the old).
fn windows_read() -> Option<Vec<Win>> {
    let out = mk::canvas::raw(&["msg", "-j", "windows"])?;
    if !out.status.success() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    // Round 169: which windows lie on the shelf (the tile brings them first).
    let shelf = mk::canvas::raw(&["msg", "-j", "workspaces"])
        .and_then(|o| serde_json::from_slice::<serde_json::Value>(&o.stdout).ok())
        .and_then(|ws| mk::window::shelf_workspace(&ws));
    Some(
        v.as_array()?
            .iter()
            .map(|w| Win {
                id: w["id"].as_u64().unwrap_or(0),
                app_id: w["app_id"].as_str().unwrap_or("?").to_string(),
                title: w["title"].as_str().unwrap_or("").to_string(),
                focus: w["is_focused"].as_bool().unwrap_or(false),
                shelved: shelf.is_some() && w["workspace_id"].as_u64() == shelf,
            })
            .collect(),
    )
}

/// The logical width of output `name` (or the first one).
fn output_width(name: Option<&str>) -> f32 {
    let fallback = 1920.0;
    let Some(out) = mk::canvas::raw(&["msg", "-j", "outputs"]) else { return fallback };
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else { return fallback };
    let list: Vec<(String, serde_json::Value)> = match v {
        serde_json::Value::Object(m) => m.into_iter().collect(),
        serde_json::Value::Array(a) => a
            .into_iter()
            .map(|o| (o["name"].as_str().unwrap_or("").to_string(), o))
            .collect(),
        _ => return fallback,
    };
    list.iter()
        .find(|(n, _)| name.is_none_or(|want| want == n))
        .and_then(|(_, o)| o.get("logical")?.get("width")?.as_f64())
        .map(|w| w as f32)
        .unwrap_or(fallback)
}

/// Start an app by name, the user's ~/.local/bin first (mkw::panel::app_start
/// without the iced crate).
fn app_start(name: &str, args: &[&str]) {
    mk::process::app_start(name, args);
}

/// A shell line in the background (mk::process::sh_spawn, round 27).
fn sh_spawn(line: &str) {
    mk::process::sh_spawn(line);
}

/// R169b: an open matrix-home keeps the folded dock awake.
fn home_open() -> bool {
    let Ok(dir) = std::fs::read_dir("/proc") else { return false };
    dir.flatten().any(|e| {
        std::fs::read_to_string(e.path().join("comm")).is_ok_and(|c| c.trim() == "matrix-home")
    })
}

/// Start this very process afresh with `extra` arguments added (exec keeps
/// the pid, so the systemd unit / niri spawn never notices). Only returns
/// when exec failed.
fn restart_with(extra: &[&str]) {
    use std::os::unix::process::CommandExt as _;
    if let Ok(exe) = mk::process::own_exe_result() {
        let mut args: Vec<String> = std::env::args().skip(1).collect();
        args.extend(extra.iter().map(|s| s.to_string()));
        let refused = std::process::Command::new(exe).args(&args).exec();
        eprintln!("[dock] Neustart misslungen: {refused}");
    }
}

/// The OSD glyph and whether it is «full» (mkw::osd_display::glyph).
fn osd_glyph(state: mk::osd::State) -> (char, bool) {
    fam::osd_glyph(state)
}

// ------------------------------------------------------------ the cards

/// What a press on a card means.
#[derive(Clone, Debug, PartialEq)]
pub enum Hit {
    /// The «Apps» tile / chip: Matrix Start.
    Apps,
    /// An app tile (by app_id — stable across list changes).
    App(String),
    Widget(Widget),
    /// Round 113 (wiggle mode): «+» adds an app, «Fertig» ends the mode.
    EditAdd,
    EditDone,
    /// Shows only (OSD, subtitle).
    None,
}

/// What a card shows.
#[derive(Clone, Debug, PartialEq)]
pub enum Face {
    Tile {
        name: String,
        maker: String,
        lines: Vec<String>,
        /// The widest of the lines (centres the text block).
        widest: f32,
        /// The running dot: None = no dot row, Some(None) = row without
        /// a dot (resting), Some(Some(focus)) = a dot.
        dot: Option<Option<bool>>,
    },
    Widget {
        glyph: Option<char>,
        label: Option<String>,
        dim: bool,
        content_w: f32,
    },
    Osd(mk::osd::State),
    /// Round 57 (owner: «just the icon»): the app's own Matrix icon
    /// (matrixkit-icons, rendered once to a file) instead of the two
    /// lines of text — dock style «icons».
    Icon {
        /// The rendered icon — or, for an app Matrix has no icon for, the
        /// bare Matrix tile (same squircle) with `mark` laid on it.
        path: std::path::PathBuf,
        /// Empty for a real icon; a symbol or the first letter otherwise.
        mark: String,
        symbol: bool,
        dot: Option<Option<bool>>,
        /// Round 114: the app runs (Some(focus)) — the glow, independent of
        /// the dot row (the duo has none: no dot, so no glow came).
        lit: Option<bool>,
    },
    Subtitle {
        px: f32,
        lines: Vec<String>,
    },
}

/// One element of the shelf: a fleet pane in the panes form, a laid
/// region of the one surface in the strip form.
#[derive(Clone, Debug, PartialEq)]
pub struct Card {
    hit: Hit,
    face: Face,
    /// Offset from the left screen edge (fleet main axis).
    x: i32,
    /// Height above the screen edge (fleet cross axis).
    lift: i32,
    w: u32,
    h: u32,
    /// Tap answer 0..1 (lit edge), quantised so a fading glow does not
    /// mint a new ring shape per frame.
    glow: f32,
    pressed: bool,
    /// Round 113 (wiggle mode): the badge — Some(true) «–» removes the pin,
    /// Some(false) «+» pins a running app.
    edit: Option<bool>,
}

impl Card {
    fn namespace(&self) -> &'static str {
        match self.face {
            // The subtitle strip wears the card glass (mkw::glass_strip).
            Face::Subtitle { .. } => "matrix-card",
            _ => "matrix-chip",
        }
    }

    fn kind(&self) -> u8 {
        match self.face {
            Face::Tile { .. } => 1,
            Face::Widget { .. } => 2,
            Face::Osd(_) => 3,
            Face::Subtitle { .. } => 4,
            Face::Icon { .. } => 5,
        }
    }
}

/// The rounded-rectangle outline of a w × h box, `inset` from its edge.
fn rounded_outline(w: f32, h: f32, r: f32, inset: f32) -> Vec<Cmd> {
    let (x0, y0, x1, y1) = (inset, inset, w - inset, h - inset);
    let r = (r - inset).clamp(0.0, ((x1 - x0) / 2.0).min((y1 - y0) / 2.0));
    vec![
        Cmd::Move(x0 + r, y0),
        Cmd::Line(x1 - r, y0),
        Cmd::Quad(x1, y0, x1, y0 + r),
        Cmd::Line(x1, y1 - r),
        Cmd::Quad(x1, y1, x1 - r, y1),
        Cmd::Line(x0 + r, y1),
        Cmd::Quad(x0, y1, x0, y1 - r),
        Cmd::Line(x0, y0 + r),
        Cmd::Quad(x0, y0, x0 + r, y0),
        Cmd::Close,
    ]
}

/// A ring (stroke) around a box — the lit edge of the tap answer and the
/// hairline of the painted chip body.
fn lay_ring(b: &mut SceneBuilder, x: f32, y: f32, w: f32, h: f32, tint: mk::Rgba, width: f32, role: Option<usize>) {
    let shape = Shape::new(w, h).op(rounded_outline(w, h, mk::radius::LARGE, width / 2.0), Style::Stroke { tint, width });
    let clear = mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    let anchor = b.quad(x, y, w, h, clear, 0.0, role, None);
    let home = Rect { x, y, w, h };
    b.pict(fam::pict_of(&shape, x, y, home, anchor));
}

/// Lay one card at (x, y). `painted`: the card carries its own chip body
/// (strip form); without, the compositor glass is the body (panes form).
fn lay_card(b: &mut SceneBuilder, engine: &mut TextEngine, card: &Card, x: f32, y: f32, painted: bool, role: Option<usize>, p: mk::Palette) {
    let (w, h) = (card.w as f32, card.h as f32);
    if painted {
        b.quad(x, y, w, h, p.surface_container.with_alpha(0.62), mk::radius::LARGE, role, None);
        lay_ring(b, x, y, w, h, p.on_surface.with_alpha(0.14), 1.0, role);
    } else {
        // An invisible body anyway: the whole card is the press target.
        b.quad(x, y, w, h, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, mk::radius::LARGE, role, None);
    }
    // chip_button: the whole chip darkens under the finger.
    if card.pressed {
        b.quad(x, y, w, h, p.on_surface.with_alpha(0.14), mk::radius::LARGE, role, None);
    }
    match &card.face {
        Face::Tile { name, maker, lines, widest, dot } => {
            let sub = mk::typo::SUBTITLE;
            let cap = mk::typo::CAPTION;
            let (_, name_h) = engine.measure_styled(name, sub.px(), sub.weight);
            let (_, maker_h) = engine.measure_styled(maker, cap.px(), cap.weight);
            let block = name_h + 1.0 + maker_h + lines.len() as f32 * crate::LINE_H as f32;
            let dot_room = if dot.is_some() { mk::spacing::XXS + DOT } else { 0.0 };
            let top = y + ((h - block - dot_room) / 2.0).max(0.0);
            let left = x + ((w - widest) / 2.0).max(mk::spacing::M.min(w / 2.0));
            b.styled(engine, name.clone(), sub, p.on_surface, left, top, role, None);
            b.styled(engine, maker.clone(), cap, p.on_surface_variant, left, top + name_h + 1.0, role, None);
            let mut ly = top + name_h + 1.0 + maker_h;
            for line in lines {
                b.styled(engine, line.clone(), cap, p.on_surface_variant, left, ly, role, None);
                ly += crate::LINE_H as f32;
            }
            if let Some(Some(focus)) = dot {
                let tint = if *focus { p.primary } else { p.on_surface_variant.with_alpha(0.55) };
                b.quad(x + w / 2.0 - DOT / 2.0, top + block + mk::spacing::XXS, DOT, DOT, tint, DOT / 2.0, role, None);
            }
        }
        Face::Icon { path, mark, symbol, dot, lit } => {
            let dot_room = if dot.is_some() { mk::spacing::XXS + DOT } else { 0.0 };
            let pad = mk::spacing::XS;
            let s = (h - 2.0 * pad - dot_room).min(w - 2.0 * pad).max(8.0);
            let (ix, iy) = (x + (w - s) / 2.0, y + pad);
            let radius = s * 0.225;
            // Round 112 (owner: «open apps are hard to recognise — with a
            // glow?»): an open app glows from behind its icon in the accent
            // colour, the focused one brighter. The dot stays as well.
            if let Some(focus) = lit {
                // Round 114/115: the running app glows (fam::lay_glow), the
                // one in front brighter.
                let room = pad.min(ix - x).min(iy - y).max(1.0);
                fam::lay_glow(b, ix, iy, s, s, radius, room, p.primary, if *focus { 1.0 } else { 0.65 }, role);
            }
            let anchor = b.quad(ix, iy, s, s, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, radius, role, None);
            b.pict(matrixkit_shell::Pict {
                path: path.clone(),
                box_w: s as u32,
                box_h: s as u32,
                fit: matrixkit_image::Fit::Contain,
                x: ix,
                y: iy,
                radius,
                home: Rect::of(&b.scene.placed[anchor]),
                anchor,
                clip: None,
            });
            // A stand-in's mark in the middle of the Matrix tile.
            if !mark.is_empty() {
                if *symbol {
                    let family = matrixkit_scene::symbol_family();
                    let px = mk::typo::symbol_px(s * 0.48);
                    let (mw, mh) = engine.measure_family(family, mark, px, mk::typo::Weight::Normal);
                    b.text(engine, mark.clone(), px, mk::typo::Weight::Normal, family, p.primary, ix + (s - mw) / 2.0, iy + (s - mh) / 2.0, role, None);
                } else {
                    let st = mk::typo::TextStyle { size: s * 0.40, weight: mk::typo::Weight::Semibold };
                    let (mw, mh) = engine.measure_styled(mark, st.px(), st.weight);
                    b.styled(engine, mark.clone(), st, p.primary, ix + (s - mw) / 2.0, iy + (s - mh) / 2.0, role, None);
                }
            }
            if let Some(Some(focus)) = dot {
                let tint = if *focus { p.primary } else { p.on_surface_variant.with_alpha(0.55) };
                b.quad(x + w / 2.0 - DOT / 2.0, iy + s + mk::spacing::XXS, DOT, DOT, tint, DOT / 2.0, role, None);
            }
        }
        Face::Widget { glyph, label, dim, content_w } => {
            let ink = p.on_surface;
            let ink2 = p.on_surface_variant;
            let mut cx = x + (w - content_w) / 2.0;
            let cy = y + h / 2.0;
            if let Some(g) = glyph {
                let px = mk::typo::symbol_px(mk::font_size::LARGE);
                let gs = g.to_string();
                let (gw, gh) = engine.measure_family(fam_symbols(), &gs, px, mk::typo::Weight::Normal);
                let tint = if *dim { ink2.with_alpha(0.5) } else { ink };
                b.text(engine, gs, px, mk::typo::Weight::Normal, fam_symbols(), tint, cx, cy - gh / 2.0, role, None);
                cx += gw + mk::spacing::S;
            }
            if let Some(label) = label {
                let st = mk::typo::CALLOUT;
                let (_, lh) = engine.measure_styled(label, st.px(), st.weight);
                b.styled(engine, label.clone(), st, if glyph.is_some() { ink2 } else { ink }, cx, cy - lh / 2.0, role, None);
            }
        }
        Face::Osd(state) => {
            // The Dynamic Dock chip: glyph and level up top, words below.
            let (glyph, full) = osd_glyph(*state);
            let track_x = x + 56.0;
            let track_w = (w - 56.0 - 18.0).max(40.0);
            let track_y = y + h / 2.0 - 14.0;
            b.node(&fam::level_bar(state.percent / 100.0, full, track_w, p), track_x, track_y, role, None);
            let px = mk::typo::symbol_px(22.0);
            let gs = glyph.to_string();
            let (_, gh) = engine.measure_family(fam_symbols(), &gs, px, mk::typo::Weight::Normal);
            let tint = if full { p.on_surface } else { p.on_surface_variant };
            b.text(engine, gs, px, mk::typo::Weight::Normal, fam_symbols(), tint, x + 18.0, track_y + 4.0 - gh / 2.0, role, None);
            b.styled(engine, state.text(), mk::typo::CAPTION, p.on_surface_variant, track_x, track_y + 12.0, role, None);
        }
        Face::Subtitle { px, lines } => {
            let line_h = px * 1.3;
            let mut ly = y + ((h - line_h * lines.len() as f32) / 2.0).max(0.0);
            for line in lines {
                let (lw, _) = engine.measure_styled(line, *px, mk::typo::Weight::Normal);
                b.text(engine, line.clone(), *px, mk::typo::Weight::Normal, matrixkit_text::FAMILY, p.on_surface, x + (w - lw) / 2.0, ly, role, None);
                ly += line_h;
            }
        }
    }
    // The tap answer: the edge lights up in the accent and fades.
    if card.glow > 0.01 {
        lay_ring(b, x, y, w, h, p.primary.with_alpha(0.3 + 0.6 * card.glow), 1.0 + card.glow, role);
    }
    // Round 113: the wiggle mode's badge in the top-left corner.
    if let Some(remove) = card.edit {
        let d = 18.0;
        let (bx, by) = (x + 1.0, y + 1.0);
        let fill = if remove { p.surface_container_high } else { p.primary };
        let ink = if remove { p.on_surface } else { p.on_primary };
        b.quad(bx, by, d, d, fill.with_alpha(0.95), d / 2.0, role, None);
        // a bar (and a second one across for «+»): crisp at any scale
        b.quad(bx + 5.0, by + d / 2.0 - 1.0, d - 10.0, 2.0, ink, 1.0, role, None);
        if !remove {
            b.quad(bx + d / 2.0 - 1.0, by + 5.0, 2.0, d - 10.0, ink, 1.0, role, None);
        }
    }
}

fn fam_symbols() -> &'static str {
    matrixkit_scene::symbol_family()
}

// ------------------------------------------------------------ the dock

/// What the threads bring home.
pub enum Work {
    /// Round 27: how bright the wallpaper is behind the tile row.
    Backdrop(Option<(String, f32)>),
    Windows(Vec<Win>),
    Osd(mk::osd::State),
    Cinema,
    Subtitle,
    Radios(Option<bool>, Option<bool>, Option<String>),
    Clipboard(Option<String>),
    Clock(String),
}

/// A press in flight: what it hit, where and when it began, and whether
/// the long press already answered.
struct Press {
    hit: Hit,
    at: (f64, f64),
    since: Instant,
    source: Source,
    fired: bool,
    /// Round 113: the second stage (wiggle mode) answered.
    edited: bool,
}

struct Dock {
    palette: mk::Palette,
    watcher: mk::PaletteWatcher,
    /// The one-surface form (pinned panel) instead of the fleet.
    strip: bool,
    /// The main surface spans the whole edge (anchored left AND right).
    full_width: bool,
    autohide: bool,
    /// The output the dock is pinned to (`--output`).
    output: Option<String>,
    screen_w: f32,
    labels: HashMap<String, String>,
    pins: Vec<String>,
    /// Round 57: dock style «icons» (contract `dock-style`).
    icons: bool,
    /// Rendered icon files per app id (None = Matrix has no icon for it).
    icon_files: HashMap<String, Option<std::path::PathBuf>>,
    /// Bumped with every palette change: fresh file names, so the picture
    /// cache never shows the old colours.
    icon_epoch: u32,
    /// Round 88: the icon theme the files above were drawn in.
    icon_theme: Option<String>,
    widgets: Vec<Widget>,
    windows: Vec<Win>,
    tiles: Vec<ShelfTile>,
    osd: Option<(mk::osd::State, Instant)>,
    cinema: bool,
    subtitle: Option<(usize, String)>,
    flash: mk::motion::TapFlash<Hit>,
    bounce: HashMap<String, Instant>,
    press: Option<Press>,
    hidden: bool,
    last_poke: Instant,
    /// Widget states (desktop extras).
    clock: String,
    battery: Option<(u8, bool)>,
    wlan: Option<bool>,
    bluetooth: Option<bool>,
    net: Option<String>,
    hardline_on: bool,
    clip_preview: Option<String>,
    /// The cards measured in the last view().
    cards: Vec<Card>,
    /// The pane painter's own picture store (ring shapes).
    atlas: ImageAtlas,
    loom: Option<Loom<Work>>,
    /// Time since the last animation frame (the glow runs at ~16 Hz).
    anim_acc: f32,
    pulses: u32,
    /// Round 27: (wallpaper, luma) behind the tile row — glass ink follows it.
    backdrop: Option<(String, f32)>,
    /// Round 113: the wiggle mode (mk::edit), its clock, and the pin being
    /// dragged (app id, horizontal offset in points).
    editing: bool,
    edit_t: f32,
    drag: Option<(String, f64)>,
    /// Round 113: what each pane last painted (position aside) — a wiggle
    /// only moves the panes; before, every refresh repainted them all.
    painted: std::cell::RefCell<HashMap<String, Card>>,
}

impl Dock {
    /// Measure the wallpaper behind the tile row off the surface (round 27).
    fn backdrop_ask(&self) {
        let (Some(loom), Some(out)) = (&self.loom, self.output.clone()) else { return };
        let band = (LIFT + tile_height() as i32) as f32;
        loom.spin(move || Work::Backdrop(matrixkit_image::backdrop::strip(&out, |w, h| (0.0, h - band, w, band))));
    }

    fn new(strip: bool, full_width: bool, autohide: bool, output: Option<String>) -> Self {
        let screen_w = output_width(output.as_deref());
        let pins = crate::pins_read();
        let windows = windows_read().unwrap_or_default();
        let tiles = shelf_tiles(&pins, &windows);
        Dock {
            palette: mk::Palette::load().unwrap_or_default(),
            watcher: mk::PaletteWatcher::new(),
            icons: mk::settings::read("dock-style").as_deref() == Some("icons"),
            icon_files: HashMap::new(),
            icon_epoch: 0,
            icon_theme: matrixkit_icons::look(),
            strip,
            full_width,
            autohide,
            output,
            screen_w,
            labels: mk::apps::index().into_iter().map(|e| (e.app_id, e.name)).collect(),
            pins,
            widgets: crate::widgets_read(),
            windows,
            tiles,
            osd: None,
            cinema: mk::cinema::active(),
            subtitle: mk::cinema::subtitle::read(),
            flash: mk::motion::TapFlash::default(),
            bounce: HashMap::new(),
            press: None,
            hidden: false,
            last_poke: Instant::now(),
            clock: String::new(),
            battery: crate::battery_read(),
            wlan: None,
            bluetooth: None,
            net: None,
            hardline_on: crate::hardline_running(),
            clip_preview: None,
            cards: Vec::new(),
            atlas: ImageAtlas::new(),
            loom: None,
            anim_acc: 0.0,
            pulses: 0,
            backdrop: None,
            editing: mk::edit::active(),
            edit_t: 0.0,
            drag: None,
            painted: std::cell::RefCell::new(HashMap::new()),
        }
    }

    /// The name a tile shows (iced `label_of`): the launcher index is the
    /// one source; «Matrix X» under the maker «Matrix» drops the prefix.
    fn label_of(&mut self, app_id: &str) -> String {
        if !self.labels.contains_key(app_id) {
            // An app the index did not know at start: ask its desktop file
            // once, and remember the answer (also a miss).
            let name = mk::apps::display_name(app_id).unwrap_or_else(|| app_id.to_string());
            self.labels.insert(app_id.to_string(), name);
        }
        let full = self.labels.get(app_id).map(String::as_str).unwrap_or(app_id);
        match full.strip_prefix("Matrix ") {
            Some(rest) if maker_of(app_id) == "Matrix" => rest.to_string(),
            _ if full == app_id => {
                let stem = full.strip_prefix("matrix-").unwrap_or(full);
                let mut chars = stem.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => full.to_string(),
                }
            }
            _ => full.to_string(),
        }
    }

    fn glow_of(&self, hit: &Hit) -> f32 {
        let starting = match hit {
            Hit::App(id) => self.bounce.get(id).copied(),
            _ => None,
        };
        let g = mk::motion::tap_glow(self.flash.since(hit), mk::motion::TAP_FLASH_S, starting);
        (g * 10.0).round() / 10.0
    }

    fn pressed(&self, hit: &Hit) -> bool {
        self.press.as_ref().is_some_and(|p| !p.fired && &p.hit == hit)
    }

    /// A two-line tile card (app or «Apps»).
    fn tile_card(&self, engine: &mut TextEngine, hit: Hit, name: String, maker: String, lines: Vec<String>, dot: Option<Option<bool>>) -> Card {
        let sub = mk::typo::SUBTITLE;
        let cap = mk::typo::CAPTION;
        let mut widest = engine.measure_styled(&name, sub.px(), sub.weight).0;
        widest = widest.max(engine.measure_styled(&maker, cap.px(), cap.weight).0);
        for l in &lines {
            widest = widest.max(engine.measure_styled(l, cap.px(), cap.weight).0);
        }
        let w = (widest + 2.0 * mk::spacing::M + TEXT_SAFETY).ceil() as u32;
        let h = tile_height() + lines.len() as u32 * crate::LINE_H;
        let glow = self.glow_of(&hit);
        let pressed = self.pressed(&hit);
        Card { hit, face: Face::Tile { name, maker, lines, widest, dot }, x: 0, lift: LIFT, w, h, glow, pressed, edit: None }
    }

    /// Round 57: an icon card (dock style «icons»). Without a Matrix icon
    /// the app gets a plain squircle with `mark` (symbol or first letter).
    fn icon_card(&mut self, hit: Hit, app_id: &str, mark: String, symbol: bool, dot: Option<Option<bool>>, lit: Option<bool>) -> Option<Card> {
        let (path, mark) = match self.icon_file(app_id) {
            Some(icon) => (icon, String::new()),
            None => (self.tile_file()?, mark),
        };
        let h = tile_height();
        let w = h.saturating_sub(if dot.is_some() { (mk::spacing::XXS + DOT) as u32 } else { 0 }).max(32);
        let glow = self.glow_of(&hit);
        let pressed = self.pressed(&hit);
        Some(Card { hit, face: Face::Icon { path, mark, symbol, dot, lit }, x: 0, lift: LIFT, w, h, glow, pressed, edit: None })
    }

    /// The bare Matrix tile as a file (stand-ins stand on it), per palette.
    fn tile_file(&mut self) -> Option<std::path::PathBuf> {
        const KEY: &str = " tile";
        if let Some(found) = self.icon_files.get(KEY) {
            return found.clone();
        }
        let found = matrixkit_icons::tile_png(&self.palette).and_then(|png| {
            let dir = mk::runtime::path("matrix-dock-icons");
            std::fs::create_dir_all(&dir).ok()?;
            let file = dir.join(format!("_tile-{}.png", self.icon_epoch));
            std::fs::write(&file, png).ok()?;
            Some(file)
        });
        self.icon_files.insert(KEY.to_string(), found.clone());
        found
    }

    /// The app's Matrix icon as a file in the runtime directory, drawn once
    /// per palette (matrixkit-icons; the palette's colours are in it).
    fn icon_file(&mut self, app_id: &str) -> Option<std::path::PathBuf> {
        if let Some(found) = self.icon_files.get(app_id) {
            return found.clone();
        }
        // Round 95: the kit's icon file (chosen theme, else Matrix; cached
        // per app, size, theme and palette — shared with the bar's app list).
        let found = matrixkit_icons::icon_file(app_id, None, &self.palette, 256);
        self.icon_files.insert(app_id.to_string(), found.clone());
        found
    }

    /// A widget chip — None when it has nothing to say on this device now
    /// (no battery, no radio). The menus of `user`/`matrix` are not in the
    /// house yet.
    fn widget_card(&self, engine: &mut TextEngine, wd: Widget) -> Option<Card> {
        let (glyph, label, dim): (Option<char>, Option<String>, bool) = match wd {
            Widget::Apps => (Some(symbol::APPS), Some(String::from("Apps")), false),
            Widget::Central => (Some(symbol::TUNE), None, false),
            Widget::Capture => (Some(symbol::IMAGE), None, false),
            Widget::Hardline => {
                let on = self.hardline_on;
                (Some(if on { symbol::KEY } else { symbol::LOCK }), Some(String::from(if on { "An" } else { "Aus" })), !on)
            }
            Widget::Clock => (None, Some(self.clock.clone()).filter(|c| !c.is_empty()), false),
            Widget::Clipboard => {
                let preview = self.clip_preview.as_deref().map(|t| {
                    let z: String = t.split_whitespace().collect::<Vec<_>>().join(" ");
                    if z.chars().count() > 18 {
                        z.chars().take(17).collect::<String>() + "…"
                    } else {
                        z
                    }
                });
                (Some(symbol::CONTENT_COPY), Some(preview.unwrap_or_else(|| String::from("Zwischenablage"))), false)
            }
            Widget::Battery => {
                let (percent, loading) = self.battery?;
                (Some(if loading { symbol::BATTERY_CHARGING } else { symbol::BATTERY_FULL }), Some(format!("{percent} %")), false)
            }
            Widget::Wlan => {
                let on = self.wlan?;
                let label = match (on, &self.net) {
                    (true, Some(n)) => n.chars().take(18).collect(),
                    (true, None) => String::from("WLAN"),
                    (false, _) => String::from("WLAN aus"),
                };
                (Some(symbol::WIFI), Some(label), !on)
            }
            Widget::Bluetooth => {
                let on = self.bluetooth?;
                (Some(symbol::BLUETOOTH), None, !on)
            }
            Widget::User | Widget::Matrix => return None,
        };
        if glyph.is_none() && label.is_none() {
            return None;
        }
        let mut content_w = 0.0;
        if let Some(g) = glyph {
            content_w += engine
                .measure_family(fam_symbols(), &g.to_string(), mk::typo::symbol_px(mk::font_size::LARGE), mk::typo::Weight::Normal)
                .0;
        }
        if let Some(l) = &label {
            if glyph.is_some() {
                content_w += mk::spacing::S;
            }
            let st = mk::typo::CALLOUT;
            content_w += engine.measure_styled(l, st.px(), st.weight).0;
        }
        let w = (content_w + 2.0 * mk::spacing::M + 2.0 * mk::spacing::S + TEXT_SAFETY).ceil() as u32;
        let hit = Hit::Widget(wd);
        let glow = self.glow_of(&hit);
        let pressed = self.pressed(&hit);
        Some(Card { hit, face: Face::Widget { glyph, label, dim, content_w }, x: 0, lift: LIFT, w, h: ELEMENT_H, glow, pressed, edit: None })
    }

    /// The shelf of this moment, measured: tiles from the left, the OSD
    /// chip after them, the widgets at the end — or, in cinema mode, only
    /// the subtitle strip.
    fn plan(&mut self, engine: &mut TextEngine) -> Vec<Card> {
        if self.hidden {
            return Vec::new();
        }
        let p_screen = self.screen_w;
        if self.cinema {
            let Some((size, text)) = self.subtitle.clone() else { return Vec::new() };
            let px = mk::cinema::subtitle::px(size);
            let margin = mk::cinema::subtitle::MARGIN;
            let w = (p_screen as i32 - 2 * margin).max(80) as u32;
            let wrapped = engine.wrap_styled(&text, px, mk::typo::Weight::Normal, w as f32 - 32.0);
            let lines = wrapped.lines().map(str::to_string).collect();
            return vec![Card {
                hit: Hit::None,
                face: Face::Subtitle { px, lines },
                x: margin,
                lift: margin,
                w,
                h: mk::cinema::subtitle::strip_height(size),
                glow: 0.0,
                pressed: false,
                edit: None,
            }];
        }
        let shelf = mk::device::shelf();
        let lead_apps = shelf.apps_leads && self.widgets.contains(&Widget::Apps);
        let widget_list: Vec<Widget> = self.widgets.iter().copied().filter(|w| !(lead_apps && *w == Widget::Apps)).collect();
        let widgets: Vec<Card> = widget_list.into_iter().filter_map(|w| self.widget_card(engine, w)).collect();
        let widget_room: i32 = widgets.iter().map(|c| c.w as i32 + GAP).sum::<i32>() + if widgets.is_empty() || !self.strip { 0 } else { mk::spacing::L as i32 };
        let room = p_screen as i32 - 2 * SIDE - widget_room;

        let mut cards: Vec<Card> = Vec::new();
        if lead_apps {
            let icon = if self.icons { self.icon_card(Hit::Apps, "matrix-start", symbol::APPS.to_string(), true, None, None) } else { None };
            cards.push(icon.unwrap_or_else(|| self.tile_card(engine, Hit::Apps, String::from(crate::APPS_NAME), String::from(crate::APPS_MAKER), Vec::new(), None)));
        }
        let dot_row = shelf.tile_height.is_none();
        let tiles = self.tiles.clone();
        let mut used: i32 = cards.iter().map(|c| c.w as i32 + GAP).sum();
        for t in &tiles {
            let name = self.label_of(&t.app_id);
            let lines = if shelf.window_lines { window_lines(t, &name) } else { Vec::new() };
            // The window lines say «running» more clearly than the dot.
            let dot = dot_row.then(|| (t.running() && lines.is_empty()).then_some(t.focus));
            // Round 57: dock style «icons» — the Matrix icon alone.
            let mark = name.chars().next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_default();
            let lit = t.running().then_some(t.focus);
            let icon = if self.icons && lines.is_empty() { self.icon_card(Hit::App(t.app_id.clone()), &t.app_id, mark, false, dot, lit) } else { None };
            let card = match icon {
                Some(c) => c,
                None => self.tile_card(engine, Hit::App(t.app_id.clone()), name, maker_of(&t.app_id).to_string(), lines, dot),
            };
            // Paging instead of pushing the widgets away (R172l): what
            // does not fit stays off the row.
            if used + card.w as i32 > room && !cards.is_empty() {
                break;
            }
            used += card.w as i32 + GAP;
            cards.push(card);
        }
        // Round 113: in the wiggle mode pins carry «–», running apps «+»,
        // and the row ends in «+» (add an app) and «Fertig».
        if self.editing {
            for c in cards.iter_mut() {
                if let Hit::App(id) = &c.hit {
                    c.edit = Some(self.pins.contains(id));
                }
            }
            cards.push(self.edit_card(engine, Hit::EditAdd, Some(symbol::ADD), None));
            cards.push(self.edit_card(engine, Hit::EditDone, None, Some(String::from("Fertig"))));
        }
        if let Some((state, _)) = self.osd {
            cards.push(Card { hit: Hit::None, face: Face::Osd(state), x: 0, lift: LIFT, w: OSD_W, h: tile_height(), glow: 0.0, pressed: false, edit: None });
        }
        let first_widget = cards.len();
        cards.extend(widgets);
        // Positions along the row.
        let mut x = SIDE;
        for (i, c) in cards.iter_mut().enumerate() {
            // The strip sets the widgets apart; the fleet knows no air of
            // its own (a spacer would be a glass surface).
            if self.strip && i == first_widget && i > 0 {
                x += mk::spacing::L as i32;
            }
            c.x = x;
            x += c.w as i32 + GAP;
        }
        cards
    }

    /// Round 113: a card of the wiggle mode («+», «Fertig»), sized like a widget.
    fn edit_card(&self, engine: &mut TextEngine, hit: Hit, glyph: Option<char>, label: Option<String>) -> Card {
        let mut content_w = 0.0;
        if let Some(g) = glyph {
            content_w += engine.measure_family(fam_symbols(), &g.to_string(), mk::typo::symbol_px(mk::font_size::LARGE), mk::typo::Weight::Normal).0;
        }
        if let Some(l) = &label {
            let st = mk::typo::CALLOUT;
            content_w += engine.measure_styled(l, st.px(), st.weight).0;
        }
        let w = (content_w + 2.0 * mk::spacing::M + 2.0 * mk::spacing::S + TEXT_SAFETY).ceil().max(ELEMENT_H as f32) as u32;
        let glow = self.glow_of(&hit);
        let pressed = self.pressed(&hit);
        Card { hit, face: Face::Widget { glyph, label, dim: false, content_w }, x: 0, lift: LIFT + (tile_height() as i32 - ELEMENT_H as i32) / 2, w, h: ELEMENT_H, glow, pressed, edit: None }
    }

    /// Round 113: into the wiggle mode — the second stage of a long press
    /// (a second buzz), or another bar / the menu asked.
    fn edit_enter(&mut self, by_press: bool) -> Refresh {
        if by_press {
            mk::feel::stage();
            // the first stage's menu goes
            sh_spawn("PATH=$HOME/.local/bin:$PATH exec matrix-context hide");
            mk::edit::enter();
        }
        eprintln!("[dock] Bearbeiten: an");
        self.editing = true;
        self.edit_t = 0.0;
        self.hidden = false;
        self.last_poke = Instant::now();
        Refresh::Panes
    }

    fn edit_leave(&mut self, tell: bool) -> Refresh {
        if tell {
            mk::edit::leave();
        }
        eprintln!("[dock] Bearbeiten: aus");
        self.editing = false;
        self.drag = None;
        Refresh::Panes
    }

    /// Pin or unpin (the badge on a tile in the wiggle mode).
    fn pin_toggle(&mut self, id: &str) {
        let mut pins = self.pins.clone();
        if let Some(i) = pins.iter().position(|p| p == id) {
            pins.remove(i);
            eprintln!("[dock] Bearbeiten: «{id}» gelöst");
        } else {
            pins.push(id.to_string());
            eprintln!("[dock] Bearbeiten: «{id}» angeheftet");
        }
        mk::feel::select();
        self.pins_store(pins);
    }

    fn pins_store(&mut self, pins: Vec<String>) {
        mk::settings::write("dock-pins", &pins.join(" "));
        self.pins = pins;
        self.retile();
    }

    /// A dragged pin dropped: its new place among the pins.
    fn drag_drop(&mut self) {
        let Some((id, dx)) = self.drag.take() else { return };
        let pinned: Vec<&Card> = self.cards.iter().filter(|c| matches!(&c.hit, Hit::App(a) if self.pins.contains(a))).collect();
        let centres: Vec<f32> = pinned.iter().map(|c| c.x as f32 + c.w as f32 / 2.0).collect();
        let Some(from) = pinned.iter().position(|c| c.hit == Hit::App(id.clone())) else { return };
        let to = mk::edit::slot_at(&centres, centres[from] + dx as f32);
        let mut order: Vec<String> = pinned.iter().filter_map(|c| if let Hit::App(a) = &c.hit { Some(a.clone()) } else { None }).collect();
        mk::edit::move_item(&mut order, from, to);
        // pins not on the row (paged off) keep their place behind
        for p in &self.pins {
            if !order.contains(p) {
                order.push(p.clone());
            }
        }
        if order != self.pins {
            eprintln!("[dock] Bearbeiten: «{id}» verschoben");
            mk::feel::select();
            self.pins_store(order);
        }
    }

    /// Re-derive the tiles from pins and windows; true = the row changed.
    fn retile(&mut self) -> bool {
        let fresh = shelf_tiles(&self.pins, &self.windows);
        // A starting pin pulses until its window arrives (or 8 s pass).
        self.bounce.retain(|id, since| {
            since.elapsed() < BOUNCE_MAX && !fresh.iter().any(|t| &t.app_id == id && t.running())
        });
        if fresh != self.tiles {
            self.tiles = fresh;
            return true;
        }
        false
    }

    /// The form needs a fresh picture: panes repaint the fleet, the strip
    /// its one surface.
    fn redraw(&self) -> Refresh {
        if self.strip {
            Refresh::Dirty
        } else {
            Refresh::Panes
        }
    }

    // ---------------------------------------------------- the gestures

    fn press_begin(&mut self, hit: Hit, x: f64, y: f64, source: Source) -> Refresh {
        self.last_poke = Instant::now();
        if self.hidden {
            return self.wake();
        }
        if hit == Hit::None {
            return Refresh::Calm;
        }
        self.press = Some(Press { hit, at: (x, y), since: Instant::now(), source, fired: false, edited: false });
        self.redraw()
    }

    fn press_travel(&mut self, x: f64, y: f64) -> Refresh {
        self.last_poke = Instant::now();
        // Round 113: in the wiggle mode a pin follows the finger.
        if self.editing {
            if let Some(p) = &self.press {
                if let Hit::App(id) = &p.hit {
                    let dx = x - p.at.0;
                    if self.pins.contains(id) && (self.drag.is_some() || dx.abs() > SLOP) {
                        self.drag = Some((id.clone(), dx));
                        return Refresh::Panes;
                    }
                }
            }
            return Refresh::Calm;
        }
        let wandered = self
            .press
            .as_ref()
            .is_some_and(|p| (x - p.at.0).abs() > SLOP || (y - p.at.1).abs() > SLOP);
        if wandered {
            self.press = None;
            return self.redraw();
        }
        Refresh::Calm
    }

    /// The press ends: a tap if it ends on what it began on and the long
    /// press has not answered already.
    fn press_end(&mut self, hit: Option<Hit>) -> Refresh {
        let Some(p) = self.press.take() else { return Refresh::Calm };
        if self.drag.is_some() {
            self.drag_drop();
            return Refresh::Panes;
        }
        if !p.fired && hit.as_ref().is_none_or(|h| *h == p.hit) {
            self.tap(p.hit);
        }
        self.redraw()
    }

    /// The long press (touch) answers while the finger still rests.
    fn press_hold(&mut self) -> Refresh {
        let hit = match &mut self.press {
            Some(p) if !p.fired && p.source == Source::Touch => {
                p.fired = true;
                p.hit.clone()
            }
            _ => return Refresh::Calm,
        };
        if self.editing {
            return Refresh::Calm;
        }
        self.context(&hit);
        self.redraw()
    }

    fn tap(&mut self, hit: Hit) {
        // Round 113: the wiggle mode's taps.
        if self.editing {
            match hit {
                Hit::App(id) => self.pin_toggle(&id),
                Hit::EditAdd => {
                    self.flash.tap(Hit::EditAdd);
                    // the bar's app list in «pick» mode: a tap pins
                    let out = self.output.clone().unwrap_or_default();
                    let from = self.cards.iter().find(|c| c.hit == Hit::EditAdd).map(|c| format!(" {} {}", c.x, c.w)).unwrap_or_default();
                    if !mk::bar::ask(&format!("apps-pick {out}{from}")) {
                        eprintln!("[dock] Bearbeiten: keine Leiste für die App-Auswahl");
                    }
                }
                Hit::EditDone => {
                    let _ = self.edit_leave(true);
                }
                _ => {}
            }
            return;
        }
        match hit {
            Hit::Apps => {
                self.flash.tap(Hit::Apps);
                // Round 35: the top bar shows the app card on our panel (no
                // iced); without a listening bar, Matrix Start as before.
                let out = self.output.clone().unwrap_or_default();
                // Round 61: tell the bar where the tile is — the app card
                // grows out of it («apps DSI-1 <x> <w>», logical px).
                let from = self.cards.iter().find(|c| c.hit == Hit::Apps).map(|c| format!(" {} {}", c.x, c.w)).unwrap_or_default();
                if !mk::bar::ask(&format!("apps {out}{from}")) {
                    app_start("matrix-start", &[]);
                }
            }
            Hit::App(id) => {
                self.flash.tap(Hit::App(id.clone()));
                let Some(t) = self.tiles.iter().find(|t| t.app_id == id).cloned() else { return };
                // Round 169: the kit's tile rule — a window on the shelf
                // comes back first (Prism's dialog stayed unreachable).
                let wins: Vec<mk::window::TileWin> =
                    t.windows.iter().map(|w| mk::window::TileWin { id: w.id, focus: w.focus, shelved: w.shelved }).collect();
                match mk::window::tile_tap(&wins, mk::device::shelf().tap_shelves) {
                    mk::window::TileTap::Shelve => {
                        // R209 (duo): the app in front, tapped again, goes on
                        // the shelf — the duo's window has no shelf light.
                        eprintln!("[dock] Kachel «{id}»: vorne — wandert auf die Ablage");
                        mk::window::shelf();
                    }
                    mk::window::TileTap::Bring(win) => {
                        eprintln!("[dock] Kachel «{id}»: Fenster {win} geholt");
                        // Fetched from the shelf first if it was put away.
                        mk::canvas::window_bring(win);
                    }
                    mk::window::TileTap::Start => {
                        if t.pinned {
                            eprintln!("[dock] Kachel «{id}»: ruhender Pin erwacht");
                            app_start(&id, &[]);
                            self.bounce.insert(id, Instant::now());
                        }
                    }
                }
            }
            Hit::Widget(w) => {
                self.flash.tap(Hit::Widget(w));
                match w {
                    Widget::Apps => app_start("matrix-start", &[]),
                    Widget::Central | Widget::Wlan | Widget::Bluetooth => app_start("matrix-central", &[]),
                    Widget::Capture => app_start("matrix-capture", &[]),
                    Widget::Hardline => {
                        // R187: OFF ends the node completely, ON starts it.
                        if self.hardline_on {
                            crate::hardline_stop();
                        } else {
                            app_start("hardline-node", &["serve"]);
                        }
                        self.hardline_on = !self.hardline_on;
                    }
                    Widget::Clipboard => {
                        // The history is an external menu (the dock never grows).
                        let bottom = LIFT + ELEMENT_H as i32 + 8;
                        // Round 68: our screen as the last word — the menu opens here.
                        let out = self.output.clone().unwrap_or_default();
                        sh_spawn(&format!("PATH=$HOME/.local/bin:$PATH exec matrix-context clips 0 {bottom} {out}"));
                    }
                    Widget::Clock | Widget::Battery | Widget::User | Widget::Matrix => {}
                }
            }
            Hit::EditAdd | Hit::EditDone | Hit::None => {}
        }
    }

    /// The tile menu: an external menu (`matrix-context dock …`) right
    /// above the tile — the dock itself never grows.
    fn context(&mut self, hit: &Hit) {
        let Hit::App(id) = hit else { return };
        let Some(t) = self.tiles.iter().find(|t| &t.app_id == id) else { return };
        let Some(card) = self.cards.iter().find(|c| &c.hit == hit) else { return };
        mk::feel::hold();
        let centre = card.x as f32 + card.w as f32 / 2.0;
        let offset_x = centre - self.screen_w / 2.0;
        let bottom = card.lift + card.h as i32 + 8;
        let ids = if t.windows.is_empty() {
            String::from("-")
        } else {
            t.windows.iter().map(|w| w.id.to_string()).collect::<Vec<_>>().join(",")
        };
        eprintln!("[dock] Kontextmenü für «{id}»");
        sh_spawn(&format!(
            "PATH=$HOME/.local/bin:$PATH exec matrix-context dock {} {} {} {} {} {}",
            offset_x as i32,
            bottom,
            t.app_id,
            u8::from(t.pinned),
            ids,
            // Round 68: our screen — the menu opens above the tile, here.
            self.output.clone().unwrap_or_default()
        ));
    }

    /// R168: waking from the fold.
    fn wake(&mut self) -> Refresh {
        if !self.hidden {
            return Refresh::Calm;
        }
        eprintln!("[dock] wacht auf");
        self.hidden = false;
        self.last_poke = Instant::now();
        Refresh::Panes
    }

    fn hit_of(&self, role: Option<usize>) -> Hit {
        role.and_then(|i| self.cards.get(i)).map(|c| c.hit.clone()).unwrap_or(Hit::None)
    }

    /// Settings the house cannot change in a running surface (the
    /// exclusive zone is set once): a changed auto-hide restarts the dock.
    fn settings_check(&mut self) -> Refresh {
        let autohide = mk::settings::read("dock-autohide").as_deref() == Some("on");
        if autohide != self.autohide {
            eprintln!("[dock] Automatisch ausblenden gewechselt — das Dock startet neu");
            restart_with(&[]);
            self.autohide = autohide;
        }
        let mut changed = false;
        let pins = crate::pins_read();
        if pins != self.pins {
            self.pins = pins;
            changed |= self.retile();
        }
        let widgets = crate::widgets_read();
        if widgets != self.widgets {
            self.widgets = widgets;
            changed = true;
        }
        // Round 57: the face (tiles / icons) from Settings, live.
        let icons = mk::settings::read("dock-style").as_deref() == Some("icons");
        if icons != self.icons {
            self.icons = icons;
            changed = true;
        }
        if changed {
            eprintln!("[dock] Regal neu gelesen: {} Kacheln, {} Widgets", self.tiles.len(), self.widgets.len());
            return Refresh::Panes;
        }
        Refresh::Calm
    }

    /// The occasional readings of the desktop widgets — on threads, so the
    /// loop never waits for nmcli, bluetoothctl or wl-paste.
    fn widget_readings(&mut self) {
        let Some(loom) = self.loom.clone() else { return };
        if self.widgets.contains(&Widget::Clock) {
            // Round 178: one reading at a time per kind (a hung tool no longer gains a thread every pulse).
            loom.spin_once("dock-clock", || {
                let out = std::process::Command::new("date").arg("+%H:%M").output();
                Work::Clock(out.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default())
            });
        }
        if self.widgets.contains(&Widget::Clipboard) {
            loom.spin_once("dock-clipboard", || Work::Clipboard(crate::zw_maintain()));
        }
        if self.widgets.iter().any(|w| w.needs_network()) && self.pulses % 15 == 0 {
            loom.spin_once("dock-radios", || Work::Radios(crate::wlan_read(), crate::bt_read(), crate::net_read()));
        }
    }
}

/// The compositor's event stream on a thread: every burst (100 ms) reads
/// the window list once and sends it home if it changed.
fn watch_windows(loom: Loom<Work>) {
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    // Round 50: the kit's event follow (reconnect, kill AND reap).
    mk::canvas::follow_events(false, move |line| line.is_none() || tx.send(()).is_ok());
    std::thread::spawn(move || {
        let mut last: Option<Vec<Win>> = None;
        while rx.recv().is_ok() {
            std::thread::sleep(Duration::from_millis(100));
            while rx.try_recv().is_ok() {}
            if let Some(list) = windows_read() {
                if last.as_ref() != Some(&list) {
                    last = Some(list.clone());
                    loom.feed(Work::Windows(list));
                }
            }
        }
    });
}

/// The small files the dock follows (OSD channel, cinema, subtitle) — a
/// stat every 80 ms on a thread; only a change wakes the loop.
fn watch_files(loom: Loom<Work>) {
    std::thread::spawn(move || {
        let stamp = |p: &std::path::Path| -> Option<SystemTime> { std::fs::metadata(p).and_then(|m| m.modified()).ok() };
        let osd_path = std::path::PathBuf::from(mk::osd::path());
        let cinema_path = mk::cinema::path();
        let subtitle_path = mk::cinema::subtitle::path();
        // Consume the state at start: only CHANGES morph the shelf (the
        // first-dropout of Lager 42/47).
        let mut osd = stamp(&osd_path);
        let mut cinema = stamp(&cinema_path);
        let mut subtitle = stamp(&subtitle_path);
        loop {
            std::thread::sleep(Duration::from_millis(80));
            let now = stamp(&osd_path);
            if now != osd {
                osd = now;
                if let Some((state, _)) = mk::osd::read() {
                    loom.feed(Work::Osd(state));
                }
            }
            let now = stamp(&cinema_path);
            if now != cinema {
                cinema = now;
                loom.feed(Work::Cinema);
            }
            let now = stamp(&subtitle_path);
            if now != subtitle {
                subtitle = now;
                loom.feed(Work::Subtitle);
            }
        }
    });
}

impl shell::Client for Dock {
    type Card = Card;
    type Work = Work;

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn warp(&mut self, loom: &Loom<Work>) {
        self.loom = Some(loom.clone());
        watch_windows(loom.clone());
        watch_files(loom.clone());
        self.widget_readings();
        self.backdrop_ask();
    }

    fn woven(&mut self, work: Work) -> Refresh {
        match work {
            Work::Backdrop(b) => {
                // the glass ink follows the backdrop: every pane anew
                self.painted.borrow_mut().clear();
                if b.is_some() && b != self.backdrop {
                    eprintln!("[dock] Hintergrund hinter dem Regal: {:.2}", b.as_ref().map(|b| b.1).unwrap_or(0.0));
                    self.backdrop = b;
                    return self.redraw();
                }
                Refresh::Calm
            }
            Work::Windows(list) => {
                self.windows = list;
                if self.retile() {
                    eprintln!("[dock] Regal wechselt: {} Kacheln", self.tiles.len());
                    return Refresh::Panes;
                }
                Refresh::Calm
            }
            Work::Osd(state) => {
                eprintln!("[dock] OSD-Verwandlung: {}", state.text());
                self.osd = Some((state, Instant::now()));
                Refresh::Panes
            }
            Work::Cinema => {
                let now = mk::cinema::active();
                if now == self.cinema {
                    return Refresh::Calm;
                }
                self.cinema = now;
                self.subtitle = mk::cinema::subtitle::read();
                eprintln!("[dock] Kino {}", if now { "an — die Kacheln treten zur Seite" } else { "aus" });
                Refresh::Panes
            }
            Work::Subtitle => {
                let fresh = mk::cinema::subtitle::read();
                if fresh == self.subtitle {
                    return Refresh::Calm;
                }
                self.subtitle = fresh;
                if self.cinema {
                    Refresh::Panes
                } else {
                    Refresh::Calm
                }
            }
            Work::Radios(wlan, bluetooth, net) => {
                if (wlan, bluetooth, &net) == (self.wlan, self.bluetooth, &self.net) {
                    return Refresh::Calm;
                }
                self.wlan = wlan;
                self.bluetooth = bluetooth;
                self.net = net;
                Refresh::Panes
            }
            Work::Clipboard(preview) => {
                if preview == self.clip_preview {
                    return Refresh::Calm;
                }
                self.clip_preview = preview;
                Refresh::Panes
            }
            Work::Clock(clock) => {
                if clock == self.clock {
                    return Refresh::Calm;
                }
                self.clock = clock;
                Refresh::Panes
            }
        }
    }

    fn view(&mut self, engine: &mut TextEngine, _keep: &mut Keep<f32>, _atlas: &mut ImageAtlas, w: u32, _h: u32) -> Scene {
        // A main surface across the whole edge knows the screen's width.
        if self.full_width && w > 1 {
            self.screen_w = w as f32;
        }
        self.cards = self.plan(engine);
        if !self.strip || self.cards.is_empty() {
            return Scene::default();
        }
        // The strip: every card bottom-aligned in the one surface; the
        // surface itself floats LIFT (or the subtitle margin) above the edge.
        let height = self.cards.iter().map(|c| c.h).max().unwrap_or(1) as f32;
        let p = self.palette;
        let mut b = SceneBuilder::new();
        let cards = self.cards.clone();
        for (i, card) in cards.iter().enumerate() {
            let y = height - card.h as f32;
            lay_card(&mut b, engine, card, card.x as f32, y, true, Some(i), mk::glass::palette(&p, self.backdrop.as_ref().map(|b| b.1)));
        }
        b.finish()
    }

    fn extent(&mut self) -> Option<(u32, u32)> {
        if self.hidden {
            return Some((if self.full_width { 0 } else { self.screen_w as u32 }, CATCH));
        }
        if !self.strip {
            return Some((0, 1));
        }
        let height = self.cards.iter().map(|c| c.h).max().unwrap_or(1);
        let width = if self.full_width {
            0
        } else {
            self.cards.iter().map(|c| c.x + c.w as i32).max().unwrap_or(1).max(1) as u32 + SIDE as u32
        };
        Some((width, height))
    }

    fn offset(&mut self) -> Option<(i32, i32, i32, i32)> {
        if self.hidden || !self.strip {
            return Some((0, 0, 0, 0));
        }
        // The cards carry their x themselves (from the left edge), so the
        // surface keeps no side margin; it floats by their lift.
        let lift = self.cards.iter().map(|c| c.lift).min().unwrap_or(LIFT);
        Some((0, 0, lift, 0))
    }

    fn passthrough(&self) -> bool {
        // Folded, the main surface is the catch edge; otherwise it only
        // takes input where it shows something (the strip form).
        !self.hidden && (!self.strip || self.cards.is_empty())
    }

    fn fleet(&mut self) -> Vec<shell::fleet::Spec<Card>> {
        if self.strip {
            return Vec::new();
        }
        // The fleet stacks along the row by itself (start SIDE, gap GAP) —
        // the same positions plan() gave the cards (no extra air in this
        // form: a spacer would be a glass surface of its own).
        self.cards
            .iter()
            .map(|card| shell::fleet::Spec {
                width: card.w,
                height: card.h,
                cross: card.lift,
                namespace: card.namespace(),
                keyboard: false,
                // Width AND height in the signature: a new measure reopens
                // the surface (as in iced).
                sig: (card.kind(), card.w + card.h * 8192),
                pane: card.clone(),
            })
            .collect()
    }

    /// Each tile places itself on the dock's screen (the fleet alone would
    /// open it on whichever panel has focus): bottom-left, its x and lift.
    fn pane_place(&mut self, card: &Card) -> Option<shell::PanePlace> {
        // Round 113: the wiggle mode moves panes only (margins — the
        // compositor shifts them, nothing is painted); a dragged pin rides
        // the finger.
        let (mut dx, mut dy) = (0.0f32, 0.0f32);
        if self.editing && !matches!(card.hit, Hit::EditAdd | Hit::EditDone | Hit::None) {
            match &self.drag {
                Some((id, off)) if card.hit == Hit::App(id.clone()) => dx = *off as f32,
                _ => {
                    let i = self.cards.iter().position(|c| c.hit == card.hit).unwrap_or(0);
                    (dx, dy) = mk::edit::wiggle(self.edit_t, i);
                }
            }
        }
        Some(shell::PanePlace {
            anchor: shell::Anchors { top: false, right: false, bottom: true, left: true },
            margin: (0, 0, card.lift + dy.round() as i32, card.x + dx.round() as i32),
            size: (card.w, card.h),
            output: self.output.clone(),
            layer: None,
            keyboard: shell::KeyboardWish::None,
            exclusive: -1,
        })
    }

    /// The tile as a scene: the house paints it at the panel's own scale
    /// (crisp text on the duo's 2.76× panels instead of 1× pixels).
    fn pane_scene(&mut self, card: &Card, engine: &mut TextEngine, _atlas: &mut ImageAtlas, _w: u32, _h: u32) -> Option<Scene> {
        let mut b = SceneBuilder::new();
        lay_card(&mut b, engine, card, 0.0, 0.0, false, None, mk::glass::palette(&self.palette, self.backdrop.as_ref().map(|b| b.1)));
        Some(b.finish())
    }

    /// Round 113: repaint a pane only when its card changed (its place
    /// aside — the wiggle only moves it).
    fn pane_stale(&mut self, card: &Card) -> bool {
        let key = format!("{:?}", card.hit);
        let mut still = card.clone();
        still.x = 0;
        still.lift = 0;
        let mut painted = self.painted.borrow_mut();
        if painted.get(&key) == Some(&still) {
            return false;
        }
        painted.insert(key, still);
        true
    }

    fn pane(&mut self, card: &Card, engine: &mut TextEngine, canvas: &mut [u8], w: u32, h: u32) {
        let mut b = SceneBuilder::new();
        lay_card(&mut b, engine, card, 0.0, 0.0, false, None, mk::glass::palette(&self.palette, self.backdrop.as_ref().map(|b| b.1)));
        let scene = b.finish();
        shell::compose(engine, &mut self.atlas, &scene, None, canvas, w, h, 1.0);
    }

    fn pane_strike(&mut self, card: &Card, x: f64, y: f64, source: Source) -> Refresh {
        self.press_begin(card.hit.clone(), x, y, source)
    }

    /// Round 163: a right click on a tile (the panes form) is its menu — the
    /// house's default would take it for a press that never ends.
    fn pane_secondary(&mut self, card: &Card, _x: f64, _y: f64) -> Refresh {
        if self.editing {
            return Refresh::Calm;
        }
        self.context(&card.hit);
        self.redraw()
    }

    fn pane_travel(&mut self, _card: &Card, x: f64, y: f64, _source: Source) -> Refresh {
        self.press_travel(x, y)
    }

    fn pane_release(&mut self, card: &Card, x: f64, y: f64, _source: Source) -> Refresh {
        let inside = x >= 0.0 && y >= 0.0 && x <= card.w as f64 && y <= card.h as f64;
        self.press_end(inside.then(|| card.hit.clone()))
    }

    fn strike(&mut self, role: Option<usize>, x: f64, y: f64, source: Source) -> Refresh {
        let hit = self.hit_of(role);
        self.press_begin(hit, x, y, source)
    }

    fn travel(&mut self, x: f64, y: f64, _source: Source) -> Refresh {
        self.press_travel(x, y)
    }

    fn release(&mut self, role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        let hit = self.hit_of(role);
        self.press_end(Some(hit))
    }

    fn hold(&mut self, _role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        // The dock keeps its own long-press clock (glide) for panes AND
        // strip alike; the house's hold only confirms it.
        self.press_hold()
    }

    fn secondary(&mut self, role: Option<usize>, _x: f64, _y: f64) -> Refresh {
        // Round 163: the menu must be drawn too (it opened, unseen, on a right
        // click — the long press redraws, the click answered Calm).
        if self.editing {
            return Refresh::Calm;
        }
        let hit = self.hit_of(role);
        self.context(&hit);
        self.redraw()
    }

    fn motion(&mut self, _x: f64, _y: f64) -> Refresh {
        // The pointer touching the catch edge wakes the folded dock.
        self.last_poke = Instant::now();
        if self.hidden {
            return self.wake();
        }
        Refresh::Calm
    }

    fn animating(&self) -> bool {
        // Round 113: the clock runs on after the menu (first stage) until
        // the second stage could come; the wiggle mode always runs.
        let pressing = self.press.as_ref().is_some_and(|p| p.source == Source::Touch && (!p.fired || (!p.edited && !self.editing)));
        pressing || self.editing || (!mk::motion_reduced() && (self.flash.is_active() || !self.bounce.is_empty()))
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        // The long press on a pane (the house only times it on the main
        // surface).
        let due = self.press.as_ref().is_some_and(|p| {
            !p.fired && p.source == Source::Touch && p.since.elapsed().as_secs_f32() >= mk::touch::LONG_PRESS_SECS
        });
        if due {
            return self.press_hold();
        }
        // Round 113: the SECOND stage — still resting after the menu came:
        // a second buzz and the wiggle mode.
        let second = !self.editing
            && self.press.as_ref().is_some_and(|p| {
                p.fired && !p.edited && p.source == Source::Touch && p.hit != Hit::None && p.since.elapsed().as_secs_f32() >= mk::edit::PRESS_SECS
            });
        if second {
            if let Some(p) = &mut self.press {
                p.edited = true;
            }
            return self.edit_enter(true);
        }
        if self.editing {
            self.edit_t += dt;
            self.anim_acc += dt;
            // ~30 steps a second: margins only (pane_stale keeps the paint)
            if self.anim_acc >= 0.033 && !mk::motion_reduced() {
                self.anim_acc = 0.0;
                return Refresh::Panes;
            }
            return Refresh::Calm;
        }
        // The glow fades / pulses at ~16 frames a second — plenty for a
        // ring, and a fleet repaint is a full repaint of every pane.
        self.anim_acc += dt;
        if self.anim_acc < 0.06 {
            return Refresh::Calm;
        }
        self.anim_acc = 0.0;
        let glowing = self.flash.is_active() || !self.bounce.is_empty();
        if !glowing || mk::motion_reduced() {
            // Only a resting finger keeps the clock running: nothing to draw.
            return Refresh::Calm;
        }
        self.flash.tick();
        self.bounce.retain(|_, since| since.elapsed() < BOUNCE_MAX);
        self.redraw()
    }

    fn beat(&mut self) -> Refresh {
        let mut r = Refresh::Calm;
        // Round 113: the bar (or the menu) entered / left the wiggle mode.
        let editing = mk::edit::active();
        if editing != self.editing && self.press.is_none() {
            r = if editing { self.edit_enter(false) } else { self.edit_leave(false) };
        }
        // Round 88: another icon theme chosen — draw the icons anew.
        let theme = matrixkit_icons::look();
        if theme != self.icon_theme {
            self.icon_theme = theme;
            self.icon_files.clear();
            self.icon_epoch = self.icon_epoch.wrapping_add(1);
            r = Refresh::Panes;
        }
        if self.watcher.changed() {
            if let Some(fresh) = mk::Palette::load() {
                if fresh != self.palette {
                    self.palette = fresh;
                    // colours are not in the cards: every pane anew
                    self.painted.borrow_mut().clear();
                    // The icons carry the palette's colours: draw anew.
                    self.icon_files.clear();
                    self.icon_epoch = self.icon_epoch.wrapping_add(1);
                    r = Refresh::Panes;
                }
            }
        }
        // The OSD chip leaves after its dwell.
        if self.osd.is_some_and(|(_, since)| since.elapsed() >= OSD_DWELL) {
            self.osd = None;
            r = Refresh::Panes;
        }
        // R168c: fold away after three still seconds — never while a
        // finger rests, and never over an open home screen.
        if self.autohide && !self.editing && !self.hidden && self.press.is_none() && self.last_poke.elapsed().as_secs_f32() > IDLE_HIDE_S && !home_open() {
            eprintln!("[dock] faltet sich weg (automatisch ausblenden)");
            self.hidden = true;
            r = Refresh::Panes;
        }
        r
    }

    fn pulse(&mut self) -> Refresh {
        self.pulses = self.pulses.wrapping_add(1);
        // Round 83: the duo folded to one panel (or opened again) — the dock
        // moves to the panel in front; the strip is pinned at start, so it
        // starts anew in place (exec keeps the pid).
        if mk::device::is_duo() {
            let want = mk::device::duo_panel("dock-output", mk::device::DUO_DOCK_PANEL);
            if shell::output_wish().as_deref() != Some(want.as_str()) {
                eprintln!("[dock] Panel wechselt nach «{want}» — Neustart dort");
                let e = mk::process::exec_on_output(Some(&want));
                eprintln!("[dock] Neustart misslungen: {e}");
            }
        }
        // Round 132: behind the lock screen nobody sees the dock — rest.
        if mk::session_lock::active() {
            return Refresh::Calm;
        }
        // A new wallpaper: measure the backdrop again (glass ink).
        if mk::wallpaper::current() != self.backdrop.as_ref().map(|b| b.0.clone()) {
            self.backdrop_ask();
        }
        let mut r = self.settings_check();
        // R169b: an open home screen wakes the folded dock.
        if self.hidden && self.pulses % 3 == 0 && home_open() && self.wake() == Refresh::Panes {
            r = Refresh::Panes;
        }
        if self.widgets.contains(&Widget::Battery) && self.pulses % 30 == 0 {
            let fresh = crate::battery_read();
            if fresh != self.battery {
                self.battery = fresh;
                r = Refresh::Panes;
            }
        }
        if self.widgets.contains(&Widget::Hardline) {
            let on = crate::hardline_running();
            if on != self.hardline_on {
                self.hardline_on = on;
                r = Refresh::Panes;
            }
        }
        if self.pulses % 3 == 0 {
            self.widget_readings();
        }
        // The screen may change (rotation, a new monitor) — the pinned
        // panel's width is asked anew now and then.
        if !self.full_width && self.pulses % 15 == 0 {
            let w = output_width(self.output.as_deref());
            if (w - self.screen_w).abs() > 0.5 {
                self.screen_w = w;
                r = Refresh::Panes;
            }
        }
        r
    }
}

/// The panel the dock belongs on: `--output NAME`, else on a duo the
/// settings key `dock-output` or DSI-1 (iced: mkw::panel::output_option).
fn wished_output() -> Option<String> {
    shell::output_wish().or_else(|| {
        mk::device::is_duo().then(|| mk::device::duo_panel("dock-output", mk::device::DUO_DOCK_PANEL))
    })
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let has = |flag: &str| args.iter().any(|a| a == flag);
    // The house reads the panel from `--output` alone; a duo without one
    // starts itself again with its dock panel named (exec keeps the pid).
    let output = wished_output();
    if let (Some(name), None) = (&output, shell::output_wish()) {
        eprintln!("[dock] Duo: das Dock gehört auf «{name}» — Neustart mit --output");
        restart_with(&["--output", name]);
    }
    // Since the fleet can place its surfaces on a named screen (R216
    // PanePlace), the duo gets the panes form too: real compositor glass
    // per tile, like the bar's chips (02.11.2026: «glass missing in the dock?»).
    // `--strip` keeps the one-surface form with painted bodies.
    let strip = has("--strip") && !has("--panes");
    let reserve = has("--reserve") || has("--reservieren");
    let autohide = mk::settings::read("dock-autohide").as_deref() == Some("on");
    // A reserved edge must be anchored to both sides (layer-shell grants an
    // exclusive zone only to an edge, never to a corner); the panes form
    // always spans the edge (an invisible, click-through strip).
    let full_width = reserve || !strip;
    let tile_h = tile_height() as i32;
    // The reserve ends exactly at the shelf's upper silhouette (R192):
    // lift + tile. The strip floats by its bottom margin, which the
    // compositor adds to the zone (smithay LayerMap::arrange).
    let zone = if reserve && !autohide {
        if strip {
            tile_h
        } else {
            LIFT + tile_h
        }
    } else {
        0
    };
    // The player learns from this how much a taller subtitle strip takes.
    mk::cinema::subtitle::publish_dock_zone(LIFT + tile_h);
    eprintln!(
        "[dock] Matrix Dock im Fensterhaus: Form {}, Ausgang {}, Zone {zone}, pid={}",
        if strip { "Streifen (eine Fläche)" } else { "Flotte (Glas je Kachel)" },
        output.as_deref().unwrap_or("vom Compositor"),
        std::process::id()
    );
    shell::run(
        Dock::new(strip, full_width, autohide, output),
        shell::Options {
            mount: shell::Mount::Layer {
                // The strip paints its own chip bodies: its surface must
                // NOT match a glass rule (a slab behind the tiles, R172d).
                namespace: String::from("matrix-shelf"),
                layer: shell::Layer::Top,
                keyboard: shell::KeyboardWish::None,
                anchor: shell::Anchors { top: false, right: full_width, bottom: true, left: true },
                size: (if full_width { 0 } else { 1 }, 1),
                margin: (0, 0, 0, 0),
                exclusive: zone,
            },
            argb: true,
            tag: "dock",
            fleet_start: SIDE,
            fleet_gap: GAP,
            fleet_flow: shell::fleet::Flow::Across,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win(id: u64, app: &str, focus: bool) -> Win {
        Win { id, app_id: app.to_string(), title: String::new(), focus, shelved: false }
    }

    #[test]
    fn pins_lead_and_running_apps_follow_by_age() {
        let pins = vec![String::from("matrix-settings"), String::from("matrix-files")];
        let windows = vec![win(9, "firefox", false), win(4, "matrix-files", true), win(2, "matrix-terminal", false)];
        let tiles = shelf_tiles(&pins, &windows);
        let ids: Vec<&str> = tiles.iter().map(|t| t.app_id.as_str()).collect();
        assert_eq!(ids, ["matrix-settings", "matrix-files", "matrix-terminal", "firefox"]);
        assert!(!tiles[0].running() && tiles[0].pinned);
        assert!(tiles[1].focus && tiles[1].running());
        assert!(!tiles[2].pinned);
    }

    #[test]
    fn the_focused_app_cycles_its_windows() {
        let tiles = shelf_tiles(&[], &[win(3, "a", false), win(5, "a", true), win(7, "a", false)]);
        assert_eq!(tiles[0].target(), Some(7));
        let calm = shelf_tiles(&[], &[win(3, "b", false), win(5, "b", false)]);
        assert_eq!(calm[0].target(), Some(3));
    }

    #[test]
    fn window_lines_skip_titles_that_repeat_the_name() {
        let mut t = shelf_tiles(&[], &[win(1, "matrix-files", false), win(2, "matrix-files", false)]).remove(0);
        t.windows[0].title = String::from("Matrix Dateien");
        t.windows[1].title = String::from("Bilder");
        assert_eq!(window_lines(&t, "Dateien"), vec![String::from("→ Bilder")]);
    }

    #[test]
    fn the_outline_stays_inside_its_box() {
        for c in rounded_outline(100.0, 56.0, 16.0, 1.0) {
            if let Cmd::Line(x, y) | Cmd::Move(x, y) = c {
                assert!((1.0..=99.0).contains(&x) && (1.0..=55.0).contains(&y));
            }
        }
    }
}
