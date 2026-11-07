//! "Softwareupdate" section — the fusion of the Matrix Updater (R89).
//!
//! Reference grammar: software update is a settings section, not an
//! own app. The engine was pulled 1:1 from Matrix Updater (app #9):
//! state from rpm-ostree, digest comparison with the build PC (SSH)
//! or the registry (skopeo, anonymous), update via the
//! slim root helper, reboot via logind. matrix-updater itself
//! has been only a switch to here since R89.

use matrixkit_widgets::iced;
use matrixkit_widgets::iced::widget::{column, container, row, Space};
use matrixkit_widgets::iced::{Element, Length, Subscription, Task};
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use std::time::Duration;

const REGISTRY_IMAGE: &str = "ghcr.io/neurosector/matrix:latest";
/// The build PC in the home network: publishes images under ~/matrix-updates/.
/// Overridable via ~/.config/matrix/update-pc (user@host).
const PC_STANDARD: &str = "benutzer@bau-rechner";
const PC_ARCHIVE: &str = "/var/tmp/matrix-pc-update.tar";

fn pc_adresse() -> String {
    mk::settings::read("update-pc").unwrap_or_else(|| String::from(PC_STANDARD))
}

/// The running state of the system (from rpm-ostree).
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Full image reference of the origin (registry OR stick archive).
    origin: String,
    /// Digest of the booted image (sha256:…).
    digest: String,
    /// Timestamp of the deployment (Unix; 0 = unknown).
    since_unix: i64,
}

#[derive(Debug, Clone, PartialEq)]
enum Phase {
    Checking,
    Current,
    UpdateAvailable,
    /// Registry unreadable: offline, private, or no skopeo.
    Error(String),
}

pub struct Panel {
    palette: mk::Palette,
    state: State,
    registry_digest: Option<String>,
    /// true = the checked source is the home PC (SSH), not ghcr.io.
    from_pc: bool,
    phase: Phase,
    /// "What is new" — commit titles from the public repo.
    news: Vec<String>,
    update_running: bool,
    /// Some(true) = update done, a restart adopts it.
    update_done: Option<bool>,
    pulse: f32,
}

#[derive(Debug, Clone)]
pub enum Msg {
    PulseTick,
    Check,
    State(State, Result<String, String>, bool),
    News(Vec<String>),
    Refresh,
    UpdateDone(bool),
    Restart,
}

/// Network permission of the SETTINGS (binding as everywhere): without
/// the permission the source is never contacted.
fn net_allowed() -> bool {
    mk::permissions::PermissionSet::load("matrix-settings")
        .allowed(mk::permissions::Permission::Network)
}

impl Panel {
    pub fn new() -> (Self, Task<Msg>) {
        let panel = Self {
            palette: mk::Palette::load().unwrap_or_default(),
            state: State::default(),
            registry_digest: None,
            from_pc: false,
            phase: Phase::Checking,
            news: Vec::new(),
            update_running: false,
            update_done: None,
            pulse: 0.0,
        };
        let task = check_task(net_allowed());
        (panel, task)
    }

    pub fn tick(&mut self, p: mk::Palette) {
        self.palette = p;
    }

    /// The pulse tick only runs while checking or installing.
    pub fn subscription(&self) -> Subscription<Msg> {
        if self.phase == Phase::Checking || self.update_running {
            mkw::tick("update-pulse", Duration::from_millis(16)).map(|_| Msg::PulseTick)
        } else {
            Subscription::none()
        }
    }

