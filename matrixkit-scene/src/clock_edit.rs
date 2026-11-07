//! CLOCK EDIT FAMILY (round 141, 05.11.2026): the lock's clock made by
//! hand, like a phone's lock screen in its edit mode — a long press on the
//! clock opens it (after the code).
//!
//! - A thin frame around the clock with a CORNER bottom right: dragging it
//!   down makes the digits taller (`stretch`), sideways wider (`resize`,
//!   round 142).
//! - «Abbr.» top left, «Fertig» top right.
//! - A tap on the clock opens a glass SHEET with the WEIGHT slider (thin …
//!   bold); the clock follows the slider live.
//!
//! THE RULE (round 143): the edit mode ADDS, it never moves. The date and
//! the clock lie exactly where the lock shows them, in the same size — the
//! frame, the corner, the bar and the sheet come on top; what may hide is
//! only what lies below the clock (player, notifications). Lay the clock
//! with the same room in both modes.
//!
//! Geometry and hits are plain functions (tested); the app keeps the state
//! and saves the values (`lock-clock-stretch`, `lock-clock-weight`).
//!
//! ```ignore
//! let frame = clock_edit::frame_of(clock);
//! clock_edit::lay_frame(&mut b, frame, fade, home, anchor);
//! let bar = clock_edit::lay_bar(&mut b, engine, wf, 12.0, fade, CANCEL, DONE, home, anchor);
//! let sheet = clock_edit::sheet(wf, bottom);
//! clock_edit::lay_sheet(&mut b, engine, &sheet, weight, map, fade, home, anchor);
//! if clock_edit::on_handle(frame, x, y) { … stretch_for(s0, y - y0, digit_w) }
//! ```

use crate::glass_clock::Look;
use matrixkit_image::Fit;
use matrixkit_shell::shapes::{Cmd, Shape, Style};
use matrixkit_shell::{Pict, SceneBuilder};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::Rect;
use std::f32::consts::PI;
use std::path::Path;

const WHITE: mk::Rgba = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

/// How much wider or narrower than its usual width the clock may get.
pub const SIZE_MIN: f32 = 0.55;
pub const SIZE_MAX: f32 = 1.6;

/// The frame's room around the clock and its corner radius.
pub const PAD: f32 = 10.0;
pub const RADIUS: f32 = 16.0;
/// How near the corner a finger must land to take it.
pub const HANDLE_HIT: f32 = 48.0;
/// The sheet's height and its slider's knob.
pub const SHEET_H: f32 = 132.0;
const PILL_H: f32 = 34.0;

/// The frame around a clock of `clock` (its glyph box).
pub fn frame_of(clock: Rect) -> Rect {
    Rect { x: clock.x - PAD, y: clock.y - PAD, w: clock.w + 2.0 * PAD, h: clock.h + 2.0 * PAD }
}

/// The corner's point: the frame's bottom right.
pub fn handle_at(frame: Rect) -> (f32, f32) {
    (frame.x + frame.w, frame.y + frame.h)
}

/// Does a finger at (x, y) take the corner?
pub fn on_handle(frame: Rect, x: f32, y: f32) -> bool {
    let (hx, hy) = handle_at(frame);
    (x - hx).hypot(y - hy) <= HANDLE_HIT
}

/// The stretch after the corner moved `dy` px down from where it was
/// taken at `stretch0` (1:1 — the digits' height follows the finger).
pub fn stretch_for(stretch0: f32, dy: f32, digit_w: f32) -> f32 {
    (stretch0 + dy / digit_w.max(1.0)).clamp(Look::STRETCH_MIN, Look::STRETCH_MAX)
}

/// Round 142: the corner in BOTH directions. The clock (glyph box `w0`×`h0`
/// at width factor `size0`, centred on its surface) was taken; the finger
/// moved (dx, dy). Width: the right edge follows the finger 1:1 — the clock
/// is centred, so it grows by 2·dx. Height: the bottom follows 1:1. Returns
/// the new width factor and stretch (digit width = `base_w` · size).
pub fn resize(w0: f32, h0: f32, size0: f32, base_w: f32, dx: f32, dy: f32) -> (f32, f32) {
    let size = (size0 * (w0 + 2.0 * dx).max(1.0) / w0.max(1.0)).clamp(SIZE_MIN, SIZE_MAX);
    let digit_w = (base_w * size).max(1.0);
    let stretch = ((h0 + dy) / digit_w).clamp(Look::STRETCH_MIN, Look::STRETCH_MAX);
    (size, stretch)
}

