//! Matrix Zentrale — app #13, the control center of the bar family.
//!
//! A floating panel top right below the bar. Since R122 phase 2 the
//! rows are no longer hard-wired: the panel renders the user's list
//! from the control registry (mkw::central) in the user's order, and
//! carries the reference edit flow inside itself — minus badges to
//! remove, ▲▼ to reorder, and an add gallery with categories and a
//! search field. The list lives in the settings key `central-controls`.
//!
//! All levers are command bridges (wpctl, nmcli, bluetoothctl,
//! ddcutil, brightnessctl) — rows without a backend do not appear in
//! the panel and turn gray in the gallery.

use matrixkit_widgets::iced;
use matrixkit_widgets::iced_layershell;
use matrixkit_widgets::iced::widget::{button, column, container, mouse_area, row, Space};
use matrixkit_widgets::iced::{Element, Length, Subscription, Task};
use matrixkit_widgets::iced_layershell::to_layer_message;
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use matrixkit_widgets::tick;

mod licht;

const WIDTH: u32 = 340;
/// Transparent gutter on both tile sides (badge floats there, OUTSIDE
/// the glass body — the compositor insets the body by this much).
const GUTTER: u32 = 30;
/// Footer pill height.
const FOOTER_H: u32 = 36;
/// Height of an mkw row (title row + inner air), for the panel size.
const ROW: f32 = 50.0;
// Since R125 the card window is exactly content-sized — the old
// fullscreen head allowance would show as empty glass.
const HEAD: f32 = 6.0;
const SECTION_TITLE: f32 = 24.0;
/// Gallery metrics: the box is sized from its CONTENT (search field,
/// category labels, entry rows) and capped only by the screen.
const GALLERY_SEARCH: f32 = 44.0;
const GALLERY_LABEL: f32 = 26.0;
const GALLERY_ROW: f32 = 52.0;
const GALLERY_EMPTY: f32 = 60.0;

fn main() -> Result<(), iced_layershell::Error> {
    // Camp 34 of the iced replacement: --erstlicht is the window-house
    // path (shadow doctrine).
    if std::env::args().nth(1).as_deref() == Some("--erstlicht") {
        if let Err(e) = licht::run() {
            eprintln!("[zentrale] Fensterhaus-Fehler: {e}");
        }
        return Ok(());
    }
    matrixkit_widgets::present_mailbox();

    // Toggle: if the Zentrale already ran, it is closed now — so are we.
    if !mkw::panel_toggle() {
        return Ok(());
    }

    // Our sections float on the compositor glass card (R125).
    mkw::sections_on_glass();

    // R125 glass: the daemon pattern carries TWO windows — the veil
    // (dismiss layer, bar strip stays free) and the CARD, exactly
    // panel-sized with namespace matrix-card, so the compositor lays
    // its glass body under the panel like under the bar.
    mkw::app_canvas::run::<App>("matrix-central", mkw::app_canvas::background())
}

impl mkw::app_canvas::Daemon for App {
    type Msg = Msg;
    fn new() -> (Self, Task<Msg>) {
        App::new()
    }
    fn update(&mut self, msg: Msg) -> Task<Msg> {
        App::update(self, msg)
    }
    fn view(&self, window: iced::window::Id) -> Element<'_, Msg> {
        App::view(self, window)
    }
    fn subscription(&self) -> iced::Subscription<Msg> {
        App::subscription(self)
    }
}

/// "Volume: 0.45 [MUTED]" → (0.45, true)
pub(crate) fn sound_read(target: &str) -> Option<(f32, bool)> {
    let z = mk::command::first_line("wpctl", &["get-volume", target])?;
    let value: f32 = z.split_whitespace().nth(1)?.parse().ok()?;
    Some((value, z.contains("MUTED")))
}

pub(crate) fn wlan_read() -> Option<bool> {
    mk::command::first_line("nmcli", &["radio", "wifi"]).map(|z| z == "enabled")
}

pub(crate) fn bt_read() -> Option<bool> {
    let out = std::process::Command::new("bluetoothctl")
        .arg("show")
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    if !s.contains("Controller") {
        return None;
    }
    Some(s.lines().any(|l| l.trim() == "Powered: yes"))
}

/// Where the brightness comes from: laptop backlight or monitor DDC.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum BrightnessSource {
    Backlight,
    Ddc,
}

/// First the laptop backlight (Surface, fast), then monitor DDC
/// (desktop, sluggish — hence all of this runs asynchronously).
pub(crate) fn brightness_read() -> Option<(f32, BrightnessSource)> {
    // brightnessctl -m: "intel_backlight,backlight,48000,50%,96000"
    if let Some(z) = mk::command::first_line("brightnessctl", &mk::device::brightnessctl_args(&["-m"])) {
        if let Some(p) = z.split(',').nth(3) {
            if let Ok(value) = p.trim_end_matches('%').parse::<f32>() {
                return Some((value, BrightnessSource::Backlight));
            }
        }
    }
    // ddcutil --brief: "VCP 10 C 70 100"
    let z = mk::command::first_line("ddcutil", &["getvcp", "10", "--brief"])?;
    let value = z.split_whitespace().nth(3)?.parse().ok()?;
    Some((value, BrightnessSource::Ddc))
}

