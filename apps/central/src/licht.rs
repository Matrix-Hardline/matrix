//! Matrix Zentrale, first-light version (iced replacement,
//! camp 34/36 — GRABUNG-ICED-ABLOESUNG.md): the control centre
//! on the ISLAND FLEET.
//!
//! Since camp 36 the control centre is again what it was on the iced
//! path: ONE glass per category group. The main surface is only
//! the veil now (a click closes); the groups live on
//! fleet surfaces (matrix-card), which LISTEN since camp 35 —
//! flip switches, drag sliders (pane_strike/travel/release,
//! surface-local), Refresh::Panes repaints with fresh cards.
//! The registry stays verbatim (REGISTRY + user list),
//! the readers/writers the same; brightness lives on the
//! loom (DDC is sluggish).
//!
//! Since camp 44 it carries the EDIT FLOW (R122 phase 2,
//! first cut): the footer island toggles, removable
//! rows carry ✕ (fixed ones stay), the GALLERY island offers
//! everything not yet placed — every change saves immediately
//! (central-controls, the registry family).
//!
//! Notes for later: dragging/reordering rows, gallery search,
//! rotation lock, slider family with the second customer,
//! compositor glass under the islands (the iced islands carry it).

use crate::{
    bt_read, brightness_read, brightness_set, sound_read, wlan_read, BrightnessSource,
};
use matrixkit_shell::{self as shell, Refresh, Scene, Source};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{lay_out, Keep, Node, Placed, Rect};
use matrixkit_widgets as mkw;
use matrixkit_widgets::scene as fam;

const WIDTH: u32 = 340;
const PAD: f32 = 14.0;
/// The row padding of the reference (mkw::row_item: S + 2 top and
/// bottom). The row height follows from that and the TALLEST
/// element — in a switch row that is the switch.
const ROW_PAD: f32 = mk::spacing::S + 2.0;

/// A switch row: switch height + padding. Before, a 44 stood
/// here — measured at the reference it is 51 logical pixels
/// (finger probe 17.08.2026), and the rule yields exactly that.
fn row_toggle() -> f32 {
    mk::control::TOGGLE_TRACK_H + 2.0 * ROW_PAD
}

/// A slider row is just as tall — the slider is
/// TRAILING, it does not drive the height.
fn row_slider() -> f32 {
    row_toggle()
}

/// The slider width of the reference (mkw::bar_slider .width(150)).
const SLIDER_W: f32 = 150.0;
/// A row with a subtitle needs the second text line.
/// With a subtitle a second text line is added — it grows
/// along with the font size setting.
fn row_subtitle() -> f32 {
    // The TEXT STACK drives the height as soon as it is taller than the
    // switch: title line + 2 + subtitle line, then the padding.
    // (Switch height + ONE line was too coarse — 20 physical pixels
    // too tall, measured 17.08.2026.)
    let stack = mk::typo::BODY.px() * matrixkit_text::LINE_HEIGHT
        + 2.0
        + mk::typo::CAPTION.px() * matrixkit_text::LINE_HEIGHT;
    stack.max(mk::control::TOGGLE_TRACK_H) + 2.0 * ROW_PAD
}

/// Which rows carry a subtitle. The reference gives one ONLY to
/// the three accessibility switches (matrix-zentrale/main.rs);
/// volume, microphone, Wi-Fi and Bluetooth stand there without a
/// description, although mkw::central knows one for all of them
/// (finger probe 16.08.2026).
fn shows_subtitle(key: &str) -> bool {
    matches!(key, "motion" | "transparency" | "contrast")
}

/// The measuring snapshot of the states.
/// What the control centre's threads bring home (round 178: the state joined the brightness).
pub enum Work {
    Brightness(Option<(f32, BrightnessSource)>),
    Stand(Stand),
}

#[derive(PartialEq, Clone, Default)]
pub struct Stand {
    volume: Option<(f32, bool)>,
    mic_muted: Option<bool>,
    wlan: Option<bool>,
    bluetooth: Option<bool>,
    motion_reduced: bool,
    transparency_reduced: bool,
    contrast_up: bool,
}

fn stand_read() -> Stand {
    Stand {
        volume: sound_read("@DEFAULT_AUDIO_SINK@"),
        mic_muted: sound_read("@DEFAULT_AUDIO_SOURCE@").map(|(_, m)| m),
        wlan: wlan_read(),
        bluetooth: bt_read(),
        motion_reduced: mk::motion_reduced(),
        transparency_reduced: mk::transparency_reduced(),
        contrast_up: mk::contrast_up(),
    }
}

