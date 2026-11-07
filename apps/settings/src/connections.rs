//! Verbindungen — WLAN and Bluetooth in Matrix Einstellungen (R44).
//!
//! Reference grammar (System Settings → WLAN/Bluetooth): the master
//! switch on top, below it the active network, then "Andere Netzwerke"
//! with signal and lock; Bluetooth as a device list with a connect
//! button. Substrate: nmcli and bluetoothctl — both async via
//! Task::perform so the scan never stalls the surface.

use matrixkit_widgets::iced;
use matrixkit_widgets::iced::widget::{column, container, row, Space};
use matrixkit_widgets::iced::{Element, Length, Task};
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;

#[derive(Debug, Clone, Default)]
pub struct Network {
    pub ssid: String,
    pub signal: u8,
    /// A password protects it (WPA/WEP).
    pub secured: bool,
    /// NetworkManager knows a profile for it (it connects without asking).
    pub saved: bool,
    pub active: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Device {
    pub mac: String,
    pub name: String,
    pub connected: bool,
    /// BlueZ's kind («audio-headset», «audio-card» …).
    pub icon: String,
}

impl Device {
    /// Round 160: the kit's kind (mouse, keyboard, headphones …).
    pub fn kind(&self) -> mk::bluetooth::Kind {
        mk::bluetooth::kind_of(&self.icon, &self.name)
    }
}

impl From<mk::bluetooth::Device> for Device {
    fn from(d: mk::bluetooth::Device) -> Device {
        Device { mac: d.mac, name: d.name, connected: d.connected, icon: d.icon }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Status {
    pub wlan_on: bool,
    pub networks: Vec<Network>,
    pub bt_on: bool,
    pub devices: Vec<Device>,
    /// Found by the last scan, not paired yet (headphones to pair).
    pub nearby: Vec<Device>,
}

pub struct Panel {
    pub palette: mk::Palette,
    status: Status,
    loading: bool,
    /// SSID a password is currently asked for + input.
    password_for: Option<String>,
    password: String,
    message: Option<(bool, String)>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Reload,
    Loaded(Status),
    WlanSwitch(bool),
    BtSwitch(bool),
    Connect(String, bool),
    PasswordHint(String),
    PasswordSubmit,
    PasswordCancel,
    BtConnect(String, bool),
    /// Look for new devices (ten seconds), then pair one (pair, trust,
    /// connect — headphones need no PIN).
    BtScan,
    BtPair(String, String),
    Result(bool, String),
}

fn command(program: &str, args: &[&str]) -> Option<String> {
    mk::command::text_from(program, args)
}

/// One nmcli terse field: `\:` is part of the value, `:` separates.
fn terse_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut escape = false;
    for c in line.chars() {
        if escape {
            current.push(c);
            escape = false;
        } else if c == '\\' {
            escape = true;
        } else if c == ':' {
            fields.push(std::mem::take(&mut current));
        } else {
            current.push(c);
        }
    }
    fields.push(current);
    fields
}

pub(crate) fn status_gather() -> Status {
    status_gather_with(true)
}

/// Round 170: what is KNOWN, at once (~0.3 s on the duo): the WLAN list as
/// NetworkManager last saw it, without a fresh scan (that one took 6.7 s,
/// and the page stood there with everything «off» — the owner 05.11.2026).
pub(crate) fn status_quick() -> Status {
    status_gather_with(false)
}

fn status_gather_with(scan: bool) -> Status {
    let wlan_on = command("nmcli", &["radio", "wifi"])
        .map(|s| s.trim() == "enabled")
        .unwrap_or(false);
    let mut networks: Vec<Network> = Vec::new();
    // The saved WLAN profiles (03.11.2026: «saved» meant «secured» before —
    // every protected network said «gespeichert», a new hotspot too).
    let profiles: Vec<String> = command("nmcli", &["-t", "-f", "NAME,TYPE", "connection", "show"])
        .map(|out| out.lines().map(terse_fields).filter(|f| f.len() >= 2 && f[1].contains("wireless")).map(|f| f[0].clone()).collect())
        .unwrap_or_default();
    if wlan_on {
        if let Some(out) = command(
            "nmcli",
            &["-t", "-f", "IN-USE,SSID,SIGNAL,SECURITY", "dev", "wifi", "list", "--rescan", if scan { "auto" } else { "no" }],
        ) {
            for line in out.lines() {
                let f = terse_fields(line);
                if f.len() < 4 || f[1].is_empty() {
                    continue;
                }
                // The same SSID often radios on several bands — the strongest
                // wins, the active one anyway.
                let net = Network {
                    active: f[0] == "*",
                    ssid: f[1].clone(),
                    signal: f[2].parse().unwrap_or(0),
                    secured: !f[3].is_empty() && f[3] != "--",
                    saved: profiles.iter().any(|p| *p == f[1]),
                };
                if let Some(old) = networks.iter_mut().find(|n| n.ssid == net.ssid) {
                    if net.active || net.signal > old.signal {
                        let war_active = old.active;
                        *old = net;
                        old.active = old.active || war_active;
                    }
                } else {
                    networks.push(net);
                }
            }
        }
        networks.sort_by(|a, b| b.active.cmp(&a.active).then(b.signal.cmp(&a.signal)));
        networks.truncate(12);
    }

    // Bluetooth through the Kit family (`mk::bluetooth`).
    let bt_on = mk::bluetooth::powered();
    let (devices, nearby) = if bt_on {
        (mk::bluetooth::paired().into_iter().map(Device::from).collect(), mk::bluetooth::nearby(20).into_iter().map(Device::from).collect())
    } else {
        (Vec::new(), Vec::new())
    };

    Status { wlan_on, networks, bt_on, devices, nearby }
}

impl Panel {
    pub fn new() -> (Self, Task<Msg>) {
        (
            Self {
                palette: mk::Palette::load().unwrap_or_default(),
                status: Status::default(),
                loading: true,
                password_for: None,
                password: String::new(),
                message: None,
            },
            Task::perform(async { status_gather() }, Msg::Loaded),
        )
    }

