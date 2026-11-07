//! Matrix Hilfe (help), first-light version (iced replacement, Lager 28 —
//! GRABUNG-ICED-ABLOESUNG.md): the sixth app move — the READER.
//!
//! The knowledge base (contents.rs) stays verbatim; only the way
//! onto the screen changes: sidebar with categories
//! (symbol + counter + selection pill), card overview, full-text
//! search over all chapters, and the READING VIEW — the first long
//! flowing text of the window house: TextEngine::wrap breaks the
//! article to the column width (Lager 27), the wheel scrolls it
//! through the view box (the viewport pattern from Lager 14: the
//! text's measure box moves with the offset, the diff sees
//! the movement, the clip cuts the glyphs).
//!
//! Since Lager 29 the cards and the hit list SCROLL too: the
//! quad view box (Placed.clip) cuts them at the edge —
//! rasteriser, hit test and diff hold the same box.
//!
//! Since Lager 31 the help carries the FOCUS MODEL: mkw::Focus
//! (the same state logic as in the iced path), Tab wanders through
//! categories and cards, Shift+Tab backwards, Enter acts, Escape
//! clears — the ring is LAYOUT (fam::ring, 2 px mantle).
//!
//! Since Lager 41 scrolling is SPRUNG: the wheel sets targets, the
//! scroll spring moves there — notch upon notch adds momentum, the
//! view box cuts as before.
//!
//! Since Lager 32 articles GLIDE in: the reading spring
//! (mk::motion::Spring, verbatim that of the iced path) hangs on the
//! house's frame clock — animating()/glide() run only
//! while the spring swings; when it rests, the clock falls silent.
//!
//! Notes: --suche/HelpLink CLI in the light path, auto-scroll
//! to the focused element.

use crate::contents::CATEGORIES;
use matrixkit_shell::{self as shell, Refresh, Scene, Source, Text};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{lay_out, Keep, Node, Placed, Rect};
use matrixkit_widgets as mkw;
use matrixkit_widgets::scene as fam;

const WIDTH: u32 = 760;
const HEIGHT: u32 = 540;
const SIDEBAR_W: f32 = 200.0;
const SEARCH_W: f32 = 240.0;
/// Roles from here on are the frame traffic lights.
const FRAME_BASE: usize = 900;
/// Roles from here on belong to the root layer (Lager 94).
const ROOT_BASE: usize = 1200;
/// What a hit means (the deed pattern from Lager 26).
enum Deed {
    Category(usize),
    Article(usize),
    Hit(usize, usize),
    Back,
    HistoryBack,
    HistoryForward,
    SearchFocus,
}

struct Help {
    palette: mk::Palette,
    category: usize,
    article: Option<usize>,
    history: Vec<(usize, Option<usize>)>,
    forward: Vec<(usize, Option<usize>)>,
    search: String,
    cursor: usize,
    anchor: Option<usize>,
    search_focus: bool,
    blink: bool,
    frame_active: bool,
    lights_hover: bool,
    /// The MAGNIFIER of the window bar (Lager 91).
    head_zoom: fam::HeaderMotion,
    /// The ROOT LAYER (Lager 94).
    root: fam::RootPane,
    /// Is the window ROLLED UP into its bar? (window key
    /// on the right — the reference carries it in every window bar.)
    rolled: bool,
    /// Is the window MAXIMISED? (Lager 92 — the house reports it.)
    zoomed: bool,
    /// Last painted window size and bar height — the
    /// window key rolls up onto it.
    last_w: u32,
    last_h: u32,
    head_h: u32,
    /// The size BEFORE rolling up — handed back once when
    /// unrolling. Without it the window stayed small: extent()
    /// returned None, and the window house then does nothing
    /// (measured on 16.08.2026, real click via uinput).
    before: Option<(u32, u32)>,
    frame_lights: Rect,
    /// The reader's view box and its scroll range.
    pane: Rect,
    roll_max: f32,
    /// The SCROLL SPRING (Lager 41): the wheel sets targets, the spring
    /// moves there — soft scrolling instead of hard jumps.
    roll: mk::motion::Spring,
    /// On the next view() set the offset to 0 (article change).
    roll_reset: bool,
    deeds: Vec<Deed>,
    /// The focus model (Lager 31): mkw::Focus is pure
    /// state logic — the same as in the iced path. focus_deeds
    /// translates the focus position into the deed of the last scene.
    focus: mkw::Focus,
    focus_deeds: Vec<usize>,
    /// The READING SPRING (Lager 32): articles glide in —
    /// the same mk::motion::Spring as in the iced path. The counter
    /// proves that the clock really falls silent after resting.
    read_spring: mk::motion::Spring,
    glides: u32,
}

