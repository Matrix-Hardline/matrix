//! Matrix Kontext, first-light version (iced replacement, stage 47
//! — GRABUNG-ICED-ABLOESUNG.md): the twelfth app move — the
//! context service, and the surface learns PASS-THROUGH.
//!
//! The resident menu service (R120): the daemon surface lives
//! permanently as an overlay, but between the menus it is
//! PASS-THROUGH (empty input region, stage 47) — no click of the
//! session gets eaten. A call writes the request into the
//! request file (contract unchanged: leinwand <x> <y> / clips
//! <offset> <bottom>), the watch thread feeds it through the
//! loom, the menu appears at the point — a click on the entry
//! acts, a click beside it closes, both make it pass-through
//! again.
//!
//! Dock mode (stage 48) and the shared daemon choice over the socket
//! (stage 49) have moved in — whoever binds the socket serves, and
//! all existing callers feed it. Notes for later: menu entrance
//! (spring), keyboard arrows in the menu.

use crate::{clips_read, request_path, ENTRIES};
use matrixkit_shell::{self as shell, Refresh, Scene, Source, Text};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{lay_out, Keep, Node, Placed, Rect};
use matrixkit_widgets as mkw;
use matrixkit_widgets::scene as fam;

const VEIL: usize = 900;
const ROW_H: f32 = 36.0;
const PAD: f32 = 8.0;
/// The reference's symbol chip (mkw::ui::menu_glyph_circle).
const CHIP: f32 = 28.0;
/// The left indent in the menu: the reference pads TWICE with S —
/// once the shell (menu_shell), once the row
/// (.padding([6, S])). With only one padding the chip sat eight
/// logical pixels too far left (finger probe 17.08.2026).
const INSET: f32 = 2.0 * mk::spacing::S;

/// What the service is showing right now (dock mode is a note for later).
enum Target {
    Hidden,
    Canvas { x: f32, y: f32 },
    Clips { offset_x: f32, bottom: f32, entries: Vec<String> },
    Dock { offset_x: f32, bottom: f32, app_id: String, pinned: bool, window: Vec<u64> },
}

struct Kontext {
    target: Target,
    palette: mk::Palette,
    /// Role = index into the rows of the open menu.
    deeds: Vec<usize>,
    /// Own start arguments (daemon choice, stage 49) — the first
    /// request, in memory instead of via the request file.
    initial: Option<Vec<String>>,
    /// Size and position of the open menu (stage 82). The SURFACE is
    /// the menu — the compositor only lays its glass under a
    /// menu-sized surface; a full-screen one would get it across the
    /// whole width. Computed while painting (that is where the
    /// text measurer is), reported afterwards via extent() and offset().
    geom: Option<(f32, f32, f32, f32)>,
    /// Round 68: the screen the request came from — the menu and its
    /// veil open THERE as panes (the dock lives on DSI-1, the main
    /// surface on whatever screen the daemon started).
    output: Option<String>,
    /// Round 68: the rows of the menu pane, (top, bottom, deed) in pane
    /// pixels — the pane's own hit test.
    row_hits: Vec<(f32, f32, usize)>,
}

/// The two panes of an open menu.
#[derive(Clone, PartialEq, Debug)]
pub enum Pane {
    Veil,
    Menu,
}

impl Kontext {
    fn new(initial: Option<Vec<String>>) -> Self {
        Kontext {
            target: Target::Hidden,
            palette: mk::Palette::load().unwrap_or_default(),
            deeds: Vec::new(),
            initial,
            geom: None,
            output: None,
            row_hits: Vec::new(),
        }
    }

    /// Menu size of the canvas entries (measured in view; here
    /// the fixed row grammar).
    fn canvas_size(engine: &mut TextEngine) -> (f32, f32) {
        let body = mk::typo::BODY;
        let mut w: f32 = 0.0;
        for (_, title, _) in ENTRIES {
            let (tw, _) = engine.measure_styled(title, body.px(), body.weight);
            w = w.max(tw);
        }
        (PAD * 2.0 + 30.0 + w + 16.0, PAD * 2.0 + ENTRIES.len() as f32 * ROW_H)
    }
}

