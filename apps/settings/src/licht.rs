//! Matrix Einstellungen in the window house (R212, the owner 02.11.2026:
//! «a settings app without iced, to replace the old one»).
//!
//! The sidebar of all fourteen areas, the page on the right (on a narrow
//! window: the list first, then the page with «‹ Einstellungen»). Pages
//! are laid with the Kit's `fam::FormPage`; the data comes from the same
//! functions the iced panels use (overview, connections, language,
//! input devices) — one truth, two faces, until the iced face retires.
//!
//! Stage 1 carries: Übersicht, Allgemein, Verbindungen (Wi-Fi with
//! password, Bluetooth with pairing), Bedienung, Sprache & Region. The
//! other areas say so and open the previous version (`--iced`) on that
//! area — nothing is lost while the rest moves house.

use crate::{AREAS, AREA_TITLE, APP_ID, APPEARANCES, HARDLINE_AREA, RADIUS_MAX, RADIUS_MIN};
use matrixkit_image::ImageAtlas;
use matrixkit_scene::scene as fam;
use matrixkit_shell::{self as shell, Loom, Refresh, Scene, SceneBuilder, Source};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{Keep, Rect};

const CHROME_BASE: usize = 500;
const ROOT_BASE: usize = 700;
/// Sidebar entries `SIDE_BASE + position in areas()` (round 70: the order
/// is no longer the area number); the narrow form's way back.
const SIDE_BASE: usize = 100;
const BACK: usize = 130;
// Leinwand (canvas) > Erscheinungsbild, Formen & Symbole (appearance, shapes & symbols) (round 70: from «Allgemein»).
const APPEAR_BASE: usize = 1000;
const MOTION: usize = 1010;
const GLASS: usize = 1011;
const TEXT_SIZE: usize = 1012; // and +1
const RADIUS: usize = 1014; // and +1
const SYMBOL_BASE: usize = 1020;
const ICON_BASE: usize = 1025;
/// Round 88: the icon theme (+ index into `matrixkit_icons::themes()`).
const THEME_BASE: usize = 2860; // the theme families, +0..+7
/// 06.11.2026: the icon theme as two dropdowns — family, then its variant.
const VARIANT_BASE: usize = 2868; // +0..+5
const THEME_OPEN: usize = 2876;
const VARIANT_OPEN: usize = 2877;
const FAMILIES_SHOWN: usize = 8;
const VARIANTS_SHOWN: usize = 6;
/// Round 96: theme icons in the Matrix form.
const THEME_FORM: usize = 2875;
// Verbindungen.
const WLAN: usize = 1100;
const BT: usize = 1101;
const SCAN: usize = 1102;
const RELOAD: usize = 1103;
const PW_OK: usize = 1105;
const PW_CANCEL: usize = 1106;
/// The password card's eye and its veil (03.11.2026).
const PW_EYE: usize = 1107;
const PW_VEIL: usize = 1108;
const NET_BASE: usize = 1200;
const BT_DEV_BASE: usize = 1300;
const NEARBY_BASE: usize = 1400;
// Bedienung.
const PAD_SPEED: usize = 1500;
const MOUSE_SPEED: usize = 1501;
const NATURAL: usize = 1502;
const TAP: usize = 1503;
// Sprache & Region (language & region).
const LANG_BASE: usize = 1600;
const REGION_BASE: usize = 1610;
const LAYOUT_BASE: usize = 1620;
/// «In der bisherigen Fassung öffnen» (open in the previous version).
const OPEN_OLD: usize = 1700;
// Leiste & Dock (bar & dock) (round 57).
const DOCK_STYLE_BASE: usize = 1730; // and +1
const DOCK_HIDE: usize = 1735;
// Leiste: notifications as the island (round 105).
const BAR_ISLAND: usize = 1736;
// Ton (sound) — hearing protection (R213).
const EAR_ON: usize = 1800;
const EAR_MAX: usize = 1801;
const EAR_START: usize = 1802;
const EAR_EVENTS: usize = 1803;
/// Per Bluetooth audio device: Kopfhörer (headphones, +0) | Lautsprecher (speaker, +1), two roles each.
const KIND_BASE: usize = 1900;
/// Paired laptops (Morpheus): withdraw, two taps.
const LAPTOP_BASE: usize = 2620;
const LAPTOPS_SHOWN: usize = 40;
// Ton (sound): output, input, sounds.
const OUT_BASE: usize = 2100;
const IN_BASE: usize = 2200;
const OUT_VOL: usize = 2300;
const IN_VOL: usize = 2301;
const TEST_LEFT: usize = 2302; // and +1 right
const MIC_TEST: usize = 2304;
const SOUNDS_ALL: usize = 2305;
const SOUND_BASE: usize = 2400;
const PLAY_BASE: usize = 2500;
/// Background (area 5, 02.11.2026: «I miss backgrounds»): for which
/// appearance, and the gallery tiles.
const WP_FOR_BASE: usize = 2690;
// Anzeige (display) (round 70, the owner: «a group Display with sizes …»).
/// Display size choices (+0 … +3), see `SCALES`.
const SCALE_BASE: usize = 2800;
const BRIGHT: usize = 2810;
const IDENTIFY: usize = 2811;
const AUTO_BRIGHT: usize = 2812;
// Night Shift (round 71): Aus · Automatisch · Immer (off · automatic · always, +0..+2), warmth slider.
const NIGHT_BASE: usize = 2820; // +0..+3: Aus · Automatisch · Zeitplan · Immer (off · automatic · schedule · always)
const NIGHT_WARM: usize = 2825;
/// Round 72: the schedule's edges, −/+ half an hour each.
const NIGHT_FROM: usize = 2830; // and +1
const NIGHT_TO: usize = 2832; // and +1
// Drehung (round 73): «Automatisch drehen» and the fixed orientation (+0..+3).
const ROT_AUTO: usize = 2840;
const ROT_BASE: usize = 2841;
// Bildschirme (round 85): the back panel fully off while folded.
const BACK_OFF: usize = 2846;
// Round 158: the screen on USB-C — shut like a laptop, folded back as a
// second monitor, sound, resolution (+0..+1) and size (+0..+2).
// Round 169: moved out of THEME_BASE's 2860..2872 (the two collided).
const DESK_CLOSED: usize = 2900;
const DESK_FOLDED: usize = 2901;
const DESK_SOUND: usize = 2902;
const DESK_RES_BASE: usize = 2903;
const DESK_SCALE_BASE: usize = 2906;
// Ruhezustand (round 78): deep sleep on/off and how soon (+0..+3, `SLEEP_DELAYS`).
const DEEP_ON: usize = 2850;
// Bildschirmaufnahme (round 102): record what the device plays along.
const FILM_SOUND: usize = 2880;
// Bedienung (operation) (round 120): vibration on/off (mk::feel).
const HAPTICS: usize = 2881;
/// Round 161: the mouse wheel's speed (+0..+2, mk::input::WHEEL_SPEEDS).
const WHEEL_BASE: usize = 2910;
/// Round 164: a long press on nothing moves the window.
const WINDOW_MOVE: usize = 2915;
/// Round 135: the lock screen's clock — glass or solid, tall or regular.
const LOCK_GLASS: usize = 2882;
const LOCK_TALL: usize = 2883;
/// Round 136: the power key locks (duo-powerkey reads the same file).
const LOCK_POWER: usize = 2884;
const DEEP_DELAY_BASE: usize = 2851;
/// Display sizes on the Duo — only scales that divide both panel sides
/// (1350 × 1800) exactly, plus the proven 2.75; any other overlaps the
/// panels by a point and makes them noise («dsi FIFO UNDERFLOW»).
/// duo-haltung reads the same file and allows the same four.
const SCALES: [(f32, &str); 4] = [(3.0, "Grösser"), (2.75, "Standard"), (2.5, "Mehr Platz"), (2.25, "Noch mehr Platz")];
/// The area «Anzeige».
const DISPLAY_AREA: usize = 14;
const WP_BASE: usize = 2700;
/// Gallery thumbnails on disk (16:9; tiles draw at most ~220 px wide).
const THUMB_W: u32 = 448;
const THUMB_H: u32 = 252;

/// Below this width the sidebar and the page take turns.
const NARROW: f32 = 620.0;
const SIDE_W: f32 = 236.0;

/// The areas this face carries already.
const CARRIED: [usize; 10] = [0, 1, 3, 8, 10, 4, 7, 5, 6, 14];

/// A symbol per area (sidebar).
const AREA_GLYPH: [char; AREAS] = [
    '\u{e871}', '\u{e8b8}', '\u{e8d7}', '\u{e63e}', '\u{e8f1}', '\u{e1bc}', '\u{e30e}', '\u{e050}', '\u{e30f}', '\u{e86f}', '\u{e894}', '\u{e7fb}',
    '\u{e32a}', '\u{e157}', '\u{e30c}',
];

/// SIDEBAR GROUPS (owner 02.11.2026: «Connections expands as a group,
/// below it Wi-Fi and Bluetooth; if I tap something else, it goes back to
/// normal»): an area with parts shows them indented below itself while it
/// is the chosen one. (area, [(part name, symbol)]); roles `SUB_BASE + k`.
const GROUPS: [(usize, &[(&str, char)]); 4] = [
    // Morpheus (06.11.2026): the laptops trusted over the USB cable.
    (3, &[("WLAN", '\u{e63e}'), ("Bluetooth", '\u{e1a7}'), ("Laptops", '\u{e31e}')]),
    // Round 70 (owner 03.11.2026): «Anzeige» with sizes, brightness and
    // screens; «Leinwand» with appearance and forms (both out of «Allgemein»).
    (14, &[("Grössen", '\u{e262}'), ("Helligkeit", '\u{e1ae}'), ("Night Shift", '\u{ef44}'), ("Drehung", '\u{e1c1}'), ("Bildschirme", '\u{e30c}'), ("Ruhezustand", '\u{e8ac}')]),
    // Round 74: «Hintergrund» joined «Leinwand» (owner: «Background should also go into the canvas group»).
    (4, &[("Erscheinungsbild", '\u{e40a}'), ("Formen & Symbole", '\u{e574}'), ("Hintergrund", '\u{e3f4}'), ("Sitzung", '\u{e8f1}')]),
    (7, &[("Ausgabe", '\u{e050}'), ("Eingabe", '\u{e029}'), ("Klänge", '\u{e7f4}'), ("Gehörschutz", '\u{e310}')]),
];
const SUB_BASE: usize = 140;

/// The parts of an area (empty: no group).
fn parts(area: usize) -> &'static [(&'static str, char)] {
    GROUPS.iter().find(|(a, _)| *a == area).map(|(_, p)| *p).unwrap_or(&[])
}

/// The area's key for the previous version's wish file (`area_wish`).
const AREA_KEY: [&str; AREAS] =
    ["overview", "general", "update", "connections", "canvas", "wallpaper", "bars", "sounds", "controls", "dev", "language", "users", "apps", "hardline", "display"];

enum Work {
    /// The status, and whether it is the full one (after a fresh scan).
    Conn(crate::connections::Status, bool),
    /// A connection attempt came back: (ok, ssid, secured-and-saved?).
    Joined(bool, String),
    Note(bool, String),
    /// Background gallery: (original, small thumbnail) — made off the surface.
    Thumbs(Vec<(std::path::PathBuf, std::path::PathBuf)>),
}

/// A press on a list: (in the sidebar?, the role it landed on) — tap or
/// scroll is decided by the kit (`mk::touch::Press`).
type Press = mk::touch::Press<(bool, Option<usize>)>;

struct Settings {
    /// Background: images to choose from, and for which appearance (0 light, 1 dark).
    wp_gallery: Vec<std::path::PathBuf>,
    /// Small thumbnails per original (matrixkit_image::thumbs), once made.
    wp_thumbs: std::collections::HashMap<std::path::PathBuf, std::path::PathBuf>,
    wp_for: usize,
    /// The selected group is folded again (its head tapped once more, or
    /// back to the list on a narrow window).
    folded: bool,
    palette: mk::Palette,
    chrome: fam::Chrome,
    root: fam::RootPane,
    loom: Option<Loom<Work>>,
    area: usize,
    /// The chosen part of a group area (None: the whole area).
    part: Option<usize>,
    /// Narrow window: the sidebar list is shown (not a page).
    menu: bool,
    /// The window is narrow (sidebar and page take turns).
    narrow: bool,
    side_roll: f32,
    side_max: f32,
    page_roll: f32,
    page_max: f32,
    side_rect: Rect,
    page_rect: Rect,
    press: Option<Press>,
    sliders: Vec<(usize, Rect)>,
    dragging: Option<usize>,
    receipt: mk::Flash,
    beats: u32,
    // Allgemein.
    appearance: usize,
    // Anzeige (display) (round 70).
    /// Index into `SCALES`.
    display_scale: usize,
    /// Screen brightness 0..1 and its source (None: not adjustable).
    bright: Option<f32>,
    bright_src: Option<mk::quick::Light>,
    /// (name, model, mode, scale) per screen, from niri.
    screens: Vec<(String, String, String, String)>,
    /// «Bildschirme erkennen»: the numbers stand until then.
    ident_until: Option<std::time::Instant>,
    motion_reduced: bool,
    glass: bool,
    text_scale: usize,
    corner_radius: u8,
    symbol_set: usize,
    icon_style: usize,
    /// Which icon dropdown is open: 0 none, 1 the theme, 2 its variant.
    theme_open: u8,
    // Übersicht (overview) (the iced panel's measuring, shared).
    overview: Option<crate::overview::Panel>,
    // Verbindungen.
    conn: crate::connections::Status,
    conn_busy: bool,
    /// Round 170: a first answer arrived — before it, the page says «a
    /// moment» instead of showing every switch off.
    conn_known: bool,
    /// The device being paired now (its name) — shown while it lasts.
    pairing: Option<String>,
    /// A paired laptop's «Entziehen» tapped once — the second tap withdraws (the kit's two taps).
    laptop_armed: mk::confirm::Armed<usize>,
    pw_for: Option<String>,
    pw: String,
    /// The password card: shown in clear, a join running, the last refusal.
    pw_reveal: bool,
    pw_busy: bool,
    pw_error: Option<String>,
    // Bedienung.
    controller: Vec<String>,
    pad_speed: f32,
    mouse_speed: f32,
    natural: bool,
    tap: bool,
    // Ton: the hearing guard (`mk::hearing`).
    ear: mk::hearing::Guard,
    // Ton: devices, levels, system sounds (the iced panel's data, `sounds`).
    sinks: Vec<crate::sounds::Device>,
    sources: Vec<crate::sounds::Device>,
    out_vol: f32,
    in_vol: f32,
    sound_conf: crate::sounds::Settings,
    /// Microphone test (`mk::audio::MicMeter`).
    mic: mk::audio::MicMeter,
    // Sprache & Region (language & region).
    lang_idx: usize,
    region_idx: usize,
    layout_idx: usize,
}

