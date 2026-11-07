//! Matrix Start, first-light version (iced replacement, Lager 23/39
//! — GRABUNG-ICED-ABLOESUNG.md): the SEARCH PALETTE, since Lager 39
//! with the big extras of the iced version.
//!
//! The palette floats as a veil over everything (overlay, Argb):
//! typing filters, arrows choose, Enter acts, Escape closes.
//! Since Lager 39 it understands: the CALCULATOR (compute() verbatim —
//! „12*8+4" becomes „= 100", Enter copies), the OPEN WINDOWS
//! (window_fetch(), Enter jumps there — with a window bonus like
//! iced), the ACTIONS (actions() verbatim: lock, appearance,
//! log out, restart …) and ICONS in the rows (Matrix apps
//! live from the palette via matrixkit_icons + atlas; foreign ones
//! carry the initial).
//!
//! Since Lager 40 it also speaks COMMANDS (intents/R156:
//! keyword + {query}, among them the web search) and FILES
//! (R155: the living seeker index; opening reports usage
//! back, a missing daemon quietly leaves the section empty).
//!
//! Notes: roles of the hit list (the view box is
//! ready), theme icons of foreign apps.

use crate::{
    actions, app_start, compute, index_build, intent_resolve, intents_load, items, seeker_find,
    seeker_report_used, window_fetch, Entry, Intent, Target, WindowInfo,
};
use matrixkit_shell::{self as shell, Pict, Refresh, Scene, Source, Text};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{lay_out, Keep, Node, Placed, Rect};
use matrixkit_widgets as mkw;
use matrixkit_widgets::scene as fam;

/// Roles: 0..ROWS-1 the hit rows, VEIL closes.
const VEIL: usize = 900;
/// The panel measures of the reference (main.rs: WIDTH 620 minus 2·M
/// outside, HEIGHT 520 fixed). The card here is the inside, i.e.
/// without its own padding (finger probe 17.08.2026).
const CARD_W: f32 = 620.0 - 2.0 * mk::spacing::M - 2.0 * mk::spacing::S;
const CARD_H: f32 = 520.0;
/// This many hits the reference holds (main.rs: truncate(40)) —
/// visible is what fits the fixed panel height, the rest comes
/// when scrolling (17.08.2026).
const ROWS: usize = 40;
/// The icon size of the reference (main.rs: ICON = 32) — with 24
/// the rows were 12 logical pixels tighter than there (finger probe
/// 17.08.2026).
const ICON: f32 = 32.0;

/// A hit row of the round: face + deed.
struct HitRow {
    name: String,
    category: &'static str,
    glyph: Option<char>,
    app_icon: Option<String>,
    target: Target,
}

struct Palette {
    index: Vec<Entry>,
    windows: Vec<WindowInfo>,
    intents: Vec<Intent>,
    search: String,
    cursor: usize,
    anchor: Option<usize>,
    sel: usize,
    /// The scroll area of the hit list: the reference holds 40
    /// hits and scrolls inside them; without it the light path showed only
    /// what happened to fit (finger probe 17.08.2026).
    liste: Rect,
    roll: mk::motion::Spring,
    roll_max: f32,
    blink: bool,
    palette: mk::Palette,
    /// The rows of the last round (role = index).
    shown: Vec<HitRow>,
    /// App icons (24×24 RGBA) — None: the initial stands in.
    icons: std::collections::HashMap<String, Option<(u32, u32, Vec<u8>)>>,
    puts: std::collections::HashSet<String>,
    epoch: u64,
    /// Round 88: the icon theme the icons were baked in.
    icon_theme: Option<String>,
}

