//! Leiste & Dock — as a panel of Matrix Einstellungen (fusion R41).
//! Was app #24 (matrix-anpassen): edit bar zones, dock pins and the
//! second dock row, with wiggle mode and widget library.

use matrixkit_widgets::iced;
use matrixkit_widgets::iced::widget::{button, column, container, row, Space};
use matrixkit_widgets::iced::{Alignment, Element, Length, Subscription, Task};
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use mkw::color;
use std::time::Duration;

/// R174h: the two dock faces, in the order the contract knows them.
const DOCK_STYLES: [&str; 2] = ["Kacheln", "Symbole"];

/// Dock/catalog symbol sizes: bound to the tile geometry.
const DOCK_SYMBOL: f32 = 26.0;
const KATALOG_SYMBOL: f32 = 22.0;

// ----------------------------------------------------------------- Zonen

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    Left,
    Center,
    Right,
}

impl Zone {
    const ALL: [Zone; 3] = [Zone::Left, Zone::Center, Zone::Right];
    fn title(self) -> &'static str {
        match self {
            Zone::Left => "Links",
            Zone::Center => "Mitte",
            Zone::Right => "Rechts",
        }
    }
}

// ---------------------------------------------------------------- Widgets

/// The ten bar widgets — identical to the bar's `enum Widget` so the
/// library offers exactly what the bar knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarWidget {
    Matrix,
    Focus,
    Clock,
    Pulse,
    Bell,
    Tray,
    Battery,
    Central,
    User,
    Capture,
}

impl BarWidget {
    const ALL: [BarWidget; 10] = [
        BarWidget::Matrix,
        BarWidget::Focus,
        BarWidget::Clock,
        BarWidget::Pulse,
        BarWidget::Bell,
        BarWidget::Tray,
        BarWidget::Battery,
        BarWidget::Central,
        BarWidget::User,
        BarWidget::Capture,
    ];

    fn key(self) -> &'static str {
        match self {
            BarWidget::Matrix => "matrix",
            BarWidget::Focus => "focus",
            BarWidget::Clock => "clock",
            BarWidget::Pulse => "pulse",
            BarWidget::Bell => "bell",
            BarWidget::Tray => "tray",
            BarWidget::Battery => "battery",
            BarWidget::Central => "central",
            BarWidget::User => "user",
            BarWidget::Capture => "capture",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        // R103 migration: German legacy values map onto the English keys.
        let name = match name {
            "fokus" => "focus",
            "uhr" => "clock",
            "puls" => "pulse",
            "glocke" => "bell",
            "akku" => "battery",
            "zentrale" => "central",
            "nutzer" => "user",
            "aufnahme" => "capture",
            "zwischenablage" => "clipboard",
            other => other,
        };
        BarWidget::ALL.into_iter().find(|w| w.key() == name)
    }

    fn title(self) -> &'static str {
        match self {
            BarWidget::Matrix => "Matrix-Menü",
            BarWidget::Focus => "Fokus",
            BarWidget::Clock => "Uhr",
            BarWidget::Pulse => "Puls",
            BarWidget::Bell => "Glocke",
            BarWidget::Tray => "Ablage",
            BarWidget::Battery => "Akku",
            BarWidget::Central => "Zentrale",
            BarWidget::User => "Nutzer",
            BarWidget::Capture => "Aufnahme",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            BarWidget::Matrix => "Das Systemmenü ganz links",
            BarWidget::Focus => "Titel der aktiven App",
            BarWidget::Clock => "Uhrzeit und Datum",
            BarWidget::Pulse => "CPU, RAM, Ton, Netz",
            BarWidget::Bell => "Mitteilungen & Nicht stören",
            BarWidget::Tray => "System-Tray-Symbole",
            BarWidget::Battery => "Ladestand",
            BarWidget::Central => "Kontrollzentrum",
            BarWidget::User => "Sitzungsmenü",
            BarWidget::Capture => "Bildschirmfoto & -film",
        }
    }

    fn glyph(self) -> char {
        match self {
            BarWidget::Matrix => mkw::symbol::APPS,
            BarWidget::Focus => mkw::symbol::TOUCH_APP,
            BarWidget::Clock => mkw::symbol::SCHEDULE,
            BarWidget::Pulse => mkw::symbol::MONITORING,
            BarWidget::Bell => mkw::symbol::NOTIFICATIONS,
            BarWidget::Tray => mkw::symbol::WIDGETS,
            BarWidget::Battery => mkw::symbol::BATTERY_FULL,
            BarWidget::Central => mkw::symbol::TUNE,
            BarWidget::User => mkw::symbol::PERSON,
            BarWidget::Capture => mkw::symbol::IMAGE,
        }
    }
}