impl Settings {
    fn new() -> Settings {
        let read = crate::conf_read;
        let language = mk::settings::read("languages").unwrap_or_default();
        let region = mk::settings::read("region").unwrap_or_default();
        let layout = mk::settings::read("keyboard-layout").unwrap_or_default();
        // Round 70: «Allgemein» moved house — its key opens «Leinwand».
        let wish = crate::area_wish().unwrap_or(0);
        let start = match wish {
            1 | 5 => 4,
            a => a,
        };
        let mut s = Settings {
            palette: mk::Palette::load().unwrap_or_default(),
            chrome: fam::Chrome::new("Einstellungen", CHROME_BASE),
            root: fam::RootPane::new(APP_ID, &[]),
            loom: None,
            area: start,
            part: None,
            menu: false,
            narrow: false,
            side_roll: 0.0,
            side_max: 0.0,
            page_roll: 0.0,
            page_max: 0.0,
            side_rect: Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 },
            page_rect: Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 },
            press: None,
            sliders: Vec::new(),
            wp_gallery: Vec::new(),
            wp_thumbs: Default::default(),
            wp_for: if mk::wallpaper::light_mode() { 0 } else { 1 },
            folded: false,
            dragging: None,
            receipt: mk::Flash::new(4.0),
            beats: 0,
            laptop_armed: mk::confirm::Armed::new(),
            display_scale: {
                let want: f32 = read("display-scale").and_then(|v| v.trim().parse().ok()).unwrap_or(2.75);
                SCALES.iter().position(|(v, _)| (v - want).abs() < 0.01).unwrap_or(1)
            },
            bright: None,
            bright_src: None,
            screens: Vec::new(),
            ident_until: None,
            appearance: match read("appearance").as_deref() {
                Some("light") | Some("hell") => 1,
                Some("dark") | Some("dunkel") => 2,
                _ => 0,
            },
            motion_reduced: matches!(read("motion").as_deref(), Some("reduced") | Some("reduziert")),
            glass: read("glass").as_deref() != Some("off"),
            text_scale: mk::settings::read("text-size").and_then(|w| mk::typo::SIZES.iter().position(|s| *s == w)).unwrap_or(1),
            corner_radius: read("corner-radius").and_then(|s| s.parse::<u8>().ok()).unwrap_or(mk::radius::NORMAL as u8).clamp(RADIUS_MIN, RADIUS_MAX),
            symbol_set: usize::from(read("symbol-set").as_deref() == Some("sharp")),
            icon_style: usize::from(matches!(read("icon-style").as_deref(), Some("tinted") | Some("getoent"))),
            theme_open: 0,
            overview: None,
            conn: Default::default(),
            conn_busy: false,
            conn_known: false,
            pairing: None,
            pw_for: None,
            pw: String::new(),
            pw_reveal: false,
            pw_busy: false,
            pw_error: None,
            controller: crate::input_devices::devices_read(),
            pad_speed: mk::settings::read("pad-speed").and_then(|s| s.parse().ok()).unwrap_or(5.0),
            mouse_speed: mk::settings::read("mouse-speed").and_then(|s| s.parse().ok()).unwrap_or(5.0),
            natural: mk::settings::read("natural-scrolling").is_some_and(|s| s == "on" || s == "an"),
            tap: mk::settings::read("typing-clicks").map(|s| s == "on" || s == "an").unwrap_or(true),
            ear: mk::hearing::Guard::load(),
            sinks: Vec::new(),
            sources: Vec::new(),
            out_vol: 0.0,
            in_vol: 0.0,
            sound_conf: crate::sounds::Settings::load(),
            mic: Default::default(),
            lang_idx: crate::language::LANGUAGES.iter().position(|(w, _)| *w == language).unwrap_or(0),
            region_idx: crate::language::REGIONS.iter().position(|(w, _)| *w == region).unwrap_or(0),
            layout_idx: crate::language::LAYOUTS.iter().position(|(w, _)| *w == layout).unwrap_or(0),
        };
        s.enter(start);
        // Round 78: a wished part ("display:5") — enter() folds the group first.
        if let Some(p) = crate::wish_part() {
            s.part = Some(p);
        }
        s
    }

    /// The areas shown (Hardline is hidden on a duo, like before).
    /// The sidebar order (round 70): «Allgemein» is gone — its parts live in
    /// «Anzeige» (sizes) and «Leinwand» (appearance, forms); «Anzeige»
    /// takes its place below «Übersicht».
    /// Round 74: the background page — «Leinwand › Hintergrund» (the old
    /// area 5 lives on only as a key).
    fn wallpaper_page(&self) -> bool {
        self.area == 5 || (self.area == 4 && self.part == Some(2))
    }

    fn areas(&self) -> Vec<usize> {
        let duo = mk::device::panels().is_some();
        let mut order = vec![0, DISPLAY_AREA];
        order.extend((2..AREAS).filter(|a| *a != DISPLAY_AREA && *a != 5));
        order.into_iter().filter(|a| !(duo && *a == HARDLINE_AREA)).collect()
    }

    /// Go to an area: fresh scroll, its data.
    fn enter(&mut self, area: usize) {
        // «Allgemein» moved house (round 70): its old key leads to «Leinwand».
        let wallpaper = area == 5;
        let area = if area == 1 || area == 5 { 4 } else { area };
        self.area = area.min(AREAS - 1);
        if self.area == DISPLAY_AREA {
            self.display_read();
        }
        // Another area folds the group (its parts start over) — the old
        // background key opens «Leinwand › Hintergrund» (round 74).
        self.part = if wallpaper { Some(2) } else { None };
        self.page_roll = 0.0;
        self.pw_for = None;
        if self.area == 0 && self.overview.is_none() {
            self.overview = Some(crate::overview::Panel::new());
        }
        if self.area == 3 {
            self.gather();
        }
        if self.area == 8 {
            self.controller = crate::input_devices::devices_read();
        }
        // Leaving Ton ends a running microphone test.
        self.mic(false);
        if self.area == 7 {
            self.sound_read();
        }
    }

    /// Ton's devices and levels, fresh.
    fn sound_read(&mut self) {
        let (sinks, sources) = crate::sounds::devices_read();
        self.sinks = sinks;
        self.sources = sources;
        self.out_vol = crate::sounds::volume_amt_read("@DEFAULT_AUDIO_SINK@");
        self.in_vol = crate::sounds::volume_amt_read("@DEFAULT_AUDIO_SOURCE@");
    }

    /// The microphone test on or off (the Kit's meter).
    fn mic(&mut self, on: bool) {
        if on {
            self.mic.start();
        } else {
            self.mic.stop();
        }
    }

    /// Play a file (system sound, test tone).
    fn play(path: &std::path::Path) {
        mk::audio::play(path);
    }

    /// Connections, gathered off the surface.
    fn gather(&mut self) {
        if let Some(loom) = &self.loom {
            self.conn_busy = true;
            // Round 170: what is known at once, then the fresh scan.
            loom.spin(|| Work::Conn(crate::connections::status_quick(), false));
            loom.spin(|| Work::Conn(crate::connections::status_gather(), true));
        }
    }

    /// Run a blocking command off the surface, then gather again.
    fn then_gather(&mut self, job: impl Fn() + Send + Sync + 'static) {
        if let Some(loom) = &self.loom {
            self.conn_busy = true;
            loom.spin(move || {
                job();
                Work::Conn(crate::connections::status_gather(), true)
            });
        }
    }

    fn note(&mut self, text: impl Into<String>) {
        self.receipt.set(text);
    }

    /// Round 70: what «Anzeige» shows — brightness and the screens (niri).
    fn display_read(&mut self) {
        match mk::quick::brightness() {
            Some((pct, src)) => {
                self.bright = Some((pct / 100.0).clamp(0.0, 1.0));
                self.bright_src = Some(src);
            }
            None => {
                self.bright = None;
                self.bright_src = None;
            }
        }
        self.screens.clear();
        let Some(out) = mk::canvas::raw(&["msg", "-j", "outputs"]) else { return };
        let Ok(all) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else { return };
        let Some(map) = all.as_object() else { return };
        let mut names: Vec<&String> = map.keys().collect();
        names.sort();
        for name in names {
            let o = &map[name];
            let Some(logical) = o.get("logical").filter(|l| !l.is_null()) else { continue };
            let model = [o.get("make"), o.get("model")]
                .iter()
                .filter_map(|v| v.and_then(|v| v.as_str()))
                .filter(|v| !v.is_empty() && *v != "Unknown")
                .collect::<Vec<_>>()
                .join(" ");
            let mode = o
                .get("current_mode")
                .and_then(|i| i.as_u64())
                .and_then(|i| o.get("modes")?.get(i as usize))
                .map(|m| {
                    let hz = m.get("refresh_rate").and_then(|r| r.as_f64()).unwrap_or(0.0) / 1000.0;
                    format!("{}×{}, {hz:.0} Hz", m.get("width").and_then(|v| v.as_u64()).unwrap_or(0), m.get("height").and_then(|v| v.as_u64()).unwrap_or(0))
                })
                .unwrap_or_default();
            let scale = logical.get("scale").and_then(|v| v.as_f64()).map(|v| format!("{:.2}×", v).replace('.', ",")).unwrap_or_default();
            self.screens.push((name.clone(), if model.is_empty() { String::from("Bildschirm") } else { model }, mode, scale));
        }
    }

    fn set_appearance(&mut self, i: usize) {
        self.appearance = i.min(2);
        let stop = |names: &[&str]| {
            let mut args = vec!["--user", "stop"];
            args.extend_from_slice(names);
            crate::silent("systemctl", &args);
        };
        match self.appearance {
            1 | 2 => {
                let light = self.appearance == 1;
                crate::conf_write("appearance", if light { "light" } else { "dark" });
                // A fixed mode pauses the sun's rhythm.
                stop(&["theme-sun.timer", "theme-sun-light.timer", "theme-sun-dark.timer"]);
                stop(&["theme-sonne.timer", "theme-sonne-light.timer", "theme-sonne-dark.timer"]);
                mk::light_set(light);
                self.note(if light { "Erscheinung: Hell ✓ — Sonnen-Rhythmus pausiert" } else { "Erscheinung: Dunkel ✓ — Sonnen-Rhythmus pausiert" });
            }
            _ => {
                crate::conf_write("appearance", "automatisch");
                for unit in ["theme-sun.timer", "theme-sun.service", "theme-sonne.timer", "theme-sonne.service"] {
                    crate::silent("systemctl", &["--user", "start", unit]);
                }
                self.note("Erscheinung: Automatisch ✓ — folgt der Sonne");
            }
        }
    }

    /// Open the previous version on this area (stage 1 carries not all).
    fn open_old(&mut self) {
        let path = mk::runtime::path("matrix-settings-area");
        let _ = std::fs::write(path, AREA_KEY[self.area]);
        if let Ok(me) = matrixkit_theme::process::own_exe_result() {
            let mut cmd = std::process::Command::new(me);
            cmd.arg("--iced");
            mk::process::spawn_reaped(&mut cmd);
        }
        self.note("Die bisherige Fassung öffnet sich …");
    }

    /// A tap (released without moving) on `role`.
    fn act(&mut self, role: usize) -> Refresh {
        mk::feel::tap();
        match role {
            r if (SIDE_BASE..SIDE_BASE + AREAS).contains(&r) => {
                // Round 70: the role is the POSITION in the sidebar.
                let Some(a) = self.areas().get(r - SIDE_BASE).copied() else { return Refresh::Calm };
                // The open group's head again: fold it (or unfold it).
                if a == self.area && !parts(a).is_empty() {
                    self.folded = !self.folded;
                    if self.folded {
                        self.part = None;
                    }
                    return Refresh::Dirty;
                }
                self.folded = false;
                self.enter(a);
                // On a narrow window a group only unfolds in the list —
                // its parts open the page.
                self.menu = self.narrow && !parts(a).is_empty();
            }
            r if (SUB_BASE..SUB_BASE + 10).contains(&r) => {
                self.part = Some(r - SUB_BASE);
                self.page_roll = 0.0;
                self.menu = false;
            }
            BACK => {
                // Out to the list: the group folds (one tap opens it again).
                self.menu = true;
                if !parts(self.area).is_empty() {
                    self.folded = true;
                    self.part = None;
                }
            }
            OPEN_OLD => self.open_old(),
            r if (APPEAR_BASE..APPEAR_BASE + 3).contains(&r) => self.set_appearance(r - APPEAR_BASE),
            r if (SCALE_BASE..SCALE_BASE + SCALES.len()).contains(&r) => {
                let i = r - SCALE_BASE;
                if i != self.display_scale {
                    self.display_scale = i;
                    // duo-haltung watches this file and lays the screens anew.
                    mk::settings::write("display-scale", &format!("{}", SCALES[i].0));
                    self.note(format!("Anzeigegrösse «{}» ✓", SCALES[i].1));
                }
            }
            BAR_ISLAND => {
                let on = mk::island::enabled();
                mk::island::set_enabled(!on);
                self.note(if on { "Mitteilungen wieder als Fenster" } else { "Mitteilungen in der Leiste ✓" });
            }
            LOCK_GLASS => {
                let on = lock_glass();
                mk::settings::write("lock-clock-glass", if on { "0" } else { "0.85" });
                self.note(if on { "Sperr-Uhr deckend" } else { "Sperr-Uhr aus Glas ✓" });
            }
            LOCK_POWER => {
                let on = lock_on_power();
                mk::settings::write("lock-on-power", if on { "off" } else { "on" });
                self.note(if on { "Power-Taste schaltet nur aus" } else { "Power-Taste sperrt ✓" });
            }
            LOCK_TALL => {
                let on = lock_tall();
                mk::settings::write("lock-clock-stretch", if on { "1.9" } else { "2.6" });
                self.note(if on { "Sperr-Uhr normal hoch" } else { "Sperr-Uhr hoch ✓" });
            }
            r if (WHEEL_BASE..WHEEL_BASE + mk::input::WHEEL_SPEEDS.len()).contains(&r) => {
                mk::input::set_wheel_choice(r - WHEEL_BASE);
                self.note(format!("Mausrad «{}» ✓", mk::input::WHEEL_SPEEDS[r - WHEEL_BASE].1));
            }
            WINDOW_MOVE => {
                let on = mk::input::window_move();
                mk::input::set_window_move(!on);
                self.note(if on { "Fenster verschieben aus" } else { "Fenster verschieben an ✓" });
            }
            HAPTICS => {
                let on = mk::feel::enabled();
                mk::feel::set_enabled(!on);
                if !on {
                    mk::feel::confirm();
                }
                self.note(if on { "Vibration aus" } else { "Vibration an ✓" });
            }
            FILM_SOUND => {
                let on = mk::film::sound();
                mk::film::set_sound(!on);
                self.note(if on { "Bildschirmaufnahmen ohne Ton" } else { "Bildschirmaufnahmen mit Ton ✓" });
            }
            DESK_CLOSED => {
                let on = mk::display::desk_closed();
                mk::display::set_desk_closed(!on);
                self.note(if on { "Zuklappen sperrt wie bisher" } else { "Zugeklappt wie ein Laptop ✓" });
            }
            DESK_FOLDED => {
                let on = mk::display::desk_folded();
                mk::display::set_desk_folded(!on);
                self.note(if on { "Zurückgeklappt bleiben Leiste und Dock auf dem Duo" } else { "Zurückgeklappt: Leiste und Dock auf dem Bildschirm ✓" });
            }
            DESK_SOUND => {
                let on = mk::display::desk_sound();
                mk::display::set_desk_sound(!on);
                self.note(if on { "Ton bleibt am Duo" } else { "Ton über den Bildschirm ✓" });
            }
            r if (DESK_RES_BASE..DESK_RES_BASE + mk::display::DESK_RESOLUTIONS.len()).contains(&r) => {
                mk::display::set_desk_choice("desk-resolution", &mk::display::DESK_RESOLUTIONS, r - DESK_RES_BASE);
                self.note(format!("Auflösung {} ✓", mk::display::DESK_RESOLUTIONS[r - DESK_RES_BASE].1));
            }
            r if (DESK_SCALE_BASE..DESK_SCALE_BASE + mk::display::DESK_SCALES.len()).contains(&r) => {
                mk::display::set_desk_choice("desk-scale", &mk::display::DESK_SCALES, r - DESK_SCALE_BASE);
                self.note(format!("Grösse «{}» ✓", mk::display::DESK_SCALES[r - DESK_SCALE_BASE].1));
            }
            BACK_OFF => {
                let on = mk::display::back_panel_off();
                mk::display::set_back_panel_off(!on);
                self.note(if on { "Hinterer Bildschirm wird nur dunkel" } else { "Hinterer Bildschirm geht ganz aus ✓" });
            }
            DEEP_ON => {
                let on = mk::power::deep_sleep();
                mk::power::set_deep_sleep(!on);
                self.note(if on { "Tiefschlaf aus" } else { "Tiefschlaf an ✓" });
            }
            r if (DEEP_DELAY_BASE..DEEP_DELAY_BASE + mk::power::SLEEP_DELAYS.len()).contains(&r) => {
                let (secs, name) = mk::power::SLEEP_DELAYS[r - DEEP_DELAY_BASE];
                mk::power::set_deep_sleep_delay(secs);
                self.note(format!("Einschlafen nach {name} ✓"));
            }
            AUTO_BRIGHT => {
                let on = mk::display::auto_brightness();
                mk::display::set_auto_brightness(!on);
                self.note(if on { "Automatische Helligkeit aus" } else { "Automatische Helligkeit an ✓" });
            }
            r if (NIGHT_BASE..NIGHT_BASE + 4).contains(&r) => {
                let mode = [mk::night::Mode::Off, mk::night::Mode::Auto, mk::night::Mode::Schedule, mk::night::Mode::On][r - NIGHT_BASE];
                mk::night::set_mode(mode);
                self.note(match mode {
                    mk::night::Mode::Off => "Night Shift aus",
                    mk::night::Mode::Auto => "Night Shift abends ✓",
                    mk::night::Mode::Schedule => "Night Shift nach Zeitplan ✓",
                    mk::night::Mode::On => "Night Shift an ✓",
                });
            }
            r if (NIGHT_FROM..NIGHT_TO + 2).contains(&r) => {
                // Half-hour steps; the edges wrap around midnight.
                let (mut from, mut to) = mk::night::schedule();
                let step = |m: u32, up: bool| if up { (m + 30) % 1440 } else { (m + 1440 - 30) % 1440 };
                match r - NIGHT_FROM {
                    0 => from = step(from, false),
                    1 => from = step(from, true),
                    2 => to = step(to, false),
                    _ => to = step(to, true),
                }
                if from != to {
                    mk::night::set_schedule(from, to);
                }
            }
            ROT_AUTO => {
                let on = mk::display::auto_rotation();
                mk::display::set_auto_rotation(!on);
                self.note(if on { "Drehsperre an — die Ausrichtung bleibt" } else { "Automatisch drehen ✓" });
            }
            r if (ROT_BASE..ROT_BASE + mk::display::ORIENTATIONS.len()).contains(&r) => {
                let (key, name) = mk::display::ORIENTATIONS[r - ROT_BASE];
                mk::display::set_orientation(key);
                self.note(format!("Ausrichtung «{name}» festgehalten ✓"));
            }
            IDENTIFY => {
                if self.screens.is_empty() {
                    self.display_read();
                }
                self.ident_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(3));
                return Refresh::Panes;
            }
            MOTION => {
                self.motion_reduced = !self.motion_reduced;
                crate::conf_write("motion", if self.motion_reduced { "reduced" } else { "full" });
                self.note("Bewegung gespeichert ✓");
            }
            r if (DOCK_STYLE_BASE..DOCK_STYLE_BASE + 2).contains(&r) => {
                let icons = r == DOCK_STYLE_BASE + 1;
                mk::settings::write("dock-style", if icons { "icons" } else { "tiles" });
                self.note(if icons { "Dock zeigt nur noch Icons ✓" } else { "Dock zeigt Kacheln mit Namen ✓" });
            }
            DOCK_HIDE => {
                let on = mk::settings::read("dock-autohide").as_deref() == Some("on");
                mk::settings::write("dock-autohide", if on { "off" } else { "on" });
                self.note(if on { "Dock bleibt stehen ✓" } else { "Dock blendet sich aus ✓" });
            }
            GLASS => {
                self.glass = !self.glass;
                crate::conf_write("glass", if self.glass { "on" } else { "off" });
                crate::write_glass_rule(self.glass);
                self.note("Liquid Glass gespeichert ✓");
            }
            r if r == TEXT_SIZE || r == TEXT_SIZE + 1 => {
                self.text_scale = if r == TEXT_SIZE { self.text_scale.saturating_sub(1) } else { (self.text_scale + 1).min(mk::typo::SIZES.len() - 1) };
                mk::settings::write("text-size", mk::typo::SIZES[self.text_scale]);
                self.note("Textgrösse gespeichert ✓ — gilt für neu gestartete Apps");
            }
            r if r == RADIUS || r == RADIUS + 1 => {
                self.corner_radius = if r == RADIUS { self.corner_radius.saturating_sub(2).max(RADIUS_MIN) } else { (self.corner_radius + 2).min(RADIUS_MAX) };
                crate::conf_write("corner-radius", &self.corner_radius.to_string());
                self.note("Eckenradius gespeichert ✓");
            }
            r if (SYMBOL_BASE..SYMBOL_BASE + 2).contains(&r) => {
                self.symbol_set = r - SYMBOL_BASE;
                crate::conf_write("symbol-set", if self.symbol_set == 1 { "sharp" } else { "rounded" });
                self.note("Symbolstil gespeichert ✓ — gilt für neu gestartete Apps");
            }
            THEME_FORM => {
                let on = matrixkit_icons::matrix_form();
                matrixkit_icons::set_matrix_form(!on);
                self.note(if on { "Symbole in ihrer eigenen Form" } else { "Symbole in Matrix-Form ✓" });
            }
            THEME_OPEN => self.theme_open = if self.theme_open == 1 { 0 } else { 1 },
            VARIANT_OPEN => self.theme_open = if self.theme_open == 2 { 0 } else { 2 },
            r if (THEME_BASE..THEME_BASE + FAMILIES_SHOWN).contains(&r) => {
                self.theme_open = 0;
                if let Some(f) = matrixkit_icons::theme_families().get(r - THEME_BASE).cloned() {
                    matrixkit_icons::set_theme(&f.name);
                    self.note(format!("Symbolthema «{}» ✓", f.name));
                }
            }
            r if (VARIANT_BASE..VARIANT_BASE + VARIANTS_SHOWN).contains(&r) => {
                self.theme_open = 0;
                let current = matrixkit_icons::current_theme().unwrap_or_else(|| String::from(matrixkit_icons::MATRIX_THEME));
                let family = matrixkit_icons::family_of(&current);
                if let Some(f) = matrixkit_icons::theme_families().into_iter().find(|f| f.name == family) {
                    if let Some(v) = f.variants.get(r - VARIANT_BASE) {
                        matrixkit_icons::set_theme(v);
                        self.note(format!("Symbolthema «{}» · {} ✓", f.name, mk::language::tr(&matrixkit_icons::variant_label(&f.name, v))));
                    }
                }
            }
            r if (ICON_BASE..ICON_BASE + 2).contains(&r) => {
                self.icon_style = r - ICON_BASE;
                crate::conf_write("icon-style", if self.icon_style == 1 { "tinted" } else { "standard" });
                crate::icons_rerender();
                self.note("Icon-Stil gespeichert ✓ — Icons werden neu gezeichnet");
            }
            WLAN => {
                let on = !self.conn.wlan_on;
                self.then_gather(move || {
                    let _ = std::process::Command::new("nmcli").args(["radio", "wifi", if on { "on" } else { "off" }]).status();
                    std::thread::sleep(std::time::Duration::from_millis(if on { 2500 } else { 300 }));
                });
            }
            RELOAD => self.gather(),
            r if (NET_BASE..NET_BASE + 100).contains(&r) => {
                if let Some(net) = self.conn.networks.get(r - NET_BASE).cloned() {
                    if net.active {
                        return Refresh::Calm;
                    }
                    // Protected and not known: the password card at once — a
                    // join without one only fails after half a minute.
                    if net.secured && !net.saved {
                        self.pw_for = Some(net.ssid.clone());
                        self.pw.clear();
                        self.pw_error = None;
                        self.pw_reveal = false;
                        return Refresh::Dirty;
                    }
                    if let Some(loom) = &self.loom {
                        self.conn_busy = true;
                        let ssid = net.ssid.clone();
                        loom.spin(move || {
                            // At most 15 s: a saved profile without a valid
                            // password waited up to a minute (03.11.2026).
                            let ok = std::process::Command::new("nmcli").args(["-w", "15", "dev", "wifi", "connect", &ssid]).output().map(|o| o.status.success()).unwrap_or(false);
                            Work::Joined(ok, ssid.clone())
                        });
                    }
                }
            }
            PW_OK => self.join_with_password(),
            PW_CANCEL => {
                self.pw_for = None;
                self.pw.clear();
                self.pw_error = None;
                self.pw_busy = false;
            }
            PW_EYE => self.pw_reveal = !self.pw_reveal,
            PW_VEIL => {}
            BT => {
                let on = !self.conn.bt_on;
                self.then_gather(move || {
                    mk::bluetooth::power(on);
                });
            }
            SCAN => self.then_gather(|| mk::bluetooth::scan(10)),
            r if (BT_DEV_BASE..BT_DEV_BASE + 100).contains(&r) => {
                if let Some(d) = self.conn.devices.get(r - BT_DEV_BASE).cloned() {
                    self.then_gather(move || {
                        mk::bluetooth::connect(&d.mac, !d.connected);
                    });
                }
            }
            r if (LAPTOP_BASE..LAPTOP_BASE + LAPTOPS_SHOWN).contains(&r) => {
                let i = r - LAPTOP_BASE;
                if !self.laptop_armed.tap(i) {
                    // First tap: ask once more.
                    mk::feel::warn();
                } else {
                    if let Some(l) = crate::laptops::list().get(i).cloned() {
                        match crate::laptops::withdraw(&l) {
                            Ok(()) => {
                                mk::feel::confirm();
                                self.note(format!("«{}» hat keinen Zugriff mehr ✓", l.name));
                            }
                            Err(why) => self.note(format!("«{}»: {why}", l.name)),
                        }
                    }
                }
            }
            r if (NEARBY_BASE..NEARBY_BASE + 100).contains(&r) => {
                if self.pairing.is_some() {
                    return Refresh::Calm;
                }
                if let (Some(d), Some(loom)) = (self.conn.nearby.get(r - NEARBY_BASE).cloned(), &self.loom) {
                    eprintln!("[settings] koppelt {} ({})", d.name, d.mac);
                    self.pairing = Some(d.name.clone());
                    loom.spin(move || match mk::bluetooth::pair_session(&d.mac) {
                        Ok(()) => Work::Note(true, format!("Mit «{}» gekoppelt und verbunden", d.name)),
                        Err(why) => Work::Note(false, format!("«{}»: {why}", d.name)),
                    });
                }
            }
            r if (KIND_BASE..KIND_BASE + 200).contains(&r) => {
                let (i, k) = ((r - KIND_BASE) / 2, (r - KIND_BASE) % 2);
                if let Some(d) = self.conn.devices.get(i).cloned() {
                    let kind = if k == 0 { mk::hearing::Kind::Headphones } else { mk::hearing::Kind::Speaker };
                    mk::hearing::set_device_kind(&d.mac, kind);
                    self.note(format!("«{}» gilt jetzt als {} ✓", d.name, if k == 0 { "Kopfhörer" } else { "Lautsprecher" }));
                }
            }
            r if (OUT_BASE..OUT_BASE + 100).contains(&r) || (IN_BASE..IN_BASE + 100).contains(&r) => {
                let out = r < IN_BASE;
                let list = if out { &self.sinks } else { &self.sources };
                if let Some(d) = list.get(r - if out { OUT_BASE } else { IN_BASE }) {
                    let (id, name) = (d.id, d.name.clone());
                    crate::sounds::wpctl(&["set-default", &id.to_string()]);
                    self.sound_read();
                    self.note(format!("{} ✓", if out { format!("Ausgabe: {name}") } else { format!("Eingabe: {name}") }));
                }
            }
            r if r == TEST_LEFT || r == TEST_LEFT + 1 => {
                if let Some(path) = crate::sounds::test_tone(r - TEST_LEFT) {
                    Self::play(&path);
                    self.note(if r == TEST_LEFT { "Testton links …" } else { "Testton rechts …" });
                }
            }
            MIC_TEST => {
                let on = !self.mic.running();
                self.mic(on);
            }
            SOUNDS_ALL => {
                let on = !self.sound_conf.on("all");
                self.sound_conf.set("all", on);
                self.note(if on { "Systemklänge an ✓" } else { "Systemklänge aus" });
            }
            r if (SOUND_BASE..SOUND_BASE + crate::sounds::EVENTS.len()).contains(&r) => {
                let (key, name, _) = crate::sounds::EVENTS[r - SOUND_BASE];
                let on = !self.sound_conf.on(key);
                self.sound_conf.set(key, on);
                self.note(format!("{name}: {} ✓", if on { "an" } else { "aus" }));
            }
            r if (WP_FOR_BASE..WP_FOR_BASE + 2).contains(&r) => {
                self.wp_for = r - WP_FOR_BASE;
            }
            r if (WP_BASE..WP_BASE + 200).contains(&r) => {
                if let Some(path) = self.wp_gallery.get(r - WP_BASE).cloned() {
                    let light = self.wp_for == 0;
                    // The wallpaper app watches this file, repaints and
                    // derives the palette (matugen) — nothing else to do.
                    mk::wallpaper::write(light, &path.to_string_lossy());
                    eprintln!("[settings] Hintergrund {} = {}", if light { "hell" } else { "dunkel" }, path.display());
                    let name = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                    self.note(format!("Hintergrund ({}) gesetzt: {}", if light { "Hell" } else { "Dunkel" }, mk::elide(&name, 24)));
                }
            }
            r if (PLAY_BASE..PLAY_BASE + crate::sounds::EVENTS.len()).contains(&r) => {
                let (file, name, _) = crate::sounds::EVENTS[r - PLAY_BASE];
                Self::play(&crate::sounds::sound_folder().join(format!("{file}.wav")));
                self.note(format!("▶ {name}"));
            }
            EAR_ON => {
                self.ear.on = !self.ear.on;
                self.ear.save();
                self.note(if self.ear.on { "Gehörschutz an ✓" } else { "Gehörschutz aus" });
            }
            NATURAL => {
                self.natural = !self.natural;
                mk::settings::write("natural-scrolling", if self.natural { "on" } else { "off" });
                self.note("Natürliches Scrollen gespeichert ✓");
            }
            TAP => {
                self.tap = !self.tap;
                mk::settings::write("typing-clicks", if self.tap { "on" } else { "off" });
                self.note("Tippen zum Klicken gespeichert ✓");
            }
            r if (LANG_BASE..LANG_BASE + 10).contains(&r) => {
                self.lang_idx = (r - LANG_BASE).min(crate::language::LANGUAGES.len() - 1);
                mk::settings::write("languages", crate::language::LANGUAGES[self.lang_idx].0);
                self.note("Sprache gespeichert ✓ — gilt für neu gestartete Apps");
            }
            r if (REGION_BASE..REGION_BASE + 10).contains(&r) => {
                self.region_idx = (r - REGION_BASE).min(crate::language::REGIONS.len() - 1);
                mk::settings::write("region", crate::language::REGIONS[self.region_idx].0);
                self.note("Region gespeichert ✓");
            }
            r if (LAYOUT_BASE..LAYOUT_BASE + 10).contains(&r) => {
                self.layout_idx = (r - LAYOUT_BASE).min(crate::language::LAYOUTS.len() - 1);
                let layout = crate::language::LAYOUTS[self.layout_idx].0;
                mk::settings::write("keyboard-layout", layout);
                crate::language::input_kdl_write(layout);
                self.note("Tastatur-Layout gespeichert ✓");
            }
            _ => return Refresh::Calm,
        }
        Refresh::Dirty
    }

    /// THE WI-FI PASSWORD CARD (03.11.2026, «without iced»): a veil over the
    /// window and a card in its upper part (the on-screen keyboard takes the
    /// lower one): the network, the field, an eye, «Verbinden» / «Abbrechen».
    fn lay_password_card(&self, b: &mut SceneBuilder, engine: &mut TextEngine, ssid: &str, wf: f32, hf: f32) {
        let p = self.palette;
        b.quad(0.0, 0.0, wf, hf, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.45 }, 0.0, Some(PW_VEIL), None);
        let w = (wf - 2.0 * mk::spacing::L).clamp(240.0, 440.0);
        let h = 230.0_f32;
        let (x, y) = ((wf - w) / 2.0, (hf * 0.08).max(mk::spacing::M));
        b.quad(x, y, w, h, p.surface_container, 22.0, Some(PW_VEIL), None);
        let pad = mk::spacing::L;
        let about = if self.pw_busy {
            String::from("Verbinde …")
        } else if let Some(e) = &self.pw_error {
            e.clone()
        } else {
            String::from("Passwort für dieses Netz eingeben")
        };
        let mut cy = fam::lay_card_head(b, engine, &format!("WLAN «{ssid}»"), &about, PW_CANCEL, x + pad / 2.0, y + pad / 2.0, w - pad, None, p);
        // The field (kit family, round 44): dots or clear text, a caret, the eye.
        let fh = 48.0;
        fam::lay_secret_field(b, engine, Rect { x: x + pad, y: cy, w: w - 2.0 * pad, h: fh }, &self.pw, self.pw_reveal, self.blink_on(), self.pw_error.is_some(), "Passwort", (Some(PW_VEIL), Some(PW_EYE)), p.on_surface, p.surface_container_high, p);
        cy += fh + mk::spacing::L;
        let label = if self.pw_busy { "Verbinde …" } else { "Verbinden" };
        let cw = fam::pill_width(engine, None, "Abbrechen");
        let vw = fam::pill_width(engine, Some('\u{e63e}'), label);
        fam::lay_pill(b, engine, None, "Abbrechen", x + w - pad - vw - mk::spacing::S - cw, cy, None, Some(PW_CANCEL), fam::PillTone::Quiet, p);
        fam::lay_pill(b, engine, Some('\u{e63e}'), label, x + w - pad - vw, cy, None, Some(PW_OK), fam::PillTone::Accent, p);
    }

    fn join_with_password(&mut self) {
        // The card stays open while it joins (and tells a refusal).
        let (Some(ssid), pw) = (self.pw_for.clone(), self.pw.clone()) else { return };
        if pw.is_empty() || self.pw_busy {
            return;
        }
        if let Some(loom) = &self.loom {
            self.conn_busy = true;
            self.pw_busy = true;
            self.pw_error = None;
            loom.spin(move || {
                // A broken old profile of this network goes first (a refused
                // hotspot left one; joining again made «… 1» duplicates).
                let _ = std::process::Command::new("nmcli").args(["connection", "delete", "id", &ssid]).output();
                let ok = std::process::Command::new("nmcli").args(["-w", "20", "dev", "wifi", "connect", &ssid, "password", &pw]).output().map(|o| o.status.success()).unwrap_or(false);
                Work::Joined(ok, ssid.clone())
            });
        }
    }

    /// A slider moved (live) — `done` writes it.
    fn slide(&mut self, role: usize, v: f32, done: bool) {
        let speed = (1.0 + v * 9.0).round();
        // Percent in steps of five, 5 … 100.
        let pct = ((5.0 + v * 95.0) / 5.0).round() as u32 * 5;
        match role {
            NIGHT_WARM => {
                // The service glides there; write whole percents only.
                if (v - mk::night::warmth()).abs() >= 0.01 || done {
                    mk::night::set_warmth(v);
                }
            }
            BRIGHT => {
                let old = self.bright.unwrap_or(0.0);
                if (v - old).abs() < 0.01 && !done {
                    return;
                }
                // Never fully dark from a slider (5 % floor).
                let v = v.max(0.05);
                self.bright = Some(v);
                if let Some(src) = self.bright_src {
                    mk::quick::brightness_commit(v * 100.0, src);
                }
            }
            OUT_VOL | IN_VOL => {
                // A finger moves many times a second: only a whole percent
                // starts a wpctl (it spawned dozens per second before).
                let old = if role == OUT_VOL { self.out_vol } else { self.in_vol };
                if (v - old).abs() < 0.01 && !done {
                    return;
                }
                if role == OUT_VOL {
                    // On headphones the slider stops at the hearing guard's cap.
                    self.out_vol = v.min(mk::hearing::key_limit());
                    crate::sounds::wpctl(&["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{:.2}", self.out_vol)]);
                } else {
                    self.in_vol = v;
                    crate::sounds::wpctl(&["set-volume", "@DEFAULT_AUDIO_SOURCE@", &format!("{v:.2}")]);
                }
            }
            EAR_MAX | EAR_START | EAR_EVENTS => {
                match role {
                    EAR_MAX => self.ear.max = pct,
                    EAR_START => self.ear.start = pct,
                    _ => self.ear.events = pct,
                }
                if done {
                    self.ear.save();
                    self.note("Gehörschutz gespeichert ✓");
                }
            }
            PAD_SPEED => {
                self.pad_speed = speed;
                if done {
                    mk::settings::write("pad-speed", &format!("{speed:.0}"));
                    self.note("Zeigertempo (Controller) gespeichert ✓");
                }
            }
            MOUSE_SPEED => {
                self.mouse_speed = speed;
                if done {
                    mk::settings::write("mouse-speed", &format!("{speed:.0}"));
                    self.note("Zeigertempo (Maus) gespeichert ✓");
                }
            }
            _ => {}
        }
    }

    // ---------------- pages ----------------

    fn page(&mut self, engine: &mut TextEngine, f: &mut fam::FormPage) {
        let title = AREA_TITLE[self.area];
        match self.area {
            0 => {
                f.heading(engine, "Übersicht", Some("Was in diesem Gerät steckt und wie es ihm gerade geht."));
                if let Some(o) = &self.overview {
                    f.section(engine, "Gerät");
                    f.value(engine, Some('\u{e30a}'), &o.device, Some(&o.base), &o.hostname);
                    f.value(engine, None, "Kernel", None, &o.kernel);
                    f.value(engine, None, "Prozessor", Some(&o.cpu_model), &format!("{} Kerne", o.cpu_cores));
                    f.section(engine, "Jetzt");
                    f.value(engine, Some('\u{e322}'), "Prozessorlast", None, &format!("{:.0} %", o.cpu));
                    if let Some(t) = o.temperature {
                        f.value(engine, Some('\u{e1ff}'), "Temperatur", None, &format!("{t:.0} °C"));
                    }
                    let gb = |b: u64| b as f64 / 1_073_741_824.0;
                    f.value(engine, Some('\u{e322}'), "Arbeitsspeicher", None, &format!("{:.1} von {:.1} GB", gb(o.mem_used), gb(o.mem_total)));
                    f.value(engine, Some('\u{e1db}'), "Speicher", None, &format!("{:.0} von {:.0} GB belegt", gb(o.disk_used), gb(o.disk_total)));
                    if let Some((level, charging)) = o.battery {
                        // Round 75: the time left (smoothed draw, shared with the bar).
                        let left = mk::power::hours_left().map(|(h, c)| mk::power::label(h, c));
                        f.value(engine, Some('\u{e1a4}'), "Akku", left.as_deref(), &match (charging, mk::device::battery_draw()) {
                        (true, _) => format!("{level} % · lädt"),
                        (false, Some((_, w))) => format!("{level} % · {:.1} W", w).replace('.', ","),
                        (false, None) => format!("{level} %"),
                    });
                    }
                    // Round 75: does the whole chip ever rest? (the deep-sleep dig)
                    if let Some((aosd, cxsd)) = mk::power::chip_sleeps() {
                        let rest = if aosd + cxsd == 0 { String::from("noch nie seit dem Start") } else { format!("{}× seit dem Start", aosd + cxsd) };
                        f.value(engine, Some('\u{ef44}'), "Tiefschlaf des Chips", Some("Je öfter, desto länger hält der Akku"), &rest);
                    }
                    f.value(engine, Some('\u{e63e}'), "Netz", Some(&o.ip), &format!("↓ {:.0} kB/s · ↑ {:.0} kB/s", o.rx_rate / 1024.0, o.tx_rate / 1024.0));
                    // Round 98: who holds the memory — RAM and GPU memory (on the
                    // Duo the GPU's is RAM too and shows in no process list).
                    if !o.apps.is_empty() {
                        f.section(engine, "Speicher je App");
                        for a in o.apps.iter().take(6) {
                            let desc = if a.gpu_mb > 0 { format!("{} MB Arbeitsspeicher · {} MB Grafik", a.ram_mb, a.gpu_mb) } else { format!("{} MB Arbeitsspeicher", a.ram_mb) };
                            f.value(engine, None, &mk::memory::app_label(&a.name), Some(&desc), &format!("{} MB", a.total_mb()));
                        }
                    }
                }
            }
            3 => {
                let (wlan, bt, laptops) = match self.part {
                    Some(0) => (true, false, false),
                    Some(1) => (false, true, false),
                    Some(_) => (false, false, true),
                    None => (true, true, true),
                };
                let (head, about) = match self.part {
                    Some(0) => ("WLAN", "Netze über NetworkManager — Passwörter tippst immer du selbst."),
                    Some(1) => ("Bluetooth", "Geräte über BlueZ — Kopfhörer, Lautsprecher, Tastaturen."),
                    Some(_) => ("Laptops", "Laptops, denen du in Morpheus vertraut hast. Sie sehen deine Dateien und Fotos nur über das USB-Kabel."),
                    None => ("Verbindungen", "WLAN über NetworkManager, Bluetooth über BlueZ — Passwörter tippst immer du selbst."),
                };
                f.heading(engine, head, Some(about));
                let c = &self.conn;
                if !self.conn_known {
                    // Round 170: say that it is reading — not every switch «off».
                    f.value(engine, Some('\u{e88b}'), "Einen Moment …", Some("WLAN und Bluetooth werden gelesen"), "");
                }
                if wlan && self.conn_known {
                f.section(engine, "WLAN");
                f.toggle(engine, WLAN, "WLAN", None, c.wlan_on, false);
                if c.wlan_on {
                    for (i, net) in c.networks.iter().enumerate() {
                        let desc = format!("{} %{}{}", net.signal, if net.secured { " · geschützt" } else { " · offen" }, if net.saved { " · gespeichert" } else { "" });
                        if net.active {
                            f.value(engine, Some('\u{e63e}'), &net.ssid, Some(&desc), "Verbunden");
                        } else {
                            f.nav(engine, NET_BASE + i, Some('\u{e63e}'), &net.ssid, Some(&desc), Some("Verbinden"));
                        }
                    }
                    if c.networks.is_empty() && !self.conn_busy {
                        f.value(engine, None, "Keine Netze gefunden", None, "");
                    }
                    if self.conn_busy {
                        // Round 170: the fresh scan runs — known networks stand already.
                        f.value(engine, Some('\u{e8b6}'), "Sucht weitere Netze …", Some("Das dauert ein paar Sekunden"), "");
                    }
                }
                }
                if bt && self.conn_known {
                f.section(engine, "Bluetooth");
                f.toggle(engine, BT, "Bluetooth", None, c.bt_on, false);
                if c.bt_on {
                    for (i, d) in c.devices.iter().enumerate() {
                        // Round 160: each device with its own symbol (mouse, keyboard, headphones …).
                        let state = format!("{} · {}", d.kind().word(), if d.connected { "Verbunden" } else { "Gekoppelt" });
                        f.button(engine, BT_DEV_BASE + i, Some(d.kind().glyph()), &d.name, Some(&state), if d.connected { "Trennen" } else { "Verbinden" }, fam::PillTone::Quiet);
                        // Sound devices: headphones (the hearing guard acts) or a speaker.
                        if d.icon.starts_with("audio") {
                            let chosen = mk::hearing::device_kind(&d.mac);
                            let kind = chosen.unwrap_or_else(|| mk::hearing::suggest(Some(&d.icon), None));
                            let how = if chosen.is_some() { "von dir gewählt" } else { "erkannt — tippe, um es zu ändern" };
                            let desc = format!("{how} · Gehörschutz {}", if kind == mk::hearing::Kind::Headphones { "wirkt" } else { "ruht" });
                            f.choice(engine, KIND_BASE + 2 * i, "Gilt als", Some(&desc), &["Kopfhörer", "Lautsprecher"], usize::from(kind == mk::hearing::Kind::Speaker));
                        }
                    }
                    if c.devices.is_empty() {
                        f.value(engine, None, "Keine gekoppelten Geräte", None, "");
                    }
                    f.section(engine, "Neues Gerät");
                    if let Some(name) = &self.pairing {
                        // While pairing: what happens, and what to do.
                        let dots = ".".repeat(1 + (self.beats as usize % 3));
                        f.value(engine, Some('\u{e1a8}'), &format!("Koppelt mit «{name}»{dots}"), Some("Kopplungsmodus halten — das dauert bis zu 50 Sekunden"), "");
                    }
                    f.button(engine, SCAN, Some('\u{e310}'), "Gerät in den Kopplungsmodus bringen — Kopfhörer, Maus, Tastatur", Some("dann suchen — gefundene Geräte erscheinen hier"), if self.conn_busy { "Sucht …" } else { "Suchen" }, fam::PillTone::Quiet);
                    for (i, d) in c.nearby.iter().enumerate() {
                        let label = if self.pairing.as_deref() == Some(d.name.as_str()) { "Koppelt …" } else { "Koppeln" };
                        f.button(engine, NEARBY_BASE + i, Some(d.kind().glyph()), &d.name, None, label, if self.pairing.is_some() { fam::PillTone::Quiet } else { fam::PillTone::Accent });
                    }
                }
                }
                if laptops {
                    // Morpheus (06.11.2026): who may see this duo's files over the cable — withdrawn with two taps.
                    f.section(engine, "Gekoppelte Laptops");
                    let paired = crate::laptops::list();
                    for (i, l) in paired.iter().enumerate().take(LAPTOPS_SHOWN) {
                        let desc = if l.since.is_empty() { String::from("Morpheus · nur über das USB-Kabel") } else { format!("Gekoppelt am {} · nur über das USB-Kabel", l.since) };
                        let armed = self.laptop_armed.is(&i);
                        f.button(engine, LAPTOP_BASE + i, Some('\u{e31e}'), &l.name, Some(&desc), if armed { "Wirklich entziehen?" } else { "Entziehen" }, if armed { fam::PillTone::Accent } else { fam::PillTone::Quiet });
                    }
                    if paired.is_empty() {
                        f.value(engine, Some('\u{e31e}'), "Kein Laptop gekoppelt", Some("In Morpheus am Laptop «Koppeln» tippen — das Duo fragt dann hier, ob es vertrauen soll"), "");
                    }
                }
                if wlan || bt {
                    f.section(engine, "");
                    f.button(engine, RELOAD, Some('\u{e5d5}'), "Neu suchen", None, if self.conn_busy { "Sucht …" } else { "Neu suchen" }, fam::PillTone::Quiet);
                }
            }
            8 => {
                f.heading(engine, "Bedienung", Some("Controller, Maus und Trackpad."));
                // Round 120: the feel (mk::feel) — on the duo, the motor.
                if mk::device::is_duo() {
                    f.section(engine, "Berührung");
                    f.toggle(engine, HAPTICS, "Vibration", Some("Spürbare Antwort beim Tippen, Halten, Bestätigen und bei Fehlern"), mk::feel::enabled(), false);
                }
                // Round 161: the mouse (the duo at the TV: right click = long press).
                f.section(engine, "Maus");
                let names: Vec<&str> = mk::input::WHEEL_SPEEDS.iter().map(|(_, n)| *n).collect();
                f.toggle(engine, WINDOW_MOVE, "Fenster verschieben", Some("Lange auf eine freie Stelle drücken (Finger oder Maus): das Fenster wackelt und lässt sich ziehen, auch auf einen anderen Bildschirm"), mk::input::window_move(), false);
                f.choice(engine, WHEEL_BASE, "Mausrad", Some("Wie weit ein Dreh am Rad scrollt — ein Rechtsklick wirkt wie langes Antippen"), &names, mk::input::wheel_choice());
                f.section(engine, "Controller");
                if self.controller.is_empty() {
                    f.value(engine, Some('\u{e30f}'), "Kein Controller verbunden", Some("Xbox- und PlayStation-Controller steuern den Zeiger"), "");
                } else {
                    for c in self.controller.clone() {
                        f.value(engine, Some('\u{e30f}'), &c, None, "verbunden");
                    }
                }
                f.slider(engine, PAD_SPEED, "Zeigertempo", Some(&format!("{:.0} von 10", self.pad_speed)), (self.pad_speed - 1.0) / 9.0);
                f.section(engine, "Maus und Trackpad");
                f.slider(engine, MOUSE_SPEED, "Zeigertempo", Some(&format!("{:.0} von 10", self.mouse_speed)), (self.mouse_speed - 1.0) / 9.0);
                f.toggle(engine, NATURAL, "Natürliches Scrollen", Some("Inhalt folgt den Fingern"), self.natural, false);
                f.toggle(engine, TAP, "Tippen zum Klicken", None, self.tap, false);
                f.section(engine, "Tastatur");
                f.value(engine, Some('\u{e312}'), "Wiederholung", None, &format!("{} ms · {}/s", mk::input::KEY_REPEAT_DELAY_MS, mk::input::KEY_REPEAT_RATE));
            }
            10 => {
                f.heading(engine, "Sprache & Region", Some("Sprache und Formate gelten für neu gestartete Apps — das Tastatur-Layout ab der nächsten Anmeldung."));
                // The names straight from the static tables.
                let names = |list: &[(&'static str, &'static str)]| list.iter().map(|(_, n)| *n).collect::<Vec<&'static str>>();
                let (langs, regions, layouts) = (names(&crate::language::LANGUAGES), names(&crate::language::REGIONS), names(&crate::language::LAYOUTS));
                f.section(engine, "Sprache");
                f.choice(engine, LANG_BASE, "Sprache", None, &langs, self.lang_idx);
                f.section(engine, "Region");
                f.choice(engine, REGION_BASE, "Formate", Some("Zahlen, Datum und Uhrzeit"), &regions, self.region_idx);
                f.section(engine, "Tastatur");
                f.choice(engine, LAYOUT_BASE, "Layout", None, &layouts, self.layout_idx);
            }
            7 => {
                let part = self.part;
                let show = |k: usize| part.is_none() || part == Some(k);
                let (head, about) = match part {
                    Some(0) => ("Ausgabe", "Wohin der Ton geht und wie laut."),
                    Some(1) => ("Eingabe", "Welches Mikrofon zuhört."),
                    Some(2) => ("Klänge", "Die Systemklänge von Matrix — einzeln an und aus, mit Hörprobe."),
                    Some(_) => ("Gehörschutz", "Schutz deiner Ohren auf Kopfhörern."),
                    None => ("Ton", "Was du hörst und wie laut."),
                };
                f.heading(engine, head, Some(about));
                if show(0) {
                    f.section(engine, "Ausgabegerät");
                    for (k, d) in self.sinks.iter().enumerate() {
                        // Round 157: the device's own symbol (a TV shows a TV).
                        f.pick(engine, OUT_BASE + k, Some(d.kind().glyph()), &d.name, None, d.standard);
                    }
                    if self.sinks.is_empty() {
                        f.value(engine, None, "Keine Ausgabe gefunden", None, "");
                    }
                    let cap = mk::hearing::key_limit();
                    let desc = if cap < 1.0 { format!("{:.0} % — Gehörschutz: höchstens {:.0} %", self.out_vol * 100.0, cap * 100.0) } else { format!("{:.0} %", self.out_vol * 100.0) };
                    f.slider(engine, OUT_VOL, "Lautstärke", Some(&desc), self.out_vol);
                    f.section(engine, "Lautsprecher prüfen");
                    f.button(engine, TEST_LEFT, Some('\u{e050}'), "Links", Some("Ein kurzer Ton nur links"), "Testton", fam::PillTone::Quiet);
                    f.button(engine, TEST_LEFT + 1, Some('\u{e050}'), "Rechts", Some("Ein kurzer Ton nur rechts"), "Testton", fam::PillTone::Quiet);
                    f.section(engine, "Bildschirmaufnahme");
                    f.toggle(engine, FILM_SOUND, "Ton mitschneiden", Some("Was das Gerät abspielt, kommt mit in den Film"), mk::film::sound(), false);
                }
                if show(1) {
                    f.section(engine, "Eingabegerät");
                    for (k, d) in self.sources.iter().enumerate() {
                        // Round 169: the input's own symbol (microphone, headset, Bluetooth).
                        f.pick(engine, IN_BASE + k, Some(d.input_kind().glyph()), &d.name, None, d.standard);
                    }
                    if self.sources.is_empty() {
                        f.value(engine, None, "Kein Mikrofon gefunden", Some("Kopfhörer mit Mikrofon erscheinen hier, sobald sie verbunden sind"), "");
                    } else {
                        f.slider(engine, IN_VOL, "Eingangspegel", Some(&format!("{:.0} %", self.in_vol * 100.0)), self.in_vol);
                        let on = self.mic.running();
                        f.toggle(engine, MIC_TEST, "Mikrofon testen", Some("Sprich — der Balken zeigt, was ankommt"), on, false);
                        if on {
                            // Round 169: in decibels — the duo's quiet microphone barely moved a linear bar.
                            f.meter(engine, Some('\u{e029}'), "Pegel", None, self.mic.loudness());
                        }
                    }
                }
                if show(2) {
                    f.section(engine, "Systemklänge");
                    let all = self.sound_conf.on("all");
                    f.toggle(engine, SOUNDS_ALL, "Systemklänge", Some("Alle Klänge von Matrix auf einmal"), all, false);
                    if all {
                        f.section(engine, "Einzeln");
                        for (i, (key, name, desc)) in crate::sounds::EVENTS.iter().enumerate() {
                            f.toggle_with_key(engine, SOUND_BASE + i, ('\u{e037}', PLAY_BASE + i), name, Some(desc), self.sound_conf.on(key), false);
                        }
                        f.note(engine, "Schlüssel erkannt und Wächter-Ruf klingen am Anmeldebildschirm immer.");
                    }
                }
                if show(3) {
                f.section(engine, "Gehörschutz");
                f.toggle(engine, EAR_ON, "Gehörschutz", Some("Auf Kopfhörern: Lautstärke gedeckelt, neue Töne blenden sanft ein, Benachrichtigungen leiser"), self.ear.on, false);
                let at = |p: u32| (p as f32 - 5.0) / 95.0;
                let locked = !self.ear.on;
                if !locked {
                    f.slider(engine, EAR_MAX, "Höchste Lautstärke", Some(&format!("{} % — lauter geht es auf Kopfhörern nicht", self.ear.max)), at(self.ear.max));
                    f.slider(engine, EAR_START, "Startlautstärke", Some(&format!("{} % — wenn Kopfhörer sich verbinden", self.ear.start)), at(self.ear.start));
                    f.slider(engine, EAR_EVENTS, "Benachrichtigungen", Some(&format!("{} % ihrer Lautstärke — System- und Hinweistöne", self.ear.events)), at(self.ear.events));
                }
                f.note(engine, "Wirkt auf Bluetooth- und Kabelkopfhörer, nicht auf den Lautsprecher. Jeder neue Ton beginnt leise und wird in gut einer Sekunde lauter.");
                // The proof: what the guard sees and last did (live).
                f.section(engine, "Zustand");
                match mk::hearing::Status::read() {
                    None => f.value(engine, Some('\u{e002}'), "Gehörschutz-Dienst läuft nicht", Some("matrix-earguard ist nicht gestartet"), ""),
                    Some(st) => {
                        match &st.phones {
                            Some(name) => f.value(engine, Some('\u{e310}'), name, Some("Kopfhörer — der Schutz wirkt"), &format!("{} % · Grenze {} %", st.volume, self.ear.max)),
                            None => f.value(engine, Some('\u{e050}'), st.output.as_deref().unwrap_or("Lautsprecher"), Some("Lautsprecher — der Schutz ruht"), &format!("{} %", st.volume)),
                        }
                        let ago = mk::format::ago;
                        match &st.last {
                            Some((what, at)) => f.value(engine, Some('\u{e86c}'), what, Some(&format!("{} · {} Eingriffe seit dem Start", ago(*at), st.count)), ""),
                            None => f.value(engine, Some('\u{e86c}'), "Noch nicht eingegriffen", None, ""),
                        }
                    }
                }
                }
            }
            6 => {
                f.heading(engine, "Leiste & Dock", Some("Was am Rand der Leinwand wohnt: oben die Leiste, unten das Dock."));
                f.section(engine, "Leiste");
                f.toggle(engine, BAR_ISLAND, "Mitteilungen in der Leiste", Some("Die Glaselemente fliessen zu einer Insel zusammen: App-Symbol und Text laufen durch, nach oben wischen schliesst"), mk::island::enabled(), false);
                f.section(engine, "Dock");
                let icons = mk::settings::read("dock-style").as_deref() == Some("icons");
                f.choice(engine, DOCK_STYLE_BASE, "Darstellung", Some("Kacheln zeigen Name und Hersteller, «Nur Icons» das Matrix-Icon der App"), &["Kacheln", "Nur Icons"], icons as usize);
                let hides = mk::settings::read("dock-autohide").as_deref() == Some("on");
                f.toggle(engine, DOCK_HIDE, "Automatisch ausblenden", Some("Das Dock faltet sich nach drei ruhigen Sekunden weg"), hides, false);
                f.section(engine, "Leiste und Dock anpassen");
                f.nav(engine, OPEN_OLD, Some(AREA_GLYPH[6]), "Widgets und angeheftete Apps", Some("Zieht in der nächsten Stufe um — bis dahin in der bisherigen Fassung"), None);
            }
            _ if self.wallpaper_page() => {
                // Wait for the loom (the first frame may come before it).
                if self.wp_gallery.is_empty() && self.loom.is_some() {
                    self.wp_gallery = crate::canvas::gallery_collect().into_iter().map(|(p, _)| p).collect();
                    // Thumbnails off the surface: the tiles never decode originals.
                    if let Some(loom) = &self.loom {
                        let all = self.wp_gallery.clone();
                        loom.spin(move || {
                            Work::Thumbs(all.into_iter().filter_map(|o| matrixkit_image::thumbs::ensure(&o, THUMB_W, THUMB_H).map(|t| (o, t))).collect())
                        });
                    }
                }
                f.heading(engine, "Hintergrund", Some("Das Bild der Leinwand — je eines für Hell und Dunkel. Die Farben von Matrix folgen dem Bild."));
                let now = mk::wallpaper::read(self.wp_for == 0).or_else(mk::wallpaper::current).unwrap_or_default();
                let name = std::path::Path::new(&now).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| String::from("—"));
                f.section(engine, "Aktuell");
                f.choice(engine, WP_FOR_BASE, "Für", Some("Welches Erscheinungsbild du gerade wählst"), &["Hell", "Dunkel"], self.wp_for);
                f.value(engine, Some('\u{e3f4}'), "Bild", None, &mk::elide(&name, 28));
                f.section(engine, if self.wp_gallery.is_empty() { "Keine Bilder gefunden — lege welche in Bilder/Hintergrundbilder" } else { "Bilder" });
            }
            4 => {
                // Round 70: a group — appearance and forms came from «Allgemein».
                let part = self.part;
                let show = |k: usize| part.is_none() || part == Some(k);
                let (head, about) = match part {
                    Some(0) => ("Erscheinungsbild", "Hell, Dunkel, Glas und Bewegung."),
                    Some(1) => ("Formen & Symbole", "Ecken, Symbole und App-Icons."),
                    Some(_) => ("Sitzung", "Wie die Leinwand startet."),
                    None => ("Leinwand", "Wie Matrix aussieht — und die unendliche Fläche, auf der deine Fenster leben."),
                };
                f.heading(engine, head, Some(about));
                if show(0) {
                    f.section(engine, "Erscheinungsbild");
                    f.choice(engine, APPEAR_BASE, "Erscheinung", Some("Automatisch folgt dem Sonnenstand"), &["Automatisch", "Hell", "Dunkel"], self.appearance);
                    f.toggle(engine, GLASS, "Liquid Glass", Some("Leiste, Dock und Karten aus Glas"), self.glass, false);
                    f.toggle(engine, MOTION, "Bewegung reduzieren", Some("Weniger Animation, Übergänge springen"), self.motion_reduced, false);
                    // Round 135: the lock screen's clock.
                    f.section(engine, "Sperrbildschirm");
                    f.toggle(engine, LOCK_GLASS, "Uhr aus Glas", Some("Die grosse Uhr bricht das Hintergrundbild wie die Leiste — sonst deckend"), lock_glass(), false);
                    f.toggle(engine, LOCK_TALL, "Hohe Uhr", Some("Schmale, hohe Ziffern"), lock_tall(), false);
                    if mk::device::is_duo() {
                        f.toggle(engine, LOCK_POWER, "Mit der Power-Taste sperren", Some("Ausschalten sperrt — beim Einschalten wartet die Sperre wie am Handy"), lock_on_power(), false);
                    }
                }
                if show(1) {
                    f.section(engine, "Formen & Symbole");
                    f.stepper(engine, RADIUS, "Eckenradius", None, &format!("{} px", self.corner_radius), self.corner_radius > RADIUS_MIN, self.corner_radius < RADIUS_MAX);
                    f.choice(engine, SYMBOL_BASE, "Symbolstil", None, &["Rund", "Eckig"], self.symbol_set);
                    // Round 88: Matrix's own icons or an installed theme (WhiteSur …).
                    // 06.11.2026 (owner: «two dropdown groups»): the theme
                    // family first (Matrix, Adwaita, WhiteSur …), then its
                    // variant (Legacy, Hoher Kontrast · Dunkel, Hell — high contrast · dark, light).
                    let families = matrixkit_icons::theme_families();
                    let current = matrixkit_icons::current_theme().unwrap_or_else(|| String::from(matrixkit_icons::MATRIX_THEME));
                    let family_name = matrixkit_icons::family_of(&current);
                    let at = families.iter().position(|fam| fam.name == family_name).unwrap_or(0);
                    let names: Vec<&str> = families.iter().map(|fam| fam.name.as_str()).take(FAMILIES_SHOWN).collect();
                    let own = at > 0 && !matrixkit_icons::brings_app_icons(&family_name);
                    let about = if families.len() <= 1 {
                        "Weitere Themen (z. B. WhiteSur) nach ~/.local/share/icons legen"
                    } else if own {
                        "Die Apps zeigen ihre eigenen Symbole — wie bei GNOME"
                    } else {
                        "Apps ohne Symbol im Thema behalten ihr Matrix-Icon"
                    };
                    f.dropdown(engine, THEME_OPEN, THEME_BASE, "Symbolthema", Some(about), &names, at, self.theme_open == 1);
                    if at == 0 {
                        f.choice(engine, ICON_BASE, "App-Icons", None, &["Standard", "Getönt"], self.icon_style);
                    } else {
                        let fam = &families[at];
                        if fam.variants.len() > 1 {
                            let labels: Vec<String> = fam.variants.iter().take(VARIANTS_SHOWN).map(|v| matrixkit_icons::variant_label(&fam.name, v)).collect();
                            let labels: Vec<&str> = labels.iter().map(String::as_str).collect();
                            let vat = fam.variants.iter().position(|v| *v == current).unwrap_or(0);
                            f.dropdown(engine, VARIANT_OPEN, VARIANT_BASE, "Variante", None, &labels, vat, self.theme_open == 2);
                        }
                        f.toggle(engine, THEME_FORM, "In Matrix-Form", Some("Die Symbole des Themas bekommen die Kachel der Matrix-Icons"), matrixkit_icons::matrix_form(), false);
                    }
                }
                if show(3) {
                    f.section(engine, "Sitzung");
                    let canvas = matrixkit_widgets::session_is_canvas();
                    f.value(engine, Some('\u{e8f1}'), "Modus", Some("Wechsel beim Anmelden («Matrix Leinwand» oder «Niri»)"), if canvas { "Unendliche Leinwand" } else { "Klassisch" });
                    f.nav(engine, OPEN_OLD, Some('\u{e8b8}'), "Geist, Navigation, Touch", Some("Diese Feineinstellungen ziehen in der nächsten Stufe um — bis dahin in der bisherigen Fassung"), None);
                }
                let _ = APPEARANCES;
            }
            DISPLAY_AREA => {
                // Round 70 (owner: «sizes, screen brightness, screen identification»).
                // Opened at start (not through enter): read once here.
                if self.screens.is_empty() {
                    self.display_read();
                }
                let part = self.part;
                let show = |k: usize| part.is_none() || part == Some(k);
                let (head, about) = match part {
                    Some(0) => ("Grössen", "Wie gross Schrift und Anzeige sind."),
                    Some(1) => ("Helligkeit", "Wie hell der Bildschirm leuchtet."),
                    Some(2) => ("Night Shift", "Wärmere Farben am Abend — schont die Augen und den Schlaf."),
                    Some(3) => ("Drehung", "Ob der Bildschirm sich mit dem Gerät dreht."),
                    Some(4) => ("Bildschirme", "Welche Bildschirme angeschlossen sind — und welcher welcher ist."),
                    Some(_) => ("Ruhezustand", "Was geschieht, wenn die Bildschirme dunkel sind."),
                    None => ("Anzeige", "Grössen, Helligkeit, Night Shift, Drehung, Bildschirme und Ruhezustand."),
                };
                f.heading(engine, head, Some(about));
                if show(0) {
                    f.section(engine, "Grössen");
                    let size_name = mk::typo::SIZES.get(self.text_scale).copied().unwrap_or("");
                    f.stepper(engine, TEXT_SIZE, "Textgrösse", Some("Schrift in Matrix-Apps"), size_name, self.text_scale > 0, self.text_scale + 1 < mk::typo::SIZES.len());
                    if mk::device::panels().is_some() {
                        let names: Vec<&str> = SCALES.iter().map(|(_, n)| *n).collect();
                        f.choice(engine, SCALE_BASE, "Anzeigegrösse", Some("Alles auf dem Bildschirm grösser oder kleiner — gilt sofort"), &names, self.display_scale);
                    } else {
                        f.value(engine, Some('\u{e30c}'), "Anzeigegrösse", Some("Skalierung des Bildschirms"), &self.screens.first().map(|s| s.3.clone()).unwrap_or_else(|| String::from("—")));
                    }
                }
                if show(1) {
                    f.section(engine, "Helligkeit");
                    if mk::device::panels().is_some() {
                        // Round 71: duo-haltung follows the light sensor and learns
                        // from every manual change (the slider stays usable).
                        f.toggle(engine, AUTO_BRIGHT, "Automatisch", Some("Folgt dem Umgebungslicht und lernt, wenn du nachregelst"), mk::display::auto_brightness(), false);
                        // What the light sensor last measured (duo-haltung publishes it).
                        let lux = mk::display::ambient_lux().map(|v| format!("{v:.0} Lux")).unwrap_or_else(|| String::from("noch kein Wert"));
                        f.value(engine, Some('\u{e518}'), "Umgebungslicht", Some("Der Sensor meldet sich, wenn sich das Licht ändert"), &lux);
                    }
                    match self.bright {
                        Some(v) => f.slider(engine, BRIGHT, "Bildschirmhelligkeit", Some(&format!("{:.0} %", v * 100.0)), v),
                        None => f.value(engine, Some('\u{e1ae}'), "Bildschirmhelligkeit", Some("Dieser Bildschirm lässt sich nicht regeln"), "—"),
                    }
                }
                if show(2) {
                    f.section(engine, "Night Shift");
                    let mode = match mk::night::mode() {
                        mk::night::Mode::Off => 0,
                        mk::night::Mode::Auto => 1,
                        mk::night::Mode::Schedule => 2,
                        mk::night::Mode::On => 3,
                    };
                    f.choice(engine, NIGHT_BASE, "Night Shift", Some("Automatisch: vom Sonnenuntergang bis zum Sonnenaufgang"), &["Aus", "Automatisch", "Zeitplan", "Immer"], mode);
                    if mode == 2 {
                        let (from, to) = mk::night::schedule();
                        let hm = |m: u32| format!("{:02}:{:02}", m / 60, m % 60);
                        f.stepper(engine, NIGHT_FROM, "Von", None, &hm(from), true, true);
                        f.stepper(engine, NIGHT_TO, "Bis", None, &hm(to), true, true);
                    }
                    let warmth = mk::night::warmth();
                    let kelvin = mk::night::MILDEST_K - warmth * (mk::night::MILDEST_K - mk::night::WARMEST_K);
                    f.slider(engine, NIGHT_WARM, "Wärme", Some(&format!("{:.0} K", kelvin)), warmth);
                }
                if show(3) && mk::device::panels().is_some() {
                    f.section(engine, "Drehung");
                    let auto = mk::display::auto_rotation();
                    f.toggle(engine, ROT_AUTO, "Automatisch drehen", Some("Der Bildschirm folgt der Lage des Geräts"), auto, false);
                    let now = mk::display::orientation();
                    let at = now.as_deref().and_then(|o| mk::display::ORIENTATIONS.iter().position(|(k, _)| *k == o)).unwrap_or(0);
                    let names: Vec<&str> = mk::display::ORIENTATIONS.iter().map(|(_, n)| *n).collect();
                    f.choice(engine, ROT_BASE, "Ausrichtung", Some(if auto { "Wählen hält die Ausrichtung fest" } else { "Festgehalten — dreht sich nicht mit" }), &names, at);
                }
                if show(4) {
                    f.section(engine, "Bildschirme");
                    for (i, (name, model, mode, scale)) in self.screens.iter().enumerate() {
                        f.value(engine, Some('\u{e30c}'), &format!("{} · {name}", i + 1), Some(&format!("{model} — {mode}")), scale);
                    }
                    f.nav(engine, IDENTIFY, Some('\u{e8f4}'), "Bildschirme erkennen", Some("Zeigt auf jedem Bildschirm drei Sekunden seine Nummer"), None);
                    // Round 85: how the duo is held, and what the back panel does.
                    if mk::device::panels().is_some() {
                        f.section(engine, "Zurückgeklappt");
                        // Round 159: with a screen on USB-C the desk says how the duo is used.
                        let held = match (mk::device::desk(), mk::device::fold()) {
                            (desk @ (mk::device::Desk::Shut { .. } | mk::device::Desk::Second { .. }), _) => desk.words(),
                            (_, mk::device::Fold::Open) => String::from("Aufgeklappt — beide Bildschirme"),
                            (_, mk::device::Fold::Folded { front }) => format!("Zurückgeklappt — vorne das {}", mk::device::panel_label(&front)),
                        };
                        f.value(engine, Some('\u{e1b1}'), "Haltung", Some("Topbar, Dock und Fenster kommen auf den vorderen Bildschirm"), &held);
                        f.toggle(engine, BACK_OFF, "Hinteren Bildschirm ganz ausschalten", Some("Schwarz statt nur dunkel; beim Aufklappen blinkt der vordere kurz. Gilt beim nächsten Zurückklappen"), mk::display::back_panel_off(), false);
                        // Round 158: a screen on USB-C (TV, monitor) — the duo as a computer.
                        f.section(engine, "Bildschirm am USB-C");
                        let now = mk::canvas::external_screen().map(|s| format!("{} · {}×{}", s.label, s.width, s.height)).unwrap_or_else(|| String::from("Nicht angeschlossen"));
                        f.value(engine, Some('\u{e333}'), "Angeschlossen", Some("Maus und Tastatur per Bluetooth — dann arbeitet das Duo wie ein Computer"), &now);
                        f.toggle(engine, DESK_CLOSED, "Zugeklappt wie ein Laptop", Some("Das Duo bleibt dunkel, Leiste, Dock und Fenster sind auf dem Bildschirm. Aus: Zuklappen sperrt wie bisher"), mk::display::desk_closed(), false);
                        f.toggle(engine, DESK_FOLDED, "Zurückgeklappt als zweiter Monitor", Some("Leiste und Dock gehen auf den Bildschirm, das vordere Panel bleibt an"), mk::display::desk_folded(), false);
                        f.toggle(engine, DESK_SOUND, "Ton über den Bildschirm", Some("Beim Einstecken spielt der Ton dort; beim Ausstecken wieder am Duo"), mk::display::desk_sound(), false);
                        let names: Vec<&str> = mk::display::DESK_RESOLUTIONS.iter().map(|(_, n)| *n).collect();
                        f.choice(engine, DESK_RES_BASE, "Auflösung", Some("4K nur mit 30 Hz — zwei der vier USB-C-Leitungen tragen USB 3"), &names, mk::display::desk_choice("desk-resolution", &mk::display::DESK_RESOLUTIONS, 0));
                        let names: Vec<&str> = mk::display::DESK_SCALES.iter().map(|(_, n)| *n).collect();
                        f.choice(engine, DESK_SCALE_BASE, "Grösse auf dem Bildschirm", Some("Wie gross Fenster und Schrift dort erscheinen"), &names, mk::display::desk_choice("desk-scale", &mk::display::DESK_SCALES, 1));
                    }
                }
                if show(5) {
                    // Round 78: the sleep service (duo-schlaf) reads these two settings.
                    f.section(engine, "Ruhezustand");
                    if mk::power::has_deep_sleep() {
                        let on = mk::power::deep_sleep();
                        f.toggle(engine, DEEP_ON, "Tiefschlaf", Some("Bei dunklen Bildschirmen schläft das Gerät tief — die Power-Taste weckt es"), on, false);
                        if on {
                            let delay = mk::power::deep_sleep_delay();
                            let at = mk::power::SLEEP_DELAYS.iter().position(|(s, _)| *s == delay).unwrap_or(1);
                            let names: Vec<&str> = mk::power::SLEEP_DELAYS.iter().map(|(_, n)| *n).collect();
                            f.choice(engine, DEEP_DELAY_BASE, "Einschlafen nach", Some("Es wartet, solange Ton läuft, ein Kabel steckt oder das Gerät zugeklappt ist"), &names, at);
                        }
                        let span = |s: u32| if s >= 3600 { format!("{} h {} min", s / 3600, s % 3600 / 60) } else if s >= 60 { format!("{} min", s / 60) } else { format!("{s} s") };
                        let log = mk::power::sleep_today().unwrap_or_default();
                        let today = if log.count == 0 { String::from("noch nicht") } else { format!("{}× · {}", log.count, span(log.secs)) };
                        // Round 79: the measured draw while asleep (charge counter before/after).
                        let draw = log.sleep_ma.map(|ma| format!(" · im Schlaf ≈ {} mA", ma.round() as i32));
                        let last = log.last.map(|(at, s)| format!("Zuletzt {at} für {}{}", span(s), draw.as_deref().unwrap_or("")));
                        f.value(engine, Some('\u{e8ac}'), "Heute geschlafen", last.as_deref(), &today);
                    } else {
                        f.value(engine, Some('\u{e8ac}'), "Tiefschlaf", Some("Auf diesem Gerät regelt das System den Ruhezustand"), "—");
                    }
                }
            }
            _ => {
                f.heading(engine, title, Some("Dieser Bereich zieht in der nächsten Stufe ins Fensterhaus um."));
                f.section(engine, "");
                f.nav(engine, OPEN_OLD, Some(AREA_GLYPH[self.area]), &format!("{title} öffnen"), Some("in der bisherigen Fassung der Einstellungen"), None);
            }
        }
    }

    fn blink_on(&self) -> bool {
        self.beats % 2 == 0
    }

    /// The sidebar (or the narrow form's list) — the Kit's `fam::lay_nav`.
    /// Entry i is area i (only the last area, Hardline, can be hidden).
    fn lay_side(&mut self, engine: &mut TextEngine, b: &mut SceneBuilder, area: Rect) {
        let entries: Vec<fam::NavEntry> = self
            .areas()
            .into_iter()
            .map(|a| fam::NavEntry { title: AREA_TITLE[a], glyph: AREA_GLYPH[a], parts: parts(a), dot: !CARRIED.contains(&a) })
            .collect();
        // The kit counts entries by position (round 70: not the area number).
        let at = self.areas().iter().position(|a| *a == self.area);
        let selected = at.filter(|_| !self.menu || !parts(self.area).is_empty());
        let height = fam::lay_nav(b, engine, &entries, selected, self.part, !self.folded, area, self.side_roll, SIDE_BASE, SUB_BASE, self.palette);
        self.side_max = (height - area.h).max(0.0);
    }

    /// The background gallery: 16:9 tiles, three a row (two when narrow);
    /// the image worn now carries an accent ring. Returns the end y.
    fn lay_gallery(&self, b: &mut SceneBuilder, y: f32, x: f32, w: f32, page: Rect) -> f32 {
        let p = self.palette;
        let now = mk::wallpaper::read(self.wp_for == 0).or_else(mk::wallpaper::current).unwrap_or_default();
        let cols = if w < 420.0 { 2 } else { 3 };
        let gap = mk::spacing::S;
        let tw = (w - gap * (cols as f32 - 1.0)) / cols as f32;
        let th = tw * 9.0 / 16.0;
        let ring = 3.0;
        for (i, path) in self.wp_gallery.iter().enumerate() {
            let (cx, cy) = (x + (i % cols) as f32 * (tw + gap), y + (i / cols) as f32 * (th + gap));
            let on = path.to_string_lossy() == now;
            if on {
                b.quad(cx - ring, cy - ring, tw + 2.0 * ring, th + 2.0 * ring, p.primary, 14.0 + ring, Some(WP_BASE + i), Some(page));
            }
            let home = Rect { x: cx, y: cy, w: tw, h: th };
            let anchor = b.quad(cx, cy, tw, th, p.surface_container_high, 12.0, Some(WP_BASE + i), Some(page));
            // Only the small thumbnail is ever drawn; until it exists the
            // tile stays a calm placeholder (and every tile is laid each
            // frame, so the atlas keeps them while scrolling).
            let Some(thumb) = self.wp_thumbs.get(path) else { continue };
            b.pict(shell::Pict {
                path: thumb.clone(),
                box_w: tw as u32,
                box_h: th as u32,
                fit: matrixkit_image::Fit::Cover,
                x: cx,
                y: cy,
                radius: 12.0,
                home,
                anchor,
                clip: Some(page),
            });
        }
        let rows = self.wp_gallery.len().div_ceil(cols);
        y + rows as f32 * (th + gap) + mk::spacing::L
    }

    fn view_inner(&mut self, engine: &mut TextEngine, w: u32, h: u32) -> Scene {
        let (wf, hf) = (w as f32, h as f32);
        let p = self.palette;
        let mut b = SceneBuilder::new();
        let top = self.chrome.build(engine, &mut b, w, h, p);
        if self.chrome.rolled {
            return b.finish();
        }
        let narrow = wf < NARROW;
        self.narrow = narrow;
        let body = Rect { x: 0.0, y: top, w: wf, h: (hf - top).max(0.0) };
        if narrow && self.menu {
            self.side_rect = body;
            self.page_rect = Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 };
            self.lay_side(engine, &mut b, body);
        } else {
            let (side, page) = if narrow {
                (Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 }, body)
            } else {
                (Rect { x: 0.0, y: top, w: SIDE_W, h: body.h }, Rect { x: SIDE_W, y: top, w: wf - SIDE_W, h: body.h })
            };
            self.side_rect = side;
            self.page_rect = page;
            if !narrow {
                b.quad(side.x, side.y, side.w, side.h, p.surface_container.with_alpha(0.55), 0.0, None, None);
                self.lay_side(engine, &mut b, side);
            }
            let m = mk::spacing::L;
            let cw = (page.w - 2.0 * m).min(680.0);
            let cx = page.x + (page.w - cw) / 2.0;
            let mut y0 = page.y + m - self.page_roll;
            if narrow {
                let (bw, bh) = fam::lay_pill(&mut b, engine, Some('\u{e5cb}'), "Einstellungen", cx, y0, None, Some(BACK), fam::PillTone::Quiet, p);
                let _ = bw;
                y0 += bh + mk::spacing::M;
            }
            let mut f = fam::FormPage::new(&mut b, cx, y0, cw, Some(page), p);
            self.page(engine, &mut f);
            let (end, sliders) = f.finish();
            self.sliders = sliders;
            // Background: the gallery as picture tiles under the form.
            let end = if self.wallpaper_page() { self.lay_gallery(&mut b, end, cx, cw, page) } else { end };
            self.page_max = (end + self.page_roll - (page.y + page.h)).max(0.0);
        }
        // The receipt of the last change, at the bottom.
        if let Some(note) = self.receipt.text() {
            let note = note.to_string();
            let wn = fam::pill_width(&mut *engine, Some('\u{e5ca}'), &note).min(wf - 32.0);
            let x = if narrow { (wf - wn) / 2.0 } else { SIDE_W + (wf - SIDE_W - wn) / 2.0 };
            fam::lay_pill(&mut b, engine, Some('\u{e5ca}'), &note, x, hf - 56.0, Some(wn), None, fam::PillTone::Accent, p);
        }
        if let Some(ssid) = self.pw_for.clone() {
            self.lay_password_card(&mut b, engine, &ssid, wf, hf);
        }
        let mut scene = b.finish();
        if self.root.visible() {
            let mut rb = SceneBuilder::new();
            rb.scene = scene;
            let about = fam::RootAbout {
                name: "Matrix Einstellungen",
                version: env!("CARGO_PKG_VERSION"),
                description: "Die Einstellungen von Matrix — im Fensterhaus, ohne iced.",
            };
            self.root.lay(engine, &mut rb, &about, wf, hf, p, ROOT_BASE);
            scene = rb.finish();
        }
        scene
    }
}