/// An island: its control keys plus a snapshot of their values —
/// the CARD of the fleet surface (camp 35: the card says who).
#[derive(Clone)]
pub struct Island {
    keys: Vec<&'static str>,
    stand: Stand,
    brightness: Option<f32>,
    palette: mk::Palette,
    /// Camp 44: in the edit flow the rows carry ✕ instead of
    /// controls.
    edit: bool,
}

/// The card of a control-centre surface (camp 44): a
/// control group, the footer (Bearbeiten/Fertig) or the
/// gallery of controls not yet placed.
#[derive(Clone)]
pub enum Insel {
    Group(Island),
    Footer { edit: bool },
    Gallery(Vec<&'static str>),
}

const FOOTER_H: u32 = 46;
const GALLERY_HEAD: f32 = 30.0;

enum RowKind {
    Toggle(bool),
    Slider(f32),
    Action,
    /// Edit flow: the row shows ✕ (or nothing if
    /// the control is fixed in place — sliders always stay).
    Edit(bool),
}

impl Island {
    /// The island's row measure — ONE truth for painting and
    /// hitting (whoever measures, hits).
    fn rows(&self) -> Vec<(&'static str, f32, f32, RowKind)> {
        let mut out = Vec::new();
        // The card is EXACTLY its rows: at the reference each
        // row carries its own padding (S + 2), the card adds
        // none around it. With PAD at top and bottom every card was
        // 28 logical pixels too tall — measured 180 instead of 124
        // physical between two cards (17.08.2026).
        let mut y = 0.0;
        for key in &self.keys {
            if self.edit {
                let removable =
                    mkw::central::find(key).map(|c| !c.fixed).unwrap_or(true);
                out.push((*key, y, row_toggle(), RowKind::Edit(removable)));
                y += row_toggle();
                continue;
            }
            let kind = match *key {
                "mute" => RowKind::Toggle(self.stand.volume.map(|(_, m)| m).unwrap_or(false)),
                "mic" => RowKind::Toggle(self.stand.mic_muted.unwrap_or(false)),
                "wlan" => RowKind::Toggle(self.stand.wlan.unwrap_or(false)),
                "bluetooth" => RowKind::Toggle(self.stand.bluetooth.unwrap_or(false)),
                "motion" => RowKind::Toggle(self.stand.motion_reduced),
                "transparency" => RowKind::Toggle(self.stand.transparency_reduced),
                "contrast" => RowKind::Toggle(self.stand.contrast_up),
                "volume" => RowKind::Slider(
                    self.stand.volume.map(|(v, _)| v.clamp(0.0, 1.0)).unwrap_or(0.0),
                ),
                "brightness" => RowKind::Slider(
                    self.brightness.map(|v| (v / 100.0).clamp(0.0, 1.0)).unwrap_or(0.0),
                ),
                _ => RowKind::Action,
            };
            // Rows WITH a subtitle are taller — the reference lets
            // row_item grow with its content; here a fixed
            // surcharge suffices, because all subtitles are one line
            // (finger probe 16.08.2026).
            let has_sub = shows_subtitle(key);
            let h = match kind {
                RowKind::Slider(_) => row_slider(),
                _ if has_sub => row_subtitle(),
                _ => row_toggle(),
            };
            out.push((*key, y, h, kind));
            y += h;
        }
        out
    }

    fn height(&self) -> u32 {
        self.rows().last().map(|(_, y, h, _)| y + h).unwrap_or(0.0).ceil() as u32
    }

    /// The slider track of a row (surface-local).
    fn track_of(&self, key: &str) -> Option<Rect> {
        // The same groove that pane() paints — otherwise the finger
        // grabs at nothing.
        self.rows().iter().find(|(k, _, _, _)| *k == key).map(|(_, y, _, _)| Rect {
            x: WIDTH as f32 - PAD - SLIDER_W,
            y: y + 24.0,
            w: SLIDER_W,
            h: 4.0,
        })
    }
}

struct Zentrale {
    palette: mk::Palette,
    controls: Vec<String>,
    stand: Stand,
    brightness: Option<(f32, BrightnessSource)>,
    loom: Option<shell::Loom<Work>>,
    /// The running slider drag: (key, track of the island).
    drag: Option<(&'static str, Rect)>,
    /// The edit flow (camp 44).
    edit: bool,
}

impl Zentrale {
    fn new() -> Self {
        Zentrale {
            palette: mk::Palette::load().unwrap_or_default(),
            controls: mkw::central::load(),
            stand: stand_read(),
            brightness: None,
            loom: None,
            drag: None,
            edit: false,
        }
    }