/// The frame and its corner.
pub fn lay_frame(b: &mut SceneBuilder, frame: Rect, fade: f32, home: Rect, anchor: usize) {
    let (w, h) = (frame.w.round().max(8.0), frame.h.round().max(8.0));
    let edge = 1.5;
    let r = RADIUS;
    let shape = Shape::new(w + 8.0, h + 8.0)
        .op(crate::glass_clock::rounded(edge / 2.0, edge / 2.0, w - edge / 2.0, h - edge / 2.0, r), Style::Stroke { tint: WHITE.with_alpha(0.55 * fade), width: edge })
        // The corner: a bold quarter round on the frame's bottom right.
        .op(vec![Cmd::Arc { cx: w - r, cy: h - r, r: r + 2.0, start: 0.0, sweep: PI / 2.0, connect: false }], Style::Stroke { tint: WHITE.with_alpha(0.95 * fade), width: 5.0 });
    b.pict(crate::scene::pict_of(&shape, frame.x, frame.y, home, anchor));
}

/// «Abbr.» and «Fertig» at the top of a surface `wf` wide; returns their
/// targets (roles `cancel` and `done`).
#[allow(clippy::too_many_arguments)]
pub fn lay_bar(b: &mut SceneBuilder, engine: &mut TextEngine, wf: f32, top: f32, fade: f32, cancel: usize, done: usize, home: Rect, anchor: usize) -> (Rect, Rect) {
    let style = mk::typo::CALLOUT;
    let mut pill = |text: &str, right: bool, role: usize| -> Rect {
        let (tw, th) = engine.measure_family(matrixkit_text::FAMILY, text, style.px(), mk::typo::Weight::Semibold);
        let w = tw + 28.0;
        let x = if right { wf - 16.0 - w } else { 16.0 };
        // The target (invisible quad) and the pill as a shape picture: a
        // quad would lie under the wallpaper picture (round 149).
        b.quad(x, top, w, PILL_H, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, PILL_H / 2.0, Some(role), None);
        crate::scene::lay_plate(b, Rect { x, y: top, w, h: PILL_H }, PILL_H / 2.0, WHITE.with_alpha(if right { 0.92 } else { 0.30 } * fade), home, anchor, None);
        let ink = if right { mk::Rgba { r: 0.08, g: 0.08, b: 0.09, a: fade } } else { WHITE.with_alpha(fade) };
        b.text(engine, text, style.px(), mk::typo::Weight::Semibold, matrixkit_text::FAMILY, ink, x + 14.0, top + (PILL_H - th) / 2.0, Some(role), None);
        Rect { x, y: top, w, h: PILL_H }
    };
    let c = pill("Abbr.", false, cancel);
    let d = pill("Fertig", true, done);
    (c, d)
}

/// Where the sheet lies: across a surface `wf` wide, its bottom at `bottom`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sheet {
    pub area: Rect,
    /// The slider's track (the knob's centre runs along it).
    pub track: Rect,
}

pub fn sheet(wf: f32, bottom: f32) -> Sheet {
    let w = (wf - 24.0).min(560.0).floor();
    let area = Rect { x: ((wf - w) / 2.0).floor(), y: (bottom - SHEET_H - 12.0).floor(), w, h: SHEET_H };
    let inset = 52.0;
    let track = Rect { x: area.x + inset, y: area.y + 80.0, w: area.w - 2.0 * inset, h: 6.0 };
    Sheet { area, track }
}

/// The weight under a finger at `x` (0 … 1).
pub fn weight_at(sheet: &Sheet, x: f32) -> f32 {
    crate::scene::slider_value(sheet.track, x)
}

/// Does a finger at (x, y) take the slider (the track's row, generously)?
pub fn on_slider(sheet: &Sheet, x: f32, y: f32) -> bool {
    let t = sheet.track;
    x >= t.x - 30.0 && x <= t.x + t.w + 30.0 && (y - (t.y + t.h / 2.0)).abs() <= 30.0
}

pub fn inside(r: Rect, x: f32, y: f32) -> bool {
    x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
}

/// The sheet's pages (round 147): a STACK like a phone's widget stack —
/// one setting per page, a vertical swipe snaps to the next, dots on the
/// right say where you are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Weight,
    Body,
}