impl shell::Client for Settings {
    type Card = String;
    type Work = Work;

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn warp(&mut self, loom: &Loom<Work>) {
        self.loom = Some(loom.clone());
        if self.area == 3 {
            self.gather();
        }
    }

    fn woven(&mut self, work: Work) -> Refresh {
        match work {
            Work::Conn(status, full) => {
                // A quick answer after the full one is stale: drop it.
                if full || self.conn_busy {
                    self.conn = status;
                    self.conn_known = true;
                }
                if full {
                    self.conn_busy = false;
                }
            }
            Work::Joined(ok, ssid) => {
                self.conn_busy = false;
                let with_card = self.pw_busy;
                self.pw_busy = false;
                if ok {
                    self.pw_for = None;
                    self.pw.clear();
                    self.pw_error = None;
                    self.note(format!("Mit «{ssid}» verbunden ✓"));
                } else {
                    // Refused (or it needs a password after all): the card
                    // opens or stays open and says so.
                    self.pw_for = Some(ssid);
                    self.pw.clear();
                    self.pw_error = with_card.then(|| String::from("Passwort abgelehnt — bitte nochmals"));
                }
                self.gather();
            }
            Work::Thumbs(list) => {
                self.wp_thumbs.extend(list);
            }
            Work::Note(ok, text) => {
                eprintln!("[settings] {text}");
                self.conn_busy = false;
                self.pairing = None;
                self.note(if ok { format!("{text} ✓") } else { text });
                self.gather();
            }
        }
        Refresh::Dirty
    }