impl Palette {
    fn new() -> Self {
        let index = index_build();
        Palette {
            windows: window_fetch(&index),
            intents: intents_load(),
            index,
            search: String::new(),
            cursor: 0,
            anchor: None,
            sel: 0,
            liste: Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 },
            roll: mk::motion::Spring::smooth(0.0),
            roll_max: 0.0,
            blink: true,
            palette: mk::Palette::load().unwrap_or_default(),
            shown: Vec::new(),
            icons: std::collections::HashMap::new(),
            puts: std::collections::HashSet::new(),
            epoch: 0,
            icon_theme: matrixkit_icons::look(),
        }
    }

    /// The hits of the round — ranked like the iced version:
    /// calculator first, windows with a bonus, apps, actions only with
    /// input.
    fn rows(&self) -> Vec<HitRow> {
        let s = self.search.trim();
        let empty = s.is_empty();
        let mut kand: Vec<(u32, HitRow)> = Vec::new();

        if let Some(erg) = compute(s) {
            kand.push((
                100_000,
                HitRow {
                    name: format!("= {erg}"),
                    category: "Rechnen",
                    glyph: Some(mkw::symbol::CODE),
                    app_icon: None,
                    target: Target::Copy(erg),
                },
            ));
        }

        for f in &self.windows {
            let p = items(&f.name, s).into_iter().chain(items(&f.title, s)).max();
            let Some(p) = p else { continue };
            let display = if f.title.is_empty() || f.title == f.name {
                f.name.clone()
            } else {
                format!("{} — {}", f.name, f.title)
            };
            let bonus = if empty { 4 } else { 15 };
            let app_icon = self
                .index
                .iter()
                .find(|e| e.name == f.name)
                .map(|e| e.app_id.clone());
            kand.push((
                p + bonus,
                HitRow {
                    name: display,
                    category: "Fenster",
                    glyph: None,
                    app_icon,
                    target: Target::Window(f.id),
                },
            ));
        }

        for (i, e) in self.index.iter().enumerate() {
            if let Some(p) = items(&e.name, s) {
                kand.push((
                    p,
                    HitRow {
                        name: e.name.clone(),
                        category: "App",
                        glyph: None,
                        app_icon: Some(e.app_id.clone()),
                        target: Target::App(i),
                    },
                ));
            }
        }

        // 3a) Intents (R156): keyword + {query} — the
        // Siri grammar without the voice (verbatim iced).
        if !empty {
            let (first, rest) = match s.split_once(' ') {
                Some((f, r)) => (f.to_lowercase(), r.trim().to_string()),
                None => (s.to_lowercase(), String::new()),
            };
            for i in &self.intents {
                let keyword_hit = i.keywords.iter().any(|k| *k == first);
                let p = if i.accepts_query && keyword_hit && !rest.is_empty() {
                    Some(35)
                } else {
                    items(&i.title, s)
                        .into_iter()
                        .chain(i.keywords.iter().filter_map(|k| items(k, s)))
                        .max()
                };
                let Some(p) = p else { continue };
                let query =
                    if i.accepts_query && keyword_hit { rest.clone() } else { String::new() };
                let name = if query.is_empty() {
                    i.title.clone()
                } else {
                    format!("{} — \u{201e}{query}\u{201c}", i.title)
                };
                kand.push((
                    p + 6,
                    HitRow {
                        name,
                        category: "Befehl",
                        glyph: None,
                        app_icon: Some(i.app_id.clone()),
                        target: Target::Intent(intent_resolve(&i.exec, &query)),
                    },
                ));
            }
        }

        // 3b) Files from the living index (R155) — only with
        // input, a small malus keeps apps in front; a missing
        // daemon answers with an empty section, never with a stall.
        if !empty {
            let home = std::env::var("HOME").unwrap_or_default();
            for (p, path) in seeker_find(s) {
                let name = std::path::Path::new(&path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone());
                let place = std::path::Path::new(&path)
                    .parent()
                    .map(|d| d.to_string_lossy().replacen(&home, "~", 1))
                    .unwrap_or_default();
                kand.push((
                    (p - 3).max(1) as u32,
                    HitRow {
                        name: format!("{name} — {place}"),
                        category: "Datei",
                        glyph: Some(mkw::symbol::FILE),
                        app_icon: None,
                        target: Target::File(path),
                    },
                ));
            }
        }

        if !empty {
            for (name, sym, target) in actions() {
                if let Some(p) = items(name, s) {
                    kand.push((
                        p + 5,
                        HitRow {
                            name: name.to_string(),
                            category: "Aktion",
                            glyph: Some(sym),
                            app_icon: None,
                            target,
                        },
                    ));
                }
            }
        }

        kand.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.name.cmp(&b.1.name)));
        kand.into_iter().take(ROWS).map(|(_, z)| z).collect()
    }

    fn icon_key(&self, app: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("\u{0}picon:{}:{}", app, self.epoch))
    }

    /// Bake an icon — once per app and epoch.
    ///
    /// First the MATRIX family (painted from the palette), then the
    /// foreign icon from the icon theme. Until the finger probe on
    /// 16.08.2026 the second route was missing entirely: every foreign app carried
    /// only its initial, while the reference showed its logo.
    /// SVGs go through the canvas rasteriser and PNG bytes into the
    /// atlas (straight channels, see Canvas::png).
    fn bake_icon(&mut self, app_id: &str, desktop_icon: Option<&str>) {
        let p = self.palette;
        let wish = desktop_icon.map(|s| s.to_string());
        self.icons.entry(app_id.to_string()).or_insert_with(|| {
            // Round 88: the chosen icon theme first (WhiteSur …), else Matrix.
            if let Some(baked) = matrixkit_icons::app_png(app_id, wish.as_deref(), &p, ICON as u32)
                .and_then(|png| matrixkit_image::ImageAtlas::unpack(&png, ICON as u32, ICON as u32))
            {
                return Some(baked);
            }
            let name = wish?;
            let path = matrixkit_icons::foreign_icon_path(&name)?;
            let bytes = std::fs::read(&path).ok()?;
            if path.extension().is_some_and(|e| e == "svg") {
                let svg = matrixkit_canvas::Svg::parse(&bytes)?;
                let sheet = svg.raster(ICON as u32, ICON as u32)?;
                let png = sheet.png()?;
                matrixkit_image::ImageAtlas::unpack(&png, ICON as u32, ICON as u32)
            } else {
                matrixkit_image::ImageAtlas::unpack(&bytes, ICON as u32, ICON as u32)
            }
        });
    }

    /// The deed of a row — called by click AND Enter.
    fn act(&mut self, row: usize) -> Refresh {
        let Some(z) = self.shown.get(row) else { return Refresh::Calm };
        match &z.target {
            Target::App(i) => {
                if let Some(e) = self.index.get(*i) {
                    eprintln!("[start] startet: {} ({})", e.name, e.app_id);
                    app_start(e);
                }
            }
            Target::Window(id) => {
                eprintln!("[start] springt zu Fenster {id}: „{}“", z.name);
                mkw::canvas::focus_window(*id);
            }
            Target::Copy(text) => {
                let ok = mk::clipboard::copy(text);
                eprintln!(
                    "[start] Rechner: „{text}“ {}",
                    if ok { "kopiert" } else { "— wl-copy fehlt" }
                );
            }
            Target::Command { cmd, .. } => {
                eprintln!("[start] Aktion: {cmd}");
                let mut parts = cmd.split_whitespace();
                if let Some(bin) = parts.next() {
                    let _ = std::process::Command::new(bin).args(parts).spawn();
                }
            }
            Target::Appearance => {
                eprintln!("[start] Erscheinung umschalten");
                mk::light_toggle();
            }
            Target::Logout => {
                eprintln!("[start] Abmelden");
                mk::canvas::logout(false);
            }
            Target::Intent(argv) => {
                eprintln!("[start] Befehl: {}", argv.join(" "));
                if let Some((bin, rest)) = argv.split_first() {
                    let _ = std::process::Command::new(bin).args(rest).spawn();
                }
            }
            Target::File(path) => {
                eprintln!("[start] öffnet Datei: {path}");
                seeker_report_used(path);
                // Round 178: the kit's opener (xdg-open is missing on the Duo; the child was never reaped).
                mk::open::open(path);
            }
        }
        Refresh::Close
    }
}