pub const PAGES: [Page; 2] = [Page::Weight, Page::Body];

impl Page {
    pub fn title(self) -> &'static str {
        match self {
            Page::Weight => "Dicke",
            Page::Body => "Deckkraft",
        }
    }

    /// The page's value in `v`.
    pub fn value(self, v: &Values) -> f32 {
        match self {
            Page::Weight => v.weight,
            Page::Body => v.body,
        }
    }

    fn set(self, v: &mut Values, x: f32) {
        match self {
            Page::Weight => v.weight = x,
            Page::Body => v.body = x,
        }
    }
}

/// The sheet: live glass (`map`, a lens card of the sheet's size) or a dark
/// plate; the pages at `pos` (0 = the first, fractions while it snaps), each
/// with its title, its two ends drawn, the slider; the page dots.
#[allow(clippy::too_many_arguments)]
pub fn lay_sheet(b: &mut SceneBuilder, engine: &mut TextEngine, sheet: &Sheet, v: &Values, pos: f32, map: Option<&Path>, fade: f32, home: Rect, anchor: usize) {
    let a = sheet.area;
    match map {
        Some(map) => b.pict(Pict { path: map.to_path_buf(), box_w: a.w as u32, box_h: a.h as u32, fit: Fit::Stretch, x: a.x, y: a.y, radius: 26.0, home, anchor, clip: None }),
        None => {
            b.quad(a.x, a.y, a.w, a.h, mk::Rgba { r: 0.10, g: 0.10, b: 0.11, a: 0.82 * fade }, 26.0, None, None);
        }
    }
    let clip = Some(a);
    let mut st = crate::stack::Stack::new(PAGES.len());
    st.pos = pos;
    for (k, off) in st.visible(a.h) {
        let page = &PAGES[k];
        let ts = mk::typo::HEADLINE;
        let title = page.title();
        let (tw, _) = engine.measure_family(matrixkit_text::FAMILY, title, ts.px(), mk::typo::Weight::Semibold);
        b.text(engine, title, ts.px(), mk::typo::Weight::Semibold, matrixkit_text::FAMILY, WHITE.with_alpha(fade), a.x + (a.w - tw) / 2.0, a.y + 20.0 + off, None, clip);
        let t = Rect { y: sheet.track.y + off, ..sheet.track };
        let cy = t.y + t.h / 2.0;
        for end in 0..2usize {
            let shape = match page {
                // The clock's own digit at both weights: the meaning, drawn.
                Page::Weight => {
                    let look = Look { digit_w: 13.0, stretch: 2.0, glass: 0.0, weight: end as f32, body: Look::BODY, ink: WHITE };
                    let (mut shape, _, _) = crate::glass_clock::glass_shape("8", &look);
                    for op in &mut shape.ops {
                        if let Style::Stroke { tint, .. } = &mut op.style {
                            *tint = WHITE.with_alpha(0.85 * fade);
                        }
                    }
                    shape
                }
                // A clear ring and a solid disc.
                Page::Body => {
                    let shape = Shape::new(22.0, 22.0);
                    if end == 0 {
                        shape.op(vec![Cmd::Circle(11.0, 11.0, 9.0)], Style::Stroke { tint: WHITE.with_alpha(0.85 * fade), width: 1.6 })
                    } else {
                        shape.op(vec![Cmd::Circle(11.0, 11.0, 10.0)], Style::Fill(WHITE.with_alpha(0.9 * fade)))
                    }
                }
            };
            let gx = if end == 0 { a.x + 18.0 } else { a.x + a.w - 30.0 - shape.w };
            let mut pict = crate::scene::pict_of(&shape, gx, cy - shape.h / 2.0, home, anchor);
            pict.clip = clip;
            b.pict(pict);
        }
        crate::scene::lay_glass_slider(b, t, page.value(v), fade, clip, home, anchor);
    }
    crate::stack::lay_dots(b, a, pos, PAGES.len(), fade, home, anchor);
}

/// The clock's adjustable values (what «Fertig» saves).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Values {
    pub stretch: f32,
    pub weight: f32,
    /// The width against the usual width (`SIZE_MIN` … `SIZE_MAX`).
    pub width: f32,
    /// Round 147: the glass' body (`Look::body`).
    pub body: f32,
}