impl Kontext {
    /// Round 68: where the menu stands and how large it is — at request
    /// time (width and row height are fixed, no text measuring needed),
    /// against the size of the REQUEST's screen.
    fn place(&mut self) {
        if matches!(self.target, Target::Hidden) {
            self.geom = None;
            return;
        }
        let (w, h) = (0u32, 0u32);
        // Die Karte samt Zeilen.
        let rows = self.rows();
        // The width is FIXED as in the reference
        // (mkw::ui::MENU_WIDTH, menu_size returns it unchanged)
        // — content-driven it was 37 logical pixels
        // narrower, and the text block sat correspondingly further
        // right (finger probe 17.08.2026).
        let card_w: f32 = mkw::ui::MENU_WIDTH;
        let card_h = PAD * 2.0 + rows.len() as f32 * ROW_H;
        // The edges are checked against the SCREEN, not against
        // our own surface — since stage 82 that is only as large
        // as the menu.
        let (sw, sh) = self
            .output
            .as_deref()
            .and_then(mk::canvas::output_size)
            .unwrap_or_else(mkw::glass::screen_size);
        let (screen_w, screen_h) = if sw > 0.0 && sh > 0.0 {
            (sw, sh)
        } else {
            (w as f32, h as f32)
        };
        let (cx, cy) = match &self.target {
            Target::Canvas { x, y } => {
                let mut cx = *x;
                let mut cy = *y;
                if cx + card_w > screen_w {
                    cx = (cx - card_w).max(0.0);
                }
                if cy + card_h > screen_h {
                    cy = (cy - card_h).max(0.0);
                }
                (cx, cy)
            }
            Target::Clips { offset_x, bottom, .. }
            | Target::Dock { offset_x, bottom, .. } => (
                // Centre of the SCREEN, not of our own surface.
                (screen_w / 2.0 + offset_x - card_w / 2.0).max(0.0),
                screen_h - bottom - card_h,
            ),
            Target::Hidden => (0.0, 0.0),
        };
        self.geom = Some((cx, cy, card_w, card_h));
    }