    /// Ctrl+R from the app: check freshly (reference refreshable).
    pub fn refresh_button(&mut self) -> Task<Msg> {
        if self.update_running {
            return Task::none();
        }
        self.update(Msg::Check)
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::PulseTick => {
                self.pulse = (self.pulse + 1.0 / 90.0).fract();
            }
            Msg::Check => {
                self.phase = Phase::Checking;
                self.update_done = None;
                return check_task(net_allowed());
            }
            Msg::State(state, registry, from_pc) => {
                self.state = state;
                self.from_pc = from_pc;
                match registry {
                    Ok(digest) => {
                        self.phase = if !self.state.digest.is_empty()
                            && self.state.digest == digest
                        {
                            Phase::Current
                        } else {
                            Phase::UpdateAvailable
                        };
                        self.registry_digest = Some(digest);
                    }
                    Err(reason) => self.phase = Phase::Error(reason),
                }
            }
            Msg::News(n) => self.news = n,
            Msg::Refresh => {
                if !self.update_running {
                    self.update_running = true;
                    let from_pc = self.from_pc;
                    return Task::perform(
                        async move {
                            if from_pc {
                                update_from_pc()
                            } else {
                                update_execute()
                            }
                        },
                        Msg::UpdateDone,
                    );
                }
            }
            Msg::UpdateDone(ok) => {
                self.update_running = false;
                self.update_done = Some(ok);
            }
            Msg::Restart => {
                // logind allows the active session to restart without root.
                let _ = std::process::Command::new("systemctl").arg("reboot").spawn();
            }
        }
        Task::none()
    }

    pub fn footer_text(&self) -> String {
        String::from(
            "Updates kommen als komplettes System-Image — Zurückrollen jederzeit im Startmenü",
        )
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let p = self.palette;

        // --- status section (the reference software-update panel) ---
        let net = net_allowed();
        let (symbol, color, title, description): (char, mk::Rgba, String, String) =
            if self.update_running {
                (
                    mkw::symbol::MONITORING,
                    p.primary,
                    String::from("Update wird installiert …"),
                    String::from("Das kann einige Minuten dauern — Fenster darf offen bleiben."),
                )
            } else if self.update_done == Some(true) {
                (
                    mkw::symbol::CHECK,
                    p.primary,
                    String::from("Update installiert"),
                    String::from("Beim nächsten Start ist es aktiv."),
                )
            } else if self.update_done == Some(false) {
                (
                    mkw::symbol::WARNING,
                    p.error,
                    String::from("Update fehlgeschlagen"),
                    String::from("Erneut versuchen — oder Details im Systemprotokoll."),
                )
            } else {
                match &self.phase {
                    Phase::Checking => (
                        mkw::symbol::SEARCH,
                        p.on_surface_variant,
                        String::from("Suche nach Updates …"),
                        String::from("Vergleiche mit der Matrix-Quelle."),
                    ),
                    Phase::Current => (
                        mkw::symbol::CHECK,
                        p.primary,
                        String::from("Matrix ist auf dem neuesten Stand"),
                        format!("Stand von {}", self.state_since_text()),
                    ),
                    Phase::UpdateAvailable => (
                        mkw::symbol::ARROW_DOWNWARD,
                        p.primary,
                        String::from("Ein Update ist verfügbar"),
                        String::from("Ein Klick holt es — der Neustart übernimmt es."),
                    ),
                    Phase::Error(reason) => (
                        mkw::symbol::WARNING,
                        p.on_surface_variant,
                        String::from("Quelle nicht erreichbar"),
                        reason.clone(),
                    ),
                }
            };

        // Dynamic texts: direct row instead of mkw::zeile (no Box::leak).
        let mut status_text = column![mkw::txt(title, mk::typo::HEADLINE, p.on_surface)].spacing(2);
        status_text = status_text.push(mkw::txt(description, mk::typo::CAPTION, p.on_surface_variant));
        let mut status_row = row![
            mkw::symbol::<Msg>(symbol, mk::icon_size::LARGE, color),
            container(status_text).width(Length::Fill),
        ]
        .spacing(mk::spacing::M)
        .align_y(iced::Alignment::Center);
        if let Some(k) = self.status_button(p, net) {
            status_row = status_row.push(k);
        }
        let mut status_lines: Vec<Element<'_, Msg>> = vec![container(status_row)
            .padding(iced::Padding {
                top: mk::spacing::M,
                right: mk::spacing::M,
                bottom: mk::spacing::M,
                left: mk::spacing::M,
            })
            .into()];
        if self.update_running || self.phase == Phase::Checking {
            status_lines.push(
                container(mkw::pulse(self.pulse, p.primary))
                    .padding(iced::Padding {
                        left: mk::spacing::M,
                        right: mk::spacing::M,
                        bottom: mk::spacing::S,
                        ..iced::Padding::ZERO
                    })
                    .into(),
            );
        }

        // --- Dieses System ---
        let digest_short = |d: &str| -> String {
            d.strip_prefix("sha256:").unwrap_or(d).chars().take(12).collect()
        };
        let stand_text = format!("{} · {}", digest_short(&self.state.digest), self.state_since_text());
        let system_lines: Vec<Element<'_, Msg>> = vec![
            mkw::row_item("Quelle", Some(source_readable(&self.state.origin)), None, None, p),
            container(
                column![
                    mkw::txt("Stand", mk::typo::BODY, p.on_surface),
                    mkw::txt(stand_text, mk::typo::CAPTION, p.on_surface_variant),
                ]
                .spacing(2),
            )
            .padding(iced::Padding {
                top: mk::spacing::S + 2.0,
                right: mk::spacing::M,
                bottom: mk::spacing::S + 2.0,
                left: mk::spacing::M,
            })
            .into(),
        ];

        // --- what is new ---
        let mut content = column![
            mkw::section("SOFTWAREUPDATE", status_lines, p),
            Space::new().height(mk::spacing::L),
            mkw::section("DIESES SYSTEM", system_lines, p),
        ]
        .spacing(0);
        if !self.news.is_empty() {
            let new_lines: Vec<Element<'_, Msg>> = self
                .news
                .iter()
                .map(|t| {
                    mkw::row_item(
                        t.as_str(),
                        None,
                        Some(mkw::symbol::<Msg>(
                            mkw::symbol::CHEVRON_RIGHT,
                            mk::font_size::SMALL,
                            p.on_surface_variant,
                        )),
                        None,
                        p,
                    )
                })
                .collect();
            content = content
                .push(Space::new().height(mk::spacing::L))
                .push(mkw::section("WAS IST NEU", new_lines, p));
        }
        content.into()
    }

    /// "Stand von …": relative while fresh (reference culture);
    /// from two weeks on the absolute date.
    fn state_since_text(&self) -> String {
        if self.state.since_unix == 0 {
            return String::from("unbekannt");
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(self.state.since_unix);
        if now - self.state.since_unix < 14 * 86400 {
            mk::time::relative(self.state.since_unix, now)
        } else {
            time_readable(self.state.since_unix)
        }
    }

    /// The ONE button on the right of the status row — depending on phase.
    fn status_button(&self, p: mk::Palette, net: bool) -> Option<Element<'_, Msg>> {
        use mkw::buttons::*;
        if self.update_running {
            return None;
        }
        if self.update_done == Some(true) {
            return Some(
                mkw::button("Neu starten", Style::Prominent, Role::Normal, Size::Normal, p, Some(Msg::Restart))
                    .into(),
            );
        }
        let e: Element<'_, Msg> = match &self.phase {
            Phase::UpdateAvailable => {
                mkw::button("Aktualisieren", Style::Prominent, Role::Normal, Size::Normal, p, Some(Msg::Refresh)).into()
            }
            Phase::Checking => return None,
            _ => mkw::button(
                "Erneut suchen",
                Style::Tinted,
                Role::Normal,
                Size::Normal,
                p,
                net.then_some(Msg::Check),
            )
            .into(),
        };
        Some(e)
    }
}