impl Values {
    /// The settings keys (`~/.config/matrix/…`).
    pub const KEYS: [&'static str; 4] = ["lock-clock-stretch", "lock-clock-weight", "lock-clock-width", "lock-clock-body"];

    /// The lock clock as the user made it (defaults where nothing is saved).
    pub fn load() -> Values {
        Self::from_lookup(|k| mk::settings::read(k))
    }

    /// Round 166: from any source of settings — the login screen reads the
    /// person's shared look (mk::login_share) instead of its own.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Values {
        let read = |k: &str, d: f32, lo: f32, hi: f32| get(k).and_then(|s| s.trim().parse::<f32>().ok()).filter(|v| v.is_finite()).unwrap_or(d).clamp(lo, hi);
        Values {
            stretch: read(Self::KEYS[0], 1.9, Look::STRETCH_MIN, Look::STRETCH_MAX),
            weight: read(Self::KEYS[1], Look::WEIGHT, 0.0, 1.0),
            width: read(Self::KEYS[2], 1.0, SIZE_MIN, SIZE_MAX),
            body: read(Self::KEYS[3], Look::BODY, 0.0, 1.0),
        }
    }

    /// «Fertig»: keep them.
    pub fn save(&self) {
        for (k, v) in Self::KEYS.iter().zip([self.stretch, self.weight, self.width, self.body]) {
            mk::settings::write(k, &format!("{v:.3}"));
        }
    }
}

/// What a finger is holding in the edit mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Grab {
    /// The frame's corner (round 142: both ways).
    Corner { x0: f32, y0: f32, w0: f32, h0: f32, size0: f32, base_w: f32 },
    /// The slider of the page shown.
    Slider,
}

/// What a touch did.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    /// Nothing to redraw.
    Calm,
    /// A value or the sheet changed: redraw.
    Changed,
    /// «Abbr.»: the values are back where they were.
    Cancelled,
    /// «Fertig»: save the values.
    Done,
}

/// Where the edit mode's targets lie this frame (on the touched surface).
#[derive(Debug, Clone, Copy, Default)]
pub struct Targets {
    /// The clock's glyph box and its digit width.
    pub clock: Option<(Rect, f32)>,
    /// The sheet, when open and on this surface.
    pub sheet: Option<Sheet>,
    /// The roles of «Abbr.» and «Fertig».
    pub cancel: usize,
    pub done: usize,
}

/// THE EDIT SESSION (round 146, from the lock): the whole touch logic of the
/// edit mode — corner, slider, tap on the clock, «Abbr.»/«Fertig» — apart
/// from any surface, tested. The app lays the targets and passes them in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Session {
    pub start: Values,
    pub sheet: bool,
    pub grab: Option<Grab>,
    /// Round 148: the sheet's pages (kit family stack).
    pub stack: crate::stack::Stack,
}

impl Session {
    pub fn new(start: Values) -> Session {
        Session { start, sheet: false, grab: None, stack: crate::stack::Stack::new(PAGES.len()) }
    }

    /// How the live clock picture keeps up (`glass_clock::Live`).
    pub fn pace(&self) -> crate::glass_clock::Pace {
        match self.grab {
            Some(Grab::Corner { .. }) => crate::glass_clock::Pace::Stretch,
            // The weight changes the shape: a draft. The body is a lens
            // value only: still (no new map at all).
            Some(Grab::Slider) if PAGES[self.stack.page] == Page::Weight => crate::glass_clock::Pace::Draft,
            _ => crate::glass_clock::Pace::Still,
        }
    }

    pub fn strike(&mut self, v: &mut Values, role: Option<usize>, x: f32, y: f32, t: &Targets) -> Outcome {
        if role == Some(t.cancel) {
            *v = self.start;
            return Outcome::Cancelled;
        }
        if role == Some(t.done) {
            return Outcome::Done;
        }
        if self.sheet {
            if let Some(sh) = t.sheet {
                if self.stack.settled() && on_slider(&sh, x, y) {
                    self.grab = Some(Grab::Slider);
                    PAGES[self.stack.page].set(v, weight_at(&sh, x));
                    return Outcome::Changed;
                }
                if inside(sh.area, x, y) {
                    // Anywhere else on the sheet: the pages follow the finger.
                    self.stack.press(y);
                    return Outcome::Calm;
                }
            }
        }
        if let Some((r, digit_w)) = t.clock {
            let frame = frame_of(r);
            if on_handle(frame, x, y) {
                self.grab = Some(Grab::Corner { x0: x, y0: y, w0: r.w, h0: r.h, size0: v.width, base_w: digit_w / v.width.max(0.01) });
                return Outcome::Calm;
            }
            if inside(frame, x, y) {
                self.sheet = !self.sheet;
                return Outcome::Changed;
            }
        }
        if self.sheet {
            self.sheet = false;
            return Outcome::Changed;
        }
        Outcome::Calm
    }