    /// What the gallery offers: everything from the registry that is
    /// not yet placed in the panel.
    fn missing(&self) -> Vec<&'static str> {
        mkw::central::REGISTRY
            .iter()
            .filter(|c| !self.controls.iter().any(|k| k == c.key))
            .map(|c| c.key)
            .collect()
    }

    /// The groups: consecutive controls of the same category
    /// share one glass (literally the iced island forming).
    fn islands(&self) -> Vec<Island> {
        let mut out: Vec<(std::mem::Discriminant<mkw::central::Category>, Island)> = Vec::new();
        for key in &self.controls {
            let Some(ctl) = mkw::central::find(key) else { continue };
            let cat = std::mem::discriminant(&ctl.category);
            match out.last_mut() {
                Some((last, island)) if *last == cat => island.keys.push(ctl.key),
                _ => out.push((
                    cat,
                    Island {
                        keys: vec![ctl.key],
                        stand: self.stand.clone(),
                        brightness: self.brightness.map(|(v, _)| v),
                        palette: self.palette,
                        edit: self.edit,
                    },
                )),
            }
        }
        out.into_iter().map(|(_, i)| i).collect()
    }

    /// Track position → apply value (the slider follows the finger).
    fn slide(&mut self, key: &'static str, x: f64, track: Rect) -> Refresh {
        let t = (((x as f32) - track.x) / track.w).clamp(0.0, 1.0);
        match key {
            "volume" => {
                let _ = mk::command::silent(
                    "wpctl",
                    &["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{t:.2}")],
                );
                if let Some((_, muted)) = self.stand.volume {
                    self.stand.volume = Some((t, muted));
                }
            }
            "brightness" => {
                if let Some((_, q)) = self.brightness {
                    self.brightness = Some((t * 100.0, q));
                }
            }
            _ => {}
        }
        Refresh::Panes
    }
}

impl shell::Client for Zentrale {
    type Card = Insel;
    type Work = Work;

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn warp(&mut self, loom: &shell::Loom<Work>) {
        self.loom = Some(loom.clone());
        loom.spin(|| Work::Brightness(brightness_read()));
    }

    fn woven(&mut self, w: Work) -> Refresh {
        match w {
            Work::Brightness(Some((v, q))) => {
                eprintln!("[zentrale] Helligkeit vom Faden: {v:.0} ({q:?})");
                self.brightness = Some((v, q));
                Refresh::Panes
            }
            Work::Stand(neu) if neu != self.stand && self.drag.is_none() => {
                self.stand = neu;
                Refresh::Panes
            }
            _ => Refresh::Calm,
        }
    }

    fn base(&mut self, _atlas: &mut matrixkit_image::ImageAtlas, buf: &mut [u8], _w: u32, _h: u32) {
        // The veil: black at 25 % — the session stays perceptible.
        for px in buf.chunks_exact_mut(4) {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
            px[3] = 64;
        }
    }

    fn view(
        &mut self,
        _engine: &mut TextEngine,
        _keep: &mut Keep<f32>,
        _atlas: &mut matrixkit_image::ImageAtlas,
        w: u32,
        h: u32,
    ) -> Scene {
        // The main surface is ONLY the veil (role 0 closes);
        // everything else lives on the islands.
        let clear = mk::Rgba { a: 0.0, ..self.palette.surface };
        let mut placed = Vec::new();
        lay_out(
            &Node::Quad { w: w as f32, h: h as f32, color: clear, radius: 0.0 },
            0.0,
            0.0,
            &mut placed,
        );
        Scene {
            placed,
            roles: vec![Some(0)],
            texts: Vec::new(),
            picts: Vec::new(),
            curves: Vec::new(),
        }
    }