    fn rows(&self) -> Vec<(Option<char>, String, usize)> {
        match &self.target {
            Target::Canvas { .. } => ENTRIES
                .iter()
                .enumerate()
                .map(|(i, (g, t, _))| (Some(*g), t.to_string(), i))
                .collect(),
            Target::Dock { pinned, window, .. } => {
                // The tile grammar of the iced path, literally:
                // pin toggle, quit only with windows, customise always.
                let mut v = vec![(
                    None,
                    String::from(if *pinned { "Loslösen" } else { "Anpinnen" }),
                    0,
                )];
                if !window.is_empty() {
                    v.push((None, String::from("Beenden"), 1));
                }
                v.push((Some(mkw::symbol::TUNE), String::from("Anpassen …"), 2));
                v
            }
            Target::Clips { entries, .. } => {
                let mut v: Vec<(Option<char>, String, usize)> = entries
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let kurz: String = e.chars().take(40).collect();
                        (None, if e.chars().count() > 40 { format!("{kurz}…") } else { kurz }, i)
                    })
                    .collect();
                v.push((None, String::from("Leeren"), usize::MAX));
                v
            }
            Target::Hidden => Vec::new(),
        }
    }

    /// Round 68: the menu pane's picture (it sits at 0,0 — the pane IS the
    /// menu) and its row hits.
    fn menu_scene(&mut self, engine: &mut TextEngine) -> Scene {
        let p = self.palette;
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let symfam = mkw::symbol_family();
        let body = mk::typo::BODY;
        let mut scene: Vec<Placed> = Vec::new();
        let mut roles: Vec<Option<usize>> = Vec::new();
        let mut texts: Vec<Text> = Vec::new();
        self.deeds = Vec::new();
        self.row_hits.clear();
        let Some((_, _, card_w, card_h)) = self.geom else { return Scene::default() };
        let rows = self.rows();
        // From here on the SURFACE is the menu: everything sits at 0,0.
        let (cx, cy) = (0.0f32, 0.0f32);
        let mut placed = Vec::new();
        lay_out(
            // On compositor GLASS the app contributes NOTHING to the
            // silhouette (R125): the background comes from the glass. That
            // only works since the SURFACE is menu-sized (stage 82)
            // — with a full-screen one the glass rule shaped
            // the whole screen, and without a background the text stood
            // on bare wallpaper.
            &Node::Quad {
                w: card_w,
                h: card_h,
                color: if mkw::ui::menu_glass_on() { clear } else { p.surface },
                radius: 14.0,
            },
            cx,
            cy,
            &mut placed,
        );
        scene.extend(placed);
        roles.push(None);
        let home = Rect { x: cx, y: cy, w: card_w, h: card_h };
        let mut y = cy + PAD;
        for (glyph, title, id) in rows {
            let deed_i = self.deeds.len();
            self.deeds.push(id);
            self.row_hits.push((y, y + ROW_H, deed_i));
            let first = scene.len();
            let mut placed = Vec::new();
            lay_out(
                &Node::Quad { w: card_w - PAD * 2.0, h: ROW_H - 4.0, color: clear, radius: 8.0 },
                cx + PAD,
                y,
                &mut placed,
            );
            scene.extend(placed);
            roles.push(Some(deed_i));
            if let Some(g) = glyph {
                // The CHIP: a circle behind the glyph, like the
                // reference (mkw::ui::menu_glyph_circle) — 28x28 in
                // on_surface over surface_container_high at 8 %.
                // We painted the bare glyph (finger probe
                // 17.08.2026).
                let chip_y = y + (ROW_H - 4.0 - CHIP) / 2.0;
                let chip = scene.len();
                let mut chip_placed = Vec::new();
                lay_out(
                    &Node::Quad {
                        w: CHIP,
                        h: CHIP,
                        color: p.on_surface.over(p.surface_container_high, 0.08),
                        radius: CHIP / 2.0,
                    },
                    cx + INSET,
                    chip_y,
                    &mut chip_placed,
                );
                scene.extend(chip_placed);
                roles.push(Some(deed_i));
                let gs = g.to_string();
                let px = mk::typo::symbol_px(mk::icon_size::SMALL + 2.0);
                let (gw, gh) = engine.measure_family(symfam, &gs, px, mk::typo::Weight::Normal);
                texts.push(Text {
                    s: gs,
                    px,
                    weight: mk::typo::Weight::Normal,
                    family: symfam,
                    color: p.on_surface_variant,
                    // Mittig IM Chip.
                    x: cx + INSET + (CHIP - gw) / 2.0,
                    y: chip_y + (CHIP - gh) / 2.0,
                    home,
                    // Above the chip, not below it (stage 70).
                    anchor: chip,
                    clip: None,
                });
            }
            let (_, th) = engine.measure_styled(&title, body.px(), body.weight);
            texts.push(Text {
                s: title,
                px: body.px(),
                weight: body.weight,
                family: matrixkit_text::FAMILY,
                color: p.on_surface,
                // An entry WITHOUT a symbol starts at the left padding,
                // as in the reference — only with a symbol does the
                // label move behind its column
                // (finger probe 17.08.2026).
                // Behind the chip lies the gap S.
                x: cx + INSET + if glyph.is_some() { CHIP + mk::spacing::S } else { 0.0 },
                y: y + (ROW_H - 4.0 - th) / 2.0,
                home,
                anchor: first,
                clip: None,
            });
            y += ROW_H;
        }
        let _ = Self::canvas_size(engine);

        Scene { placed: scene, roles, texts, picts: Vec::new(), curves: Vec::new() }
    }
}