/// The library for the dock: the pinnable Matrix apps (app_id, title,
/// glyph). Order = display in the gallery.
const DOCK_KATALOG: &[(&str, &str, char)] = &[
    ("matrix-settings", "Einstellungen", mkw::symbol::TUNE),
    ("matrix-colors", "Farben", mkw::symbol::PALETTE),
    ("matrix-start", "Start", mkw::symbol::SEARCH),
    ("matrix-files", "Dateien", mkw::symbol::FOLDER),
    ("matrix-web", "Web", mkw::symbol::PUBLIC),
    ("matrix-codes", "Codes", mkw::symbol::KEY),
    ("matrix-keys", "Schlüssel", mkw::symbol::USB),
    ("matrix-updater", "Updater", mkw::symbol::RESTART),
    ("matrix-notifications", "Mitteilungen", mkw::symbol::NOTIFICATIONS),
    ("matrix-sounds", "Klänge", mkw::symbol::VOLUME_UP),
    ("matrix-sysmon", "Systemmonitor", mkw::symbol::MONITORING),
    ("matrix-icons-app", "Icon Composer", mkw::symbol::APPS),
    ("matrix-capture", "Aufnahme", mkw::symbol::IMAGE),
    ("matrix-help", "Hilfe", mkw::symbol::HELP),
];

fn dock_title(app_id: &str) -> &str {
    DOCK_KATALOG
        .iter()
        .find(|(id, _, _)| *id == app_id)
        .map(|(_, t, _)| *t)
        .unwrap_or(app_id)
}

fn dock_glyph(app_id: &str) -> char {
    DOCK_KATALOG
        .iter()
        .find(|(id, _, _)| *id == app_id)
        .map(|(_, _, g)| *g)
        .unwrap_or(mkw::symbol::APPS)
}

// ---- reine Serialisierung (testbar) --------------------------------------

const BAR_STANDARD: &str = "matrix fokus | uhr | akku puls tray glocke zentrale nutzer";
const DOCK_STANDARD: &str = "matrix-settings";
const ROW2_DEFAULT: &str = "apps";

/// The widgets of the second dock row: (key, title, glyph, subtitle).
const ROW2_CATALOG: &[(&str, &str, char, &str)] = &[
    ("apps", "Apps", mkw::symbol::APPS, "Der App-Launcher-Knopf"),
    ("central", "Zentrale", mkw::symbol::TUNE, "Kontrollzentrum"),
    ("clock", "Uhr", mkw::symbol::SCHEDULE, "Uhrzeit"),
    ("clipboard", "Zwischenablage", mkw::symbol::CONTENT_COPY, "Kopien-Verlauf"),
    ("capture", "Aufnahme", mkw::symbol::IMAGE, "Bildschirmfoto & -film"),
    // R173: the displays that used to live only in the top bar. The
    // user menu leads the group — it is the way out of the session.
    ("user", "Nutzer", mkw::symbol::PERSON, "Sperren, abmelden, neu starten"),
    ("matrix", "Matrix-Menü", mkw::symbol::WIDGETS, "Über, Einstellungen, Update"),
    ("battery", "Batterie", mkw::symbol::BATTERY_FULL, "Ladestand"),
    ("wlan", "WLAN", mkw::symbol::WIFI, "Verbundenes Netz"),
    ("bluetooth", "Bluetooth", mkw::symbol::BLUETOOTH, "Funk an oder aus"),
];

/// R103 migration: German legacy dock values map onto English keys.
fn line2_modern(key: &str) -> &str {
    match key {
        "zentrale" => "central",
        "uhr" => "clock",
        "zwischenablage" => "clipboard",
        "aufnahme" => "capture",
        // R173: the widgets that came down from the bar keep the bar's
        // German legacy spellings too — one vocabulary for both rows.
        "nutzer" => "user",
        "akku" => "battery",
        "wifi" => "wlan",
        other => other,
    }
}

fn line2_entry(key: &str) -> Option<&'static (&'static str, &'static str, char, &'static str)> {
    let key = line2_modern(key);
    ROW2_CATALOG.iter().find(|(id, _, _, _)| *id == key)
}