    /// From the host per tick: palette follows. (No auto rescan — WLAN
    /// scans cost power; reload sits as a button in the view.)
    pub fn tick(&mut self, p: mk::Palette) {
        self.palette = p;
    }

    pub fn footer_text(&self) -> String {
        if self.loading {
            String::from("Netze werden gesucht \u{2026}")
        } else {
            String::from("WLAN über NetworkManager, Bluetooth über BlueZ — Passwörter tippst immer du selbst")
        }
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Reload => {
                self.loading = true;
                self.message = None;
                return Task::perform(async { status_gather() }, Msg::Loaded);
            }
            Msg::Loaded(s) => {
                self.status = s;
                self.loading = false;
            }
            Msg::WlanSwitch(on) => {
                self.loading = true;
                return Task::perform(
                    async move {
                        let _ = std::process::Command::new("nmcli")
                            .args(["radio", "wifi", if on { "on" } else { "off" }])
                            .status();
                        // The radio needs a moment until networks appear.
                        std::thread::sleep(std::time::Duration::from_millis(if on { 2500 } else { 300 }));
                        status_gather()
                    },
                    Msg::Loaded,
                );
            }
            Msg::BtSwitch(on) => {
                self.loading = true;
                return Task::perform(
                    async move {
                        mk::bluetooth::power(on);
                        status_gather()
                    },
                    Msg::Loaded,
                );
            }
            Msg::Connect(ssid, saved) => {
                // nmcli connects known profiles without a password; only if
                // that fails and the network is secured do we ask for
                // the key.
                self.message = None;
                self.loading = true;
                return Task::perform(
                    async move {
                        let ok = std::process::Command::new("nmcli")
                            .args(["dev", "wifi", "connect", &ssid])
                            .output()
                            .map(|o| o.status.success())
                            .unwrap_or(false);
                        (ok, ssid, saved)
                    },
                    |(ok, ssid, saved)| {
                        if ok {
                            Msg::Result(true, format!("Mit \u{201e}{ssid}\u{201c} verbunden"))
                        } else if saved {
                            Msg::PasswordHint(format!("\u{0}{ssid}"))
                        } else {
                            Msg::Result(false, format!("Verbindung mit \u{201e}{ssid}\u{201c} fehlgeschlagen"))
                        }
                    },
                );
            }
            Msg::PasswordHint(t) => {
                // NUL prefix = internal marker: open the password field for SSID.
                if let Some(ssid) = t.strip_prefix('\u{0}') {
                    self.password_for = Some(ssid.to_string());
                    self.password.clear();
                    self.loading = false;
                } else {
                    self.password = t;
                }
            }
            Msg::PasswordSubmit => {
                let (Some(ssid), pw) = (self.password_for.take(), std::mem::take(&mut self.password))
                else {
                    return Task::none();
                };
                if pw.is_empty() {
                    return Task::none();
                }
                self.loading = true;
                return Task::perform(
                    async move {
                        let ok = std::process::Command::new("nmcli")
                            .args(["dev", "wifi", "connect", &ssid, "password", &pw])
                            .output()
                            .map(|o| o.status.success())
                            .unwrap_or(false);
                        (ok, ssid)
                    },
                    |(ok, ssid)| {
                        Msg::Result(
                            ok,
                            if ok {
                                format!("Mit \u{201e}{ssid}\u{201c} verbunden")
                            } else {
                                format!("Passwort für \u{201e}{ssid}\u{201c} abgelehnt")
                            },
                        )
                    },
                );
            }
            Msg::PasswordCancel => {
                self.password_for = None;
                self.password.clear();
            }
            Msg::BtConnect(mac, connect) => {
                self.loading = true;
                return Task::perform(
                    async move {
                        mk::bluetooth::connect(&mac, connect);
                        status_gather()
                    },
                    Msg::Loaded,
                );
            }
            Msg::BtScan => {
                self.loading = true;
                self.message = None;
                return Task::perform(
                    async {
                        mk::bluetooth::scan(10);
                        status_gather()
                    },
                    Msg::Loaded,
                );
            }
            Msg::BtPair(mac, name) => {
                self.loading = true;
                self.message = None;
                return Task::perform(
                    async move {
                        let ok = mk::bluetooth::pair(&mac);
                        (ok, name)
                    },
                    |(ok, name)| {
                        Msg::Result(
                            ok,
                            if ok {
                                format!("Mit \u{201e}{name}\u{201c} gekoppelt und verbunden")
                            } else {
                                format!("Kopplung mit \u{201e}{name}\u{201c} fehlgeschlagen — ist das Gerät im Kopplungsmodus?")
                            },
                        )
                    },
                );
            }
            Msg::Result(ok, text) => {
                self.message = Some((ok, text));
                self.loading = true;
                return Task::perform(async { status_gather() }, Msg::Loaded);
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let p = self.palette;
        let mut column = column![].spacing(mk::spacing::L);

        if let Some((ok, text)) = &self.message {
            column = column.push(mkw::notice(*ok, text.clone(), p));
        }

        // --- WLAN ---
        let mut wlan_lines: Vec<Element<'_, Msg>> = vec![mkw::row_toggle(
            "WLAN",
            None,
            Some(mkw::symbol::<Msg>(mkw::symbol::WIFI, mk::icon_size::MEDIUM, p.primary)),
            self.status.wlan_on,
            p,
            Some(Msg::WlanSwitch(!self.status.wlan_on)),
        )];
        if self.status.wlan_on {
            for net in &self.status.networks {
                let strength = mkw::symbol::WIFI;
                let mut right = row![].spacing(mk::spacing::S).align_y(iced::Alignment::Center);
                if net.saved {
                    right = right.push(mkw::symbol::<Msg>(mkw::symbol::LOCK, mk::icon_size::XSMALL, p.on_surface_variant));
                }
                right = right.push(mkw::txt(
                    format!("{} %", net.signal),
                    mk::typo::CAPTION,
                    p.on_surface_variant,
                ));
                if net.active {
                    right = right.push(mkw::txt("Verbunden", mk::typo::CAPTION, p.primary));
                } else {
                    right = right.push(mkw::button(
                        "Verbinden",
                        mkw::buttons::Style::Tinted,
                        mkw::buttons::Role::Normal,
                        mkw::buttons::Size::Small,
                        p,
                        Some(Msg::Connect(net.ssid.clone(), net.saved)),
                    ));
                }
                wlan_lines.push(mkw::row_item(
                    &net.ssid,
                    None,
                    Some(mkw::symbol::<Msg>(strength, mk::icon_size::MEDIUM, if net.active { p.primary } else { p.on_surface_variant })),
                    Some(right.into()),
                    p,
                ));
                // Password row directly below the affected network.
                if self.password_for.as_deref() == Some(net.ssid.as_str()) {
                    wlan_lines.push(
                        container(
                            row![
                                container(mkw::input_field(
                                    "Passwort",
                                    &self.password,
                                    Msg::PasswordHint,
                                    Some(Msg::PasswordSubmit),
                                    true,
                                    p,
                                ))
                                .width(Length::Fill),
                                mkw::button(
                                    "Verbinden",
                                    mkw::buttons::Style::Prominent,
                                    mkw::buttons::Role::Normal,
                                    mkw::buttons::Size::Small,
                                    p,
                                    Some(Msg::PasswordSubmit),
                                ),
                                mkw::button(
                                    "Abbrechen",
                                    mkw::buttons::Style::Borderless,
                                    mkw::buttons::Role::Normal,
                                    mkw::buttons::Size::Small,
                                    p,
                                    Some(Msg::PasswordCancel),
                                ),
                            ]
                            .spacing(mk::spacing::S)
                            .align_y(iced::Alignment::Center),
                        )
                        .padding(iced::Padding {
                            left: mk::spacing::M,
                            right: mk::spacing::M,
                            top: 4.0,
                            bottom: mk::spacing::S,
                        })
                        .into(),
                    );
                }
            }
            if self.status.networks.is_empty() && !self.loading {
                wlan_lines.push(mkw::row_item("Keine Netze gefunden", None, None, None, p));
            }
        }
        column = column.push(mkw::section("WLAN", wlan_lines, p));