/// Check job: state + registry digest + news in parallel.
fn check_task(net_allowed: bool) -> Task<Msg> {
    let stand_task = Task::perform(
        async move {
            let state = stand_read();
            if !net_allowed {
                let e = Err(String::from(
                    "Netzwerk-Berechtigung ist aus (App-Name → PermissionSet)",
                ));
                return (state, e, false);
            }
            // Source 1: the build PC in the home network (fresher than any registry).
            match pc_digest() {
                Ok(d) => (state, Ok(d), true),
                // Source 2: the public registry.
                Err(_) => (state, registry_digest(), false),
            }
        },
        |(s, r, pc)| Msg::State(s, r, pc),
    );
    if net_allowed {
        Task::batch([
            stand_task,
            Task::perform(async { news_fetch() }, Msg::News),
        ])
    } else {
        stand_task
    }
}

/// Read the booted state from rpm-ostree (works without root).
fn stand_read() -> State {
    let out = std::process::Command::new("rpm-ostree")
        .args(["status", "--json"])
        .output();
    let Ok(out) = out else { return State::default() };
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else {
        return State::default();
    };
    let empty = Vec::new();
    let deployments = json["deployments"].as_array().unwrap_or(&empty);
    let booted = deployments
        .iter()
        .find(|d| d["booted"].as_bool() == Some(true));
    let Some(b) = booted else { return State::default() };
    let origin = b["container-image-reference"]
        .as_str()
        .or_else(|| b["origin"].as_str())
        .unwrap_or("?")
        .to_string();
    let digest = b["container-image-reference-digest"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let since_unix = b["timestamp"].as_i64().unwrap_or(0);
    State { origin, digest, since_unix }
}

fn time_readable(unix: i64) -> String {
    // Without chrono: date handles the localization.
    std::process::Command::new("date")
        .args(["-d", &format!("@{unix}"), "+%-d. %B %Y, %H:%M"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("@{unix}"))
}

/// Digest of the image published on the build PC (~/matrix-updates/
/// digest.txt, written by the build-and-publish script).
/// Reachable only in the home network — else Err and registry fallback.
fn pc_digest() -> Result<String, String> {
    let out = std::process::Command::new("ssh")
        .args([
            "-o", "BatchMode=yes",
            "-o", "ConnectTimeout=4",
            &pc_adresse(),
            "head -1 ~/matrix-updates/digest.txt",
        ])
        .output()
        .map_err(|_| String::from("ssh fehlt"))?;
    if out.status.success() {
        let d = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if d.starts_with("sha256:") {
            return Ok(d);
        }
    }
    Err(String::from("PC nicht erreichbar"))
}

/// Update from the PC: fetch the archive via scp, then rebase (polkit
/// allows rpm-ostree for the user — no root helper needed).
fn update_from_pc() -> bool {
    let fetched = std::process::Command::new("scp")
        .args([
            "-o", "BatchMode=yes",
            &format!("{}:matrix-updates/matrix.tar", pc_adresse()),
            PC_ARCHIVE,
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !fetched {
        return false;
    }
    let rebase = std::process::Command::new("rpm-ostree")
        .args([
            "rebase",
            &format!("ostree-unverified-image:oci-archive:{PC_ARCHIVE}"),
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if rebase {
        return true;
    }
    // The ref trap (Surface, 2026-07-19): if the system ALREADY runs from
    // this archive path, rebase refuses with "Old and new refs are equal" —
    // although the tar content is new. Only `upgrade` pulls the same
    // ref freshly (deploy doctrine from the build PC).
    std::process::Command::new("rpm-ostree")
        .arg("upgrade")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Digest of the newest image — anonymous (the Matrix image is public).
fn registry_digest() -> Result<String, String> {
    let out = std::process::Command::new("skopeo")
        .args([
            "inspect",
            "--format",
            "{{.Digest}}",
            &format!("docker://{REGISTRY_IMAGE}"),
        ])
        .output()
        .map_err(|_| String::from("skopeo fehlt auf diesem System"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        let error = String::from_utf8_lossy(&out.stderr);
        if error.contains("unauthorized") || error.contains("authentication") {
            Err(String::from("Quelle ist privat — auf GitHub „Public“ schalten"))
        } else {
            Err(String::from("Offline oder Quelle nicht erreichbar"))
        }
    }
}

/// "What is new": commit titles of the public repo (best effort).
fn news_fetch() -> Vec<String> {
    let out = std::process::Command::new("curl")
        .args([
            "-sf",
            "-m",
            "8",
            "https://api.github.com/repos/Neurosector/matrix/commits?per_page=8",
        ])
        .output();
    let Ok(out) = out else { return Vec::new() };
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else {
        return Vec::new();
    };
    json.as_array()
        .map(|commits| {
            commits
                .iter()
                .filter_map(|c| c["commit"]["message"].as_str())
                .filter_map(|m| m.lines().next())
                .map(|z| {
                    // Display polish (R139): no hard cut-off — a
                    // visible ellipsis where the line must end.
                    mk::elide(z, 96)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Update via the narrow root helper (sudoers NOPASSWD, only this one).
fn update_execute() -> bool {
    std::process::Command::new("sudo")
        .args(["-n", "/usr/bin/matrix-update-helper"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Origin human-readable: registry, stick archive or unknown.
fn source_readable(origin: &str) -> &'static str {
    if origin.contains("matrix-pc-update") || origin.contains("matrix-neu.tar") {
        "Vom Bau-PC — Updates kommen aus dem Heimnetz"
    } else if origin.contains("ghcr.io/neurosector/matrix") {
        "Matrix-Quelle (ghcr.io) — Updates kommen automatisch hierher"
    } else if origin.contains("oci-archive") {
        "USB-Stick-Installation — das erste Update stellt auf die Matrix-Quelle um"
    } else {
        "Eigene Quelle"
    }
}