fn zones_parse(conf: &str) -> (Vec<BarWidget>, Vec<BarWidget>, Vec<BarWidget>) {
    let mut zones = conf.splitn(3, '|').map(|z| {
        z.split_whitespace()
            .filter_map(BarWidget::from_name)
            .collect::<Vec<_>>()
    });
    (
        zones.next().unwrap_or_default(),
        zones.next().unwrap_or_default(),
        zones.next().unwrap_or_default(),
    )
}

fn zones_string(l: &[BarWidget], m: &[BarWidget], r: &[BarWidget]) -> String {
    let part = |z: &[BarWidget]| {
        z.iter()
            .map(|w| w.key())
            .collect::<Vec<_>>()
            .join(" ")
    };
    format!("{} | {} | {}", part(l), part(m), part(r))
}

/// The wiggle of the customize tiles is OFF (owner, 01.11.2026): it ran
/// a 33 ms tick around the clock — whichever settings page was open — and
/// kept the duo composing (settings ~30 % CPU, niri ~25 %). The tiles stay
/// still; `offset` and the tick remain for a later wiggle that runs only
/// while this page is in view.
fn wobble_on() -> bool {
    false
}

// ------------------------------------------------------------------- App

pub struct Panel {
    pub palette: mk::Palette,
    left: Vec<BarWidget>,
    center: Vec<BarWidget>,
    right: Vec<BarWidget>,
    pins: Vec<String>,
    /// Second dock row (dock-widgets), only known keys.
    line2: Vec<String>,
    target: Zone,
    phase: f32,
    wobble: bool,
}

#[derive(Debug, Clone)]
pub enum Msg {
    JiggleTick,
    TargetChoice(Zone),
    BarAdd(BarWidget),
    BarRemove(Zone, usize),
    BarLeft(Zone, usize),
    BarRight(Zone, usize),
    DockAdd(String),
    DockRemove(usize),
    DockShiftOver(usize, i32),
    Row2Add(String),
    Row2Remove(usize),
    Row2Move(usize, i32),
    Standard,
    /// R168c: auto-hide toggles — the strip-and-swipe grammar.
    BarAutoHide(bool),
    DockAutoHide(bool),
    /// R174h: which face the dock wears — tiles or icons.
    DockStyle(usize),
}

impl Panel {
    pub fn new() -> Self {
        let (left, center, right) =
            zones_parse(&mk::settings::read("bar-widgets").unwrap_or_else(|| BAR_STANDARD.into()));
        let pins = mk::settings::read("dock-pins")
            .unwrap_or_else(|| DOCK_STANDARD.into())
            .split_whitespace()
            .map(String::from)
            .collect();
        let line2 = mk::settings::read("dock-widgets")
            .unwrap_or_else(|| ROW2_DEFAULT.into())
            .split_whitespace()
            .filter(|w| line2_entry(w).is_some())
            .map(|w| line2_modern(w).to_string())
            .collect();
        Self {
            palette: mk::Palette::load().unwrap_or_default(),
            left,
            center,
            right,
            pins,
            line2,
            target: Zone::Right,
            phase: 0.0,
            wobble: wobble_on(),
        }
    }

    /// From the host per tick: palette follows, wiggle culture read freshly.
    pub fn tick(&mut self, p: mk::Palette) {
        self.palette = p;
        self.wobble = wobble_on();
    }

    fn zone_mut(&mut self, z: Zone) -> &mut Vec<BarWidget> {
        match z {
            Zone::Left => &mut self.left,
            Zone::Center => &mut self.center,
            Zone::Right => &mut self.right,
        }
    }

    fn zone_ref(&self, z: Zone) -> &[BarWidget] {
        match z {
            Zone::Left => &self.left,
            Zone::Center => &self.center,
            Zone::Right => &self.right,
        }
    }

    fn placed(&self, w: BarWidget) -> bool {
        self.left.contains(&w) || self.center.contains(&w) || self.right.contains(&w)
    }

    /// One tile to the left: swap within the zone; at the left edge
    /// wander to the end of the previous zone (the three zones as one band).
    fn bar_left(&mut self, z: Zone, i: usize) {
        if i > 0 {
            self.zone_mut(z).swap(i - 1, i);
            return;
        }
        match z {
            Zone::Left => {}
            Zone::Center => {
                if !self.center.is_empty() {
                    let w = self.center.remove(0);
                    self.left.push(w);
                }
            }
            Zone::Right => {
                if !self.right.is_empty() {
                    let w = self.right.remove(0);
                    self.center.push(w);
                }
            }
        }
    }