        // --- Bluetooth ---
        let mut bt_lines: Vec<Element<'_, Msg>> = vec![mkw::row_toggle(
            "Bluetooth",
            None,
            Some(mkw::symbol::<Msg>(mkw::symbol::BLUETOOTH, mk::icon_size::MEDIUM, p.primary)),
            self.status.bt_on,
            p,
            Some(Msg::BtSwitch(!self.status.bt_on)),
        )];
        if self.status.bt_on {
            for g in &self.status.devices {
                let (action, target) = if g.connected {
                    ("Trennen", false)
                } else {
                    ("Verbinden", true)
                };
                bt_lines.push(mkw::row_item(
                    &g.name,
                    Some(if g.connected { "Verbunden" } else { "Gekoppelt" }),
                    Some(mkw::symbol::<Msg>(
                        mkw::symbol::BLUETOOTH,
                        mk::icon_size::MEDIUM,
                        if g.connected { p.primary } else { p.on_surface_variant },
                    )),
                    Some(
                        mkw::button(
                            action,
                            mkw::buttons::Style::Tinted,
                            mkw::buttons::Role::Normal,
                            mkw::buttons::Size::Small,
                            p,
                            Some(Msg::BtConnect(g.mac.clone(), target)),
                        )
                        .into(),
                    ),
                    p,
                ));
            }
            if self.status.devices.is_empty() {
                bt_lines.push(mkw::row_item("Keine gekoppelten Geräte", None, None, None, p));
            }
        }
        column = column.push(mkw::section("BLUETOOTH", bt_lines, p));