    pub fn travel(&mut self, v: &mut Values, x: f32, y: f32, sheet: Option<Sheet>) -> Outcome {
        match self.grab {
            Some(Grab::Corner { x0, y0, w0, h0, size0, base_w }) => {
                let (width, stretch) = resize(w0, h0, size0, base_w, x - x0, y - y0);
                v.width = width;
                v.stretch = stretch;
                Outcome::Changed
            }
            Some(Grab::Slider) => {
                if let Some(sh) = sheet {
                    PAGES[self.stack.page].set(v, weight_at(&sh, x));
                }
                Outcome::Changed
            }
            None if self.stack.held() => {
                self.stack.drag(y, sheet.map(|s| s.area.h).unwrap_or(SHEET_H));
                Outcome::Changed
            }
            None => Outcome::Calm,
        }
    }

    /// The finger lifted: a grab ends (the clock is made sharp again).
    pub fn release(&mut self) -> Outcome {
        if self.stack.release().is_some() || self.grab.take().is_some() {
            Outcome::Changed
        } else {
            Outcome::Calm
        }
    }

    /// The stack glides to its page; true while it moves.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.stack.tick(dt)
    }
}

#[cfg(test)]
mod session_tests {
    use super::*;

    fn targets(sheet_open: bool) -> Targets {
        Targets { clock: Some((Rect { x: 100.0, y: 80.0, w: 300.0, h: 120.0 }, 60.0)), sheet: sheet_open.then(|| sheet(500.0, 900.0)), cancel: 200, done: 201 }
    }

    #[test]
    fn the_corner_resizes_and_letting_go_sharpens() {
        let start = Values { stretch: 2.0, weight: 0.5, width: 1.0, body: Look::BODY };
        let mut v = start;
        let mut s = Session::new(start);
        let (hx, hy) = handle_at(frame_of(targets(false).clock.unwrap().0));
        assert_eq!(s.strike(&mut v, None, hx, hy, &targets(false)), Outcome::Calm);
        assert_eq!(s.pace(), crate::glass_clock::Pace::Stretch);
        assert_eq!(s.travel(&mut v, hx + 30.0, hy + 60.0, None), Outcome::Changed);
        assert!((v.width - 1.2).abs() < 1e-4 && v.stretch > 2.0);
        assert_eq!(s.release(), Outcome::Changed);
        assert_eq!(s.pace(), crate::glass_clock::Pace::Still);
        // «Abbr.» brings everything back.
        assert_eq!(s.strike(&mut v, Some(200), 0.0, 0.0, &targets(false)), Outcome::Cancelled);
        assert_eq!(v, start);
    }