    /// One tile to the right: swap within the zone; at the right edge
    /// wander to the start of the next zone.
    fn bar_right(&mut self, z: Zone, i: usize) {
        let len = self.zone_ref(z).len();
        if i + 1 < len {
            self.zone_mut(z).swap(i, i + 1);
            return;
        }
        match z {
            Zone::Right => {}
            Zone::Left => {
                if i < self.left.len() {
                    let w = self.left.remove(i);
                    self.center.insert(0, w);
                }
            }
            Zone::Center => {
                if i < self.center.len() {
                    let w = self.center.remove(i);
                    self.right.insert(0, w);
                }
            }
        }
    }

    fn save(&self) {
        mk::settings::write(
            "bar-widgets",
            &zones_string(&self.left, &self.center, &self.right),
        );
        mk::settings::write("dock-pins", &self.pins.join(" "));
        mk::settings::write("dock-widgets", &self.line2.join(" "));
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::BarAutoHide(on) => {
                mk::settings::write("bar-autohide", if on { "on" } else { "off" });
            }
            Msg::DockAutoHide(on) => {
                mk::settings::write("dock-autohide", if on { "on" } else { "off" });
            }
            Msg::DockStyle(i) => {
                // The dock watches this contract and changes its face
                // on the spot — no logout, no restart by hand.
                mk::settings::write("dock-style", if i == 0 { "tiles" } else { "icons" });
                // R174j (user): the tile shelf REPLACES the top bar —
                // its widgets (session menu included) moved down in
                // R173, so nothing is lost. Back on icons, the bar
                // returns at once; the watchdog also knows this rule.
                if i == 0 {
                    let _ = std::process::Command::new("pkill")
                        .args(["-x", "matrix-bar"])
                        .status();
                } else {
                    let _ = std::process::Command::new("matrix-bar")
                        .arg("--reservieren")
                        .spawn();
                }
            }
            Msg::JiggleTick => {
                // ~30 fps suffices for wiggling — half the redraw load so the
                // animation does not heat the (thermally sensitive) Surface.
                // Same angle increment per second as before.
                self.phase += 0.32;
                if self.phase > std::f32::consts::TAU * 32.0 {
                    self.phase -= std::f32::consts::TAU * 32.0;
                }
            }
            Msg::TargetChoice(z) => self.target = z,
            Msg::BarAdd(w) => {
                if !self.placed(w) {
                    let z = self.target;
                    self.zone_mut(z).push(w);
                    self.save();
                }
            }
            Msg::BarRemove(z, i) => {
                if i < self.zone_ref(z).len() {
                    self.zone_mut(z).remove(i);
                    self.save();
                }
            }
            Msg::BarLeft(z, i) => {
                self.bar_left(z, i);
                self.save();
            }
            Msg::BarRight(z, i) => {
                self.bar_right(z, i);
                self.save();
            }
            Msg::DockAdd(id) => {
                if !self.pins.contains(&id) {
                    self.pins.push(id);
                    self.save();
                }
            }
            Msg::DockRemove(i) => {
                if i < self.pins.len() {
                    self.pins.remove(i);
                    self.save();
                }
            }
            Msg::DockShiftOver(i, d) => {
                let j = i as i32 + d;
                if j >= 0 && (j as usize) < self.pins.len() {
                    self.pins.swap(i, j as usize);
                    self.save();
                }
            }
            Msg::Row2Add(id) => {
                if !self.line2.contains(&id) {
                    self.line2.push(id);
                    self.save();
                }
            }
            Msg::Row2Remove(i) => {
                if i < self.line2.len() {
                    self.line2.remove(i);
                    self.save();
                }
            }
            Msg::Row2Move(i, d) => {
                let j = i as i32 + d;
                if j >= 0 && (j as usize) < self.line2.len() {
                    self.line2.swap(i, j as usize);
                    self.save();
                }
            }
            Msg::Standard => {
                let (l, m, r) = zones_parse(BAR_STANDARD);
                self.left = l;
                self.center = m;
                self.right = r;
                self.pins = DOCK_STANDARD.split_whitespace().map(String::from).collect();
                self.line2 = ROW2_DEFAULT.split_whitespace().map(String::from).collect();
                self.save();
            }
        }
        Task::none()
    }

    /// Only the wiggle tick — everything else is subscribed by the host.
    pub fn subscription(&self) -> Subscription<Msg> {
        if self.wobble {
            mkw::tick("customize-jiggle", Duration::from_millis(33)).map(|_| Msg::JiggleTick)
        } else {
            Subscription::none()
        }
    }

    // ---- building blocks of the view -------------------------------------

    /// The wiggle offset of a tile (organically staggered via the start angle).
    fn offset(&self, seed: f32) -> (f32, f32) {
        if !self.wobble {
            return (0.0, 0.0);
        }
        let a = 1.8;
        (
            a * (self.phase + seed).sin(),
            a * (self.phase * 1.17 + seed * 1.7).cos(),
        )
    }

    /// A tiny control button below a tile (‹ − ›).
    fn mini(&self, glyph: char, tinting: bool, on: Msg) -> Element<'_, Msg> {
        let p = self.palette;
        let color = if tinting { p.error } else { p.on_surface_variant };
        button(
            container(mkw::symbol::<Msg>(glyph, mk::icon_size::SMALL, color))
                .center_x(Length::Fixed(22.0)),
        )
        .padding(iced::Padding {
            top: 2.0,
            bottom: 2.0,
            ..iced::Padding::ZERO
        })
        .style(move |_, status| mkw::panel::button_style(p, status, mk::radius::SMALL))
        .on_press(on)
        .into()
    }

    /// A placed, wiggling tile with its control row below.
    fn tile<'a>(
        &'a self,
        glyph: char,
        name: &'a str,
        seed: f32,
        control: Element<'a, Msg>,
    ) -> Element<'a, Msg> {
        let p = self.palette;
        let (dx, dy) = self.offset(seed);
        let icon = container(mkw::symbol::<Msg>(glyph, DOCK_SYMBOL, p.on_surface)).padding(iced::Padding {
            left: 2.0 + dx,
            right: 2.0 - dx,
            top: 2.0 + dy,
            bottom: 2.0 - dy,
        });
        container(
            column![
                icon,
                mkw::txt(name, mk::typo::CAPTION, p.on_surface_variant),
                control,
            ]
            .spacing(mk::spacing::XXS)
            .align_x(Alignment::Center),
        )
        .padding(mk::spacing::S)
        .width(Length::Fixed(96.0))
        .style(move |_| container::Style {
            background: Some(color(p.surface_container_high).into()),
            border: iced::Border {
                radius: mk::radius::NORMAL.into(),
                width: 1.0,
                color: color(p.outline.over(p.surface_container_high, 0.3)),
            },
            ..Default::default()
        })
        .into()
    }

    /// A library tile (for adding). Gray if already placed.
    fn gallery_tile<'a>(
        &self,
        glyph: char,
        name: &'a str,
        sub: &'a str,
        active: bool,
        on: Msg,
    ) -> Element<'a, Msg> {
        let p = self.palette;
        let front = if active { p.on_surface } else { p.on_surface_variant };
        let content = row![
            container(mkw::symbol::<Msg>(glyph, KATALOG_SYMBOL, front)).center_y(Length::Fixed(38.0)),
            column![
                mkw::txt(name, mk::typo::CAPTION, front),
                mkw::txt(sub, mk::typo::LABEL, p.on_surface_variant),
            ]
            .spacing(1),
            Space::new().width(Length::Fill),
            mkw::symbol::<Msg>(
                if active { mkw::symbol::ADD } else { mkw::symbol::CHECK },
                18.0,
                if active { p.primary } else { p.on_surface_variant },
            ),
        ]
        .spacing(mk::spacing::S)
        .align_y(Alignment::Center);
        let mut button = button(content)
            .padding(mk::spacing::S)
            .width(Length::Fixed(272.0))
            .style(move |_, status| mkw::panel::button_style(p, status, mk::radius::NORMAL));
        if active {
            button = button.on_press(on);
        }
        button.into()
    }

    /// A bar zone as a labeled tray with its wiggling tiles.
    fn zone_tray_row(&self, z: Zone) -> Element<'_, Msg> {
        let p = self.palette;
        let widgets = self.zone_ref(z);
        let mut row = row![].spacing(mk::spacing::S).align_y(Alignment::Center);
        if widgets.is_empty() {
            row = row.push(
                container(mkw::txt("leer", mk::typo::CAPTION, p.on_surface_variant))
                    .padding(mk::spacing::S),
            );
        }
        for (i, w) in widgets.iter().enumerate() {
            // ARROW_BACK points left, CHEVRON_RIGHT right; − removes.
            let control = row![
                self.mini(mkw::symbol::ARROW_BACK, false, Msg::BarLeft(z, i)),
                self.mini(mkw::symbol::REMOVE, true, Msg::BarRemove(z, i)),
                self.mini(mkw::symbol::CHEVRON_RIGHT, false, Msg::BarRight(z, i)),
            ]
            .spacing(2);
            let seed = i as f32 * 1.7 + z as u8 as f32;
            row = row.push(self.tile(w.glyph(), w.title(), seed, control.into()));
        }
        column![
            mkw::txt(z.title(), mk::typo::LABEL, p.on_surface_variant),
            container(row)
                .width(Length::Fill)
                .padding(mk::spacing::XS)
                .style(move |_| container::Style {
                    background: Some(color(p.surface_container.over(p.surface, 0.6)).into()),
                    border: iced::Border {
                        radius: mk::radius::NORMAL.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(mk::spacing::XXS)
        .into()
    }

    fn target_choice(&self) -> Element<'_, Msg> {
        let p = self.palette;
        let mut row = row![mkw::txt("Neues Widget nach:", mk::typo::CAPTION, p.on_surface_variant)]
            .spacing(mk::spacing::S)
            .align_y(Alignment::Center);
        for z in Zone::ALL {
            let selected = self.target == z;
            row = row.push(
                button(mkw::txt(
                    z.title(),
                    mk::typo::CAPTION,
                    if selected { p.on_primary } else { p.on_surface },
                ))
                .padding(iced::Padding {
                    left: mk::spacing::M,
                    right: mk::spacing::M,
                    top: mk::spacing::XS,
                    bottom: mk::spacing::XS,
                })
                .style(move |_, status| {
                    let mut s = mkw::panel::button_style(p, status, mk::radius::SMALL);
                    if selected {
                        s.background = Some(color(p.primary).into());
                    }
                    s
                })
                .on_press(Msg::TargetChoice(z)),
            );
        }
        row.into()
    }

    fn bar_editor(&self) -> Element<'_, Msg> {
        let p = self.palette;
        let mut column = column![
            mkw::txt(
                "Die drei Zonen der Top-Bar. Kacheln wackeln im Bearbeiten-Modus — ‹ › ordnen und schieben über Zonengrenzen, − entfernt.",
                mk::typo::CAPTION,
                p.on_surface_variant,
            ),
            self.zone_tray_row(Zone::Left),
            self.zone_tray_row(Zone::Center),
            self.zone_tray_row(Zone::Right),
            self.target_choice(),
        ]
        .spacing(mk::spacing::M);

        let free: Vec<BarWidget> = BarWidget::ALL
            .into_iter()
            .filter(|w| !self.placed(*w))
            .collect();
        if free.is_empty() {
            column = column.push(mkw::txt(
                "Alle Widgets sind in der Leiste.",
                mk::typo::CAPTION,
                p.on_surface_variant,
            ));
        } else {
            column = column.push(mkw::txt("BIBLIOTHEK", mk::typo::LABEL, p.on_surface_variant));
            let mut gal = column![].spacing(mk::spacing::S);
            let mut line = row![].spacing(mk::spacing::S);
            for (n, w) in free.iter().enumerate() {
                line = line.push(self.gallery_tile(
                    w.glyph(),
                    w.title(),
                    w.subtitle(),
                    true,
                    Msg::BarAdd(*w),
                ));
                if (n + 1) % 2 == 0 {
                    gal = gal.push(line);
                    line = row![].spacing(mk::spacing::S);
                }
            }
            gal = gal.push(line);
            column = column.push(gal);
        }
        column.into()
    }

    fn dock_editor(&self) -> Element<'_, Msg> {
        let p = self.palette;
        let mut row = row![].spacing(mk::spacing::S).align_y(Alignment::Center);
        if self.pins.is_empty() {
            row = row.push(
                container(mkw::txt("keine Pins", mk::typo::CAPTION, p.on_surface_variant))
                    .padding(mk::spacing::S),
            );
        }
        for (i, id) in self.pins.iter().enumerate() {
            let control = row![
                self.mini(mkw::symbol::ARROW_BACK, false, Msg::DockShiftOver(i, -1)),
                self.mini(mkw::symbol::REMOVE, true, Msg::DockRemove(i)),
                self.mini(mkw::symbol::CHEVRON_RIGHT, false, Msg::DockShiftOver(i, 1)),
            ]
            .spacing(2);
            row = row.push(self.tile(
                dock_glyph(id),
                dock_title(id),
                i as f32 * 2.3 + 11.0,
                control.into(),
            ));
        }

        let free: Vec<&(&str, &str, char)> = DOCK_KATALOG
            .iter()
            .filter(|entry| !self.pins.iter().any(|q| q == entry.0))
            .collect();

        let mut column = column![
            mkw::txt(
                "Angepinnte Apps im Dock. − löst den Pin, ‹ › ordnet.",
                mk::typo::CAPTION,
                p.on_surface_variant,
            ),
            container(row)
                .width(Length::Fill)
                .padding(mk::spacing::XS)
                .style(move |_| container::Style {
                    background: Some(color(p.surface_container.over(p.surface, 0.6)).into()),
                    border: iced::Border {
                        radius: mk::radius::NORMAL.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(mk::spacing::M);

        if free.is_empty() {
            column = column.push(mkw::txt(
                "Alle Apps sind angepinnt.",
                mk::typo::CAPTION,
                p.on_surface_variant,
            ));
        } else {
            column = column.push(mkw::txt("BIBLIOTHEK", mk::typo::LABEL, p.on_surface_variant));
            let mut gal = column![].spacing(mk::spacing::S);
            let mut line = row![].spacing(mk::spacing::S);
            for (n, (id, title, glyph)) in free.iter().enumerate() {
                line = line.push(self.gallery_tile(
                    *glyph,
                    title,
                    id,
                    true,
                    Msg::DockAdd((*id).to_string()),
                ));
                if (n + 1) % 2 == 0 {
                    gal = gal.push(line);
                    line = row![].spacing(mk::spacing::S);
                }
            }
            gal = gal.push(line);
            column = column.push(gal);
        }
        column.into()
    }

    /// Editor of the second dock row (dock-widgets): apps button, Zentrale,
    /// clock, clipboard — the same grammar as the bar zones.
    fn line2_editor(&self) -> Element<'_, Msg> {
        let p = self.palette;
        let mut row = row![].spacing(mk::spacing::S).align_y(Alignment::Center);
        if self.line2.is_empty() {
            row = row.push(
                container(mkw::txt("leer", mk::typo::CAPTION, p.on_surface_variant))
                    .padding(mk::spacing::S),
            );
        }
        for (i, id) in self.line2.iter().enumerate() {
            let Some((_, title, glyph, _)) = line2_entry(id) else { continue };
            let control = row![
                self.mini(mkw::symbol::ARROW_BACK, false, Msg::Row2Move(i, -1)),
                self.mini(mkw::symbol::REMOVE, true, Msg::Row2Remove(i)),
                self.mini(mkw::symbol::CHEVRON_RIGHT, false, Msg::Row2Move(i, 1)),
            ]
            .spacing(2);
            row = row.push(self.tile(*glyph, title, i as f32 * 2.9 + 23.0, control.into()));
        }

        let free: Vec<&(&str, &str, char, &str)> = ROW2_CATALOG
            .iter()
            .filter(|(id, _, _, _)| !self.line2.iter().any(|w| w == id))
            .collect();

        let mut column = column![
            mkw::txt(
                "Die Widget-Zeile unter den App-Icons — z. B. der Apps-Knopf oder die Zwischenablage.",
                mk::typo::CAPTION,
                p.on_surface_variant,
            ),
            container(row)
                .width(Length::Fill)
                .padding(mk::spacing::XS)
                .style(move |_| container::Style {
                    background: Some(color(p.surface_container.over(p.surface, 0.6)).into()),
                    border: iced::Border {
                        radius: mk::radius::NORMAL.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(mk::spacing::M);

        if free.is_empty() {
            column = column.push(mkw::txt(
                "Alle Widgets sind in der Zeile.",
                mk::typo::CAPTION,
                p.on_surface_variant,
            ));
        } else {
            column = column.push(mkw::txt("BIBLIOTHEK", mk::typo::LABEL, p.on_surface_variant));
            let mut gal = column![].spacing(mk::spacing::S);
            let mut line = row![].spacing(mk::spacing::S);
            for (n, (id, title, glyph, sub)) in free.iter().enumerate() {
                line = line.push(self.gallery_tile(
                    *glyph,
                    title,
                    sub,
                    true,
                    Msg::Row2Add((*id).to_string()),
                ));
                if (n + 1) % 2 == 0 {
                    gal = gal.push(line);
                    line = row![].spacing(mk::spacing::S);
                }
            }
            gal = gal.push(line);
            column = column.push(gal);
        }
        column.into()
    }

    /// The panel content — scroll/footer/card is supplied by the host.
    pub fn view(&self) -> Element<'_, Msg> {
        let p = self.palette;
        let bar_hides = mk::settings::read("bar-autohide").as_deref() == Some("on");
        let dock_hides = mk::settings::read("dock-autohide").as_deref() == Some("on");
        // R174h (user): both dock faces stay available — the tile shelf
        // with a glass chip per element, and the classic icon pill.
        let style_now = if mk::settings::read("dock-style").as_deref() == Some("icons") {
            1
        } else {
            0
        };
        column![
            mkw::section(
                "DAS DOCK",
                vec![
                    mkw::row_item(
                        "Aussehen",
                        Some("Kacheln: jede App als Glasfläche mit Name und Herkunft. Symbole: die klassische Leiste mit Programmzeichen."),
                        None,
                        Some(mkw::segments(&DOCK_STYLES[..], style_now, Msg::DockStyle, p)),
                        p,
                    ),
                ],
                p,
            ),
            mkw::section(
                "AUTOMATISCH AUSBLENDEN",
                vec![
                    mkw::row_toggle(
                        "Top-Bar automatisch ausblenden",
                        Some("Zum Einblenden: von oben nach unten wischen oder die Kante berühren"),
                        None,
                        bar_hides,
                        p,
                        Some(Msg::BarAutoHide(!bar_hides)),
                    ),
                    mkw::row_toggle(
                        "Dock automatisch ausblenden",
                        Some("Zum Einblenden: von unten nach oben wischen oder die Kante berühren"),
                        None,
                        dock_hides,
                        p,
                        Some(Msg::DockAutoHide(!dock_hides)),
                    ),
                ],
                p,
            ),
            mkw::section("TOP-BAR", vec![self.bar_editor()], p),
            mkw::section("DOCK", vec![self.dock_editor()], p),
            mkw::section("DOCK — ZWEITE ZEILE", vec![self.line2_editor()], p),
            container(mkw::button(
                "Standard wiederherstellen",
                mkw::buttons::Style::Tinted,
                mkw::buttons::Role::Normal,
                mkw::buttons::Size::Small,
                p,
                Some(Msg::Standard),
            ))
            .width(Length::Fill)
            .center_x(Length::Fill),
        ]
        .spacing(mk::spacing::L)
        .into()
    }
}

// ------------------------------------------------------------------ Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_serialize_roundtrip() {
        let (l, m, r) = zones_parse(BAR_STANDARD);
        assert_eq!(l, vec![BarWidget::Matrix, BarWidget::Focus]);
        assert_eq!(m, vec![BarWidget::Clock]);
        assert_eq!(r.len(), 6);
        // Rundreise: string -> parse -> string identisch (normalisiert).
        let s = zones_string(&l, &m, &r);
        let (l2, m2, r2) = zones_parse(&s);
        assert_eq!((l, m, r), (l2, m2, r2));
    }

    #[test]
    fn unknown_drops_silently() {
        let (l, _m, _r) = zones_parse("fokus quatsch | | ");
        assert_eq!(l, vec![BarWidget::Focus]);
    }

    #[test]
    fn push_across_zone_border() {
        let mut app = probe();
        app.left = vec![BarWidget::Focus, BarWidget::Clock];
        app.center = vec![];
        app.right = vec![];
        // last tile in left moves right -> to the start of center
        app.bar_right(Zone::Left, 1);
        assert_eq!(app.left, vec![BarWidget::Focus]);
        assert_eq!(app.center, vec![BarWidget::Clock]);
        // and back: first tile in center moves left -> to the end of left
        app.bar_left(Zone::Center, 0);
        assert_eq!(app.left, vec![BarWidget::Focus, BarWidget::Clock]);
        assert!(app.center.is_empty());
    }

    #[test]
    fn placed_detects_duplicates() {
        let mut app = probe();
        app.left = vec![BarWidget::Bell];
        assert!(app.placed(BarWidget::Bell));
        assert!(!app.placed(BarWidget::Battery));
    }

    fn probe() -> Panel {
        Panel {
            palette: mk::Palette::load().unwrap_or_default(),
            left: vec![],
            center: vec![],
            right: vec![],
            pins: vec![],
            line2: vec![],
            target: Zone::Right,
            phase: 0.0,
            wobble: false,
        }
    }
}
