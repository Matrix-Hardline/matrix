//! Developer Access — as a panel of Matrix Einstellungen (fusion R41b).
//! Was app #23 (matrix-entwicklerzugang): SSH access, network, update source,
//! polkit grant, mobile build environment. Privileged work runs via pkexec.

use matrixkit_widgets::iced::widget::{column, container, row, Space};
use matrixkit_widgets::iced::{Element, Task};
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;

const PC_STANDARD: &str = "benutzer@bau-rechner";

/// Snapshot of the access — gathered once (async), non-blocking.
#[derive(Debug, Clone, Default)]
pub struct Status {
    host: String,
    home_ip: Option<String>,
    hotspot_ip: Option<String>,
    ssh_server: bool,
    firewall_ssh: Option<bool>,
    key_short: Option<String>,
    key_full: Option<String>,
    polkit_free: bool,
    bau_environment: bool,
    sources: bool,
}

impl Status {
    fn net_text(&self) -> String {
        match (&self.home_ip, &self.hotspot_ip) {
            (Some(ip), _) => format!("Heimnetz · {ip}"),
            (None, Some(ip)) => format!("Handy-Hotspot · {ip}"),
            (None, None) => String::from("Offline"),
        }
    }
}

pub struct Panel {
    pub palette: mk::Palette,
    status: Status,
    loading: bool,
    update_pc: String,
    partner: String,
    new_key: String,
    /// Result banner: (success?, text).
    message: Option<(bool, String)>,
    /// Reachability of the peer after "Testen".
    pc_reachable: Option<bool>,
    busy: bool,
    /// derived display texts (live as long as the app — zeile_wert borrows &str).
    net_text: String,
    fp_text: String,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Reload,
    Loaded(Status),
    UpdatePcInput(String),
    PartnerInput(String),
    Save,
    KeyInput(String),
    KeyAllow,
    KeyCopy,
    SshServer(bool),
    ShareInstall,
    PeerRunTest,
    PeerTested(bool),
    Done(bool, String, bool),
}

impl Panel {
    pub fn new() -> (Self, Task<Msg>) {
        let app = Self {
            palette: mk::Palette::load().unwrap_or_default(),
            status: Status::default(),
            loading: true,
            update_pc: mk::settings::read("update-pc")
                .unwrap_or_else(|| String::from(PC_STANDARD)),
            partner: mk::settings::read("dev-partner").unwrap_or_default(),
            new_key: String::new(),
            message: None,
            pc_reachable: None,
            busy: false,
            net_text: String::from("…"),
            fp_text: String::from("…"),
        };
        (app, Task::perform(async { status_gather() }, Msg::Loaded))
    }