impl Help {
    fn new() -> Self {
        // Memory like the iced version: the help opens where it
        // was closed ("k" or "k,a", clamped against what
        // exists).
        let (category, article) = Self::state_read();
        Help {
            palette: mk::Palette::load().unwrap_or_default(),
            category,
            article,
            history: Vec::new(),
            forward: Vec::new(),
            search: String::new(),
            cursor: 0,
            anchor: None,
            search_focus: false,
            blink: true,
            frame_active: true,
            lights_hover: false,
            head_zoom: fam::HeaderMotion::new(),
            root: fam::RootPane::new("matrix-help", &[]),
            rolled: false,
            zoomed: false,
            last_w: WIDTH,
            last_h: HEIGHT,
            head_h: 50,
            before: None,
            frame_lights: Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 },
            pane: Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 },
            roll_max: 0.0,
            roll: mk::motion::Spring::smooth(0.0),
            roll_reset: false,
            deeds: Vec::new(),
            focus: mkw::Focus::new(0),
            focus_deeds: Vec::new(),
            read_spring: mk::motion::Spring::smooth(1.0),
            glides: 0,
        }
    }

    fn state_read() -> (usize, Option<usize>) {
        let Some(value) = mk::settings::read("help-state") else {
            return (0, None);
        };
        let mut parts = value.splitn(2, ',');
        let k = parts
            .next()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0)
            .min(CATEGORIES.len().saturating_sub(1));
        let a = parts
            .next()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&a| a < CATEGORIES[k].article.len());
        (k, a)
    }

    fn state_write(&self) {
        let value = match self.article {
            Some(a) => format!("{},{}", self.category, a),
            None => format!("{}", self.category),
        };
        mk::settings::write("help-state", &value);
    }

    /// Jump to a place and put the old one into the history.
    fn go(&mut self, category: usize, article: Option<usize>) {
        self.history.push((self.category, self.article));
        self.forward.clear();
        self.category = category;
        let opened = article.is_some() && article != self.article;
        self.article = article;
        self.roll_reset = true;
        self.focus.delete();
        if opened {
            // The spring tenses: the article glides in.
            // LICHT_ZEITLUPE stretches the probe (evidence photography).
            self.read_spring = if std::env::var("LICHT_ZEITLUPE").is_ok() {
                mk::motion::Spring::with(0.0, 5.0, 0.0)
            } else {
                mk::motion::Spring::smooth(0.0)
            };
            self.read_spring.retarget(1.0);
            self.glides = 0;
        }
        self.state_write();
    }

    /// Perform a deed — called by click (strike) AND key (Enter):
    /// one route, no drift.
    fn act(&mut self, r: usize, origin: &str) -> Refresh {
        let Some(deed) = self.deeds.get(r) else { return Refresh::Calm };
        match deed {
            Deed::Category(i) => {
                let i = *i;
                eprintln!("[hilfe] Tat ({origin}): Kategorie „{}“", CATEGORIES[i].name);
                self.go(i, None);
            }
            Deed::Article(i) => {
                let i = *i;
                eprintln!(
                    "[hilfe] Tat ({origin}): Artikel „{}“",
                    CATEGORIES[self.category].article[i].title
                );
                self.go(self.category, Some(i));
            }
            Deed::Hit(k, a) => {
                let (k, a) = (*k, *a);
                eprintln!(
                    "[hilfe] Tat ({origin}): Fund „{}“ in „{}“",
                    CATEGORIES[k].article[a].title,
                    CATEGORIES[k].name
                );
                self.search.clear();
                self.cursor = 0;
                self.anchor = None;
                self.go(k, Some(a));
            }
            Deed::Back => {
                eprintln!("[hilfe] Tat ({origin}): zurück zur Übersicht");
                self.go(self.category, None);
            }
            Deed::HistoryBack => {
                if let Some((k, a)) = self.history.pop() {
                    eprintln!("[hilfe] Geschichte: ‹ zurück");
                    self.forward.push((self.category, self.article));
                    self.category = k;
                    self.article = a;
                    self.roll_reset = true;
                    self.state_write();
                }
            }
            Deed::HistoryForward => {
                if let Some((k, a)) = self.forward.pop() {
                    eprintln!("[hilfe] Geschichte: › vor");
                    self.history.push((self.category, self.article));
                    self.category = k;
                    self.article = a;
                    self.roll_reset = true;
                    self.state_write();
                }
            }
            Deed::SearchFocus => {
                eprintln!("[hilfe] Tat ({origin}): das Suchfeld hört zu");
                self.search_focus = true;
            }
        }
        Refresh::Dirty
    }
}

/// A text run at a freshly laid measure box.
#[allow(clippy::too_many_arguments)]
fn put_text(
    b: &mut Build,
    s: String,
    style: mk::typo::TextStyle,
    family: &'static str,
    color: mk::Rgba,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    role: Option<usize>,
    clip: Option<Rect>,
) {
    let clear = mk::Rgba { a: 0.0, ..color };
    let first = b.scene.len();
    let mut placed = Vec::new();
    lay_out(&Node::Quad { w, h, color: clear, radius: 0.0 }, x, y, &mut placed);
    b.scene.extend(placed);
    b.roles.push(role);
    b.texts.push(Text {
        s,
        px: style.px(),
        weight: style.weight,
        family,
        color,
        x,
        y,
        home: clip.unwrap_or(Rect::of(&b.scene[first])),
        anchor: first,
        clip,
    });
}

#[derive(Default)]
struct Build {
    scene: Vec<Placed>,
    roles: Vec<Option<usize>>,
    texts: Vec<Text>,
}

impl shell::Client for Help {
    type Card = ();
    type Work = ();

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn base(&mut self, _atlas: &mut matrixkit_image::ImageAtlas, buf: &mut [u8], _w: u32, _h: u32) {
        let p = self.palette;
        let (bb, g, r) = (
            (p.surface.b * 255.0) as u8,
            (p.surface.g * 255.0) as u8,
            (p.surface.r * 255.0) as u8,
        );
        for px in buf.chunks_exact_mut(4) {
            px[0] = bb;
            px[1] = g;
            px[2] = r;
            px[3] = 0xff;
        }
    }