        // --- New devices: scan, then pair ---
        if self.status.bt_on {
            let mut new_lines: Vec<Element<'_, Msg>> = vec![mkw::row_item(
                "Kopfhörer oder Lautsprecher in den Kopplungsmodus bringen",
                Some("dann suchen — gefundene Geräte erscheinen hier"),
                Some(mkw::symbol::<Msg>(mkw::symbol::HEADPHONES, mk::icon_size::MEDIUM, p.primary)),
                Some(
                    mkw::button(
                        if self.loading { "Sucht \u{2026}" } else { "Suchen" },
                        mkw::buttons::Style::Tinted,
                        mkw::buttons::Role::Normal,
                        mkw::buttons::Size::Small,
                        p,
                        (!self.loading).then_some(Msg::BtScan),
                    )
                    .into(),
                ),
                p,
            )];
            for g in &self.status.nearby {
                new_lines.push(mkw::row_item(
                    &g.name,
                    None,
                    Some(mkw::symbol::<Msg>(mkw::symbol::BLUETOOTH, mk::icon_size::MEDIUM, p.on_surface_variant)),
                    Some(
                        mkw::button(
                            "Koppeln",
                            mkw::buttons::Style::Tinted,
                            mkw::buttons::Role::Normal,
                            mkw::buttons::Size::Small,
                            p,
                            (!self.loading).then(|| Msg::BtPair(g.mac.clone(), g.name.clone())),
                        )
                        .into(),
                    ),
                    p,
                ));
            }
            column = column.push(mkw::section("NEUES GERÄT", new_lines, p));
        }

        // --- Neu suchen ---
        column = column.push(
            container(mkw::button(
                if self.loading { "Suche läuft \u{2026}" } else { "Neu suchen" },
                mkw::buttons::Style::Tinted,
                mkw::buttons::Role::Normal,
                mkw::buttons::Size::Small,
                p,
                Some(Msg::Reload),
            ))
            .width(Length::Fill)
            .align_x(iced::Alignment::Center),
        );

        column = column.push(Space::new().height(mk::spacing::S));
        column.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terse_fields_with_escape() {
        assert_eq!(terse_fields("*:Nick\\:Netz:76:WPA2"), vec!["*", "Nick:Netz", "76", "WPA2"]);
        assert_eq!(terse_fields(" ::29:WPA2"), vec![" ", "", "29", "WPA2"]);
    }
}