    /// From the host per tick: palette follows (the message stays).
    pub fn tick(&mut self, p: mk::Palette) {
        self.palette = p;
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Reload => {
                self.loading = true;
                self.pc_reachable = None;
                return Task::perform(async { status_gather() }, Msg::Loaded);
            }
            Msg::Loaded(s) => {
                self.net_text = s.net_text();
                self.fp_text = s
                    .key_short
                    .clone()
                    .unwrap_or_else(|| String::from("kein Schlüssel"));
                self.status = s;
                self.loading = false;
            }
            Msg::UpdatePcInput(v) => self.update_pc = v,
            Msg::PartnerInput(v) => self.partner = v,
            Msg::Save => {
                mk::settings::write("update-pc", self.update_pc.trim());
                mk::settings::write("dev-partner", self.partner.trim());
                self.message = Some((true, String::from("Gegenstellen gespeichert.")));
            }
            Msg::KeyInput(v) => self.new_key = v,
            Msg::KeyAllow => {
                let key = self.new_key.trim().to_string();
                if !key.starts_with("ssh-") {
                    self.message =
                        Some((false, String::from("Das sieht nicht nach einem SSH-Schlüssel aus.")));
                    return Task::none();
                }
                self.new_key.clear();
                return Task::perform(
                    async move { key_allow(&key) },
                    |ok| {
                        Msg::Done(
                            ok,
                            if ok {
                                String::from("Schlüssel erlaubt — dieses Gerät ist jetzt erreichbar.")
                            } else {
                                String::from("Konnte den Schlüssel nicht hinterlegen.")
                            },
                            true,
                        )
                    },
                );
            }
            Msg::KeyCopy => {
                if let Some(k) = self.status.key_full.clone() {
                    return Task::perform(async move { to_clipboard(&k) }, |ok| {
                        Msg::Done(
                            ok,
                            if ok {
                                String::from("Schlüssel in die Zwischenablage kopiert.")
                            } else {
                                String::from("wl-copy fehlt — Schlüssel steht oben zum Abtippen.")
                            },
                            false,
                        )
                    });
                }
            }
            Msg::SshServer(on) => {
                self.busy = true;
                return Task::perform(async move { ssh_server_set(on) }, move |ok| {
                    Msg::Done(
                        ok,
                        if ok {
                            String::from("SSH-Server umgeschaltet.")
                        } else {
                            String::from("Abgebrochen oder fehlgeschlagen.")
                        },
                        true,
                    )
                });
            }
            Msg::ShareInstall => {
                self.busy = true;
                return Task::perform(async { share_install() }, |ok| {
                    Msg::Done(
                        ok,
                        if ok {
                            String::from("Update-Freigabe aktiv — Updates nun ohne Passwort.")
                        } else {
                            String::from("Freigabe nicht gesetzt (abgebrochen?).")
                        },
                        true,
                    )
                });
            }
            Msg::PeerRunTest => {
                let target = self.update_pc.trim().to_string();
                self.pc_reachable = None;
                return Task::perform(async move { peer_run_test(&target) }, Msg::PeerTested);
            }
            Msg::PeerTested(ok) => self.pc_reachable = Some(ok),
            Msg::Done(ok, text, reload) => {
                self.busy = false;
                self.message = Some((ok, text));
                if reload {
                    return Task::perform(async { status_gather() }, Msg::Loaded);
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let p = self.palette;
        let mut content = column![].spacing(mk::spacing::L);

        // --- Banner: message ---
        if let Some((ok, text)) = &self.message {
            content = content.push(mkw::notice(*ok, text.clone(), p));
        }

        // --- This computer ---
        content = content.push(mkw::section(
            "DIESER RECHNER",
            vec![
                mkw::row_value("Rechner", None, &self.status.host, p),
                mkw::row_value("Netzwerk", Some("Heim = ich baue & rolle aus · Hotspot = mobiler Dev-Zugang"), &self.net_text, p),
                mkw::row_toggle(
                    "SSH-Server",
                    Some("Fernzugang für Entwicklung & Updates"),
                    None,
                    self.status.ssh_server,
                    p,
                    (!self.busy).then_some(Msg::SshServer(!self.status.ssh_server)),
                ),
                mkw::row_value(
                    "Firewall (SSH)",
                    None,
                    match self.status.firewall_ssh {
                        Some(true) => "offen",
                        Some(false) => "blockiert",
                        None => "—",
                    },
                    p,
                ),
            ],
            p,
        ));

        // --- my key ---
        content = content.push(mkw::section(
            "MEIN SCHLÜSSEL",
            vec![
                mkw::row_value("Fingerabdruck", Some("Damit meldet sich dieses Gerät bei anderen an"), &self.fp_text, p),
                mkw::row_button(
                    "Öffentlichen Schlüssel kopieren",
                    Some("Auf der Gegenstelle unter Zugang gewähren einfügen"),
                    "Kopieren",
                    p,
                    Msg::KeyCopy,
                ),
            ],
            p,
        ));

        // --- grant access ---
        content = content.push(mkw::section(
            "ZUGANG GEWÄHREN",
            vec![
                container(
                    column![
                        mkw::txt(
                            "Öffentlichen Schlüssel eines anderen Geräts einfügen, um ihm Zugang zu diesem Rechner zu geben:",
                            mk::typo::CAPTION,
                            p.on_surface_variant,
                        ),
                        Space::new().height(mk::spacing::XS),
                        mkw::text_field(
                            "",
                            &self.new_key,
                            "ssh-ed25519 AAAA… name@gerät",
                            Msg::KeyInput,
                            Some(Msg::KeyAllow),
                            None,
                            false,
                            p,
                        ),
                        Space::new().height(mk::spacing::XS),
                        mkw::button("Zugang erlauben", mkw::buttons::Style::Prominent, mkw::buttons::Role::Normal, mkw::buttons::Size::Normal, p, (!self.new_key.trim().is_empty()).then_some(Msg::KeyAllow)),
                    ]
                    .spacing(0),
                )
                .padding(mk::spacing::M)
                .into(),
            ],
            p,
        ));

        // --- Peers ---
        let test_text = match self.pc_reachable {
            Some(true) => "erreichbar ✓",
            Some(false) => "nicht erreichbar",
            None => "Testen",
        };
        content = content.push(mkw::section(
            "GEGENSTELLEN",
            vec![
                container(
                    column![
                        mkw::txt("Bau-PC / Update-Quelle", mk::typo::CAPTION, p.on_surface_variant),
                        Space::new().height(mk::spacing::XXS),
                        mkw::text_field(
                            "",
                            &self.update_pc,
                            "benutzer@bau-rechner",
                            Msg::UpdatePcInput,
                            Some(Msg::Save),
                            None,
                            false,
                            p,
                        ),
                        Space::new().height(mk::spacing::XS),
                        mkw::txt("Partner-Gerät (optional)", mk::typo::CAPTION, p.on_surface_variant),
                        Space::new().height(mk::spacing::XXS),
                        mkw::text_field(
                            "",
                            &self.partner,
                            "benutzer@partner-geraet",
                            Msg::PartnerInput,
                            Some(Msg::Save),
                            None,
                            false,
                            p,
                        ),
                        Space::new().height(mk::spacing::S),
                        row![
                            mkw::button("Speichern", mkw::buttons::Style::Tinted, mkw::buttons::Role::Normal, mkw::buttons::Size::Normal, p, Some(Msg::Save)),
                            Space::new().width(mk::spacing::S),
                            mkw::button(test_text, mkw::buttons::Style::Tinted, mkw::buttons::Role::Normal, mkw::buttons::Size::Normal, p, (!self.update_pc.trim().is_empty()).then_some(Msg::PeerRunTest)),
                        ],
                    ]
                    .spacing(0),
                )
                .padding(mk::spacing::M)
                .into(),
            ],
            p,
        ));

        // --- Update approval & tools ---
        content = content.push(mkw::section(
            "UPDATE & WERKZEUGE",
            vec![
                if self.status.polkit_free {
                    mkw::row_value(
                        "Update-Freigabe",
                        Some("Updates laufen ohne Passwort"),
                        "aktiv ✓",
                        p,
                    )
                } else {
                    mkw::row_button(
                        "Update-Freigabe",
                        Some("Einmalig einrichten (fragt nach dem Passwort)"),
                        "Einrichten",
                        p,
                        Msg::ShareInstall,
                    )
                },
                mkw::row_value(
                    "Bau-Umgebung",
                    Some("Container zum Kompilieren unterwegs"),
                    if self.status.bau_environment { "bereit ✓" } else { "nicht eingerichtet" },
                    p,
                ),
                mkw::row_value(
                    "Quelltext",
                    None,
                    if self.status.sources { "vorhanden ✓" } else { "fehlt" },
                    p,
                ),
            ],
            p,
        ));

        content.into()
    }
}

// ---------------------------------------------------------------- Probes

fn sh(cmd: &str) -> Option<String> {
    let out = std::process::Command::new("sh").args(["-c", cmd]).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn status_gather() -> Status {
    let home = std::env::var("HOME").unwrap_or_default();
    let host = std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| String::from("?"));

    // Classify IPs.
    let addr = sh("ip -brief addr 2>/dev/null").unwrap_or_default();
    let mut home_ip = None;
    let mut hotspot_ip = None;
    for wort in addr.split_whitespace() {
        if let Some(ip) = wort.strip_suffix("/24").or_else(|| wort.split('/').next()) {
            if ip.starts_with("192.168.") {
                home_ip.get_or_insert_with(|| ip.to_string());
            } else if ip.starts_with("172.20.10.") {
                hotspot_ip.get_or_insert_with(|| ip.to_string());
            }
        }
    }

    let ssh_server = sh("systemctl is-active sshd 2>/dev/null")
        .map(|s| s == "active")
        .unwrap_or(false);

    let firewall_ssh = sh("firewall-cmd --query-service=ssh 2>/dev/null")
        .map(|s| s == "yes")
        .or_else(|| sh("command -v firewall-cmd").map(|_| false));

    let pub_path = format!("{home}/.ssh/id_ed25519.pub");
    let key_full = std::fs::read_to_string(&pub_path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let key_short = sh(&format!("ssh-keygen -lf {pub_path} 2>/dev/null"))
        .and_then(|s| s.split_whitespace().nth(1).map(String::from));

    let polkit_free =
        std::path::Path::new("/etc/polkit-1/rules.d/90-matrix-update.rules").exists();

    let bau_environment = sh("distrobox list 2>/dev/null | grep -qi matrixdev && echo ja")
        .as_deref()
        == Some("ja");
    let sources = std::path::Path::new(&format!("{home}/matrixkit-dev/matrixkit/Cargo.toml")).exists();

    Status {
        host,
        home_ip,
        hotspot_ip,
        ssh_server,
        firewall_ssh,
        key_short,
        key_full,
        polkit_free,
        bau_environment,
        sources,
    }
}

// ---------------------------------------------------------------- Actions

/// Foreign key into authorized_keys — no root needed.
fn key_allow(key: &str) -> bool {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return false,
    };
    let dir = format!("{home}/.ssh");
    if std::fs::create_dir_all(&dir).is_err() {
        return false;
    }
    let path = format!("{dir}/authorized_keys");
    let present = std::fs::read_to_string(&path).unwrap_or_default();
    if present.lines().any(|z| z.trim() == key.trim()) {
        return true; // already there
    }
    use std::io::Write;
    let ok = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| writeln!(f, "{key}"))
        .is_ok();
    if ok {
        let _ = std::process::Command::new("chmod").args(["600", &path]).status();
    }
    ok
}

fn to_clipboard(text: &str) -> bool {
    use std::io::Write;
    let Ok(mut child) = std::process::Command::new("wl-copy")
        .stdin(std::process::Stdio::piped())
        .spawn()
    else {
        return false;
    };
    if let Some(mut pipe_in) = child.stdin.take() {
        let _ = pipe_in.write_all(text.as_bytes());
    }
    child.wait().map(|s| s.success()).unwrap_or(false)
}

/// SSH server on/off + firewall — needs root → pkexec (graphical prompt).
fn ssh_server_set(on: bool) -> bool {
    let cmd = if on {
        "systemctl enable --now sshd; firewall-cmd --add-service=ssh --permanent 2>/dev/null; firewall-cmd --reload 2>/dev/null; true"
    } else {
        "systemctl disable --now sshd; true"
    };
    std::process::Command::new("pkexec")
        .args(["sh", "-c", cmd])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// polkit rule: rpm-ostree for the current user without a password.
fn share_install() -> bool {
    let user = std::env::var("USER").unwrap_or_else(|_| String::from("benutzer"));
    let rule = format!(
        "polkit.addRule(function(action, subject) {{ \
if (action.id.indexOf(\"org.projectatomic.rpmostree1\") === 0 && \
subject.user === \"{user}\") {{ return polkit.Result.YES; }} }});"
    );
    let cmd = format!(
        "mkdir -p /etc/polkit-1/rules.d && cat > /etc/polkit-1/rules.d/90-matrix-update.rules <<'RULE'\n{rule}\nRULE\nsystemctl restart polkit; true"
    );
    std::process::Command::new("pkexec")
        .args(["sh", "-c", &cmd])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn peer_run_test(target: &str) -> bool {
    std::process::Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=5",
            target,
            "true",
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