impl shell::Client for Palette {
    type Card = ();
    type Work = ();

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn base(&mut self, _atlas: &mut matrixkit_image::ImageAtlas, buf: &mut [u8], _w: u32, _h: u32) {
        // The veil: black at 45 % — premultiplied, the
        // screen below stays faintly visible (immersion doctrine).
        for px in buf.chunks_exact_mut(4) {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
            px[3] = 115;
        }
    }

    fn view(
        &mut self,
        engine: &mut TextEngine,
        _keep: &mut Keep<f32>,
        atlas: &mut matrixkit_image::ImageAtlas,
        w: u32,
        h: u32,
    ) -> Scene {
        let p = self.palette;
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let symfam = mkw::symbol_family();
        let mut scene: Vec<mk::scene::Placed> = Vec::new();
        let mut roles_v: Vec<Option<usize>> = Vec::new();
        let mut texts: Vec<Text> = Vec::new();
        let mut picts: Vec<Pict> = Vec::new();

        // The veil as hit target: a click beside it closes.
        let mut placed = Vec::new();
        lay_out(
            &Node::Quad { w: w as f32, h: h as f32, color: clear, radius: 0.0 },
            0.0,
            0.0,
            &mut placed,
        );
        scene.extend(placed);
        roles_v.push(Some(VEIL));

        // The card: field on top, hits below.
        self.shown = self.rows();
        self.sel = self.sel.min(self.shown.len().saturating_sub(1));
        let card_x = (w as f32 - CARD_W) / 2.0;
        // The reference places the panel 140 logical pixels
        // from the top edge (main.rs: padding top 140.0); the
        // thirds calculation here sat 110 pixels lower.
        // 140 is the distance of the PANEL edge; the field starts
        // lower by the panel padding.
        let card_y = 140.0 + mk::spacing::S;

        // The PANEL CARD (mkw::panel::pill): surface_container with
        // opacity, 1 px edge, floating shadow — it was missing entirely,
        // the rows lay bare on the wallpaper (finger probe
        // 16.08.2026). The height is only fixed once the rows are laid;
        // hence two placeholders that get their size further down.
        let pad = mk::spacing::S;
        let pill_ring = scene.len();
        scene.push(Placed {
            x: card_x - pad - 1.0,
            y: card_y - pad - 1.0,
            w: CARD_W + 2.0 * pad + 2.0,
            h: 0.0,
            color: p.outline.with_alpha(mkw::panel::OUTLINE),
            radius: mk::radius::NORMAL + pad,
            clip: None,
            shadow: None,
        });
        roles_v.push(None);
        let pill_fill = scene.len();
        scene.push(Placed {
            x: card_x - pad,
            y: card_y - pad,
            w: CARD_W + 2.0 * pad,
            h: 0.0,
            color: p.surface_container.with_alpha(mkw::panel::coverage()),
            radius: mk::radius::NORMAL + pad,
            clip: None,
            shadow: None,
        });
        roles_v.push(None);

        let spec = fam::FieldSpec {
            value: &self.search,
            cursor: self.cursor,
            anchor: self.anchor,
            focused: true,
            blink: self.blink,
            // Word for word like the reference (mkw::search_field in the
            // iced gate) — and the magnifier belongs INSIDE the pill.
            placeholder: "Suchen, springen oder rechnen …",
            leading: Some(mkw::symbol::SEARCH),
        };
        let field = fam::input_field(engine, &spec, CARD_W, p);
        let ffirst = scene.len();
        let mut fplaced = Vec::new();
        let (_, field_h) = lay_out(&field.node, card_x, card_y, &mut fplaced);
        roles_v.extend(std::iter::repeat(None).take(fplaced.len()));
        scene.extend(fplaced);
        // With a leading glyph the prefix sits at 6 instead of 4.
        let text_slot = ffirst + if field.leading.is_some() { 6 } else { 4 };
        if let Some(glyph) = &field.leading {
            let gq = scene[ffirst + 2];
            texts.push(Text {
                s: glyph.clone(),
                px: mk::typo::symbol_px(mk::font_size::MEDIUM),
                weight: mk::typo::Weight::Normal,
                family: mkw::symbol_family(),
                color: p.on_surface_variant,
                x: gq.x,
                y: gq.y,
                home: Rect::of(&scene[ffirst]),
                anchor: ffirst + 2,
                clip: None,
            });
        }
        let anchor_q = scene[text_slot];
        texts.push(Text {
            s: field.label.text.clone(),
            px: field.label.px,
            weight: field.label.weight,
            family: field.label.family,
            color: field.label.color,
            x: anchor_q.x,
            y: anchor_q.y,
            home: Rect::of(&scene[ffirst]),
            anchor: text_slot,
            clip: None,
        });

        // The hit rows: icon, name, category on the right; the
        // chosen one carries the primary_container pill.
        let body = mk::typo::BODY;
        let cap = mk::typo::CAPTION;
        let head = mk::typo::LABEL;
        // The scroll area: from below the field to the panel's bottom edge.
        let listen_oben = card_y + field_h + mk::spacing::S;
        self.liste = Rect {
            x: card_x - pad,
            y: listen_oben,
            w: CARD_W + 2.0 * pad,
            h: (card_y - pad + CARD_H) - listen_oben - pad,
        };
        let off = self.roll.value().max(0.0);
        let mut y = listen_oben - off;
        let rows_data: Vec<(String, &'static str, Option<char>, Option<String>)> = self
            .shown
            .iter()
            .map(|z| (z.name.clone(), z.category, z.glyph, z.app_icon.clone()))
            .collect();
        for (row, (name, category, glyph, app_icon)) in rows_data.into_iter().enumerate() {
            let picked = row == self.sel;
            // Bake the icon (Matrix apps live from the palette).
            if let Some(app) = &app_icon {
                // The Icon= value is in the index — from there the
                // baking fetches it for foreign apps.
                let desktop = self
                    .index
                    .iter()
                    .find(|e| &e.app_id == app)
                    .and_then(|e| e.icon.clone());
                self.bake_icon(app, desktop.as_deref());
                if let Some(Some((iw, ih, rgba))) = self.icons.get(app).cloned() {
                    if !self.puts.contains(app) {
                        atlas.put(&self.icon_key(app), iw, ih, rgba);
                        self.puts.insert(app.clone());
                    }
                }
            }
            let has_pict = app_icon
                .as_ref()
                .map(|a| self.icons.get(a).map(|o| o.is_some()).unwrap_or(false))
                .unwrap_or(false);
            let name_short = mk::elide(&name, 53);
            let (nw, nh) = engine.measure_styled(&name_short, body.px(), body.weight);
            let (kw, kh) = engine.measure_styled(category, cap.px(), cap.weight);
            // Like the reference: a row carries a ground ONLY when
            // chosen, and then the subtle hover tone — not every
            // row its own pill in primary_container
            // (finger probe 16.08.2026). Padding XS/S as there.
            let node = Node::Wrap {
                pad_x: mk::spacing::S,
                pad_y: mk::spacing::XS,
                background: Some((
                    if picked {
                        p.on_surface.over(p.surface_container, mkw::panel::HOVER)
                    } else {
                        clear
                    },
                    mk::radius::NORMAL,
                )),
                child: Box::new(Node::Row {
                    gap: mk::spacing::S,
                    padding: 0.0,
                    background: None,
                    children: vec![
                        Node::Quad { w: ICON, h: ICON, color: clear, radius: 0.0 },
                        Node::Quad { w: nw, h: nh, color: clear, radius: 0.0 },
                        Node::Quad {
                            w: (CARD_W
                                - 2.0 * mk::spacing::M
                                - ICON
                                - 2.0 * mk::spacing::S
                                - nw
                                - kw)
                                .max(0.0),
                            h: 0.0,
                            color: clear,
                            radius: 0.0,
                        },
                        Node::Quad { w: kw, h: kh, color: clear, radius: 0.0 },
                    ],
                }),
            };
            let first = scene.len();
            let mut placed = Vec::new();
            let (_, row_h) = lay_out(&node, card_x, y, &mut placed);
            roles_v.extend(std::iter::repeat(Some(row)).take(placed.len()));
            scene.extend(placed);
            let home = Rect::of(&scene[first]);
            let iq = scene[first + 1];
            if has_pict {
                if let Some(app) = &app_icon {
                    picts.push(Pict {
                        path: self.icon_key(app),
                        box_w: ICON as u32,
                        box_h: ICON as u32,
                        fit: matrixkit_image::Fit::Cover,
                        x: iq.x,
                        y: iq.y,
                        radius: 5.0,
                        home,
                        anchor: first + 1,
                        clip: Some(self.liste),
                    });
                }
            } else if let Some(g) = glyph {
                let gs = g.to_string();
                let (gw, gh) = engine.measure_family(symfam, &gs, mk::typo::symbol_px(18.0), mk::typo::Weight::Normal);
                texts.push(Text {
                    s: gs,
                    px: mk::typo::symbol_px(18.0),
                    weight: mk::typo::Weight::Normal,
                    family: symfam,
                    color: p.on_surface_variant,
                    x: iq.x + (ICON - gw) / 2.0,
                    y: iq.y + (ICON - gh) / 2.0,
                    home,
                    anchor: first + 1,
                    clip: Some(self.liste),
                });
            } else {
                // The initial as a small badge.
                let initial = name.chars().next().unwrap_or('•').to_uppercase().to_string();
                let (lw, lh) = engine.measure_styled(&initial, head.px(), head.weight);
                texts.push(Text {
                    s: initial,
                    px: head.px(),
                    weight: head.weight,
                    family: matrixkit_text::FAMILY,
                    color: p.on_surface_variant,
                    x: iq.x + (ICON - lw) / 2.0,
                    y: iq.y + (ICON - lh) / 2.0,
                    home,
                    anchor: first + 1,
                    clip: Some(self.liste),
                });
            }
            let nq = scene[first + 2];
            texts.push(Text {
                s: name_short,
                px: body.px(),
                weight: body.weight,
                family: matrixkit_text::FAMILY,
                color: p.on_surface,
                x: nq.x,
                y: nq.y + (ICON - nh) / 2.0,
                home,
                anchor: first + 2,
                clip: Some(self.liste),
            });
            let kq = scene[first + 4];
            texts.push(Text {
                s: category.to_string(),
                px: cap.px(),
                weight: cap.weight,
                family: matrixkit_text::FAMILY,
                color: p.on_surface_variant,
                x: kq.x,
                y: kq.y + (ICON - kh) / 2.0,
                home,
                anchor: first + 4,
                clip: Some(self.liste),
            });
            // The reference's row spacing is S (column!.spacing(S)),
            // not XS — otherwise the rows sit four logical pixels
            // too close (finger probe 17.08.2026).
            y += row_h + mk::spacing::S;
        }

        // How far it can scroll: whatever sticks out beyond
        // the area.
        let gesamt = (y + off) - listen_oben;
        self.roll_max = (gesamt - self.liste.h).max(0.0);

        // The reference keeps a FIXED panel height and scrolls inside;
        // a growing card showed fewer rows than there.
        let pill_h = CARD_H;
        scene[pill_ring].h = pill_h + 2.0;
        scene[pill_fill].h = pill_h;
        // What floats casts a shadow (Lager 64).
        fam::lift(&mut scene, pill_ring, fam::floating_shadow());

        Scene { placed: scene, roles: roles_v, texts, picts, curves: Vec::new() }
    }

    /// While the scroll spring runs, the client wants beats —
    /// without that its value stands still and the list does not scroll (measured
    /// with a real wheel event, 17.08.2026).
    fn animating(&self) -> bool {
        !self.roll.is_settled()
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        if self.roll.tick(dt) {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn axis(&mut self, x: f64, y: f64, value: f64, _keep: &mut Keep<f32>) -> Refresh {
        // Only in the list area, and only if there is something to scroll.
        let r = self.liste;
        let drin = r.w > 0.0
            && (x as f32) >= r.x
            && (x as f32) < r.x + r.w
            && (y as f32) >= r.y
            && (y as f32) < r.y + r.h;
        if drin && self.roll_max > 0.0 {
            let target = (self.roll.target + value as f32 * 4.0).clamp(0.0, self.roll_max);
            if (target - self.roll.target).abs() > f32::EPSILON {
                self.roll.retarget(target);
                return Refresh::Dirty;
            }
        }
        Refresh::Calm
    }

    fn strike(&mut self, role: Option<usize>, _x: f64, _y: f64, source: Source) -> Refresh {
        match role {
            Some(VEIL) => {
                eprintln!("[start] Schleier getroffen ({}) — die Palette weicht", source.label());
                Refresh::Close
            }
            Some(row) if row < ROWS => self.act(row),
            _ => Refresh::Calm,
        }
    }

    fn key(&mut self, key: shell::Key, shift: bool) -> Refresh {
        match key {
            shell::Key::Up => {
                self.sel = self.sel.saturating_sub(1);
                return Refresh::Dirty;
            }
            shell::Key::Down => {
                self.sel = (self.sel + 1).min(self.shown.len().saturating_sub(1));
                return Refresh::Dirty;
            }
            _ => {}
        }
        match fam::field_edit(&mut self.search, &mut self.cursor, &mut self.anchor, &key, shift) {
            fam::FieldDeed::Changed => {
                self.sel = 0;
                eprintln!("[start] Suche: „{}“", self.search);
                Refresh::Dirty
            }
            fam::FieldDeed::Submitted => {
                let sel = self.sel;
                self.act(sel)
            }
            fam::FieldDeed::Escaped => {
                eprintln!("[start] Escape — die Palette weicht");
                Refresh::Close
            }
            fam::FieldDeed::Calm => Refresh::Calm,
        }
    }

    fn beat(&mut self) -> Refresh {
        self.blink = !self.blink;
        Refresh::Dirty
    }

    fn wants_text(&self) -> bool {
        true
    }

    fn pulse(&mut self) -> Refresh {
        // Windows move — the list lives (2 s pulse).
        self.windows = window_fetch(&self.index);
        let fresh = mk::Palette::load().unwrap_or_default();
        let theme = matrixkit_icons::look();
        if fresh != self.palette || theme != self.icon_theme {
            self.palette = fresh;
            self.icon_theme = theme;
            self.icons.clear();
            self.puts.clear();
            self.epoch += 1;
            return Refresh::Full;
        }
        Refresh::Dirty
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Round 90: the GPU paints the app menu — its transparent veil keeps its
    // alpha on the GPU path now (it was the CPU, «the app menu lags»).
    if std::env::var_os("MATRIX_CPU").is_none() {
        shell::prefer_gpu();
    }
    eprintln!(
        "[start] Matrix Start: die Palette wird voll (Lager 39) — Rechner, Fenster, Aktionen, Icons, pid={}",
        std::process::id()
    );
    shell::run(
        Palette::new(),
        shell::Options {
            mount: shell::Mount::Layer {
                namespace: String::from("matrix-start"),
                layer: shell::Layer::Overlay,
                keyboard: shell::KeyboardWish::Exclusive,
                anchor: shell::Anchors::all(),
                size: (0, 0),
                margin: (0, 0, 0, 0),
                exclusive: -1,
            },
            argb: true,
            tag: "start",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}