pub(crate) fn brightness_set(value: f32, source: BrightnessSource) {
    match source {
        BrightnessSource::Backlight => {
            let value = format!("{value:.0}%");
            let _ = mk::command::silent("brightnessctl", &mk::device::brightnessctl_args(&["set", &value]));
        }
        BrightnessSource::Ddc => {
            let _ = mk::command::silent("ddcutil", &["setvcp", "10", &format!("{value:.0}")]);
        }
    }
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
enum Msg {
    WindowClosed(iced::window::Id),
    Tick,
    // R135 event family: a listener fired — re-read that state.
    AudioChanged,
    NetChanged,
    Volume(f32),
    SoundMute,
    MicMute,
    Wlan(bool),
    Bluetooth(bool),
    Brightness(f32),
    SetBrightness,
    BrightnessAvailable(Option<(f32, BrightnessSource)>),
    Motion(bool),
    Transparenz(bool),
    Contrast(bool),
    Appearance,
    RotationLock(bool),
    // R122 phase 2: the edit flow.
    Edit(bool),
    Search(String),
    CtlAdd(String),
    CtlRemove(usize),
    DragStart(usize),
    DragOver(usize),
    DragEnd,
    Run(&'static str),
    Close,
}

struct App {
    palette: mk::Palette,
    watcher: mk::PaletteWatcher,
    volume: Option<(f32, bool)>,
    mic_muted: Option<bool>,
    wlan: Option<bool>,
    bluetooth: Option<bool>,
    /// 0–100 incl. source; None = no backend (or still loading).
    brightness: Option<(f32, BrightnessSource)>,
    motion_reduced: bool,
    transparency_reduced: bool,
    contrast_up: bool,
    /// R171: Drehung eingefroren? (Vertrag `rotation-lock`.)
    rotation_lock: bool,
    /// The user's controls, in panel order (registry keys).
    controls: Vec<String>,
    edit: bool,
    search: String,
    /// Index currently lifted by the reorder handle (reference
    /// grammar: rows displace live while the drag travels).
    drag: Option<usize>,
    /// One window PER island (reference control center: every tile is
    /// its own glass) — the fleet family owns ids, signature and the
    /// prefix-keep diff (R134).
    fleet: mkw::fleet::Fleet<Pane>,
}

/// What one island window shows.
#[derive(Clone)]
enum Pane {
    /// Consecutive rendered control indices of one category run.
    Group(Vec<usize>),
    Footer,
    Gallery,
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        let mut app = App {
            palette: mk::Palette::load().unwrap_or_default(),
            watcher: mk::PaletteWatcher::new(),
            volume: sound_read("@DEFAULT_AUDIO_SINK@"),
            mic_muted: sound_read("@DEFAULT_AUDIO_SOURCE@").map(|(_, m)| m),
            wlan: wlan_read(),
            bluetooth: bt_read(),
            brightness: None,
            motion_reduced: mk::motion_reduced(),
            transparency_reduced: mk::transparency_reduced(),
            contrast_up: mk::contrast_up(),
            rotation_lock: mk::settings::read("rotation-lock").as_deref() == Some("true"),
            controls: mkw::central::load(),
            // Dev aid: MATRIX_CENTRAL_EDIT=1 opens straight into the
            // edit flow (screenshot runs over SSH cannot click).
            edit: std::env::var_os("MATRIX_CENTRAL_EDIT").is_some(),
            search: String::new(),
            drag: None,
            fleet: mkw::fleet::Fleet::new(
                mkw::panel::BAR_STRIP + mkw::panel::POPUP_TOP as i32,
                mk::spacing::M as i32,
            ),
        };
        let veil = app.open_veil();
        let panes = app.sync_panes();
        (
            app,
            Task::batch([
                veil,
                panes,
                // DDC takes seconds — the panel does not wait for it.
                Task::perform(async { brightness_read() }, Msg::BrightnessAvailable),
            ]),
        )
    }

    /// The dismiss veil, created once at startup.
    fn open_veil(&mut self) -> Task<Msg> {
        use matrixkit_widgets::iced_layershell::reexport::{
            Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings,
        };
        let veil = iced::window::Id::unique();
        self.fleet.set_veil(veil);
        Task::done(Msg::NewLayerShell {
            settings: NewLayerShellSettings {
                size: Some((0, 0)),
                anchor: Anchor::Top | Anchor::Right | Anchor::Bottom | Anchor::Left,
                margin: Some((mkw::panel::BAR_STRIP, 0, 0, 0)),
                layer: Layer::Overlay,
                keyboard_interactivity: KeyboardInteractivity::None,
                exclusive_zone: Some(-1),
                ..Default::default()
            },
            id: veil,
        })
    }

    /// The control indices currently rendered, in order.
    fn rendered_rows(&self) -> Vec<usize> {
        self.controls
            .iter()
            .enumerate()
            .filter(|(_, key)| self.edit || self.available(key))
            .map(|(i, _)| i)
            .collect()
    }

    /// Deterministic row height — every row is WRAPPED to this, so the
    /// island window fits its content exactly (glass body = window).
    fn row_height(&self, i: usize) -> u32 {
        let key = &self.controls[i];
        let desc = matches!(key.as_str(), "motion" | "transparency" | "contrast")
            || matches!(key.as_str(), "capture" | "lock")
            || (self.edit && !self.available(key));
        if desc { 64 } else { ROW as u32 }
    }

    /// The island plan: one pane per category run, footer, gallery.
    fn pane_plan(&self) -> Vec<(Pane, u32)> {
        let mut plan: Vec<(Pane, u32)> = Vec::new();
        let mut run: Vec<usize> = Vec::new();
        let mut run_cat: Option<mkw::central::Category> = None;
        let flush = |run: &mut Vec<usize>, plan: &mut Vec<(Pane, u32)>, me: &Self| {
            if !run.is_empty() {
                let h: u32 = run.iter().map(|i| me.row_height(*i)).sum::<u32>()
                    + (run.len() as u32 - 1);
                plan.push((Pane::Group(std::mem::take(run)), h));
            }
        };
        for i in self.rendered_rows() {
            let Some(c) = mkw::central::find(&self.controls[i]) else { continue };
            if run_cat != Some(c.category) {
                flush(&mut run, &mut plan, self);
                run_cat = Some(c.category);
            }
            run.push(i);
        }
        flush(&mut run, &mut plan, self);
        plan.push((Pane::Footer, FOOTER_H));
        if self.edit {
            plan.push((Pane::Gallery, self.gallery_box() as u32));
        }
        plan
    }

    /// Rebuild the island windows whenever the STRUCTURE changed —
    /// every island is its own glass window (reference control
    /// center: each tile alone is glass, nothing around it).
    fn sync_panes(&mut self) -> Task<Msg> {
        use matrixkit_widgets::iced_layershell::reexport::{
            Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings,
        };
        let footer_w: u32 = if self.edit { 96 } else { 224 };
        let specs: Vec<mkw::fleet::Spec<Pane>> = self
            .pane_plan()
            .into_iter()
            .map(|(pane, h)| {
                let kind = match pane {
                    Pane::Group(_) => 0u8,
                    Pane::Footer => 1,
                    Pane::Gallery => 2,
                };
                // Width folded into the height slot (heights < 8192).
                let sig_w = match pane {
                    Pane::Footer => footer_w,
                    _ => 0,
                };
                // Group tiles carry a transparent GUTTER on both sides —
                // the compositor insets the glass body (matrix-tile
                // rule), and the minus badges float there, outside.
                let (width, cross, namespace) = match pane {
                    Pane::Group(_) => (
                        WIDTH + 2 * GUTTER,
                        mkw::panel::POPUP_RIGHT as i32 - GUTTER as i32,
                        "matrix-tile",
                    ),
                    Pane::Footer => (
                        footer_w,
                        mkw::panel::POPUP_RIGHT as i32 + (WIDTH as i32 - footer_w as i32) / 2,
                        "matrix-card",
                    ),
                    Pane::Gallery => (WIDTH, mkw::panel::POPUP_RIGHT as i32, "matrix-card"),
                };
                let keyboard = matches!(pane, Pane::Gallery);
                mkw::fleet::Spec {
                    pane,
                    width,
                    height: h,
                    cross,
                    namespace,
                    keyboard,
                    sig: (kind, h + sig_w * 8192),
                }
            })
            .collect();
        self.fleet.sync(
            specs,
            |shell, id| {
                Task::done(Msg::NewLayerShell {
                    settings: NewLayerShellSettings {
                        size: Some((shell.width, shell.height)),
                        anchor: Anchor::Top | Anchor::Right,
                        margin: Some((shell.main, shell.cross, 0, 0)),
                        layer: Layer::Overlay,
                        keyboard_interactivity: if shell.keyboard {
                            KeyboardInteractivity::OnDemand
                        } else {
                            KeyboardInteractivity::None
                        },
                        exclusive_zone: Some(-1),
                        namespace: Some(String::from(shell.namespace)),
                        ..Default::default()
                    },
                    id,
                })
            },
            |id| Task::done(Msg::RemoveWindow(id)),
        )
    }

    /// Does this control have a live backend right now?
    fn available(&self, key: &str) -> bool {
        match key {
            "volume" | "mute" => self.volume.is_some(),
            "mic" => self.mic_muted.is_some(),
            "wlan" => self.wlan.is_some(),
            "bluetooth" => self.bluetooth.is_some(),
            "brightness" => self.brightness.is_some(),
            _ => true,
        }
    }

    /// The gallery entries that currently match search and placement.
    fn gallery_counts(&self) -> (usize, usize) {
        let need = self.search.to_lowercase();
        let mut rows = 0;
        let mut cats = 0;
        for cat in mkw::central::Category::ALL {
            let n = mkw::central::REGISTRY
                .iter()
                .filter(|c| c.category == cat && !c.fixed)
                .filter(|c| !self.controls.iter().any(|k| k == c.key))
                .filter(|c| {
                    need.is_empty()
                        || c.title.to_lowercase().contains(&need)
                        || c.subtitle.to_lowercase().contains(&need)
                })
                .count();
            if n > 0 {
                cats += 1;
                rows += n;
            }
        }
        (rows, cats)
    }

    /// The panel WITHOUT the gallery (rows, section titles, footer).
    fn base_height(&self) -> f32 {
        let mut lines = 0.0;
        let mut sections = 0.0;
        let mut last: Option<mkw::central::Category> = None;
        for key in &self.controls {
            let Some(c) = mkw::central::find(key) else { continue };
            if !self.edit && !self.available(key) {
                continue;
            }
            lines += 1.0;
            if last != Some(c.category) {
                sections += 1.0;
                last = Some(c.category);
            }
        }
        HEAD
            + lines * ROW
            + sections * (SECTION_TITLE + mk::spacing::M)
            + 34.0
            + 2.0 * mk::spacing::M
    }

    fn screen_cap(&self) -> f32 {
        (mkw::glass::screen_size().1 - mkw::panel::BAR_STRIP as f32 - 40.0).max(300.0)
    }

    /// The full content height of the gallery, spacing included.
    fn gallery_raw(&self) -> f32 {
        let (rows, cats) = self.gallery_counts();
        if rows == 0 {
            return GALLERY_SEARCH + GALLERY_EMPTY;
        }
        let items = 1.0 + cats as f32 + rows as f32;
        GALLERY_SEARCH
            + cats as f32 * GALLERY_LABEL
            + rows as f32 * GALLERY_ROW
            + (items - 1.0) * mk::spacing::XS
            + mk::spacing::M
    }

    /// The gallery box grows WITH its content (user finding 23.07:
    /// a fixed box stays small while everything piles up inside) and
    /// is capped only by the screen — beyond that the region scrolls.
    fn gallery_box(&self) -> f32 {
        let free = self.screen_cap() - self.base_height() - SECTION_TITLE - mk::spacing::M;
        self.gallery_raw().min(free.max(GALLERY_SEARCH + GALLERY_EMPTY))
    }

    fn save(&self) {
        mkw::central::save(&self.controls);
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        let task = self.handle(msg);
        // The island fleet follows every structural change.
        let sync = self.sync_panes();
        Task::batch([task, sync])
    }

    fn handle(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::WindowClosed(id) => {
                // Island windows come and go (sync_panes) — only the
                // VEIL closing means dismissal.
                if self.fleet.is_veil(id) {
                    std::process::exit(0);
                }
            }
            Msg::AudioChanged => {
                self.volume = sound_read("@DEFAULT_AUDIO_SINK@");
                self.mic_muted = sound_read("@DEFAULT_AUDIO_SOURCE@").map(|(_, m)| m);
            }
            Msg::NetChanged => {
                self.wlan = wlan_read();
            }
            Msg::Tick => {
                if self.watcher.changed() {
                    if let Some(new) = mk::Palette::load() {
                        self.palette = new;
                    }
                }
                self.volume = sound_read("@DEFAULT_AUDIO_SINK@");
                self.mic_muted = sound_read("@DEFAULT_AUDIO_SOURCE@").map(|(_, m)| m);
                self.wlan = wlan_read();
                self.bluetooth = bt_read();
                self.motion_reduced = mk::motion_reduced();
                self.transparency_reduced = mk::transparency_reduced();
                self.contrast_up = mk::contrast_up();
            }
            Msg::Volume(v) => {
                let _ = mk::command::silent("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{v:.2}")]);
                if let Some((_, muted)) = self.volume {
                    self.volume = Some((v, muted));
                }
            }
            Msg::SoundMute => {
                let _ = mk::command::silent("wpctl", &["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]);
                if let Some((v, muted)) = self.volume {
                    self.volume = Some((v, !muted));
                }
            }
            Msg::MicMute => {
                let _ = mk::command::silent("wpctl", &["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"]);
                self.mic_muted = self.mic_muted.map(|m| !m);
            }
            Msg::Wlan(on) => {
                let _ = mk::command::silent("nmcli", &["radio", "wifi", if on { "on" } else { "off" }]);
                self.wlan = Some(on);
            }
            Msg::Bluetooth(on) => {
                let _ = mk::command::silent("bluetoothctl", &["power", if on { "on" } else { "off" }]);
                self.bluetooth = Some(on);
            }
            Msg::Brightness(v) => {
                // The display follows immediately, setting happens on release.
                if let Some((_, q)) = self.brightness {
                    self.brightness = Some((v, q));
                }
            }
            Msg::SetBrightness => {
                if let Some((v, q)) = self.brightness {
                    // DDC is sluggish — setting runs in the background, the echo
                    // (None) changes nothing about the state.
                    return Task::perform(
                        async move {
                            brightness_set(v, q);
                            None
                        },
                        Msg::BrightnessAvailable,
                    );
                }
            }
            Msg::BrightnessAvailable(w) => {
                if w.is_some() {
                    self.brightness = w;
                    // Fullscreen overlay (R103): the surface never resizes.
                }
            }
            Msg::Motion(reduced) => {
                mk::settings::write("motion", if reduced { "reduced" } else { "full" });
                self.motion_reduced = reduced;
            }
            Msg::Transparenz(reduced) => {
                mk::settings::write(
                    "transparency",
                    if reduced { "reduced" } else { "full" },
                );
                self.transparency_reduced = reduced;
            }
            Msg::Contrast(up) => {
                mk::settings::write("contrast", if up { "high" } else { "normal" });
                if let Some(new) = mk::Palette::load() {
                    self.palette = new; // color immediately with our own medicine
                }
                self.contrast_up = up;
            }
            // R171: the rotation lock — freezes the orientation without
            // switching the sensor off (the service reads the contract live).
            Msg::RotationLock(on) => {
                mk::settings::write("rotation-lock", if on { "true" } else { "false" });
                self.rotation_lock = on;
            }
            Msg::Appearance => {
                mk::light_toggle(); // own color chain, no DMS
            }
            Msg::Edit(on) => {
                self.edit = on;
                if !on {
                    self.search.clear();
                }
            }
            Msg::Search(s) => self.search = s,
            Msg::CtlAdd(key) => {
                if !self.controls.contains(&key) {
                    self.controls.push(key);
                    self.save();
                }
            }
            Msg::CtlRemove(i) => {
                if i < self.controls.len() {
                    self.controls.remove(i);
                    self.save();
                }
            }
            Msg::DragStart(i) => {
                if i < self.controls.len() {
                    self.drag = Some(i);
                }
            }
            Msg::DragOver(j) => {
                // Live displacement: the lifted row travels to the row
                // the cursor enters — the list reflows immediately.
                if let Some(i) = self.drag {
                    if i != j && i < self.controls.len() && j < self.controls.len() {
                        let lifted = self.controls.remove(i);
                        self.controls.insert(j, lifted);
                        self.drag = Some(j);
                    }
                }
            }
            Msg::DragEnd => {
                if self.drag.take().is_some() {
                    self.save();
                }
            }
            Msg::Run(cmd) => {
                let _ = mk::process::spawn_reaped(&mut std::process::Command::new(cmd)); // round 51
                std::process::exit(0);
            }
            Msg::Close => std::process::exit(0),
            // variants added by the to_layer_message macro
            _ => {}
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Msg> {
        Subscription::batch([
            // R135: listeners fire, we query (Apple's zero-polling).
            // The tick is only the slow fallback for states without a
            // monitor stream (palette, bluetooth, brightness).
            tick("zentrale", std::time::Duration::from_secs(10)).map(|_| Msg::Tick),
            mkw::events::watch("audio", "pw-mon", &[], |l| l.contains("changed:"))
                .map(|_| Msg::AudioChanged),
            mkw::events::watch("network", "nmcli", &["monitor"], |_| true)
                .map(|_| Msg::NetChanged),
            iced::window::close_events().map(Msg::WindowClosed),
        ])
    }

    // ---- rows ------------------------------------------------------------

    /// The live row of one control — None if the backend is missing.
    /// R124: the trailing control MORPHS into the reorder handle when
    /// the edit flow opens; the row itself never changes layout, so
    /// the mode switch is barely a layer change. Fixed modules morph
    /// their slider away and stay otherwise untouched.
    fn control_row(&self, i: usize, key: &str) -> Option<Element<'_, Msg>> {
        let p = self.palette;
        let lead = |z: char| {
            Some(mkw::symbol(z, mk::icon_size::SMALL + 4.0, p.on_surface_variant))
        };
        let fixed = mkw::central::find(key).is_some_and(|c| c.fixed);
        match key {
            "volume" => {
                let (v, _) = self.volume?;
                Some(mkw::row_item(
                    "Lautstärke",
                    None,
                    lead(mkw::symbol::VOLUME_UP),
                    Some(self.trailing_swap(i, fixed, 
                        mkw::bar_slider(0.0..=1.0, v, 0.01, p, Msg::Volume)
                            .width(Length::Fixed(150.0))
                            .into(),
                    )),
                    p,
                ))
            }
            "mute" => {
                let (_, muted) = self.volume?;
                Some(mkw::row_item(
                    "Ton aus",
                    None,
                    lead(mkw::symbol::VOLUME_OFF),
                    Some(self.trailing_swap(i, fixed, mkw::toggle_guarded(muted, p, Some(Msg::SoundMute)))),
                    p,
                ))
            }
            "mic" => {
                let m = self.mic_muted?;
                Some(mkw::row_item(
                    "Mikrofon aus",
                    None,
                    lead(mkw::symbol::MIC_OFF),
                    Some(self.trailing_swap(i, fixed, mkw::toggle_guarded(m, p, Some(Msg::MicMute)))),
                    p,
                ))
            }
            "wlan" => {
                let on = self.wlan?;
                Some(mkw::row_item(
                    "WLAN",
                    None,
                    lead(mkw::symbol::WIFI),
                    Some(self.trailing_swap(i, fixed, mkw::toggle_guarded(on, p, Some(Msg::Wlan(!on))))),
                    p,
                ))
            }
            "bluetooth" => {
                let on = self.bluetooth?;
                Some(mkw::row_item(
                    "Bluetooth",
                    None,
                    lead(mkw::symbol::BLUETOOTH),
                    Some(self.trailing_swap(i, fixed, mkw::toggle_guarded(on, p, Some(Msg::Bluetooth(!on))))),
                    p,
                ))
            }
            "brightness" => {
                let (h, _) = self.brightness?;
                Some(mkw::row_item(
                    "Helligkeit",
                    None,
                    lead(mkw::symbol::BRIGHTNESS),
                    Some(self.trailing_swap(i, fixed, 
                        mkw::bar_slider(0.0..=100.0, h, 1.0, p, Msg::Brightness)
                            .on_release(Msg::SetBrightness)
                            .width(Length::Fixed(150.0))
                            .into(),
                    )),
                    p,
                ))
            }
            "motion" => Some(mkw::row_item(
                "Bewegung reduzieren",
                Some("Federn und Übergänge stillhalten"),
                lead(mkw::symbol::ANIMATION),
                Some(self.trailing_swap(i, fixed, mkw::toggle_guarded(
                    self.motion_reduced,
                    p,
                    Some(Msg::Motion(!self.motion_reduced)),
                ))),
                p,
            )),
            "transparency" => Some(mkw::row_item(
                "Transparenz reduzieren",
                Some("Leisten und Schleier deckend"),
                lead(mkw::symbol::OPACITY),
                Some(self.trailing_swap(i, fixed, mkw::toggle_guarded(
                    self.transparency_reduced,
                    p,
                    Some(Msg::Transparenz(!self.transparency_reduced)),
                ))),
                p,
            )),
            "contrast" => Some(mkw::row_item(
                "Erhöhter Kontrast",
                Some("Kräftigere Text- und Konturfarben"),
                lead(mkw::symbol::CONTRAST),
                Some(self.trailing_swap(i, fixed, mkw::toggle_guarded(
                    self.contrast_up,
                    p,
                    Some(Msg::Contrast(!self.contrast_up)),
                ))),
                p,
            )),
            "rotation-lock" => Some(mkw::row_item(
                "Drehung sperren",
                None,
                lead(mkw::symbol::RESTART),
                Some(self.trailing_swap(
                    i,
                    fixed,
                    mkw::toggle_guarded(self.rotation_lock, p, Some(Msg::RotationLock(!self.rotation_lock))),
                )),
                p,
            )),
            "appearance" => Some(mkw::row_item(
                "Helles Erscheinungsbild",
                None,
                lead(mkw::symbol::DARK_MODE),
                Some(self.trailing_swap(i, fixed, mkw::toggle_guarded(p.is_light, p, Some(Msg::Appearance)))),
                p,
            )),
            "capture" => Some(self.launch_row(
                i,
                mkw::symbol::IMAGE,
                "Aufnahme",
                "Bildschirmfoto & -film",
                "matrix-capture",
            )),
            "lock" => Some(self.launch_row(
                i,
                mkw::symbol::LOCK,
                "Sperren",
                "Bildschirm sofort sperren",
                "matrix-lock",
            )),
            _ => None,
        }
    }

    /// Live control ↔ reorder handle (fixed modules: ↔ nothing).
    fn trailing_swap<'b>(
        &'b self,
        i: usize,
        fixed: bool,
        ctl: Element<'b, Msg>,
    ) -> Element<'b, Msg> {
        if fixed {
            mkw::morph(ctl, Space::new(), self.edit)
        } else {
            mkw::morph(ctl, self.drag_handle(i), self.edit)
        }
    }

    /// A row that launches an app and lets the panel dismiss — the
    /// "Vorschläge" grammar of the reference. While editing, the
    /// chevron morphs into the reorder handle and the press sleeps.
    fn launch_row<'a>(
        &'a self,
        i: usize,
        glyph: char,
        title: &'a str,
        sub: &'a str,
        cmd: &'static str,
    ) -> Element<'a, Msg> {
        let p = self.palette;
        let chevron: Element<'_, Msg> =
            mkw::symbol(mkw::symbol::CHEVRON_RIGHT, mk::icon_size::SMALL, p.on_surface_variant);
        // While editing the row must behave EXACTLY like every other
        // row: no hover glow, no press — only handle and badge live.
        let edit = self.edit;
        button(mkw::row_item(
            title,
            Some(sub),
            Some(mkw::symbol(glyph, mk::icon_size::SMALL + 4.0, p.on_surface_variant)),
            Some(mkw::morph(chevron, self.drag_handle(i), self.edit)),
            p,
        ))
        .padding(0)
        .width(Length::Fill)
        .style(move |_, status| {
            if edit {
                iced::widget::button::Style::default()
            } else {
                mkw::panel::button_style(p, status, mk::radius::NORMAL)
            }
        })
        .on_press_maybe((!self.edit).then_some(Msg::Run(cmd)))
        .into()
    }

    /// The reorder handle — reference `line.3.horizontal` at the
    /// trailing edge; pressing lifts the row, entering a neighbor
    /// displaces the list live, releasing anywhere settles and saves.
    fn drag_handle(&self, i: usize) -> Element<'_, Msg> {
        let p = self.palette;
        mouse_area(
            container(mkw::symbol::<Msg>(
                mkw::symbol::REORDER,
                mk::icon_size::SMALL,
                p.on_surface_variant,
            ))
            .padding(iced::Padding {
                top: mk::spacing::S,
                right: mk::spacing::XS,
                bottom: mk::spacing::S,
                left: mk::spacing::XS,
            }),
        )
        .interaction(if self.drag.is_some() {
            iced::mouse::Interaction::Grabbing
        } else {
            iced::mouse::Interaction::Grab
        })
        .on_press(Msg::DragStart(i))
        .into()
    }

    /// One gallery row: glyph circle, wording, ADD — gray without backend.
    fn gallery_row(&self, c: &'static mkw::central::Control) -> Element<'_, Msg> {
        let p = self.palette;
        let ok = self.available(c.key);
        let front = if ok { p.on_surface } else { p.on_surface_variant };
        let sub: &str = if ok { c.subtitle } else { "nicht verfügbar" };
        let content = row![
            mkw::ui::menu_glyph_circle::<Msg>(c.glyph, front, p),
            column![
                mkw::txt(c.title, mk::typo::BODY, front),
                mkw::txt(sub, mk::typo::CAPTION, p.on_surface_variant),
            ]
            .spacing(1),
            Space::new().width(Length::Fill),
            mkw::symbol::<Msg>(
                mkw::symbol::ADD,
                18.0,
                if ok { p.primary } else { p.on_surface_variant },
            ),
        ]
        .spacing(mk::spacing::S)
        .align_y(iced::Alignment::Center);
        let mut b = button(content)
            .padding(mk::spacing::S)
            .width(Length::Fill)
            .style(move |_, status| mkw::panel::button_style(p, status, mk::radius::NORMAL));
        if ok {
            b = b.on_press(Msg::CtlAdd(c.key.to_string()));
        }
        b.into()
    }

    /// The add gallery: search field on top, then the categories with
    /// every control that is not yet placed.
    fn gallery(&self) -> Element<'_, Msg> {
        let p = self.palette;
        let need = self.search.to_lowercase();
        let mut body = column![container(mkw::search_field(
            &self.search,
            "Steuerelemente durchsuchen",
            Msg::Search,
            Msg::Search(String::new()),
            p,
        ))
        .padding(iced::Padding {
            left: mk::spacing::M,
            right: mk::spacing::M,
            top: mk::spacing::XS,
            bottom: mk::spacing::XS,
        })]
        .spacing(mk::spacing::XS);

        let mut hits = 0usize;
        for cat in mkw::central::Category::ALL {
            let free: Vec<&'static mkw::central::Control> = mkw::central::REGISTRY
                .iter()
                .filter(|c| c.category == cat && !c.fixed)
                .filter(|c| !self.controls.iter().any(|k| k == c.key))
                .filter(|c| {
                    need.is_empty()
                        || c.title.to_lowercase().contains(&need)
                        || c.subtitle.to_lowercase().contains(&need)
                })
                .collect();
            if free.is_empty() {
                continue;
            }
            body = body.push(
                container(mkw::txt(cat.title(), mk::typo::LABEL, p.on_surface_variant))
                    .padding(iced::Padding {
                        left: mk::spacing::M,
                        top: mk::spacing::XS,
                        ..iced::Padding::ZERO
                    }),
            );
            for c in free {
                hits += 1;
                body = body.push(self.gallery_row(c));
            }
        }
        if hits == 0 {
            body = body.push(
                container(mkw::txt(
                    if need.is_empty() {
                        "Alle Steuerelemente sind in der Zentrale."
                    } else {
                        "Nichts gefunden."
                    },
                    mk::typo::CAPTION,
                    p.on_surface_variant,
                ))
                .padding(mk::spacing::M),
            );
        }
        // Natural height while everything fits — there is nothing to
        // scroll then. The scroll region only takes over at the cap.
        if self.gallery_raw() <= self.gallery_box() + 0.5 {
            container(body).into()
        } else {
            container(mkw::scroll_region(body.into(), p))
                .height(Length::Fixed(self.gallery_box()))
                .into()
        }
    }

    fn view(&self, window: iced::window::Id) -> Element<'_, Msg> {
        // The veil: invisible dismiss layer — a click closes, a drag
        // release settles.
        let p = self.palette;
        let pane = self.fleet.pane(window).cloned();
        let Some(pane) = pane else {
            // The veil (or a window on its way out): a click dismisses,
            // a drag release settles.
            return mouse_area(
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .on_press(Msg::Close)
            .on_release(Msg::DragEnd)
            .into();
        };
        match pane {
            // One island window = one category run — rows straight on
            // the compositor glass, hairlines between them, no paint
            // of our own (reference: the TILE is the glass).
            Pane::Group(rows) => {
                let mut col = column![].spacing(0);
                for (n, i) in rows.iter().enumerate() {
                    let i = *i;
                    if n > 0 {
                        col = col.push(
                            container(
                                Space::new().width(Length::Fill).height(Length::Fixed(1.0)),
                            )
                            .padding(iced::Padding {
                                left: mk::spacing::M,
                                ..iced::Padding::ZERO
                            })
                            .style(move |_| container::Style {
                                background: Some(
                                    mkw::color(mk::Rgba {
                                        a: 0.25,
                                        ..p.outline
                                    })
                                    .into(),
                                ),
                                ..Default::default()
                            }),
                        );
                    }
                    let key = &self.controls[i];
                    let c = mkw::central::find(key);
                    let row_plain: Element<'_, Msg> = match self.control_row(i, key) {
                        Some(r) => r,
                        None => mkw::row_item(
                            c.map(|c| c.title).unwrap_or(""),
                            Some("nicht verfügbar"),
                            None,
                            Some(self.drag_handle(i)),
                            p,
                        ),
                    };
                    let fixed = c.is_some_and(|c| c.fixed);
                    let row_el: Element<'_, Msg> = if fixed {
                        row_plain
                    } else {
                        let lifted = self.drag == Some(i);
                        let body =
                            container(row_plain).width(Length::Fill).style(move |_| {
                                container::Style {
                                    background: lifted.then(|| {
                                        mkw::color(
                                            p.primary.over(p.surface_container_high, 0.16),
                                        )
                                        .into()
                                    }),
                                    border: iced::Border {
                                        radius: mk::radius::NORMAL.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            });
                        // The badge floats in the transparent gutter,
                        // OUTSIDE the glass body (user wish: draussen
                        // schwebend) — the window is wide enough now.
                        let badge = mkw::floating(
                            mkw::morph(
                                Space::new(),
                                mkw::ui::minus_badge(Msg::CtlRemove(i), p),
                                self.edit,
                            ),
                            -(22.0 + mk::spacing::S),
                            -11.0,
                        );
                        row![badge, mouse_area(body).on_enter(Msg::DragOver(i))]
                            .align_y(iced::Alignment::Center)
                            .into()
                    };
                    col = col.push(
                        container(row_el)
                            .height(Length::Fixed(self.row_height(i) as f32))
                            .center_y(Length::Fixed(self.row_height(i) as f32)),
                    );
                }
                mouse_area(
                    container(col).padding(iced::Padding {
                        left: GUTTER as f32,
                        right: GUTTER as f32,
                        ..iced::Padding::ZERO
                    }),
                )
                .on_release(Msg::DragEnd)
                .into()
            }
            // The footer island: the WINDOW is the glass pill — the
            // button paints nothing and fills it (user finding: a
            // pill inside a glass strip reads as clutter).
            Pane::Footer => button(
                container(mkw::txt(
                    if self.edit { "Fertig" } else { "Steuerelemente bearbeiten" },
                    mk::typo::CAPTION,
                    p.on_surface,
                ))
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            )
            .padding(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_, status| {
                mkw::panel::button_style(p, status, mk::radius::capsule(FOOTER_H as f32))
            })
            .on_press(Msg::Edit(!self.edit))
            .into(),
            // The add gallery: search + entries straight on glass.
            Pane::Gallery => mouse_area(self.gallery())
                .on_release(Msg::DragEnd)
                .into(),
        }
    }
}