impl shell::Client for Kontext {
    type Card = Pane;
    type Work = Vec<String>;

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn warp(&mut self, loom: &shell::Loom<Vec<String>>) {
        // Die eigene Start-Anfrage zuerst (Daemon-Wahl, Lager 49).
        if let Some(args) = self.initial.take() {
            loom.feed(args);
        }
        // The watch thread (25 Hz like iced): a fresh
        // request file becomes a menu request.
        let feeder = loom.clone();
        std::thread::spawn(move || {
            // Consume the initial state BEFORE the loop — otherwise
            // the watcher swallows the first real request
            // when the file is missing at first (the first-miss bug;
            // the same root lived in the OSD listener, stage 42).
            let mut last = std::fs::metadata(request_path())
                .ok()
                .and_then(|m| m.modified().ok());
            loop {
                if let Ok(meta) = std::fs::metadata(request_path()) {
                    let mtime = meta.modified().ok();
                    if mtime.is_some() && mtime != last {
                        {
                            if let Ok(raw) = std::fs::read_to_string(request_path()) {
                                let args: Vec<String> =
                                    raw.split('\u{1f}').map(str::to_string).collect();
                                feeder.feed(args);
                            }
                        }
                        last = mtime;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(40));
            }
        });
    }

    fn woven(&mut self, args: Vec<String>) -> Refresh {
        self.target = match args.first().map(String::as_str) {
            // Round 67: an explicit close (tests, other panels) — before,
            // every unknown word opened the canvas menu.
            Some("hide") => Target::Hidden,
            Some("clips") => Target::Clips {
                offset_x: args.get(1).and_then(|a| a.parse().ok()).unwrap_or(0.0),
                bottom: args.get(2).and_then(|a| a.parse().ok()).unwrap_or(140.0),
                entries: clips_read(),
            },
            Some("dock") => Target::Dock {
                offset_x: args.get(1).and_then(|a| a.parse().ok()).unwrap_or(0.0),
                bottom: args.get(2).and_then(|a| a.parse().ok()).unwrap_or(140.0),
                app_id: args.get(3).cloned().unwrap_or_default(),
                pinned: args.get(4).map(String::as_str) == Some("1"),
                window: args
                    .get(5)
                    .map(|f| f.split(',').filter_map(|t| t.parse().ok()).collect())
                    .unwrap_or_default(),
            },
            _ => Target::Canvas {
                x: args.get(1).and_then(|a| a.parse().ok()).unwrap_or(200.0),
                y: args.get(2).and_then(|a| a.parse().ok()).unwrap_or(200.0),
            },
        };
        // Round 68: the screen of the request — the dock and the clip
        // shelf name theirs (last word), the canvas menu takes the
        // focused screen.
        self.output = match &self.target {
            Target::Dock { .. } => args.get(6).filter(|o| !o.is_empty()).cloned(),
            Target::Clips { .. } => args.get(3).filter(|o| !o.is_empty()).cloned(),
            _ => None,
        }
        .or_else(mk::canvas::focused_output);
        self.place();
        match &self.target {
            Target::Canvas { x, y } => eprintln!("[kontext] Leinwand-Menü bei ({x:.0}, {y:.0})"),
            Target::Clips { entries, .. } => {
                eprintln!("[kontext] Ablage-Menü ({} Einträge)", entries.len())
            }
            Target::Dock { app_id, window, .. } => {
                eprintln!("[kontext] Kachel-Menü für {app_id} ({} Fenster)", window.len())
            }
            Target::Hidden => {}
        }
        // Panes: the veil comes (or goes) with the menu.
        Refresh::Panes
    }

    fn passthrough(&self) -> bool {
        // Round 68: the main surface never listens — menu and veil are panes.
        true
    }

    /// Round 68: the main surface is a 1×1 nothing for good (it used to
    /// BE the menu, Lager 82 — and kept its size when closing went wrong).
    fn extent(&mut self) -> Option<(u32, u32)> {
        Some((1, 1))
    }

    fn view(
        &mut self,
        _engine: &mut TextEngine,
        _keep: &mut Keep<f32>,
        _atlas: &mut matrixkit_image::ImageAtlas,
        _w: u32,
        _h: u32,
    ) -> Scene {
        // Round 68: the main surface stays a 1×1 nothing — the menu and its
        // veil are panes on the requested screen (fleet below).
        Scene::default()
    }

    fn strike(&mut self, role: Option<usize>, _x: f64, _y: f64, source: Source) -> Refresh {
        let origin = source.label();
        match role {
            Some(VEIL) => {
                eprintln!("[kontext] daneben ({origin}) — das Menü weicht");
                self.target = Target::Hidden;
                self.place();
                Refresh::Panes
            }
            Some(r) => {
                let Some(&id) = self.deeds.get(r) else { return Refresh::Calm };
                match &self.target {
                    Target::Canvas { .. } => {
                        if let Some((_, title, cmd)) = ENTRIES.get(id) {
                            eprintln!("[kontext] Eintrag ({origin}): „{title}“ → {cmd}");
                            // "name area" → wish file (R105 protocol),
                            // then start via the dev-slot path with reaping.
                            let mut parts = cmd.splitn(2, ' ');
                            let name = parts.next().unwrap_or(cmd);
                            if let Some(bereich) = parts.next() {
                                let _ = std::fs::write(
                                    format!("{}/{name}-area", crate::runtime_dir()),
                                    bereich,
                                );
                            }
                            mkw::panel::app_start(name);
                        }
                    }
                    Target::Clips { entries, .. } => {
                        if id == usize::MAX {
                            eprintln!("[kontext] Ablage geleert");
                            let _ = std::fs::remove_file(crate::clips_path());
                        } else if let Some(e) = entries.get(id) {
                            let ok = mk::clipboard::copy(e);
                            eprintln!(
                                "[kontext] Ablage ({origin}): Eintrag {id} {}",
                                if ok { "zurück in die Zwischenablage" } else { "— wl-copy fehlt" }
                            );
                        }
                    }
                    Target::Dock { app_id, pinned, window, .. } => match id {
                        0 => {
                            let mut pins: Vec<String> = mk::settings::read("dock-pins")
                                .unwrap_or_default()
                                .split_whitespace()
                                .map(String::from)
                                .collect();
                            if *pinned {
                                pins.retain(|p| p != app_id);
                            } else {
                                pins.push(app_id.clone());
                            }
                            mk::settings::write("dock-pins", &pins.join(" "));
                            eprintln!(
                                "[kontext] Kachel ({origin}): {app_id} {}",
                                if *pinned { "losgelöst" } else { "angepinnt" }
                            );
                        }
                        1 => {
                            eprintln!(
                                "[kontext] Kachel ({origin}): {} Fenster von {app_id} schließen",
                                window.len()
                            );
                            for id in window {
                                mk::canvas::window_close(*id);
                            }
                        }
                        _ => {
                            // Round 113: «Anpassen» puts dock and bar into the
                            // wiggle mode (the long press's second stage).
                            mk::edit::enter();
                            eprintln!("[kontext] Kachel ({origin}): Anpassen — Bearbeiten-Modus");
                        }
                    },
                    Target::Hidden => {}
                }
                self.target = Target::Hidden;
                self.place();
                Refresh::Panes
            }
            None => Refresh::Calm,
        }
    }

    /// Round 66/68: an open menu is TWO panes on the request's screen —
    /// the VEIL (glass-free `matrix-veil`, TOP layer: under the menu, over
    /// everything else; a tap beside closes) and the MENU itself
    /// (`matrix-card`, OVERLAY, exactly menu-sized so niri lays its glass
    /// as a card). Closed: no panes at all.
    fn fleet(&mut self) -> Vec<shell::fleet::Spec<Pane>> {
        let Some((_, _, w, h)) = self.geom else { return Vec::new() };
        // A new screen needs new surfaces: its name rides in the signature.
        let screen = self.output.as_deref().map_or(0, |o| o.bytes().fold(7u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32)));
        vec![
            shell::fleet::Spec { pane: Pane::Veil, width: 0, height: 0, cross: 0, namespace: "matrix-veil", keyboard: false, sig: (1, screen) },
            shell::fleet::Spec { pane: Pane::Menu, width: w.ceil() as u32, height: h.ceil() as u32, cross: 0, namespace: "matrix-card", keyboard: false, sig: (2, screen) },
        ]
    }

