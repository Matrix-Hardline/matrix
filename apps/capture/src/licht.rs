//! Matrix Aufnahme (capture), first-light version (iced replacement, Lager 67 —
//! GRABUNG-ICED-ABLOESUNG.md).
//!
//! Two surfaces, two clients: the PANEL (the capture menu under
//! its bar widget) and the THUMBNAIL (the picture that stays at the bottom
//! right for six seconds after the shot).
//!
//! The panel is almost a one-to-one mapping: the iced version builds
//! `mkw::ui::MenuItem` entries and hands them to the menu family —
//! since Lager 53 the same grammar exists without iced (`fam::menu`),
//! so only the target language changes, not the structure.
//!
//! Shadow doctrine: `--erstlicht` runs NEXT TO the iced path.

use matrixkit_shell::{self as shell, Key, Pict, Refresh, Scene, Source, Text};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{lay_out, Keep, Node, Placed, Rect};
use matrixkit_widgets as mkw;
use matrixkit_widgets::scene as fam;
use std::time::Instant;

const THUMB_VISIBLE_S: u64 = 6;
const THUMB_B: f32 = 236.0;
const THUMB_H: f32 = 156.0;

// ------------------------------------------------------------ the panel

struct Panel {
    palette: mk::Palette,
    film_running: bool,
    film_height: Option<u32>,
    native: Option<(u32, u32)>,
    hovered: Option<usize>,
    /// What each entry does — the deed list (role = index).
    deeds: Vec<Deed>,
}

#[derive(Clone)]
enum Deed {
    Action(&'static str),
    Resolution(u32),
}

impl Panel {
    fn new(film_running: bool, film_height: Option<u32>, native: Option<(u32, u32)>) -> Self {
        Self {
            palette: mk::Palette::load().unwrap_or_default(),
            film_running,
            film_height,
            native,
            hovered: None,
            deeds: Vec::new(),
        }
    }