    fn fleet(&mut self) -> Vec<shell::fleet::Spec<Insel>> {
        let cross = mkw::panel::POPUP_RIGHT as i32;
        let mut specs: Vec<shell::fleet::Spec<Insel>> = self
            .islands()
            .into_iter()
            .enumerate()
            .map(|(i, island)| shell::fleet::Spec {
                // OPEN (finger probe 16.08.2026): the reference gives
                // group tiles a TRANSPARENT gutter on both sides
                // (width WIDTH + 2*GUTTER, cross - GUTTER, namespace
                // matrix-tile, whose compositor rule sets glass-inset-x
                // 30) — the minus badges float there. Here
                // it is still missing. RETROFITTING it means not just
                // widening the surface: pane() paints with
                // absolute coordinates across the full width, so
                // EVERY paint call would have to indent by GUTTER.
                // A mere widening measurably pushed the card
                // 30 px out of the picture to the left and right.
                width: WIDTH,
                height: island.height(),
                cross,
                namespace: "matrix-card",
                // Only the gallery takes keys — as at the reference.
                keyboard: false,
                // Height in the signature: the edit flow changes
                // the row measures — the surface then reopens.
                sig: (1, i as u32 * 8192 + island.height()),
                pane: Insel::Group(island),
            })
            .collect();
        // The footer: Bearbeiten/Fertig (always there, like iced).
        // The footer is NARROWER than the cards and centred
        // beneath them — as at the reference.
        let footer_w: u32 = if self.edit { 96 } else { 224 };
        specs.push(shell::fleet::Spec {
            width: footer_w,
            height: FOOTER_H,
            cross: cross + (WIDTH as i32 - footer_w as i32) / 2,
            namespace: "matrix-card",
            keyboard: false,
            sig: (3, self.edit as u32),
            pane: Insel::Footer { edit: self.edit },
        });
        // The gallery: only in the edit flow.
        if self.edit {
            let fehlend = self.missing();
            specs.push(shell::fleet::Spec {
                width: WIDTH,
                height: (2.0 * PAD + GALLERY_HEAD + fehlend.len() as f32 * row_toggle()).ceil()
                    as u32,
                cross,
                namespace: "matrix-card",
                keyboard: true,
                sig: (4, fehlend.len() as u32),
                pane: Insel::Gallery(fehlend),
            });
        }
        specs
    }