    fn pane_place(&mut self, card: &Pane) -> Option<shell::PanePlace> {
        let mut place = match card {
            Pane::Veil => shell::PanePlace::veil(self.output.clone(), Some(shell::Layer::Top), false),
            Pane::Menu => {
                let (x, y, w, h) = self.geom?;
                shell::PanePlace::at(
                    shell::Anchors { top: true, right: false, bottom: false, left: true },
                    (y.round() as i32, 0, 0, x.round() as i32),
                    (w.ceil() as u32, h.ceil() as u32),
                    self.output.clone(),
                    Some(shell::Layer::Overlay),
                )
            }
        };
        place.keyboard = shell::KeyboardWish::None;
        Some(place)
    }

    fn pane_scene(&mut self, card: &Pane, engine: &mut TextEngine, _atlas: &mut matrixkit_image::ImageAtlas, w: u32, h: u32) -> Option<Scene> {
        match card {
            Pane::Menu => Some(self.menu_scene(engine)),
            Pane::Veil => {
                let mut placed = Vec::new();
                lay_out(&Node::Quad { w: w as f32, h: h as f32, color: mk::Rgba { a: 0.0, ..self.palette.surface }, radius: 0.0 }, 0.0, 0.0, &mut placed);
                let roles = vec![Some(VEIL); placed.len()];
                Some(Scene { placed, roles, ..Scene::default() })
            }
        }
    }