    /// The entries VERBATIM as in the iced path — only in the language
    /// of the scene menu family.
    fn entries(&mut self) -> Vec<fam::MenuEntry> {
        self.deeds.clear();
        let mut out = Vec::new();
        let item = |glyph: Option<char>, title: &str, danger: bool, deed: Option<Deed>,
                        out: &mut Vec<fam::MenuEntry>, deeds: &mut Vec<Deed>| {
            out.push(fam::MenuEntry::Item {
                glyph,
                title: title.to_string(),
                shortcut: None,
                badge: None,
                danger,
                enabled: deed.is_some(),
            });
            deeds.push(deed.unwrap_or(Deed::Resolution(u32::MAX)));
        };
        item(Some(mkw::symbol::IMAGE), "Ganzen Bildschirm fotografieren", false,
             Some(Deed::Action("photo")), &mut out, &mut self.deeds);
        item(Some(mkw::symbol::SEARCH), "Bereich auswählen …", false,
             Some(Deed::Action("region")), &mut out, &mut self.deeds);
        item(Some(mkw::symbol::APPS), "Fenster fotografieren", false,
             Some(Deed::Action("fenster")), &mut out, &mut self.deeds);
        out.push(fam::MenuEntry::Separator);
        self.deeds.push(Deed::Resolution(u32::MAX));
        if self.film_running {
            item(Some(mkw::symbol::CLOSE), "Film beenden", true,
                 Some(Deed::Action("recording-stop")), &mut out, &mut self.deeds);
        } else {
            item(Some(mkw::symbol::PLAY_ARROW), "Bildschirm filmen", false,
                 Some(Deed::Action("recording-one")), &mut out, &mut self.deeds);
            // Round 131: a duo, opened — both panels in one film.
            if mk::device::panels().is_some() && matches!(mk::device::fold(), mk::device::Fold::Open) {
                item(Some(mkw::symbol::PLAY_ARROW), "Beide Bildschirme filmen", false,
                     Some(Deed::Action("recording-both")), &mut out, &mut self.deeds);
            }
        }
        out.push(fam::MenuEntry::Separator);
        self.deeds.push(Deed::Resolution(u32::MAX));
        // The section title is a DISABLED entry — the
        // reference never hides, it greys out (Lager 53).
        item(None, "Filmauflösung", false, None, &mut out, &mut self.deeds);
        let native_title = match self.native {
            Some((b, h)) => format!("Nativ ({b} × {h})"),
            None => String::from("Nativ (Displayauflösung)"),
        };
        let mut levels = vec![(0u32, native_title)];
        for h in [1440u32, 1080, 720] {
            if self.native.is_none_or(|(_, nh)| h < nh) {
                levels.push((h, format!("{h}p")));
            }
        }
        for (h, title) in levels {
            let selected =
                if h == 0 { self.film_height.is_none() } else { self.film_height == Some(h) };
            item(selected.then_some(mkw::symbol::CHECK), &title, false,
                 Some(Deed::Resolution(h)), &mut out, &mut self.deeds);
        }
        out
    }
}

impl shell::Client for Panel {
    type Card = ();
    type Work = ();

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn view(
        &mut self,
        engine: &mut TextEngine,
        _keep: &mut Keep<f32>,
        _atlas: &mut matrixkit_image::ImageAtlas,
        w: u32,
        h: u32,
    ) -> Scene {
        let p = self.palette;
        let mut scene: Vec<Placed> = Vec::new();
        let mut roles: Vec<Option<usize>> = Vec::new();
        let mut texts: Vec<Text> = Vec::new();

        // The veil: a click beside it closes (R103 grammar).
        scene.push(Placed {
            x: 0.0, y: 0.0, w: w as f32, h: h as f32,
            color: mk::Rgba { a: 0.0, ..p.surface },
            radius: 0.0, clip: None, shadow: None,
        });
        roles.push(Some(usize::MAX));

        let entries = self.entries();
        let menu = fam::menu(engine, Some("Aufnahme"), &entries, self.hovered, p);
        let mut placed = Vec::new();
        lay_out(&menu.node, 0.0, 0.0, &mut placed);
        let (mw, mh) = placed.iter().fold((0.0f32, 0.0f32), |(aw, ah), q| {
            (aw.max(q.x + q.w), ah.max(q.y + q.h))
        });
        let _ = mh;
        // The card sits top right under the bar — the same
        // popup grammar as for every other bar panel.
        let ox = w as f32 - mw - mkw::panel::POPUP_RIGHT;
        let oy = mkw::panel::POPUP_TOP;
        let first = scene.len();
        for mut q in placed {
            q.x += ox;
            q.y += oy;
            scene.push(q);
        }
        let mut menu_roles: Vec<Option<usize>> = vec![None; scene.len() - first];
        for (i, slot) in menu.item_slots.iter().enumerate() {
            if let Some(s) = slot {
                menu_roles[*s] = Some(i);
            }
        }
        roles.extend(menu_roles);
        for (i, label) in menu.labels.iter().enumerate() {
            let q = scene[first + menu.label_slots[i]];
            texts.push(Text {
                s: label.text.clone(),
                px: label.px,
                weight: label.weight,
                family: label.family,
                color: label.color,
                x: q.x,
                y: q.y,
                home: Rect::of(&q),
                anchor: first + menu.label_slots[i],
                clip: None,
            });
        }
        Scene { placed: scene, roles, texts, picts: Vec::new(), curves: Vec::new() }
    }

    fn motion(&mut self, _x: f64, _y: f64) -> Refresh {
        Refresh::Calm
    }

    fn strike(&mut self, role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        match role {
            Some(usize::MAX) | None => Refresh::Close,
            Some(i) => match self.deeds.get(i).cloned() {
                Some(Deed::Action(mode)) => {
                    let _ = std::process::Command::new(
                        matrixkit_theme::process::own_exe_result().unwrap_or_default(),
                    )
                    .arg(mode)
                    .spawn();
                    eprintln!("[aufnahme] {mode}");
                    Refresh::Close
                }
                Some(Deed::Resolution(h)) if h != u32::MAX => {
                    if h == 0 {
                        let _ = mk::settings::write("capture-resolution", "");
                        self.film_height = None;
                    } else {
                        let _ = mk::settings::write("capture-resolution", &h.to_string());
                        self.film_height = Some(h);
                    }
                    eprintln!("[aufnahme] Filmauflösung: {h}");
                    Refresh::Dirty
                }
                _ => Refresh::Calm,
            },
        }
    }