    fn view(
        &mut self,
        engine: &mut TextEngine,
        keep: &mut Keep<f32>,
        _atlas: &mut matrixkit_image::ImageAtlas,
        w: u32,
        h: u32,
    ) -> Scene {
        let p = self.palette;
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let symfam = mkw::symbol_family();
        let mut b = Build::default();
        self.deeds = Vec::new();
        self.focus_deeds = Vec::new();
        if self.roll_reset {
            self.roll_reset = false;
            self.roll = mk::motion::Spring::smooth(0.0);
        }
        let off = self.roll.value().max(0.0);
        let _ = keep;

        // The head (frame family, like Mandelbrot).
        let header = fam::frame_header(
            engine,
            "Matrix Hilfe",
            fam::HeaderState {
                active: self.frame_active,
                lights_hover: self.lights_hover ,
                rolled_up: self.rolled,
                magnify: self.head_zoom.factors(),
            },
            w as f32,
            p,
        );
        let mut head_placed = Vec::new();
        lay_out(&header.node, 0.0, 0.0, &mut head_placed);
        self.head_zoom.learn(&header, &head_placed, 0.0, 0.0);
        // The window key rolls up onto THIS bar.
        self.last_w = w;
        self.last_h = h;
        self.head_h = header.height.ceil() as u32;
        let mut head_roles: Vec<Option<usize>> = vec![None; head_placed.len()];
        for (k, slot) in header.light_slots.iter().enumerate() {
            head_roles[*slot] = Some(FRAME_BASE + k);
        }
        // The window key gets the fourth frame role.
        head_roles[header.shade_slot] = Some(FRAME_BASE + 3);
        // The TITLE is a button (Lager 94): root layer.
        head_roles[header.title_slot] = Some(FRAME_BASE + 4);
        b.roles.extend(head_roles);
        b.scene.extend(head_placed);
        // The WINDOW KEY on the right (roll up/unroll).
        let sq = b.scene[header.shade_slot];
        b.texts.push(Text {
            s: header.shade.text.clone(),
            px: header.shade.px,
            weight: header.shade.weight,
            family: header.shade.family,
            color: header.shade.color,
            x: sq.x,
            y: sq.y,
            home: Rect::of(&sq),
            anchor: header.shade_slot,
            clip: None,
        });
        let tq = b.scene[header.title_slot];
        b.texts.push(Text {
            s: header.title.text.clone(),
            px: header.title.px,
            weight: header.title.weight,
            family: header.title.family,
            color: header.title.color,
            x: tq.x,
            y: tq.y,
            home: Rect::of(&b.scene[0]),
            anchor: header.title_slot,
            clip: None,
        });
        for (k, (g, dx, dy)) in header.glyphs.iter().enumerate() {
            let lq = b.scene[header.light_slots[k]];
            b.texts.push(Text {
                s: g.text.clone(),
                px: g.px,
                weight: g.weight,
                family: g.family,
                color: g.color,
                x: lq.x + dx,
                y: lq.y + dy,
                home: Rect::of(&b.scene[0]),
                anchor: header.light_slots[k],
                clip: None,
            });
        }
        let l0 = b.scene[header.light_slots[0]];
        let l2 = b.scene[header.light_slots[2]];
        self.frame_lights = Rect {
            x: l0.x - 2.0,
            y: l0.y - 2.0,
            w: (l2.x + l2.w) - l0.x + 4.0,
            h: l0.h + 4.0,
        };
        // The bar reports its OWN height — it grows with the
        // font size setting, the fixed SHADE_HEIGHT does
        // not (finger probe 16.08.2026).
        let top = header.height;

        // The SIDEBAR: ground, then per category a row with
        // symbol, name, counter — the chosen one carries the pill.
        let mut placed = Vec::new();
        lay_out(
            &Node::Quad {
                w: SIDEBAR_W,
                h: h as f32 - top,
                color: p.surface_container,
                radius: 0.0,
            },
            0.0,
            top,
            &mut placed,
        );
        b.scene.extend(placed);
        b.roles.push(None);
        let symbols = [
            mkw::symbol::ROCKET_LAUNCH,
            mkw::symbol::TOUCH_APP,
            mkw::symbol::SHIELD,
            mkw::symbol::PALETTE,
            mkw::symbol::CODE,
        ];
        let mut sy = top + mk::spacing::M;
        for (i, k) in CATEGORIES.iter().enumerate() {
            let picked = i == self.category;
            let glyph = symbols[i.min(symbols.len() - 1)].to_string();
            let (gw, gh) = engine.measure_family(symfam, &glyph, mk::typo::symbol_px(18.0), mk::typo::Weight::Normal);
            let name = k.name.to_string();
            let body = mk::typo::BODY;
            let (nw, nh) = engine.measure_styled(&name, body.px(), body.weight);
            let count = format!("{}", k.article.len());
            let cap = mk::typo::CAPTION;
            let (cw, chh) = engine.measure_styled(&count, cap.px(), cap.weight);
            let inner = SIDEBAR_W - 2.0 * mk::spacing::S - 2.0 * mk::spacing::S;
            let node = Node::Wrap {
                pad_x: mk::spacing::S,
                pad_y: 7.0,
                background: Some((
                    if picked { p.primary_container } else { clear },
                    mk::radius::SMALL + 2.0,
                )),
                child: Box::new(Node::Row {
                    gap: mk::spacing::S,
                    padding: 0.0,
                    background: None,
                    children: vec![
                        Node::Quad { w: gw, h: gh, color: clear, radius: 0.0 },
                        Node::Quad { w: nw, h: nh, color: clear, radius: 0.0 },
                        Node::Quad {
                            w: (inner - gw - nw - cw - 2.0 * mk::spacing::S).max(0.0),
                            h: 0.0,
                            color: clear,
                            radius: 0.0,
                        },
                        Node::Quad { w: cw, h: chh, color: clear, radius: 0.0 },
                    ],
                }),
            };
            let deed_i = self.deeds.len();
            self.deeds.push(Deed::Category(i));
            let fi = self.focus_deeds.len();
            self.focus_deeds.push(deed_i);
            let ringed = self.focus.is(fi);
            let node = fam::ring(node, ringed, mk::radius::SMALL + 2.0, p);
            let r = if ringed { 1 } else { 0 };
            let ro = if ringed { 2.0 } else { 0.0 };
            let first = b.scene.len();
            let mut placed = Vec::new();
            let (_, rh0) = lay_out(&node, mk::spacing::S - ro, sy - ro, &mut placed);
            let rh = rh0 - 2.0 * ro;
            let n = placed.len();
            b.scene.extend(placed);
            b.roles.extend(std::iter::repeat(Some(deed_i)).take(n));
            let home = Rect::of(&b.scene[first]);
            let (tint, dim) = if picked {
                (p.on_primary_container, p.on_primary_container)
            } else {
                (p.on_surface, p.on_surface_variant)
            };
            let gq = b.scene[first + r + 1];
            b.texts.push(Text {
                s: glyph,
                px: mk::typo::symbol_px(18.0),
                weight: mk::typo::Weight::Normal,
                family: symfam,
                color: if picked { p.on_primary_container } else { p.on_surface_variant },
                x: gq.x,
                y: gq.y,
                home,
                anchor: first + r + 1,
                clip: None,
            });
            let nq = b.scene[first + r + 2];
            b.texts.push(Text {
                s: name,
                px: body.px(),
                weight: body.weight,
                family: matrixkit_text::FAMILY,
                color: tint,
                x: nq.x,
                y: nq.y,
                home,
                anchor: first + r + 2,
                clip: None,
            });
            let cq = b.scene[first + r + 4];
            b.texts.push(Text {
                s: count,
                px: cap.px(),
                weight: cap.weight,
                family: matrixkit_text::FAMILY,
                color: dim,
                x: cq.x,
                y: cq.y + (nh - chh) / 2.0,
                home,
                anchor: first + r + 4,
                clip: None,
            });
            sy += rh + mk::spacing::XS;
        }

        // The head row of the surface: ‹ › (history) and the
        // search field on the right.
        let px0 = SIDEBAR_W + mk::spacing::L;
        let pane_w = w as f32 - px0 - mk::spacing::L;
        let mut hx = px0;
        let hy = top + mk::spacing::M;
        for (glyph, active, deed) in [
            (mkw::symbol::ARROW_BACK, !self.history.is_empty(), Deed::HistoryBack),
            (mkw::symbol::CHEVRON_RIGHT, !self.forward.is_empty(), Deed::HistoryForward),
        ] {
            let deed_i = self.deeds.len();
            self.deeds.push(deed);
            let (node, l) = fam::tool_button(engine, glyph, active, fam::Status::Idle, p);
            let first = b.scene.len();
            let mut placed = Vec::new();
            let (bw2, _) = lay_out(&node, hx, hy, &mut placed);
            let n = placed.len();
            b.scene.extend(placed);
            b.roles.extend(std::iter::repeat(Some(deed_i)).take(n));
            let lq = b.scene[first + 1];
            b.texts.push(Text {
                s: l.text,
                px: l.px,
                weight: l.weight,
                family: l.family,
                color: l.color,
                x: lq.x,
                y: lq.y,
                home: Rect::of(&b.scene[first]),
                anchor: first + 1,
                clip: None,
            });
            hx += bw2 + 2.0;
        }
        let spec = fam::FieldSpec {
            value: &self.search,
            cursor: self.cursor,
            anchor: self.anchor,
            focused: self.search_focus,
            blink: self.blink,
            placeholder: "Hilfe durchsuchen",
            // The magnifier belongs INSIDE the pill — the reference
            // (mkw::search_field) puts it into the edge as well.
            leading: Some(mkw::symbol::SEARCH),
        };
        let field = fam::input_field(engine, &spec, SEARCH_W, p);
        let deed_i = self.deeds.len();
        self.deeds.push(Deed::SearchFocus);
        let ffirst = b.scene.len();
        let mut fplaced = Vec::new();
        let (_, field_h) = lay_out(&field.node, w as f32 - SEARCH_W - mk::spacing::L, hy - 4.0, &mut fplaced);
        let fn_ = fplaced.len();
        b.scene.extend(fplaced);
        b.roles.extend(std::iter::repeat(Some(deed_i)).take(fn_));
        // With a leading glyph the prefix sits at 6 instead of 4
        // (edge, ground, glyph, air, selection lead, selection,
        // prefix, cursor, rest).
        let text_slot = ffirst + if field.leading.is_some() { 6 } else { 4 };
        if let Some(glyph) = &field.leading {
            let gq = b.scene[ffirst + 2];
            b.texts.push(Text {
                s: glyph.clone(),
                px: mk::typo::symbol_px(mk::font_size::MEDIUM),
                weight: mk::typo::Weight::Normal,
                family: mkw::symbol_family(),
                color: p.on_surface_variant,
                x: gq.x,
                y: gq.y,
                home: Rect::of(&b.scene[ffirst]),
                anchor: ffirst + 2,
                clip: None,
            });
        }
        let aq = b.scene[text_slot];
        b.texts.push(Text {
            s: field.label.text.clone(),
            px: field.label.px,
            weight: field.label.weight,
            family: field.label.family,
            color: field.label.color,
            x: aq.x,
            y: aq.y,
            home: Rect::of(&b.scene[ffirst]),
            anchor: text_slot,
            clip: None,
        });
        let cy = hy + field_h + mk::spacing::S;

        // --- the content: search, cards or the reader ---
        let kat = &CATEGORIES[self.category];
        let card_w = pane_w;
        let inner_w = card_w - 2.0 * mk::spacing::L;
        self.pane = Rect { x: px0, y: cy, w: pane_w, h: h as f32 - cy - mk::spacing::M };
        self.roll_max = 0.0;

        if !self.search.is_empty() {
            // Full text over ALL chapters (like the iced version),
            // capped at MAX_HITS visible cards.
            let request = self.search.to_lowercase();
            let mut hits: Vec<(usize, usize)> = Vec::new();
            for (ki, k) in CATEGORIES.iter().enumerate() {
                for (ai, a) in k.article.iter().enumerate() {
                    if a.title.to_lowercase().contains(&request)
                        || a.teaser.to_lowercase().contains(&request)
                        || a.content.to_lowercase().contains(&request)
                    {
                        hits.push((ki, ai));
                    }
                }
            }
            let title = mk::typo::TITLE;
            let line = if hits.len() == 1 {
                String::from("1 Treffer")
            } else {
                format!("{} Treffer", hits.len())
            };
            let (lw, lh) = engine.measure_styled(&line, title.px(), title.weight);
            put_text(&mut b, line, title, matrixkit_text::FAMILY, p.on_surface, px0, cy, lw, lh, None, None);
            let ly0 = cy + lh + mk::spacing::M;
            let pane = Rect { x: px0, y: ly0, w: pane_w, h: h as f32 - ly0 - mk::spacing::M };
            self.pane = pane;
            let mut y = ly0 - off;
            if hits.is_empty() {
                let bodys = mk::typo::BODY;
                let s = engine.wrap_styled(
                    &format!("Keine Treffer für „{}“ — anders formulieren oder kürzer suchen.", self.search),
                    bodys.px(),
                    bodys.weight,
                    inner_w,
                );
                let (sw, sh) = engine.measure_styled(&s, bodys.px(), bodys.weight);
                put_text(&mut b, s, bodys, matrixkit_text::FAMILY, p.on_surface_variant, px0, y, sw, sh, None, None);
            }
            for &(ki, ai) in hits.iter() {
                let a = &CATEGORIES[ki].article[ai];
                let deed_i = self.deeds.len();
                self.deeds.push(Deed::Hit(ki, ai));
                let label = mk::typo::LABEL;
                let sub = mk::typo::SUBTITLE;
                let bodys = mk::typo::BODY;
                let teaser = engine.wrap_styled(a.teaser, bodys.px(), bodys.weight, inner_w);
                let (kw, kh) = engine.measure_styled(CATEGORIES[ki].name, label.px(), label.weight);
                let (tw2, th2) = engine.measure_styled(a.title, sub.px(), sub.weight);
                let (zw, zh) = engine.measure_styled(&teaser, bodys.px(), bodys.weight);
                let node = Node::Wrap {
                    pad_x: mk::spacing::L,
                    pad_y: mk::spacing::M,
                    background: Some((p.surface_container_high, mk::CORNER_RADIUS)),
                    child: Box::new(Node::Column {
                        gap: 3.0,
                        padding: 0.0,
                        background: None,
                        children: vec![
                            Node::Quad { w: kw, h: kh, color: clear, radius: 0.0 },
                            Node::Quad { w: tw2, h: th2, color: clear, radius: 0.0 },
                            Node::Quad { w: zw, h: zh, color: clear, radius: 0.0 },
                            Node::Quad { w: inner_w, h: 0.0, color: clear, radius: 0.0 },
                        ],
                    }),
                };
                let first = b.scene.len();
                let mut placed = Vec::new();
                let (_, rh) = lay_out(&node, px0, y, &mut placed);
                let n = placed.len();
                b.scene.extend(placed);
                // The card lives only in the view box (Lager 29).
                for q in &mut b.scene[first..] {
                    q.clip = Some(pane);
                }
                b.roles.extend(std::iter::repeat(Some(deed_i)).take(n));
                let home = pane;
                for (idx, (s, st, tint)) in [
                    (a.title.to_string(), sub, p.on_surface),
                    (teaser.clone(), bodys, p.on_surface_variant),
                ]
                .into_iter()
                .enumerate()
                {
                    let q = b.scene[first + 2 + idx];
                    b.texts.push(Text {
                        s,
                        px: st.px(),
                        weight: st.weight,
                        family: matrixkit_text::FAMILY,
                        color: tint,
                        x: q.x,
                        y: q.y,
                        home,
                        anchor: first + 2 + idx,
                        clip: Some(pane),
                    });
                }
                let q = b.scene[first + 1];
                b.texts.push(Text {
                    s: CATEGORIES[ki].name.to_string(),
                    px: label.px(),
                    weight: label.weight,
                    family: matrixkit_text::FAMILY,
                    color: p.primary,
                    x: q.x,
                    y: q.y,
                    home,
                    anchor: first + 1,
                    clip: Some(pane),
                });
                y += rh + mk::spacing::M;
            }
            self.roll_max = ((y + off - ly0) - pane.h).max(0.0);
        } else if let Some(i) = self.article {
            // ------------------------------------------- the READER
            let a = &kat.article[i.min(kat.article.len() - 1)];
            // The back pill: ‹ + category name.
            let deed_i = self.deeds.len();
            self.deeds.push(Deed::Back);
            let fi = self.focus_deeds.len();
            self.focus_deeds.push(deed_i);
            let ringed = self.focus.is(fi);
            let back_glyph = mkw::symbol::ARROW_BACK.to_string();
            let (bgw, bgh) = engine.measure_family(symfam, &back_glyph, mk::typo::symbol_px(16.0), mk::typo::Weight::Normal);
            let body = mk::typo::BODY;
            let (knw, knh) = engine.measure_styled(kat.name, body.px(), body.weight);
            let node = Node::Wrap {
                pad_x: mk::spacing::S,
                pad_y: mk::spacing::XS,
                background: Some((clear, mk::CORNER_RADIUS)),
                child: Box::new(Node::Row {
                    gap: mk::spacing::XS,
                    padding: 0.0,
                    background: None,
                    children: vec![
                        Node::Quad { w: bgw, h: bgh, color: clear, radius: 0.0 },
                        Node::Quad { w: knw, h: knh, color: clear, radius: 0.0 },
                    ],
                }),
            };
            let node = fam::ring(node, ringed, mk::CORNER_RADIUS, p);
            let r = if ringed { 1 } else { 0 };
            let ro = if ringed { 2.0 } else { 0.0 };
            let first = b.scene.len();
            let mut placed = Vec::new();
            let (_, bh0) = lay_out(&node, px0 - ro, cy - ro, &mut placed);
            let bh2 = bh0 - 2.0 * ro;
            let n = placed.len();
            b.scene.extend(placed);
            b.roles.extend(std::iter::repeat(Some(deed_i)).take(n));
            let home = Rect::of(&b.scene[first]);
            let gq = b.scene[first + r + 1];
            b.texts.push(Text {
                s: back_glyph,
                px: mk::typo::symbol_px(16.0),
                weight: mk::typo::Weight::Normal,
                family: symfam,
                color: p.primary,
                x: gq.x,
                y: gq.y,
                home,
                anchor: first + r + 1,
                clip: None,
            });
            let nq = b.scene[first + r + 2];
            b.texts.push(Text {
                s: kat.name.to_string(),
                px: body.px(),
                weight: body.weight,
                family: matrixkit_text::FAMILY,
                color: p.primary,
                x: nq.x,
                y: nq.y,
                home,
                anchor: first + r + 2,
                clip: None,
            });

            let title = mk::typo::TITLE;
            let (tw2, th2) = engine.measure_styled(a.title, title.px(), title.weight);
            // The reading spring's indent: 24 px → 0 (like iced).
            let lf = self.read_spring.value().clamp(0.0, 1.2);
            let indent = (24.0 * (1.0 - lf)).max(0.0);
            let ty = cy + bh2 + mk::spacing::M + indent;
            put_text(&mut b, a.title.to_string(), title, matrixkit_text::FAMILY, p.on_surface, px0, ty, tw2, th2, None, None);

            // The FLOWING TEXT: broken (Lager 27), scrolled
            // (Lager 14) — the measure box moves with the offset,
            // the view box cuts the glyphs.
            let text_y0 = ty + th2 + mk::spacing::M;
            let view = Rect { x: px0, y: text_y0, w: pane_w, h: h as f32 - text_y0 - mk::spacing::L };
            self.pane = view;
            let bodys = mk::typo::BODY;
            let flow = engine.wrap_styled(a.content, bodys.px(), bodys.weight, inner_w);
            let (fw2, fh2) = engine.measure_styled(&flow, bodys.px(), bodys.weight);
            self.roll_max = (fh2 - view.h).max(0.0);
            let off = off.clamp(0.0, self.roll_max);
            put_text(
                &mut b,
                flow,
                bodys,
                matrixkit_text::FAMILY,
                p.on_surface,
                px0,
                text_y0 - off,
                fw2,
                fh2,
                None,
                Some(view),
            );
        } else {
            // ------------------------------------- the CARDS
            let title = mk::typo::TITLE;
            let (kw, kh) = engine.measure_styled(kat.name, title.px(), title.weight);
            put_text(&mut b, kat.name.to_string(), title, matrixkit_text::FAMILY, p.on_surface, px0, cy, kw, kh, None, None);
            let ly0 = cy + kh + mk::spacing::M;
            let pane = Rect { x: px0, y: ly0, w: pane_w, h: h as f32 - ly0 - mk::spacing::M };
            self.pane = pane;
            let mut y = ly0 - off;
            for (i, a) in kat.article.iter().enumerate() {
                let deed_i = self.deeds.len();
                self.deeds.push(Deed::Article(i));
                let fi = self.focus_deeds.len();
                self.focus_deeds.push(deed_i);
                let ringed = self.focus.is(fi);
                let sub = mk::typo::SUBTITLE;
                let bodys = mk::typo::BODY;
                let teaser = engine.wrap_styled(a.teaser, bodys.px(), bodys.weight, inner_w);
                let (tw2, th2) = engine.measure_styled(a.title, sub.px(), sub.weight);
                let (zw, zh) = engine.measure_styled(&teaser, bodys.px(), bodys.weight);
                let node = Node::Wrap {
                    pad_x: mk::spacing::L,
                    pad_y: mk::spacing::M,
                    background: Some((p.surface_container_high, mk::CORNER_RADIUS)),
                    child: Box::new(Node::Column {
                        gap: mk::spacing::XS,
                        padding: 0.0,
                        background: None,
                        children: vec![
                            Node::Quad { w: tw2, h: th2, color: clear, radius: 0.0 },
                            Node::Quad { w: zw, h: zh, color: clear, radius: 0.0 },
                            Node::Quad { w: inner_w, h: 0.0, color: clear, radius: 0.0 },
                        ],
                    }),
                };
                let node = fam::ring(node, ringed, mk::CORNER_RADIUS, p);
                let r = if ringed { 1 } else { 0 };
                let ro = if ringed { 2.0 } else { 0.0 };
                let first = b.scene.len();
                let mut placed = Vec::new();
                let (_, rh0) = lay_out(&node, px0 - ro, y - ro, &mut placed);
                let rh = rh0 - 2.0 * ro;
                let n = placed.len();
                b.scene.extend(placed);
                // The card lives only in the view box (Lager 29).
                for q in &mut b.scene[first..] {
                    q.clip = Some(pane);
                }
                b.roles.extend(std::iter::repeat(Some(deed_i)).take(n));
                let home = pane;
                let tq2 = b.scene[first + r + 1];
                b.texts.push(Text {
                    s: a.title.to_string(),
                    px: sub.px(),
                    weight: sub.weight,
                    family: matrixkit_text::FAMILY,
                    color: p.on_surface,
                    x: tq2.x,
                    y: tq2.y,
                    home,
                    anchor: first + r + 1,
                    clip: Some(pane),
                });
                let zq = b.scene[first + r + 2];
                b.texts.push(Text {
                    s: teaser,
                    px: bodys.px(),
                    weight: bodys.weight,
                    family: matrixkit_text::FAMILY,
                    color: p.on_surface_variant,
                    x: zq.x,
                    y: zq.y,
                    home,
                    anchor: first + r + 2,
                    clip: Some(pane),
                });
                y += rh + mk::spacing::M;
            }
            self.roll_max = ((y + off - ly0) - pane.h).max(0.0);
        }
        self.focus.set_count(self.focus_deeds.len());

        // The ROOT LAYER lies ABOVE everything (Lager 94).
        if self.root.visible() {
            let about = fam::RootAbout {
                name: "Matrix Hilfe",
                version: env!("CARGO_PKG_VERSION"),
                description: "Alles Wissenswerte zu MatrixKit — für Nutzer und Entwickler.",
            };
            let build = self.root.build(engine, &about, w as f32, h as f32, p);
            let first = b.scene.len();
            b.roles.extend(build.roles.into_iter().map(|r| r.map(|k| ROOT_BASE + k)));
            b.scene.extend(build.placed);
            for (l, slot) in build.labels {
                let q = b.scene[first + slot];
                b.texts.push(Text {
                    s: l.text,
                    px: l.px,
                    weight: l.weight,
                    family: l.family,
                    color: l.color,
                    x: q.x,
                    y: q.y,
                    home: Rect::of(&q),
                    anchor: first + slot,
                    clip: None,
                });
            }
        }

        Scene { placed: b.scene, roles: b.roles, texts: b.texts, picts: Vec::new(), curves: Vec::new() }
    }