    fn pane_strike(&mut self, card: &Pane, x: f64, y: f64, source: Source) -> Refresh {
        match card {
            Pane::Veil => self.strike(Some(VEIL), x, y, source),
            Pane::Menu => {
                let y = y as f32;
                match self.row_hits.iter().find(|(top, bottom, _)| y >= *top && y < *bottom) {
                    Some(&(_, _, deed)) => self.strike(Some(deed), x, y as f64, source),
                    None => Refresh::Calm,
                }
            }
        }
    }

    fn pane_hold(&mut self, card: &Pane, x: f64, y: f64, source: Source) -> Refresh {
        match card {
            Pane::Veil => self.pane_strike(card, x, y, source),
            Pane::Menu => Refresh::Calm,
        }
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

pub fn run(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Our menus lie on compositor glass cards (R125) — the
    // family then paints thin shells instead of its own background.
    // The iced path has always set the switch; here it was missing,
    // and the menu came out opaque instead of glassy (finger probe
    // 17.08.2026).
    mkw::ui::menus_on_glass();
    eprintln!("[kontext] Matrix Kontext: zwölfter App-Umzug — der durchlässige Dienst, pid={}", std::process::id());
    let initial = if args.is_empty() { None } else { Some(args) };
    shell::run(
        Kontext::new(initial),
        shell::Options {
            mount: shell::Mount::Layer {
                // matrix-CARD like the reference, not matrix-kontext: the
            // compositor rule (glass.kdl) only knows six
            // namespaces, and matrix-kontext is not one of them —
            // so the context menu lost its glass effect
            // (finger probe 16.08.2026).
            namespace: String::from("matrix-card"),
                layer: shell::Layer::Overlay,
                keyboard: shell::KeyboardWish::None,
                // Anchored top left and MENU-SIZED: the menu
                // is the surface (stage 82). extent() and offset()
                // set size and position at runtime; only then does the
                // compositor lay its glass under it as a CARD instead of over
                // the whole screen.
                anchor: shell::Anchors { top: true, right: false, bottom: false, left: true },
                size: (1, 1),
                margin: (0, 0, 0, 0),
                exclusive: -1,
            },
            argb: true,
            tag: "kontext",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}