    fn key(&mut self, key: Key, _shift: bool) -> Refresh {
        match key {
            Key::Escape => Refresh::Close,
            _ => Refresh::Calm,
        }
    }
}

pub fn run_panel(
    film_running: bool,
    film_height: Option<u32>,
    native: Option<(u32, u32)>,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[aufnahme] Panel im Ersten Licht, pid={}", std::process::id());
    shell::run(
        Panel::new(film_running, film_height, native),
        shell::Options {
            mount: shell::Mount::Layer {
                namespace: String::from("matrix-capture"),
                layer: shell::Layer::Overlay,
                keyboard: shell::KeyboardWish::OnDemand,
                anchor: shell::Anchors::all(),
                size: (0, 0),
                margin: (mkw::panel::BAR_STRIP, 0, 0, 0),
                exclusive: -1,
            },
            argb: true,
            tag: "aufnahme",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}

// ---------------------------------------------------------- the thumbnail

struct Thumb {
    palette: mk::Palette,
    path: std::path::PathBuf,
    is_photo: bool,
    since: Instant,
}

impl shell::Client for Thumb {
    type Card = ();
    type Work = ();

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn view(
        &mut self,
        engine: &mut TextEngine,
        _keep: &mut Keep<f32>,
        _atlas: &mut matrixkit_image::ImageAtlas,
        _w: u32,
        _h: u32,
    ) -> Scene {
        let p = self.palette;
        let mut scene: Vec<Placed> = Vec::new();
        let mut roles: Vec<Option<usize>> = Vec::new();
        let mut texts: Vec<Text> = Vec::new();
        let mut picts: Vec<Pict> = Vec::new();

        // The card: edge, ground, the picture inside — the card family
        // (Lager 54) carries the concentric ring.
        let card = fam::card(
            Node::Quad {
                w: THUMB_B - 2.0 * mk::spacing::M,
                h: THUMB_H - 2.0 * mk::spacing::M,
                color: mk::Rgba { a: 0.0, ..p.surface },
                radius: 0.0,
            },
            p,
        );
        lay_out(&card, 0.0, 0.0, &mut scene);
        roles.extend(std::iter::repeat(Some(0)).take(scene.len()));
        // What floats casts a shadow (Lager 64).
        fam::lift(&mut scene, 0, fam::floating_shadow());

        if self.is_photo {
            picts.push(Pict {
                path: self.path.clone(),
                box_w: (THUMB_B - 2.0 * mk::spacing::M) as u32,
                box_h: (THUMB_H - 2.0 * mk::spacing::M) as u32,
                fit: matrixkit_image::Fit::Contain,
                x: mk::spacing::M + 1.0,
                y: mk::spacing::M + 1.0,
                radius: mk::radius::SMALL,
                home: Rect::of(&scene[0]),
                anchor: 0,
                clip: None,
            });
        } else {
            // A film has no preview image — the glyph says so.
            let s = mkw::symbol::PLAY_ARROW.to_string();
            let (gw, gh) = engine.measure_family(mkw::symbol_family(), &s, mk::typo::symbol_px(44.0),
                mk::typo::Weight::Normal,
            );
            texts.push(Text {
                s,
                px: mk::typo::symbol_px(44.0),
                weight: mk::typo::Weight::Normal,
                family: mkw::symbol_family(),
                color: p.on_surface_variant,
                x: (THUMB_B - gw) / 2.0,
                y: (THUMB_H - gh) / 2.0,
                home: Rect::of(&scene[0]),
                anchor: 0,
                clip: None,
            });
        }
        Scene { placed: scene, roles, texts, picts, curves: Vec::new() }
    }

    fn strike(&mut self, _role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        // A tap opens the capture — the same gesture as in the
        // iced path.
        mkw::panel::app_start(&format!("xdg-open {}", self.path.display()));
        Refresh::Close
    }

    fn beat(&mut self) -> Refresh {
        // The dwell: six seconds, then it leaves by itself.
        if self.since.elapsed().as_secs() >= THUMB_VISIBLE_S {
            return Refresh::Close;
        }
        Refresh::Calm
    }
}

pub fn run_thumb(path: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let is_photo = path
        .extension()
        .map(|e| e.eq_ignore_ascii_case("png"))
        .unwrap_or(false);
    let edge = mkw::panel::SHADOW_MARGIN as u32;
    eprintln!("[aufnahme] Miniatur im Ersten Licht: {}", path.display());
    shell::run(
        Thumb {
            palette: mk::Palette::load().unwrap_or_default(),
            path,
            is_photo,
            since: Instant::now(),
        },
        shell::Options {
            mount: shell::Mount::Layer {
                namespace: String::from("matrix-capture"),
                layer: shell::Layer::Overlay,
                keyboard: shell::KeyboardWish::None,
                anchor: shell::Anchors { top: false, right: true, bottom: true, left: false },
                size: (THUMB_B as u32 + 2 * edge, THUMB_H as u32 + 2 * edge),
                margin: (0, 16 - mkw::panel::SHADOW_MARGIN as i32, 140 - mkw::panel::SHADOW_MARGIN as i32, 0),
                exclusive: 0,
            },
            argb: true,
            tag: "miniatur",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}