    /// The window is MAXIMISED — or free again (Lager 92).
    fn zoomed(&mut self, on: bool) {
        self.zoomed = on;
    }

    /// Rolled up, the window is only its BAR. The
    /// window house has applied the wish to windows too since Lager 77
    /// (before only to layer surfaces), otherwise the
    /// window key would be a button without effect.
    fn extent(&mut self) -> Option<(u32, u32)> {
        if self.rolled {
            Some((self.last_w.max(1), self.head_h.max(1)))
        } else {
            // Once: back to the remembered size. Afterwards
            // None again, so that a later drag at the edge does not
            // fight against the app.
            self.before.take()
        }
    }

    fn strike(&mut self, role: Option<usize>, _x: f64, _y: f64, source: Source) -> Refresh {
        let origin = source.label();
        // Root layer open: only its roles count (Lager 94).
        if self.root.is_open() {
            return match role {
                Some(t) if t >= ROOT_BASE => match self.root.strike(t - ROOT_BASE) {
                    fam::RootDeed::Help => {
                        matrixkit_widgets::help_open("Matrix Hilfe");
                        Refresh::Calm
                    }
                    fam::RootDeed::Calm => Refresh::Calm,
                    _ => Refresh::Dirty,
                },
                _ => Refresh::Calm,
            };
        }
        if let Some(t) = role {
            if t >= FRAME_BASE {
                return match t - FRAME_BASE {
                    0 => {
                        eprintln!("[hilfe] Treffer ({origin}): Ampel × — die Hilfe schließt");
                        Refresh::Close
                    }
                    // The traffic light + (Lager 92): maximise and back. How
                    // large maximised is, the compositor computes —
                    // the same route the iced path takes via winit.
                    2 => {
                        if self.rolled {
                            // Rolled up there is nothing to maximise:
                            // the click unrolls first.
                            self.rolled = false;
                            return Refresh::Full;
                        }
                        eprintln!("[hilfe] Treffer ({origin}): Ampel + — maximieren/zurück");
                        Refresh::Zoom
                    }
                    4 => {
                        self.root.toggle();
                        return Refresh::Dirty;
                    }
                    // The WINDOW KEY: the window rolls up into its bar
                    // and out again — like the reference.
                    3 => {
                        // When maximised, the window key first brings the window
                        // back: rolled up and maximised
                        // exclude each other — the requested size
                        // of a maximised window does not apply.
                        if self.zoomed {
                            return Refresh::Zoom;
                        }
                        self.rolled = !self.rolled;
                        if self.rolled {
                            self.before = Some((self.last_w, self.last_h));
                        }
                        Refresh::Full
                    }
                    _ => Refresh::Calm,
                };
            }
        }
        let was_focused = self.search_focus;
        self.search_focus = false;
        self.focus.delete();
        let Some(r) = role else {
            return if was_focused { Refresh::Dirty } else { Refresh::Calm };
        };
        self.act(r, origin)
    }