    fn base(&mut self, _atlas: &mut ImageAtlas, buf: &mut [u8], _w: u32, _h: u32) {
        let p = self.palette;
        let (b, g, r) = ((p.surface.b * 255.0) as u8, (p.surface.g * 255.0) as u8, (p.surface.r * 255.0) as u8);
        for px in buf.chunks_exact_mut(4) {
            px[0] = b;
            px[1] = g;
            px[2] = r;
            px[3] = 0xff;
        }
    }

    fn view(&mut self, engine: &mut TextEngine, _keep: &mut Keep<f32>, _atlas: &mut ImageAtlas, w: u32, h: u32) -> Scene {
        self.view_inner(engine, w, h)
    }

    fn animating(&self) -> bool {
        self.chrome.animating() || self.root.animating() || self.mic.running()
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        // The microphone's level bar lives.
        let dirty = self.chrome.glide(dt) | self.root.glide(dt) | self.mic.running();
        if dirty {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn beat(&mut self) -> Refresh {
        self.beats += 1;
        // Ton shows the guard live.
        let mut dirty = self.pw_for.is_some() || self.pairing.is_some() || self.receipt.alive() || self.area == 7;
        // An armed «Entziehen» that ran out goes back to its plain word.
        dirty |= self.laptop_armed.lapsed();
        // Ton: devices come and go (Bluetooth), levels change from outside.
        if self.area == 7 && self.beats % 6 == 0 && self.dragging.is_none() {
            self.sound_read();
        }
        // Live values every three seconds.
        if self.area == 0 && self.beats % 6 == 0 {
            if let Some(o) = &mut self.overview {
                o.measure();
                dirty = true;
            }
        }
        // A wish from the bar, dock or a context menu: go there.
        if let Some(a) = crate::area_wish() {
            self.enter(a);
            if let Some(p) = crate::wish_part() {
                self.part = Some(p);
            }
            self.menu = false;
            dirty = true;
        }
        if dirty {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    /// Round 70: «Bildschirme erkennen» — a big number on every screen for
    /// three seconds (a pane per screen, its name is the card).
    fn fleet(&mut self) -> Vec<shell::fleet::Spec<String>> {
        if self.ident_until.is_none() {
            return Vec::new();
        }
        self.screens
            .iter()
            .enumerate()
            .map(|(i, (name, ..))| shell::fleet::Spec { pane: name.clone(), width: 220, height: 180, cross: 0, namespace: "matrix-ident", keyboard: false, sig: (9, i as u32) })
            .collect()
    }

    fn pane_place(&mut self, card: &String) -> Option<shell::PanePlace> {
        let mut place = shell::PanePlace::at(shell::Anchors { top: false, right: false, bottom: false, left: false }, (0, 0, 0, 0), (220, 180), Some(card.clone()), Some(shell::Layer::Overlay));
        place.keyboard = shell::KeyboardWish::None;
        Some(place)
    }

    fn pane_scene(&mut self, card: &String, engine: &mut TextEngine, _atlas: &mut matrixkit_image::ImageAtlas, w: u32, h: u32) -> Option<Scene> {
        let p = self.palette;
        let nr = self.screens.iter().position(|s| &s.0 == card).map_or(0, |i| i + 1);
        let mut b = SceneBuilder::new();
        let (wf, hf) = (w as f32, h as f32);
        b.quad(0.0, 0.0, wf, hf, p.surface_container_high, 24.0, None, None);
        let big = nr.to_string();
        let px = 96.0;
        let (tw, th) = engine.measure_styled(&big, px, mk::typo::Weight::Semibold);
        b.text(engine, big, px, mk::typo::Weight::Semibold, matrixkit_text::FAMILY, p.on_surface, (wf - tw) / 2.0, hf * 0.42 - th / 2.0, None, None);
        let st = mk::typo::BODY;
        let (nw, nh) = engine.measure_styled(card, st.px(), st.weight);
        b.styled(engine, card.clone(), st, p.on_surface_variant, (wf - nw) / 2.0, hf - nh - 22.0, None, None);
        Some(b.finish())
    }

    fn pulse(&mut self) -> Refresh {
        if self.ident_until.is_some_and(|t| std::time::Instant::now() >= t) {
            self.ident_until = None;
            return Refresh::Panes;
        }
        let fresh = mk::Palette::load().unwrap_or_default();
        if fresh != self.palette {
            // The ground (base) changes colour too.
            self.palette = fresh;
            return Refresh::Full;
        }
        Refresh::Calm
    }

    fn zoomed(&mut self, on: bool) {
        self.chrome.set_zoomed(on);
    }

    fn extent(&mut self) -> Option<(u32, u32)> {
        self.chrome.extent()
    }

    fn headline(&self) -> f32 {
        self.chrome.headline()
    }

    fn strike(&mut self, role: Option<usize>, x: f64, y: f64, source: Source) -> Refresh {
        let (xf, yf) = (x as f32, y as f32);
        if let Some(deed) = self.root.strike_at(role, ROOT_BASE) {
            return match deed {
                fam::RootDeed::Help => {
                    matrixkit_scene::help_open("Matrix Einstellungen");
                    Refresh::Calm
                }
                fam::RootDeed::Calm => Refresh::Calm,
                _ => Refresh::Dirty,
            };
        }
        match self.chrome.strike(role) {
            Some(fam::ChromeDeed::Close) => {
                eprintln!("[settings] Ampel × ({})", source.label());
                return Refresh::Close;
            }
            Some(fam::ChromeDeed::Answer(r)) => return r,
            Some(fam::ChromeDeed::Title) => {
                self.root.toggle();
                return Refresh::Dirty;
            }
            None => {}
        }
        // A slider follows the finger at once.
        if let Some((r, track)) = role.and_then(|r| self.sliders.iter().find(|(s, _)| *s == r).copied()) {
            self.dragging = Some(r);
            self.slide(r, fam::slider_value(track, xf), false);
            return Refresh::Dirty;
        }
        let side = self.side_rect.w > 0.0 && xf < self.side_rect.x + self.side_rect.w;
        self.press = Some(Press::new(xf, yf, if side { self.side_roll } else { self.page_roll }, (side, role)));
        Refresh::Calm
    }

    fn travel(&mut self, x: f64, y: f64, _source: Source) -> Refresh {
        if let Some(r) = self.dragging {
            if let Some((_, track)) = self.sliders.iter().find(|(s, _)| *s == r).copied() {
                self.slide(r, fam::slider_value(track, x as f32), false);
                return Refresh::Dirty;
            }
        }
        let Some(pr) = &mut self.press else { return Refresh::Calm };
        let side = pr.what.0;
        match pr.travel(x as f32, y as f32, if side { self.side_max } else { self.page_max }) {
            Some(roll) if side => self.side_roll = roll,
            Some(roll) => self.page_roll = roll,
            None => return if pr.scrolling() { Refresh::Dirty } else { Refresh::Calm },
        }
        Refresh::Dirty
    }

    fn release(&mut self, _role: Option<usize>, x: f64, _y: f64, _source: Source) -> Refresh {
        if let Some(r) = self.dragging.take() {
            if let Some((_, track)) = self.sliders.iter().find(|(s, _)| *s == r).copied() {
                self.slide(r, fam::slider_value(track, x as f32), true);
            }
            return Refresh::Dirty;
        }
        match self.press.take().and_then(Press::release) {
            Some((_, Some(r))) => self.act(r),
            _ => Refresh::Calm,
        }
    }

    fn axis(&mut self, x: f64, _y: f64, value: f64, _keep: &mut Keep<f32>) -> Refresh {
        if self.side_rect.w > 0.0 && (x as f32) < self.side_rect.x + self.side_rect.w {
            self.side_roll = (self.side_roll + value as f32).clamp(0.0, self.side_max);
        } else {
            self.page_roll = (self.page_roll + value as f32).clamp(0.0, self.page_max);
        }
        Refresh::Dirty
    }

    fn key(&mut self, key: shell::Key, _shift: bool) -> Refresh {
        if self.pw_for.is_none() {
            return Refresh::Calm;
        }
        match key {
            shell::Key::Char(s) => self.pw.push_str(&s),
            shell::Key::Backspace => {
                self.pw.pop();
            }
            shell::Key::Enter => self.join_with_password(),
            shell::Key::Escape => {
                self.pw_for = None;
                self.pw.clear();
                self.pw_error = None;
            }
            _ => return Refresh::Calm,
        }
        Refresh::Dirty
    }

    fn wants_keyboard(&self) -> bool {
        self.pw_for.is_some()
    }

    fn wants_text(&self) -> bool {
        self.pw_for.is_some()
    }

    fn root(&mut self) -> Refresh {
        self.root.toggle();
        Refresh::Dirty
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    if !mk::window::single_instance(APP_ID) {
        return Ok(());
    }
    // The GPU paints: Krabbler and the microphone's level bar move (on
    // the CPU the Krabbler cost about 25 %, measured 02.11.2026).
    shell::prefer_gpu();
    let mut app = Settings::new();
    // A narrow window starts on the list unless a wish named an area.
    app.menu = false;
    eprintln!("[settings] Matrix Einstellungen im Fensterhaus, pid={}", std::process::id());
    fam::run(
        app,
        shell::Options {
            mount: shell::Mount::Window { app_id: String::from(APP_ID), title: String::from("Matrix Einstellungen"), width: 940, height: 680 },
            argb: false,
            tag: "settings",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}

/// Round 135: the lock clock's look (matrix-lock reads the same keys).
fn lock_glass() -> bool {
    mk::settings::read("lock-clock-glass").and_then(|v| v.trim().parse::<f32>().ok()).unwrap_or(0.85) >= 0.5
}

fn lock_tall() -> bool {
    mk::settings::read("lock-clock-stretch").and_then(|v| v.trim().parse::<f32>().ok()).unwrap_or(1.9) > 2.2
}

/// Round 136: the power key locks unless switched off (default on).
fn lock_on_power() -> bool {
    !mk::settings::read("lock-on-power").is_some_and(|v| v.trim() == "off")
}

#[cfg(test)]
mod id_tests {
    use super::*;

    /// Round 169: every control id range is its own — DESK_*, WHEEL_BASE and
    /// WINDOW_MOVE once lay inside THEME_BASE's range and on THEME_FORM, so a
    /// tap could change the wrong setting and «Matrix-Form» was unreachable.
    #[test]
    fn control_ids_never_overlap() {
        let ranges: Vec<(&str, usize, usize)> = vec![
            ("THEME_BASE", THEME_BASE, FAMILIES_SHOWN),
            ("VARIANT_BASE", VARIANT_BASE, VARIANTS_SHOWN),
            ("THEME_OPEN", THEME_OPEN, 1),
            ("VARIANT_OPEN", VARIANT_OPEN, 1),
            ("THEME_FORM", THEME_FORM, 1),
            ("DESK_CLOSED", DESK_CLOSED, 1),
            ("DESK_FOLDED", DESK_FOLDED, 1),
            ("DESK_SOUND", DESK_SOUND, 1),
            ("DESK_RES_BASE", DESK_RES_BASE, mk::display::DESK_RESOLUTIONS.len()),
            ("DESK_SCALE_BASE", DESK_SCALE_BASE, mk::display::DESK_SCALES.len()),
            ("WHEEL_BASE", WHEEL_BASE, mk::input::WHEEL_SPEEDS.len()),
            ("WINDOW_MOVE", WINDOW_MOVE, 1),
            ("FILM_SOUND", FILM_SOUND, 1),
            ("HAPTICS", HAPTICS, 1),
            ("LOCK_GLASS", LOCK_GLASS, 1),
            ("LOCK_TALL", LOCK_TALL, 1),
            ("LOCK_POWER", LOCK_POWER, 1),
            ("DEEP_ON", DEEP_ON, 1),
            ("BACK_OFF", BACK_OFF, 1),
            ("LAPTOP_BASE", LAPTOP_BASE, LAPTOPS_SHOWN),
            ("PLAY_BASE", PLAY_BASE, 100),
            ("WP_FOR_BASE", WP_FOR_BASE, 2),
            ("KIND_BASE", KIND_BASE, 200),
            ("OUT_BASE", OUT_BASE, 100),
            ("IN_BASE", IN_BASE, 100),
        ];
        for (i, a) in ranges.iter().enumerate() {
            for b in &ranges[i + 1..] {
                let apart = a.1 + a.2 <= b.1 || b.1 + b.2 <= a.1;
                assert!(apart, "{} ({}..{}) overlaps {} ({}..{})", a.0, a.1, a.1 + a.2, b.0, b.1, b.1 + b.2);
            }
        }
    }
}