    fn pane(&mut self, card: &Insel, engine: &mut TextEngine, canvas: &mut [u8], w: u32, h: u32) {
        let card = match card {
            Insel::Group(island) => island,
            Insel::Footer { edit } => {
                let p = self.palette;
                let mut scene: Vec<Placed> = Vec::new();
                let mut placed = Vec::new();
                lay_out(
                    &Node::Quad { w: w as f32, h: h as f32, color: p.surface, radius: 16.0 },
                    0.0,
                    0.0,
                    &mut placed,
                );
                scene.extend(placed);
                shell::paint(&scene, canvas, w, h, None);
                let body = mk::typo::BODY;
                // Word for word as the reference (main.rs).
                let label = if *edit { "Fertig" } else { "Steuerelemente bearbeiten" };
                let (lw, lh) = engine.measure_styled(label, body.px(), body.weight);
                engine.paint_clip(
                    matrixkit_text::FAMILY,
                    label,
                    body.px(),
                    body.weight,
                    p.primary,
                    (w as f32 - lw) / 2.0,
                    (h as f32 - lh) / 2.0,
                    None,
                    canvas,
                    w,
                    h,
                );
                return;
            }
            Insel::Gallery(fehlend) => {
                let p = self.palette;
                let symfam = mkw::symbol_family();
                let body = mk::typo::BODY;
                let cap = mk::typo::CAPTION;
                let mut scene: Vec<Placed> = Vec::new();
                let mut placed = Vec::new();
                lay_out(
                    &Node::Quad { w: w as f32, h: h as f32, color: p.surface, radius: 16.0 },
                    0.0,
                    0.0,
                    &mut placed,
                );
                scene.extend(placed);
                shell::paint(&scene, canvas, w, h, None);
                engine.paint_clip(
                    matrixkit_text::FAMILY,
                    "Steuer hinzufügen",
                    cap.px(),
                    mk::typo::Weight::Semibold,
                    p.on_surface_variant,
                    PAD,
                    PAD,
                    None,
                    canvas,
                    w,
                    h,
                );
                let mut y = PAD + GALLERY_HEAD;
                for key in fehlend {
                    if let Some(ctl) = mkw::central::find(key) {
                        let gs = ctl.glyph.to_string();
                        let (_, gh) =
                            engine.measure_family(symfam, &gs, mk::typo::symbol_px(18.0), mk::typo::Weight::Normal);
                        engine.paint_clip(symfam, &gs, mk::typo::symbol_px(18.0),
                            mk::typo::Weight::Normal,
                            p.on_surface_variant,
                            PAD,
                            y + (row_toggle() - gh) / 2.0 - 4.0,
                            None,
                            canvas,
                            w,
                            h,
                        );
                        let (_, th) = engine.measure_styled(ctl.title, body.px(), body.weight);
                        engine.paint_clip(
                            matrixkit_text::FAMILY,
                            ctl.title,
                            body.px(),
                            body.weight,
                            p.on_surface,
                            PAD + 30.0,
                            y + (row_toggle() - th) / 2.0 - 4.0,
                            None,
                            canvas,
                            w,
                            h,
                        );
                    }
                    y += row_toggle();
                }
                return;
            }
        };
        let p = card.palette;
        let symfam = mkw::symbol_family();
        let body = mk::typo::BODY;
        let mut scene: Vec<Placed> = Vec::new();
        let mut texts: Vec<(String, &'static str, f32, mk::typo::Weight, mk::Rgba, f32, f32)> =
            Vec::new();

        let mut placed = Vec::new();
        lay_out(
            &Node::Quad { w: w as f32, h: h as f32, color: p.surface, radius: 16.0 },
            0.0,
            0.0,
            &mut placed,
        );
        scene.extend(placed);

        for (key, y, rh, kind) in card.rows() {
            let Some(ctl) = mkw::central::find(key) else { continue };
            let glyph = ctl.glyph.to_string();
            let (_, gh) = engine.measure_family(symfam, &glyph, mk::typo::symbol_px(18.0), mk::typo::Weight::Normal);
            let (_, th) = engine.measure_styled(ctl.title, body.px(), body.weight);
            texts.push((
                glyph,
                symfam,
                18.0,
                mk::typo::Weight::Normal,
                p.on_surface_variant,
                PAD,
                y + (rh - gh) / 2.0,
            ));
            // With a SUBTITLE, title and subtitle move together
            // around the row's centre — like mkw::row_item. The
            // subtitle was missing entirely (finger probe 16.08.2026).
            let cap = mk::typo::CAPTION;
            let sub = (shows_subtitle(key) && !ctl.subtitle.is_empty()).then(|| {
                let (_, sh) = engine.measure_styled(ctl.subtitle, cap.px(), cap.weight);
                sh
            });
            let stack_h = th + sub.map(|sh| sh + 2.0).unwrap_or(0.0);
            // CENTRED in the row, like the reference
            // (row_item: align_y Center in a box with padding
            // S+2). The earlier 26/13 were guessed and put
            // the text 13 logical pixels too high (finger probe
            // 17.08.2026).
            let title_y = y + (rh - stack_h) / 2.0;
            texts.push((
                ctl.title.to_string(),
                matrixkit_text::FAMILY,
                body.px(),
                body.weight,
                p.on_surface,
                PAD + 30.0,
                title_y,
            ));
            if let Some(sh) = sub {
                let _ = sh;
                texts.push((
                    ctl.subtitle.to_string(),
                    matrixkit_text::FAMILY,
                    cap.px(),
                    cap.weight,
                    p.on_surface_variant,
                    PAD + 30.0,
                    title_y + th + 2.0,
                ));
            }
            match kind {
                RowKind::Toggle(on) => {
                    let node = fam::toggle(on, false, p);
                    let mut tmp = Vec::new();
                    let (tw2, th2) = lay_out(&node, 0.0, 0.0, &mut tmp);
                    let mut placed = Vec::new();
                    lay_out(
                        &node,
                        w as f32 - PAD - tw2,
                        y + (row_toggle() - th2) / 2.0 - 4.0,
                        &mut placed,
                    );
                    scene.extend(placed);
                }
                RowKind::Slider(v) => {
                    // The reference places the slider TRAILING next to
                    // the label (150 px, mkw::row_item), not
                    // below it across the full width
                    // (finger probe 16.08.2026).
                    let track = Rect {
                        x: w as f32 - PAD - SLIDER_W,
                        y: y + 24.0,
                        w: SLIDER_W,
                        h: 4.0,
                    };
                    let knob_x = track.x + track.w * v - 9.0;
                    for (rx, rw, rh2, ry, color, rad) in [
                        (
                            track.x,
                            track.w,
                            4.0,
                            track.y,
                            p.on_surface.over(p.surface_container_high, 0.18),
                            2.0,
                        ),
                        (track.x, (track.w * v).max(2.0), 4.0, track.y, p.primary, 2.0),
                        (knob_x, 18.0, 18.0, track.y - 7.0, p.on_primary.over(p.primary, 0.05), 9.0),
                    ] {
                        let mut placed = Vec::new();
                        lay_out(
                            &Node::Quad { w: rw, h: rh2, color, radius: rad },
                            rx,
                            ry,
                            &mut placed,
                        );
                        scene.extend(placed);
                    }
                }
                RowKind::Action => {}
                RowKind::Edit(_) => {}
            }
        }

        // Edit flow: removable rows carry ✕ on the right.
        for (key, y, rh, kind) in card.rows() {
            if let RowKind::Edit(removable) = kind {
                let _ = key;
                if removable {
                    let symfam2 = mkw::symbol_family();
                    let xs = mkw::symbol::CLOSE.to_string();
                    let (xw, xh) =
                        engine.measure_family(symfam2, &xs, 16.0, mk::typo::Weight::Normal);
                    texts.push((
                        xs,
                        symfam2,
                        16.0,
                        mk::typo::Weight::Normal,
                        p.error,
                        w as f32 - PAD - xw,
                        y + (rh - xh) / 2.0 - 4.0,
                    ));
                }
            }
        }
        shell::paint(&scene, canvas, w, h, None);
        for (s, famly, px, wt, tint, x, y) in texts {
            engine.paint_clip(famly, &s, px, wt, tint, x, y, None, canvas, w, h);
        }
    }

    fn pane_strike(&mut self, card: &Insel, x: f64, y: f64, source: Source) -> Refresh {
        let origin = source.label();
        let card = match card {
            Insel::Group(island) => island,
            Insel::Footer { .. } => {
                self.edit = !self.edit;
                eprintln!(
                    "[zentrale] Fußzeile ({origin}): Bearbeiten {}",
                    if self.edit { "AN" } else { "aus — gespeichert" }
                );
                return Refresh::Panes;
            }
            Insel::Gallery(fehlend) => {
                let idx = ((y as f32 - PAD - GALLERY_HEAD) / row_toggle()).floor();
                if idx >= 0.0 {
                    if let Some(key) = fehlend.get(idx as usize) {
                        eprintln!("[zentrale] Galerie ({origin}): „{key}“ kommt ins Panel");
                        self.controls.push(key.to_string());
                        mkw::central::save(&self.controls);
                    }
                }
                return Refresh::Panes;
            }
        };
        let Some((key, _, _, kind)) = card
            .rows()
            .into_iter()
            .find(|(_, ry, rh, _)| (y as f32) >= *ry && (y as f32) < ry + rh
        ) else {
            return Refresh::Calm;
        };
        match kind {
            RowKind::Edit(removable) => {
                if removable {
                    eprintln!("[zentrale] Bearbeiten ({origin}): „{key}“ fliegt aus dem Panel");
                    self.controls.retain(|k| k != key);
                    mkw::central::save(&self.controls);
                    return Refresh::Panes;
                }
                eprintln!("[zentrale] Bearbeiten ({origin}): „{key}“ ist fest verbaut");
                return Refresh::Calm;
            }
            RowKind::Slider(_) => {
                let Some(track) = card.track_of(key) else { return Refresh::Calm };
                eprintln!("[zentrale] Insel-Regler „{key}“ gegriffen ({origin})");
                self.drag = Some((key, track));
                self.slide(key, x, track)
            }
            RowKind::Toggle(_) => {
                eprintln!("[zentrale] Insel-Schalter ({origin}): „{key}“");
                match key {
                    "mute" => {
                        let _ = mk::command::silent(
                            "wpctl",
                            &["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"],
                        );
                        if let Some((v, m)) = self.stand.volume {
                            self.stand.volume = Some((v, !m));
                        }
                    }
                    "mic" => {
                        let _ = mk::command::silent(
                            "wpctl",
                            &["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"],
                        );
                        self.stand.mic_muted = self.stand.mic_muted.map(|m| !m);
                    }
                    "wlan" => {
                        let on = !self.stand.wlan.unwrap_or(false);
                        let _ = mk::command::silent(
                            "nmcli",
                            &["radio", "wifi", if on { "on" } else { "off" }],
                        );
                        self.stand.wlan = Some(on);
                    }
                    "bluetooth" => {
                        let on = !self.stand.bluetooth.unwrap_or(false);
                        let _ = mk::command::silent(
                            "bluetoothctl",
                            &["power", if on { "on" } else { "off" }],
                        );
                        self.stand.bluetooth = Some(on);
                    }
                    "motion" => {
                        let neu = !self.stand.motion_reduced;
                        mk::settings::write("motion", if neu { "reduced" } else { "full" });
                        self.stand.motion_reduced = neu;
                    }
                    "transparency" => {
                        let neu = !self.stand.transparency_reduced;
                        mk::settings::write("transparency", if neu { "reduced" } else { "full" });
                        self.stand.transparency_reduced = neu;
                    }
                    "contrast" => {
                        let neu = !self.stand.contrast_up;
                        mk::settings::write("contrast", if neu { "high" } else { "normal" });
                        if let Some(new) = mk::Palette::load() {
                            self.palette = new;
                        }
                        self.stand.contrast_up = neu;
                    }
                    _ => {}
                }
                Refresh::Panes
            }
            RowKind::Action => {
                eprintln!("[zentrale] Insel-Tat ({origin}): „{key}“");
                match key {
                    "appearance" => mk::light_toggle(),
                    "lock" => mkw::panel::app_start("matrix-lock"),
                    "capture" => mkw::panel::app_start("matrix-capture"),
                    _ => {}
                }
                Refresh::Calm
            }
        }
    }

    fn pane_travel(&mut self, _card: &Insel, x: f64, _y: f64, _source: Source) -> Refresh {
        let Some((key, track)) = self.drag else { return Refresh::Calm };
        self.slide(key, x, track)
    }

    fn pane_release(&mut self, _card: &Insel, x: f64, _y: f64, _source: Source) -> Refresh {
        let Some((key, track)) = self.drag.take() else { return Refresh::Calm };
        let r = self.slide(key, x, track);
        if key == "brightness" {
            if let Some((v, q)) = self.brightness {
                eprintln!("[zentrale] Helligkeit gesetzt: {v:.0} (beim Loslass, gesponnen)");
                if let Some(loom) = &self.loom {
                    loom.spin(move || {
                        brightness_set(v, q);
                        Work::Brightness(None)
                    });
                }
            }
        } else {
            eprintln!("[zentrale] Insel-Regler „{key}“ losgelassen");
        }
        r
    }

    fn strike(&mut self, role: Option<usize>, _x: f64, _y: f64, source: Source) -> Refresh {
        if role == Some(0) {
            eprintln!("[zentrale] Schleier getroffen ({}) — die Zentrale weicht", source.label());
            return Refresh::Close;
        }
        Refresh::Calm
    }

    fn key(&mut self, key: shell::Key, _shift: bool) -> Refresh {
        if key == shell::Key::Escape {
            eprintln!("[zentrale] Escape — die Zentrale weicht");
            return Refresh::Close;
        }
        Refresh::Calm
    }

    fn pulse(&mut self) -> Refresh {
        let fresh = mk::Palette::load().unwrap_or_default();
        if fresh != self.palette {
            self.palette = fresh;
            return Refresh::Panes;
        }
        if self.drag.is_some() {
            return Refresh::Calm;
        }
        // Round 178: the state (nmcli, bluetoothctl …) is read on a thread, one reading at a time — on the loop a
        // hung tool froze the control centre.
        if let Some(loom) = &self.loom {
            loom.spin_once("central-stand", || Work::Stand(stand_read()));
        }
        Refresh::Calm
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!(
        "[zentrale] Matrix Zentrale: die Inseln hören zu (Lager 36), pid={}",
        std::process::id()
    );
    shell::run(
        Zentrale::new(),
        shell::Options {
            mount: shell::Mount::Layer {
                namespace: String::from("matrix-zentrale"),
                layer: shell::Layer::Overlay,
                keyboard: shell::KeyboardWish::OnDemand,
                anchor: shell::Anchors::all(),
                size: (0, 0),
                margin: (0, 0, 0, 0),
                exclusive: -1,
            },
            argb: true,
            tag: "zentrale",
            fleet_start: mkw::panel::BAR_STRIP + mkw::panel::POPUP_TOP as i32,
            fleet_gap: mk::spacing::M as i32,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}