    #[test]
    fn a_tap_on_the_clock_opens_the_sheet_and_the_slider_drafts() {
        let start = Values { stretch: 2.0, weight: 0.5, width: 1.0, body: Look::BODY };
        let mut v = start;
        let mut s = Session::new(start);
        assert_eq!(s.strike(&mut v, None, 250.0, 140.0, &targets(false)), Outcome::Changed);
        assert!(s.sheet);
        let sh = targets(true).sheet.unwrap();
        s.strike(&mut v, None, sh.track.x, sh.track.y, &targets(true));
        assert_eq!(s.pace(), crate::glass_clock::Pace::Draft);
        assert_eq!(v.weight, 0.0);
        s.travel(&mut v, sh.track.x + sh.track.w, sh.track.y, Some(sh));
        assert_eq!(v.weight, 1.0);
        s.release();
        // Swipe the stack up a third of a page: the body page; its slider
        // changes the body, without a new map (pace Still).
        let mid = sh.area.y + sh.area.h * 0.3;
        s.strike(&mut v, None, sh.area.x + 20.0, mid, &targets(true));
        s.travel(&mut v, sh.area.x + 20.0, mid - sh.area.h / 3.0, Some(sh));
        assert_eq!(s.release(), Outcome::Changed);
        assert_eq!(s.stack.page, 1);
        while s.tick(0.016) {}
        assert_eq!(s.stack.pos, 1.0);
        s.strike(&mut v, None, sh.track.x + sh.track.w, sh.track.y, &targets(true));
        assert_eq!(v.body, 1.0);
        assert_eq!(v.weight, 1.0, "the weight stays");
        assert_eq!(s.pace(), crate::glass_clock::Pace::Still);
        s.release();
        // A tap beside everything closes the sheet; «Fertig» says done.
        assert_eq!(s.strike(&mut v, None, 5.0, 5.0, &targets(true)), Outcome::Changed);
        assert!(!s.sheet);
        assert_eq!(s.strike(&mut v, Some(201), 0.0, 0.0, &targets(false)), Outcome::Done);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_corner_is_taken_near_the_frame_bottom_right() {
        let f = frame_of(Rect { x: 100.0, y: 80.0, w: 300.0, h: 200.0 });
        assert_eq!(handle_at(f), (410.0, 290.0));
        assert!(on_handle(f, 400.0, 285.0));
        assert!(on_handle(f, 440.0, 320.0));
        assert!(!on_handle(f, 250.0, 180.0), "the clock's middle is not the corner");
    }

    #[test]
    fn the_corner_stretches_one_to_one_within_bounds() {
        // 60 px digits: 30 px down = half a digit taller.
        assert!((stretch_for(2.0, 30.0, 60.0) - 2.5).abs() < 1e-5);
        assert_eq!(stretch_for(2.0, -600.0, 60.0), Look::STRETCH_MIN);
        assert_eq!(stretch_for(2.0, 600.0, 60.0), Look::STRETCH_MAX);
    }

    #[test]
    fn the_corner_resizes_both_ways() {
        // 4 digits of 60 px ≈ 300 px wide, 120 px high at stretch 2.
        let (size, stretch) = resize(300.0, 120.0, 1.0, 60.0, 30.0, 0.0);
        assert!((size - 1.2).abs() < 1e-5, "right edge +30 → 60 px wider (centred)");
        assert!((stretch - 120.0 / 72.0).abs() < 1e-4, "the height stays put while it widens");
        let (size, stretch) = resize(300.0, 120.0, 1.0, 60.0, 0.0, 60.0);
        assert_eq!(size, 1.0);
        assert!((stretch - 3.0).abs() < 1e-5);
        // Pulled tall, then narrow: the height stays (round 144).
        let (size, stretch) = resize(300.0, 120.0, 1.0, 60.0, 0.0, 72.0);
        let tall = 60.0 * size * stretch;
        let (size2, stretch2) = resize(300.0 * size, tall, size, 60.0, -120.0, 0.0);
        assert_eq!(size2, SIZE_MIN);
        assert!((60.0 * size2 * stretch2 - tall).abs() < 0.5, "height {tall} kept");
        // Wide and low: the height stays too.
        let (size3, stretch3) = resize(300.0, 90.0, 1.0, 60.0, 90.0, 0.0);
        assert!((60.0 * size3 * stretch3 - 90.0).abs() < 0.5);
        let (size, _) = resize(300.0, 120.0, 1.0, 60.0, -1000.0, 0.0);
        assert_eq!(size, SIZE_MIN);
        let (size, _) = resize(300.0, 120.0, 1.0, 60.0, 1000.0, 0.0);
        assert_eq!(size, SIZE_MAX);
    }

    #[test]
    fn the_slider_runs_zero_to_one() {
        let s = sheet(400.0, 800.0);
        assert!(s.area.y + s.area.h <= 800.0);
        assert!(s.track.y > s.area.y && s.track.y < s.area.y + s.area.h);
        assert_eq!(weight_at(&s, s.track.x - 50.0), 0.0);
        assert_eq!(weight_at(&s, s.track.x + s.track.w + 50.0), 1.0);
        assert!((weight_at(&s, s.track.x + s.track.w / 2.0) - 0.5).abs() < 1e-5);
        assert!(on_slider(&s, s.track.x + 10.0, s.track.y + 10.0));
        assert!(!on_slider(&s, s.track.x + 10.0, s.area.y + 10.0), "the title is not the slider");
    }
}