    fn depart(&mut self) -> Refresh {
        // The pointer has left the window — otherwise the magnifier would
        // stay, and the traffic-light glyphs with it (Lager 91).
        let zoom = self.head_zoom.leave();
        let war = self.lights_hover;
        self.lights_hover = false;
        if zoom || war {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn motion(&mut self, x: f64, y: f64) -> Refresh {
        let r = self.frame_lights;
        let inside = r.w > 0.0
            && (x as f32) >= r.x
            && (x as f32) < r.x + r.w
            && (y as f32) >= r.y
            && (y as f32) < r.y + r.h;
        let zoom = self.head_zoom.point(x, y);
        if inside != self.lights_hover {
            self.lights_hover = inside;
            Refresh::Dirty
        } else if zoom {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn axis(&mut self, x: f64, y: f64, value: f64, _keep: &mut Keep<f32>) -> Refresh {
        // The wheel scrolls the reader — SPRUNG since Lager 41: each
        // notch moves the TARGET, the spring moves there (soft,
        // interruptible — notch upon notch adds momentum).
        let r = self.pane;
        let inside = r.w > 0.0
            && (x as f32) >= r.x
            && (x as f32) < r.x + r.w
            && (y as f32) >= r.y
            && (y as f32) < r.y + r.h;
        if inside && self.roll_max > 0.0 {
            let target = (self.roll.target + value as f32 * 4.0).clamp(0.0, self.roll_max);
            if (target - self.roll.target).abs() > f32::EPSILON {
                self.roll.retarget(target);
                return Refresh::Dirty;
            }
        }
        Refresh::Calm
    }

    fn key(&mut self, key: shell::Key, shift: bool) -> Refresh {
        // The open root layer catches the keyboard (Lager 94).
        if self.root.key(&key, shift) {
            return Refresh::Dirty;
        }
        if !self.search_focus {
            match key {
                shell::Key::Tab => {
                    // The focus model (Lager 31): Tab wanders,
                    // Shift+Tab backwards, the ring shows.
                    if shift {
                        self.focus.back();
                    } else {
                        self.focus.onward();
                    }
                    eprintln!("[hilfe] Fokus: {:?}", self.focus.current());
                    return Refresh::Dirty;
                }
                shell::Key::Enter => {
                    if let Some(i) = self.focus.current() {
                        if let Some(&r) = self.focus_deeds.get(i) {
                            return self.act(r, "Taste");
                        }
                    }
                    return Refresh::Calm;
                }
                shell::Key::Escape => {
                    if self.focus.current().is_some() {
                        self.focus.delete();
                        return Refresh::Dirty;
                    }
                    if self.article.is_some() {
                        eprintln!("[hilfe] Escape: zurück zur Übersicht");
                        self.go(self.category, None);
                        return Refresh::Dirty;
                    }
                    return Refresh::Calm;
                }
                _ => return Refresh::Calm,
            }
        }
        match fam::field_edit(&mut self.search, &mut self.cursor, &mut self.anchor, &key, shift) {
            fam::FieldDeed::Changed => {
                eprintln!("[hilfe] Suche: „{}“", self.search);
                self.roll_reset = true;
                Refresh::Dirty
            }
            fam::FieldDeed::Submitted => {
                // Enter opens the first find.
                let request = self.search.to_lowercase();
                for (ki, k) in CATEGORIES.iter().enumerate() {
                    for (ai, a) in k.article.iter().enumerate() {
                        if a.title.to_lowercase().contains(&request)
                            || a.teaser.to_lowercase().contains(&request)
                            || a.content.to_lowercase().contains(&request)
                        {
                            eprintln!("[hilfe] Enter: erster Fund „{}“", a.title);
                            self.search.clear();
                            self.cursor = 0;
                            self.anchor = None;
                            self.search_focus = false;
                            self.go(ki, Some(ai));
                            return Refresh::Dirty;
                        }
                    }
                }
                Refresh::Calm
            }
            fam::FieldDeed::Escaped => {
                eprintln!("[hilfe] Escape: Suche geräumt");
                self.search.clear();
                self.cursor = 0;
                self.anchor = None;
                self.search_focus = false;
                Refresh::Dirty
            }
            fam::FieldDeed::Calm => Refresh::Calm,
        }
    }

    fn animating(&self) -> bool {
        !self.read_spring.is_settled()
            || !self.roll.is_settled()
            || self.head_zoom.running()
            || self.root.animating()
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        let lese = self.read_spring.tick(dt);
        let roll = self.roll.tick(dt);
        let zoom = self.head_zoom.tick(dt);
        let _ = self.root.glide(dt);
        self.glides += 1;
        if !lese && !roll && !zoom {
            eprintln!("[hilfe] Feder ruht nach {} Takten", self.glides);
            self.glides = 0;
        }
        Refresh::Dirty
    }

    fn beat(&mut self) -> Refresh {
        if self.root.beat() {
            return Refresh::Dirty;
        }
        if self.search_focus {
            self.blink = !self.blink;
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn wants_text(&self) -> bool {
        if self.root.wants_text() {
            return true;
        }
        self.search_focus
    }

    fn pulse(&mut self) -> Refresh {
        let fresh = mk::Palette::load().unwrap_or_default();
        if fresh != self.palette {
            self.palette = fresh;
            return Refresh::Full;
        }
        Refresh::Calm
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[hilfe] Matrix Hilfe: sechster App-Umzug — der Leser fließt, pid={}", std::process::id());
    fam::run(
        Help::new(),
        shell::Options {
            mount: shell::Mount::Window {
                app_id: String::from("matrix-help"),
                title: String::from("Matrix Hilfe"),
                width: WIDTH,
                height: HEIGHT,
            },
            argb: false,
            tag: "hilfe",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}
