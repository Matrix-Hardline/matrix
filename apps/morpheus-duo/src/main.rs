//! Morpheus — on the Surface Laptop 7, the Surface Duo's Matrix companion
//! (06.11.2026, no iced). Two ways:
//!
//! * «Matrix starten» — the duo restarted into Android (its default): back
//!   into Matrix from its partition (`duo-fedora.sh`, Duo-Start's way).
//! * «Matrix installieren» — a NEW duo: the bootloader checked, the live
//!   system (the factory image, RAM only) started over the cable
//!   (`morpheus-live.sh`); from then on the duo leads (backup, split, an
//!   explicit confirmation) and the laptop delivers what it asks for
//!   (`morpheus-lieferant.sh`: the backup here, the image there).
//!
//! Safety (Surface Duo doctrine): the laptop only `fastboot boot`s the known
//! start image — never `fastboot flash`, never an unlock of its own. All
//! writing happens on the duo, by its installer, after its confirmation;
//! «Trockenlauf» lets it only describe what it would do.

mod companion;
mod flow;
mod pages;

use flow::{Device, Link, State, INSTALL_STEPS, STEPS};
use pages::{Deliver, Files, Page, Photos};
use matrixkit_image::ImageAtlas;
use matrixkit_scene::scene as fam;
use matrixkit_scene::symbol;
use matrixkit_shell::{self as shell, Refresh, Scene, SceneBuilder, Source};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{Keep, Rect};
use std::io::BufRead;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const APP_ID: &str = "matrix-morpheus-duo";
const CHROME_BASE: usize = 1000;
const GO: usize = 10;
const ANDROID: usize = 11;
const DRY: usize = 12;
const WAY_BASE: usize = 20;
const LANG_BASE: usize = 30;
const KBD_BASE: usize = 40;
const LANG_OK: usize = 50;
const LANG_OPEN: usize = 51;
/// Matrix Recovery (06.11.2026): start, back up, reinstall, restore one of the backups.
const REC_START: usize = 60;
const REC_BACKUP: usize = 61;
const REC_NEW: usize = 62;
const REC_RESTORE: usize = 70;
const SHOWN_BACKUPS: usize = 4;

/// The two languages and keyboards Matrix speaks: (code, label).
const LANGS: [(&str, &str); 2] = [("de-CH", "Deutsch (Schweiz)"), ("en-GB", "English (UK)")];
const KBDS: [(&str, &str, &str); 2] = [("ch", "Schweiz (QWERTZ)", "Swiss (QWERTZ)"), ("gb", "UK (QWERTY)", "UK (QWERTY)")];

/// German or English, by the language chosen.
fn l<'a>(de: &'a str, en: &'a str) -> &'a str {
    mk::language::pick(de, en)
}

/// Where Morpheus remembers the language and keyboard (line 1, line 2).
fn choice_file() -> PathBuf {
    home().join(".config/matrix-morpheus/sprache")
}

fn choice_read() -> Option<(&'static str, &'static str)> {
    let text = std::fs::read_to_string(choice_file()).ok()?;
    let mut it = text.lines();
    let lang = if it.next()?.trim() == "en-GB" { "en-GB" } else { "de-CH" };
    let kbd = if it.next().map(str::trim) == Some("gb") { "gb" } else { "ch" };
    Some((lang, kbd))
}
const DUO_IP: [u8; 4] = [172, 16, 42, 1];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Way {
    Start,
    Install,
}

/// What Matrix Recovery is doing right now (a run of morpheus-recovery.sh).
#[derive(Debug, Clone, PartialEq)]
enum Task {
    Backup,
    Restore(PathBuf),
    Reinstall,
    Start,
}

/// One Matrix backup on this laptop (~/Sicherungen/surface-duo-<serial>/matrix-<date>/).
#[derive(Debug, Clone)]
struct Saved {
    dir: PathBuf,
    when: String,
    mb: u64,
    accounts: String,
}

/// The Matrix backups of one duo, newest first.
fn backups_of(serial: &str) -> Vec<Saved> {
    let base = home().join("Sicherungen").join(format!("surface-duo-{serial}"));
    let mut out: Vec<Saved> = std::fs::read_dir(&base)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|d| d.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("matrix-")) && d.join("matrix.tar.zst").is_file())
        .map(|dir| {
            let info = std::fs::read_to_string(dir.join("info.json")).unwrap_or_default();
            let flat: String = info.split_whitespace().collect::<Vec<_>>().join(" ");
            let when = flow::json_field(&flat, "zeit").map(str::to_string).unwrap_or_else(|| dir.file_name().map(|n| n.to_string_lossy().trim_start_matches("matrix-").to_string()).unwrap_or_default());
            let accounts = flow::json_field(&flat, "konten").unwrap_or("").replace('"', "").split(',').map(str::trim).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(", ");
            let mb = std::fs::metadata(dir.join("matrix.tar.zst")).map(|m| m.len() / 1_048_576).unwrap_or(0);
            Saved { dir, when, mb, accounts }
        })
        .collect();
    out.sort_by(|a, b| b.dir.cmp(&a.dir));
    out
}

/// Shared with the worker threads.
#[derive(Default)]
struct Wire {
    link: Option<Link>,
    lines: Vec<String>,
    /// The script ended (its exit code).
    ended: Option<i32>,
    /// A run is on: the watcher keeps its hands off the USB line.
    busy: bool,
    /// The companion (06.11.2026): a fresh answer of the duo's pairing service (None inside = no Matrix session).
    hello: Option<Option<companion::Hello>>,
}

struct Morpheus {
    chrome: fam::Chrome,
    palette: mk::Palette,
    wire: Arc<Mutex<Wire>>,
    link: Link,
    way: Way,
    dry: bool,
    steps: Vec<State>,
    log: Vec<String>,
    running: bool,
    outcome: Option<bool>,
    locked: bool,
    /// Morpheus asks language and keyboard first (06.11.2026) — and on the globe.
    asking_lang: bool,
    lang: &'static str,
    kbd: &'static str,
    image: PathBuf,
    factory: PathBuf,
    /// Matrix Recovery: the duo in the live system, Matrix already on it.
    device: Option<Device>,
    task: Option<Task>,
    saved: Vec<Saved>,
    /// The companion (06.11.2026): the page, the paired duo, files, photos.
    page: Page,
    hello: Option<companion::Hello>,
    info: Option<companion::Info>,
    inbox: Arc<Mutex<Vec<Deliver>>>,
    files: Files,
    photos: Photos,
    /// What a worker is doing (one at a time), and the last result.
    job: Option<String>,
    note: Option<(bool, String)>,
    pairing: bool,
    /// «Zugriff erlauben» is waiting for the password.
    access_busy: bool,
    /// A restore that waits for Matrix Recovery to come up.
    pending_restore: Option<PathBuf>,
    pending_restore_due: bool,
    width: f32,
}

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"))
}

/// The folder of this binary (inside the AppImage: usr/bin).
fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|e| e.parent().map(PathBuf::from))
}

/// A script: the AppImage's own copy first (offline, 06.11.2026), else the workshop (~/morpheus, ~/duo-a1).
fn find_script(name: &str) -> Option<PathBuf> {
    let mut places: Vec<PathBuf> = exe_dir().map(|d| d.join(name)).into_iter().collect();
    places.extend([home().join("morpheus").join(name), home().join("duo-a1").join(name)]);
    places.into_iter().find(|p| p.is_file())
}

/// An image Morpheus carries (usr/share/morpheus in the AppImage), else the workshop path.
fn carried(name: &str, workshop: &str) -> PathBuf {
    exe_dir()
        .map(|d| d.join("../share/morpheus").join(name))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| home().join(workshop))
}

fn port_open(port: u16) -> bool {
    let addr = std::net::SocketAddr::from((DUO_IP, port));
    std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(600)).is_ok()
}

fn output_of(cmd: &str, args: &[&str]) -> String {
    std::process::Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// The network interface of the Duo's USB gadget (1d6b:0104), if one is there.
fn duo_interface() -> Option<String> {
    std::fs::read_dir("/sys/class/net").ok()?.flatten().find_map(|e| {
        let dev = e.path().join("device/..");
        let id = |f: &str| std::fs::read_to_string(dev.join(f)).map(|s| s.trim().to_string()).unwrap_or_default();
        (id("idVendor") == "1d6b" && id("idProduct") == "0104").then(|| e.file_name().to_string_lossy().into_owned())
    })
}

/// The watcher: every 2 s what the duo is (paused while a run is on).
fn watch(wire: Arc<Mutex<Wire>>) {
    // 06.11.2026 (test on a foreign PC): a Duo that already runs Matrix (or stage 1) shows up as a USB network
    // interface — but nothing reaches it until this PC carries 172.16.42.2 there. Give it the address once per
    // appearance (morpheus-netz.sh: NetworkManager as this user, else pkexec).
    let mut addressed: Option<String> = None;
    std::thread::spawn(move || loop {
        let busy = wire.lock().map(|w| w.busy).unwrap_or(false);
        if !busy {
            let gadget = duo_interface();
            if gadget.is_none() {
                addressed = None;
            } else if gadget != addressed {
                if let Some(script) = find_script("morpheus-netz.sh") {
                    let _ = std::process::Command::new("bash")
                        .args(["-c", ". \"$1\"; netz_hoch", "morpheus-netz", &script.display().to_string()])
                        .stdin(std::process::Stdio::null())
                        .status();
                }
                addressed = gadget;
            }
            let ssh = port_open(22);
            let telnet = !ssh && port_open(23);
            let (adb, fastboot) = if ssh || telnet { (String::new(), String::new()) } else { (output_of("adb", &["devices"]), output_of("fastboot", &["devices"])) };
            let link = flow::link_of(&adb, &fastboot, ssh, telnet);
            // Matrix with a person logged in answers its pairing service (USB only).
            let hello = if ssh { companion::hello() } else { None };
            if let Ok(mut w) = wire.lock() {
                w.link = Some(link);
                w.hello = Some(hello);
            }
        }
        std::thread::sleep(Duration::from_secs(2));
    });
}

/// How to unlock the bootloader — Morpheus never does it itself: it writes
/// to the device and erases Android, and the duo asks for it on its own screen.
const UNLOCK: [(&str, &str); 5] = [
    ("Der Bootloader des Duo ist gesperrt. Entsperren löscht Android vollständig — Morpheus tut das nie selbst.", "The Duo's bootloader is locked. Unlocking erases Android completely — Morpheus never does it itself."),
    ("1. Auf dem Duo: Einstellungen › Info › 7× auf «Buildnummer» tippen (Entwickleroptionen).", "1. On the Duo: Settings › About › tap «Build number» 7 times (developer options)."),
    ("2. Entwickleroptionen: «OEM-Entsperrung» und «USB-Debugging» einschalten.", "2. Developer options: switch on «OEM unlocking» and «USB debugging»."),
    ("3. Am Laptop im Terminal: adb reboot bootloader, dann fastboot flashing unlock.", "3. In a terminal on the laptop: adb reboot bootloader, then fastboot flashing unlock."),
    ("4. Auf dem Duo mit den Lautstärketasten bestätigen. Danach Android einrichten, USB-Debugging wieder an, und hier neu beginnen.", "4. Confirm on the Duo with the volume keys. Then set up Android, switch USB debugging on again, and start again here."),
];

/// A step's name in the chosen language.
fn step_en(de: &str) -> &str {
    let en = match de {
        "Duo gefunden" => "Duo found",
        "Bootloader" => "Bootloader",
        "Startabbild geladen" => "Start image loaded",
        "Stufe 1" => "Stage 1",
        "Fedora meldet sich" => "Matrix answers",
        "Bootloader entsperrt" => "Bootloader unlocked",
        "Live-System gestartet" => "Live system started",
        "Installer auf dem Duo" => "Installer on the Duo",
        "Sicherung" => "Backup",
        "Matrix übertragen" => "Transfer Matrix",
        _ => de,
    };
    l(de, en)
}

impl Morpheus {
    fn new() -> Morpheus {
        let wire = Arc::new(Mutex::new(Wire::default()));
        watch(wire.clone());
        let chosen = choice_read();
        let (lang, kbd) = chosen.unwrap_or(("de-CH", "ch"));
        mk::language::choose(lang);
        Morpheus {
            asking_lang: chosen.is_none(),
            lang,
            kbd,
            chrome: fam::Chrome::new("Morpheus", CHROME_BASE),
            palette: mk::Palette::load().unwrap_or_default(),
            wire,
            link: Link::None,
            way: Way::Start,
            dry: false,
            steps: vec![State::Waiting; STEPS.len()],
            log: Vec::new(),
            running: false,
            outcome: None,
            locked: false,
            image: std::env::var_os("DUO_BILD").map(PathBuf::from).unwrap_or_else(|| carried("boot-duo1-fedora-stufe1-v7.img", "duo-a1/out/boot-duo1-fedora-stufe1-v7.img")),
            factory: std::env::var_os("MORPHEUS_WERK").map(PathBuf::from).unwrap_or_else(|| carried("matrix-werk.sqfs", "duo-a1/out/werk/matrix-werk.sqfs")),
            device: None,
            task: None,
            saved: Vec::new(),
            page: Page::Overview,
            hello: None,
            info: None,
            inbox: Arc::new(Mutex::new(Vec::new())),
            files: Files { laptop_dir: home(), ..Files::default() },
            photos: Photos::default(),
            job: None,
            note: None,
            pairing: false,
            access_busy: false,
            pending_restore: None,
            pending_restore_due: false,
            width: 1040.0,
        }
    }

    fn labels(&self) -> &'static [&'static str] {
        match self.way {
            Way::Start => &STEPS,
            Way::Install => &INSTALL_STEPS,
        }
    }

    /// What is missing for this way (the button sleeps, the hint says it).
    fn missing(&self) -> Option<String> {
        if !self.image.is_file() {
            return Some(format!("Startabbild fehlt: {}", self.image.display()));
        }
        match self.way {
            Way::Start => find_script("duo-fedora.sh").is_none().then(|| String::from("Startskript fehlt (duo-fedora.sh)")),
            Way::Install => {
                if ["morpheus-live.sh", "morpheus-lieferant.sh", "morpheus-recovery.sh"].iter().any(|n| find_script(n).is_none()) {
                    Some(String::from("Morpheus-Skripte fehlen (morpheus-live.sh, morpheus-lieferant.sh, morpheus-recovery.sh)"))
                } else if !self.factory.is_file() {
                    Some(format!("Werksabbild fehlt: {}", self.factory.display()))
                } else {
                    None
                }
            }
        }
    }

    fn choose(&mut self, way: Way) -> Refresh {
        if self.running || self.way == way {
            return Refresh::Calm;
        }
        mk::feel::tap();
        self.way = way;
        self.steps = vec![State::Waiting; self.labels().len()];
        self.outcome = None;
        self.locked = false;
        self.log.clear();
        Refresh::Dirty
    }

    /// 06.11.2026 (test on a foreign PC: «ich konnte Matrix starten nicht drücken»): after an installation — or in
    /// Matrix Recovery — the Duo sits in the live system with Matrix on its partition. «Start Matrix» then takes the
    /// recovery path (live system -> bootloader -> start from the partition) instead of staying greyed out.
    fn start_from_live(&self) -> bool {
        self.way == Way::Start && self.link == Link::Fedora && self.device.as_ref().is_some_and(|d| d.matrix)
    }

    fn go(&mut self) -> Refresh {
        if self.start_from_live() && !self.running {
            return self.recover(Task::Start);
        }
        if self.running || self.missing().is_some() || !self.link.can_start() {
            return Refresh::Calm;
        }
        mk::feel::tap();
        self.running = true;
        self.outcome = None;
        self.locked = false;
        self.steps = vec![State::Waiting; self.labels().len()];
        self.log.clear();
        // The start way: duo-fedora.sh. The install way: the live system,
        // then the delivery loop until the duo says it is done.
        let (script, env): (String, Vec<(&str, String)>) = match self.way {
            Way::Start => (format!("exec bash '{}' 2>&1", find_script("duo-fedora.sh").unwrap_or_default().display()), vec![("DUO_BILD", self.image.display().to_string())]),
            // Matrix already there -> Matrix Recovery (the laptop leads); else the installer + delivery loop.
            Way::Install => (
                format!(
                    "bash '{}' 2>&1 || exit $?; Z=$(bash '{}' zustand 2>&1); echo \"$Z\"; case \"$Z\" in *'\"matrix_da\": true'*) exit 0 ;; esac; exec bash '{}' 2>&1",
                    find_script("morpheus-live.sh").unwrap_or_default().display(),
                    find_script("morpheus-recovery.sh").unwrap_or_default().display(),
                    find_script("morpheus-lieferant.sh").unwrap_or_default().display()
                ),
                vec![
                    ("MORPHEUS_TROCKEN", String::from(if self.dry { "1" } else { "0" })),
                    ("MORPHEUS_WERK", self.factory.display().to_string()),
                    ("MORPHEUS_SPRACHE", self.lang.to_string()),
                    ("MORPHEUS_TASTATUR", self.kbd.to_string()),
                ],
            ),
        };
        self.device = None;
        self.spawn(script, env);
        Refresh::Dirty
    }

    /// Run one script in the background; its lines and its end come back over the wire.
    fn spawn(&mut self, script: String, env: Vec<(&'static str, String)>) {
        let wire = self.wire.clone();
        if let Ok(mut w) = wire.lock() {
            w.busy = true;
            w.ended = None;
            w.lines.clear();
        }
        std::thread::spawn(move || {
            let mut cmd = std::process::Command::new("bash");
            cmd.arg("-c").arg(&script).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::piped());
            for (k, v) in env {
                cmd.env(k, v);
            }
            let code = match cmd.spawn() {
                Ok(mut child) => {
                    if let Some(out) = child.stdout.take() {
                        for line in std::io::BufReader::new(out).lines().map_while(Result::ok) {
                            if let Ok(mut w) = wire.lock() {
                                w.lines.push(line);
                            }
                        }
                    }
                    child.wait().ok().and_then(|s| s.code()).unwrap_or(1)
                }
                Err(e) => {
                    if let Ok(mut w) = wire.lock() {
                        w.lines.push(format!("Skript nicht startbar: {e}"));
                    }
                    1
                }
            };
            if let Ok(mut w) = wire.lock() {
                w.ended = Some(code);
                w.busy = false;
            }
        });
    }

    /// One Matrix Recovery task (morpheus-recovery.sh on the laptop, the duo in its live system).
    fn recover(&mut self, task: Task) -> Refresh {
        if self.running || self.device.as_ref().is_none_or(|d| !d.matrix) {
            return Refresh::Calm;
        }
        let Some(script) = find_script("morpheus-recovery.sh") else { return Refresh::Calm };
        mk::feel::tap();
        let arg = match &task {
            Task::Backup => String::from("sichern"),
            Task::Restore(dir) => format!("wiederherstellen '{}'", dir.display()),
            Task::Reinstall => String::from("neu"),
            Task::Start => String::from("starten"),
        };
        self.running = true;
        self.outcome = None;
        self.log.clear();
        self.task = Some(task);
        self.spawn(format!("exec bash '{}' {arg} 2>&1", script.display()), vec![("MORPHEUS_WERK", self.factory.display().to_string())]);
        Refresh::Dirty
    }

    /// Back to Android from the bootloader.
    fn android(&mut self) -> Refresh {
        if self.running || self.link != Link::Bootloader {
            return Refresh::Calm;
        }
        mk::feel::tap();
        self.log.push(String::from("fastboot reboot → Android"));
        std::thread::spawn(|| {
            let _ = std::process::Command::new("fastboot").arg("reboot").status();
        });
        Refresh::Dirty
    }

    /// Take what the threads brought.
    fn take(&mut self) -> bool {
        let Ok(mut w) = self.wire.lock() else { return false };
        let mut moved = false;
        for line in w.lines.drain(..) {
            if let Some(device) = flow::read_device(&line) {
                if device.matrix {
                    self.saved = backups_of(&device.serial);
                    self.device = Some(device);
                    self.pending_restore_due = true;
                }
                self.log.push(line);
                moved = true;
                continue;
            }
            if self.task.is_some() {
                self.log.push(line);
                moved = true;
                continue;
            }
            let event = match self.way {
                Way::Start => flow::read(&line),
                Way::Install => flow::read_install(&line),
            };
            if event == flow::Event::Locked {
                self.locked = true;
            }
            flow::apply(&mut self.steps, event);
            self.log.push(line);
            moved = true;
        }
        if self.log.len() > 200 {
            self.log.drain(..self.log.len() - 200);
        }
        if let Some(code) = w.ended.take() {
            self.running = false;
            if let Some(task) = self.task.take() {
                let ok = code == 0;
                self.outcome = Some(ok);
                if ok {
                    mk::feel::confirm();
                    if task == Task::Start {
                        self.device = None; // Matrix runs from its partition now.
                    }
                }
                if let Some(d) = &self.device {
                    self.saved = backups_of(&d.serial);
                }
                return true;
            }
            if self.device.as_ref().is_some_and(|d| d.matrix) {
                // The live system found Matrix: Matrix Recovery, not a failed install.
                self.outcome = None;
                flow::apply(&mut self.steps, flow::Event::Begin(3));
                return true;
            }
            let ok = code == 0
                && match self.way {
                    Way::Start => self.steps.iter().all(|s| *s == State::Done),
                    Way::Install => flow::install_done(&self.steps),
                };
            if !ok {
                flow::apply(&mut self.steps, flow::Event::Failed);
            }
            self.outcome = Some(ok);
            if ok {
                mk::feel::confirm();
                // Installed: Matrix is on the partition now and the Duo still sits in the live system — the next
                // step is «Start Matrix» (recovery path), offered right away.
                if self.way == Way::Install && !self.dry {
                    if let Some(d) = self.device.as_mut() {
                        d.matrix = true;
                    }
                    self.way = Way::Start;
                }
            }
            moved = true;
        }
        let hello = w.hello.take();
        if let Some(link) = w.link {
            if link != self.link {
                self.link = link;
                if link == Link::None && !self.running {
                    self.device = None; // unplugged: Recovery ends
                }
                moved = true;
            }
        }
        drop(w);
        if let Some(h) = hello {
            if h != self.hello {
                self.greet(h);
                moved = true;
            }
        }
        if self.take_inbox() {
            moved = true;
        }
        if std::mem::take(&mut self.pending_restore_due) && !self.running {
            self.resume_restore();
            moved = true;
        }
        moved
    }

    /// MATRIX RECOVERY at the laptop («Morpheus, the iTunes of Matrix»):
    /// the duo, its Matrix, the four deeds, and this duo's backups.
    #[allow(clippy::too_many_arguments)]
    fn lay_recovery(&self, b: &mut SceneBuilder, engine: &mut TextEngine, dev: &Device, m: f32, w: f32, mut y: f32) -> f32 {
        let p = self.palette;
        let quiet = p.on_surface_variant;
        // The device card.
        let card_h = 92.0;
        b.quad(m, y, w, card_h, p.surface_container, 14.0, None, None);
        b.styled(engine, "Matrix Recovery", mk::typo::HEADLINE, p.on_surface, m + 16.0, y + 12.0, None, None);
        let line1 = format!("{} · {} {}", dev.model, l("Seriennummer", "Serial"), dev.serial);
        b.styled(engine, &line1, mk::typo::CAPTION, quiet, m + 16.0, y + 44.0, None, Some(Rect { x: m, y, w, h: card_h }));
        let who = if dev.accounts.is_empty() { String::from(l("noch kein Konto", "no account yet")) } else { dev.accounts.join(", ") };
        let line2 = format!("Matrix {} GB · {} {:.1} GB · {} {} · {} {}", dev.matrix_gib, l("Daten", "data"), dev.data_mb as f32 / 1024.0, l("Konten:", "accounts:"), who, l("System vom", "system of"), dev.system);
        b.styled(engine, &line2, mk::typo::CAPTION, quiet, m + 16.0, y + 64.0, None, Some(Rect { x: m, y, w, h: card_h }));
        y += card_h + 14.0;
        if dev.dry {
            b.styled(engine, l("Trockenlauf: auf dem Duo wird nichts geschrieben (Sichern liest nur).", "Dry run: nothing is written on the duo (backing up only reads)."), mk::typo::CAPTION, p.tertiary, m, y, None, None);
            y += 22.0;
        }
        // The deeds.
        let deeds: [(char, &str, usize, fam::PillTone); 3] = [
            (symbol::PLAY_ARROW, l("Matrix starten", "Start Matrix"), REC_START, fam::PillTone::Accent),
            (symbol::SAVE, l("Sichern", "Back up"), REC_BACKUP, fam::PillTone::Quiet),
            (symbol::RESTART, l("Neu installieren", "Reinstall"), REC_NEW, fam::PillTone::Quiet),
        ];
        let mut x = m;
        let mut bh = 0.0f32;
        for (g, t, role, tone) in deeds {
            let (role, tone) = if self.running { (None, fam::PillTone::Quiet) } else { (Some(role), tone) };
            let (pw, ph) = fam::lay_pill(b, engine, Some(g), t, x, y, None, role, tone, p);
            x += pw + 10.0;
            bh = bh.max(ph);
        }
        y += bh + 12.0;
        // What happens, or how it went.
        let hint: Option<(String, mk::Rgba)> = match (&self.task, self.outcome) {
            (Some(Task::Backup), _) => Some((String::from(l("Matrix wird gesichert — Kabel nicht ziehen.", "Backing up Matrix — do not unplug.")), quiet)),
            (Some(Task::Restore(_)), _) | (Some(Task::Reinstall), _) => Some((String::from(l("Bitte auf dem Duo bestätigen, dann nicht abstecken.", "Please confirm on the duo, then do not unplug.")), p.tertiary)),
            (Some(Task::Start), _) => Some((String::from(l("Das Duo startet Matrix von seiner Partition …", "The duo starts Matrix from its partition …")), quiet)),
            (None, Some(true)) => Some((String::from(l("Erledigt.", "Done.")), p.primary)),
            (None, Some(false)) => Some((String::from(l("Nicht geschafft — Protokoll unten. Auf dem Duo hat sich nichts geändert, wenn es nicht bestätigt wurde.", "Did not work — see the log. Nothing changed on the duo unless it was confirmed there.")), p.error)),
            (None, None) => Some((String::from(l("«Neu installieren» setzt nur Matrix neu auf; Android bleibt. Vorher sichern, danach wiederherstellen.", "«Reinstall» only renews Matrix; Android stays. Back up first, restore after.")), quiet)),
        };
        if let Some((text, ink)) = hint {
            for line in mk::wrap_words(&text, ((w / (mk::typo::CAPTION.px() * 0.52)) as usize).max(30), 3) {
                b.styled(engine, &line, mk::typo::CAPTION, ink, m, y, None, None);
                y += 17.0;
            }
            y += 8.0;
        }
        // This duo's backups.
        b.styled(engine, l("Sicherungen auf diesem Laptop", "Backups on this laptop"), mk::typo::LABEL, p.on_surface, m, y, None, None);
        y += 24.0;
        if self.saved.is_empty() {
            b.styled(engine, l("Noch keine Matrix-Sicherung von diesem Duo.", "No Matrix backup of this duo yet."), mk::typo::CAPTION, quiet, m, y, None, None);
            y += 24.0;
        }
        let row = 44.0;
        for (k, s) in self.saved.iter().take(SHOWN_BACKUPS).enumerate() {
            b.quad(m, y, w, row - 6.0, p.surface_container, 10.0, None, None);
            let label = l("Wiederherstellen", "Restore");
            let pw = fam::pill_width(engine, Some(symbol::DOWNLOAD), label);
            let what = format!("{} · {} MB · {}", s.when, s.mb, if s.accounts.is_empty() { "—" } else { &s.accounts });
            b.styled(engine, &what, mk::typo::BODY, p.on_surface, m + 14.0, y + 9.0, None, Some(Rect { x: m, y, w: w - pw - 20.0, h: row }));
            let (role, tone) = if self.running { (None, fam::PillTone::Quiet) } else { (Some(REC_RESTORE + k), fam::PillTone::Quiet) };
            fam::lay_pill(b, engine, Some(symbol::DOWNLOAD), label, m + w - pw - 4.0, y + 1.0, None, role, tone, p);
            y += row;
        }
        y + 6.0
    }

    /// The first page: language and keyboard — everything after speaks it,
    /// on the laptop and on the duo (passed on to the live system).
    fn lay_language(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, wf: f32, top: f32) {
        let p = self.palette;
        let m = 24.0;
        let w = wf - 2.0 * m;
        let mut y = top + m + 12.0;
        b.styled(engine, "Sprache · Language", mk::typo::LARGE_TITLE, p.on_surface, m, y, None, None);
        y += mk::typo::LARGE_TITLE.px() * 1.6;
        for line in mk::wrap_words(l("Morpheus, das Duo und Matrix sprechen danach diese Sprache.", "Morpheus, the Duo and Matrix will then speak this language."), 60, 3) {
            b.styled(engine, &line, mk::typo::BODY, p.on_surface_variant, m, y, None, None);
            y += 20.0;
        }
        y += 14.0;
        let langs: Vec<String> = LANGS.iter().map(|(_, n)| n.to_string()).collect();
        y += fam::lay_choices(b, engine, &langs, LANGS.iter().position(|(c, _)| *c == self.lang), m, y, w, LANG_BASE, p) + 18.0;
        b.styled(engine, l("Tastatur", "Keyboard"), mk::typo::CAPTION, p.on_surface_variant, m, y, None, None);
        y += 20.0;
        let kbds: Vec<String> = KBDS.iter().map(|(_, de, en)| l(de, en).to_string()).collect();
        y += fam::lay_choices(b, engine, &kbds, KBDS.iter().position(|(c, _, _)| *c == self.kbd), m, y, w, KBD_BASE, p) + 24.0;
        fam::lay_pill(b, engine, Some('▶'), l("Weiter", "Continue"), m, y, None, Some(LANG_OK), fam::PillTone::Accent, p);
    }

    /// The software page (start, install, recovery) in the content area right of the sidebar.
    fn lay(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, x0: f32, width: f32, hf: f32, top: f32) {
        let p = self.palette;
        let m = x0 + 32.0;
        let w = width - 64.0;
        let mv = 24.0;
        let mut y = top + mv;
        // The way: start or install — and the language, beside it.
        // The language, with the symbol font's «language» globe.
        const GLOBE: char = '\u{e894}';
        let lang_label = LANGS.iter().find(|(c, _)| *c == self.lang).map(|(_, n)| n.to_string()).unwrap_or_default();
        let lw = fam::pill_width(engine, Some(GLOBE), &lang_label);
        let ways = vec![String::from(l("Matrix starten", "Start Matrix")), String::from(l("Matrix installieren", "Install Matrix"))];
        let active = Some(if self.way == Way::Start { 0 } else { 1 });
        let _ = lw;
        let wh = fam::lay_choices(b, engine, &ways, active, m, y, w, WAY_BASE, p);
        y += wh + 18.0;
        // The duo's state: a dot and a word.
        let title = mk::typo::TITLE;
        b.styled(engine, "Surface Duo", title, p.on_surface, m, y, None, None);
        y += title.px() * 1.5;
        let (dot, word) = match self.link {
            Link::Fedora if self.device.as_ref().is_some_and(|d| d.matrix) => (p.primary, l("Im Recovery (Live-System, nur im Arbeitsspeicher)", "In recovery (live system, memory only)")),
            Link::Fedora => (p.primary, if self.way == Way::Install && self.running { l("Live-System läuft", "Live system running") } else { l("Matrix läuft", "Matrix running") }),
            Link::None => (p.outline, l(Link::None.says(), "No Duo connected")),
            Link::Android => (p.tertiary, l(Link::Android.says(), "Android running")),
            Link::Bootloader => (p.tertiary, l(Link::Bootloader.says(), "In the bootloader")),
            Link::Stage1 => (p.tertiary, l(Link::Stage1.says(), "Stage 1 starting")),
            Link::NoAccess => (p.error, l(Link::NoAccess.says(), "No access to the Duo — see Overview")),
            Link::Unauthorized => (p.tertiary, l(Link::Unauthorized.says(), "Confirm USB debugging on the Duo")),
        };
        b.quad(m, y + 5.0, 12.0, 12.0, dot, 6.0, None, None);
        b.styled(engine, word, mk::typo::BODY, p.on_surface_variant, m + 22.0, y, None, None);
        y += 36.0;
        if let Some(dev) = self.device.clone().filter(|d| d.matrix) {
            y = self.lay_recovery(b, engine, &dev, m, w, y);
        } else {
            // The steps.
            let row = 38.0;
            for (k, label) in self.labels().iter().enumerate() {
                let (color, says) = match self.steps.get(k).copied().unwrap_or(State::Waiting) {
                    State::Waiting => (p.outline, if self.way == Way::Install && k == 4 && self.outcome.is_some() { l("übersprungen", "skipped") } else { "" }),
                    State::Running => (p.tertiary, l("läuft …", "running …")),
                    State::Done => (p.primary, l("fertig", "done")),
                    State::Failed => (p.error, l("Fehler", "failed")),
                };
                b.quad(m, y, w, row - 6.0, p.surface_container, 10.0, None, None);
                b.quad(m + 14.0, y + (row - 6.0) / 2.0 - 6.0, 12.0, 12.0, color, 6.0, None, None);
                b.styled(engine, &format!("{}. {}", k + 1, step_en(label)), mk::typo::BODY, p.on_surface, m + 38.0, y + 6.0, None, None);
                if !says.is_empty() {
                    let (sw, _) = engine.measure_styled(says, mk::typo::CAPTION.px(), mk::typo::CAPTION.weight);
                    b.styled(engine, says, mk::typo::CAPTION, color, m + w - 14.0 - sw, y + 9.0, None, None);
                }
                y += row;
            }
            y += 10.0;
            // The buttons.
            let missing = self.missing();
            let ready = missing.is_none() && (self.link.can_start() || self.start_from_live());
            let (word, glyph) = match self.way {
                Way::Start => (l("Matrix starten", "Start Matrix"), '▶'),
                Way::Install => (if self.dry { l("Trockenlauf beginnen", "Begin dry run") } else { l("Installation beginnen", "Begin installation") }, '▶'),
            };
            let (label, tone, role) = if self.running {
                (if self.way == Way::Install { l("Läuft …", "Running …") } else { l("Startet …", "Starting …") }, fam::PillTone::Quiet, None)
            } else if ready {
                (word, fam::PillTone::Accent, Some(GO))
            } else {
                (word, fam::PillTone::Quiet, None)
            };
            let (bw, bh) = fam::lay_pill(b, engine, Some(glyph), label, m, y, None, role, tone, p);
            let mut bx = m + bw + 12.0;
            if self.way == Way::Install && !self.running {
                let dry = if self.dry { l("Trockenlauf: an", "Dry run: on") } else { l("Trockenlauf: aus", "Dry run: off") };
                let (dw, _) = fam::lay_pill(b, engine, None, dry, bx, y, None, Some(DRY), if self.dry { fam::PillTone::Accent } else { fam::PillTone::Quiet }, p);
                bx += dw + 12.0;
            }
            if self.link == Link::Bootloader && !self.running {
                fam::lay_pill(b, engine, None, l("Android starten", "Start Android"), bx, y, None, Some(ANDROID), fam::PillTone::Quiet, p);
            }
            y += bh + 14.0;
            // Why the button sleeps, or how it went.
            let mut hints: Vec<(String, mk::Rgba)> = Vec::new();
            let quiet = p.on_surface_variant;
            if let Some(why) = missing {
                hints.push((why, p.error));
            } else if self.locked {
                for (de, en) in UNLOCK {
                    hints.push((l(de, en).to_string(), quiet));
                }
            } else if self.running {
                let text = match self.way {
                    Way::Start => String::from(l("Duo nicht abstecken, bis Matrix sich meldet.", "Do not unplug the Duo until Matrix answers.")),
                    Way::Install if self.steps.get(3) == Some(&State::Running) || self.steps.get(4) == Some(&State::Running) || self.steps.get(5) == Some(&State::Running) => {
                        String::from(l("Bitte auf dem Duo weitermachen — Morpheus liefert, was es braucht. Kabel nicht ziehen.", "Please continue on the Duo — Morpheus delivers what it needs. Do not unplug the cable."))
                    }
                    Way::Install => String::from(l("Das Live-System startet (nur im Arbeitsspeicher). Kabel nicht ziehen.", "The live system is starting (in memory only). Do not unplug the cable.")),
                };
                hints.push((text, quiet));
            } else if self.outcome == Some(true) {
                hints.push((
                    String::from(match self.way {
                        Way::Start => l("Fertig — bitte auf dem Duo anmelden.", "Done — please log in on the Duo."),
                        Way::Install => l("Installiert. Jetzt «Matrix starten» — das Duo startet von seiner Partition.", "Installed. Now «Start Matrix» — the Duo starts from its partition."),
                    }),
                    quiet,
                ));
            } else if self.outcome == Some(false) {
                hints.push((String::from(l("Nicht geschafft — Protokoll unten. Erneut versuchen geht.", "Did not work — see the log below. You can try again.")), p.error));
            } else if !self.link.can_start() && self.link != Link::Fedora {
                hints.push((String::from(l("Duo per USB anstecken (Android oder Bootloader).", "Plug in the Duo over USB (Android or bootloader).")), quiet));
            } else if self.way == Way::Install {
                hints.push((String::from(l("Für ein NEUES Duo: Android wird zurückgesetzt, die Wahl triffst du auf dem Duo — mit Sicherung vorher.", "For a NEW Duo: Android is reset; you decide on the Duo — with a backup first.")), quiet));
            }
            for (text, color) in hints {
                for l in mk::wrap_words(&text, ((w / (mk::typo::CAPTION.px() * 0.52)) as usize).max(30), 3) {
                    b.styled(engine, &l, mk::typo::CAPTION, color, m, y, None, None);
                    y += 17.0;
                }
                y += 4.0;
            }
            y += 6.0;
        }
        // The scripts' last lines.
        let foot = 44.0;
        let log_h = (hf - y - foot - mv).max(0.0);
        if log_h > 40.0 {
            b.quad(m, y, w, log_h, p.surface_container_high, 12.0, None, None);
            let line_h = 18.0;
            let n = ((log_h - 16.0) / line_h).floor().max(0.0) as usize;
            let from = self.log.len().saturating_sub(n);
            let clip = Some(Rect { x: m, y, w, h: log_h });
            for (k, line) in self.log[from..].iter().enumerate() {
                b.text(engine, line.clone(), 12.5, mk::typo::Weight::Normal, matrixkit_text::FAMILY, p.on_surface_variant, m + 12.0, y + 8.0 + k as f32 * line_h, None, clip);
            }
        }
        // What it does — and does not.
        let note = if self.device.as_ref().is_some_and(|d| d.matrix) {
            String::from(l("Geschrieben wird nur auf die Matrix-Partition, nach deiner Bestätigung auf dem Duo.", "Only the Matrix partition is written, after you confirm on the duo."))
        } else {
            match self.way {
            Way::Start => String::from(l("Nur ein RAM-Start: auf dem Duo wird nichts geschrieben, Android bleibt Standard.", "Only a RAM start: nothing is written on the Duo, Android stays the default.")),
            Way::Install => String::from(l("Geschrieben wird nur auf dem Duo, von seinem Installer, nach deiner Bestätigung dort.", "Only the Duo's installer writes, on the Duo, after you confirm there.")),
            }
        };
        b.styled(engine, &note, mk::typo::CAPTION, p.outline, m, hf - mv - 18.0, None, Some(Rect { x: m, y: hf - foot - mv, w, h: foot + mv }));
    }
}

impl shell::Client for Morpheus {
    type Card = ();
    type Work = ();

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn view(&mut self, engine: &mut TextEngine, _keep: &mut Keep<f32>, _atlas: &mut ImageAtlas, w: u32, h: u32) -> Scene {
        let p = self.palette;
        let (wf, hf) = (w as f32, h as f32);
        self.width = wf;
        let mut b = SceneBuilder::new();
        b.quad(0.0, 0.0, wf, hf, p.surface, 0.0, None, None);
        let top = self.chrome.build(engine, &mut b, w, h, p);
        if !self.chrome.rolled {
            if self.asking_lang {
                self.lay_language(&mut b, engine, wf, top);
            } else {
                self.lay_side(&mut b, engine, top, hf);
                self.lay_page(&mut b, engine, wf, hf, top);
            }
        }
        b.finish()
    }

    fn animating(&self) -> bool {
        self.chrome.animating()
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        if self.chrome.glide(dt) {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
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

    fn strike(&mut self, role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        match self.chrome.strike(role) {
            Some(fam::ChromeDeed::Close) => return Refresh::Close,
            Some(fam::ChromeDeed::Title) => return Refresh::Calm,
            Some(fam::ChromeDeed::Answer(r)) => return r,
            None => {}
        }
        if let Some(r) = role {
            if !self.asking_lang {
                if let Some(done) = self.strike_page(r) {
                    return done;
                }
            }
        }
        match role {
            Some(GO) => self.go(),
            Some(ANDROID) => self.android(),
            Some(DRY) if !self.running => {
                mk::feel::tap();
                self.dry = !self.dry;
                Refresh::Dirty
            }
            Some(r) if (LANG_BASE..LANG_BASE + LANGS.len()).contains(&r) => {
                mk::feel::tap();
                self.lang = LANGS[r - LANG_BASE].0;
                self.kbd = if self.lang == "en-GB" { "gb" } else { "ch" };
                mk::language::choose(self.lang);
                Refresh::Dirty
            }
            Some(r) if (KBD_BASE..KBD_BASE + KBDS.len()).contains(&r) => {
                mk::feel::tap();
                self.kbd = KBDS[r - KBD_BASE].0;
                Refresh::Dirty
            }
            Some(LANG_OK) => {
                mk::feel::tap();
                let f = choice_file();
                if let Some(dir) = f.parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                let _ = std::fs::write(&f, format!("{}\n{}\n", self.lang, self.kbd));
                self.asking_lang = false;
                Refresh::Dirty
            }
            Some(LANG_OPEN) if !self.running => {
                mk::feel::tap();
                self.asking_lang = true;
                Refresh::Dirty
            }
            Some(REC_START) => self.recover(Task::Start),
            Some(REC_BACKUP) => self.recover(Task::Backup),
            Some(REC_NEW) => self.recover(Task::Reinstall),
            Some(r) if (REC_RESTORE..REC_RESTORE + SHOWN_BACKUPS).contains(&r) => match self.saved.get(r - REC_RESTORE) {
                Some(s) => {
                    let dir = s.dir.clone();
                    self.recover(Task::Restore(dir))
                }
                None => Refresh::Calm,
            },
            Some(r) if r == WAY_BASE => self.choose(Way::Start),
            Some(r) if r == WAY_BASE + 1 => self.choose(Way::Install),
            _ => Refresh::Calm,
        }
    }

    fn axis(&mut self, x: f64, _y: f64, value: f64, _keep: &mut Keep<f32>) -> Refresh {
        let w = self.width;
        self.axis_page(x, value, w)
    }

    fn dropped(&mut self, paths: Vec<PathBuf>, _x: f64, _y: f64) -> Refresh {
        self.dropped_files(paths)
    }

    fn depart(&mut self) -> Refresh {
        self.chrome.depart()
    }

    fn motion(&mut self, x: f64, y: f64) -> Refresh {
        self.chrome.motion(x, y)
    }

    fn beat(&mut self) -> Refresh {
        if self.take() {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }
}

fn main() {
    // Offline (06.11.2026): the tools Morpheus carries (fastboot, adb, nc, telnet) beside this binary come first —
    // for the app's own device check and for every script it starts.
    if let Some(dir) = exe_dir() {
        let path = std::env::var_os("PATH").unwrap_or_default();
        let mut parts = vec![dir];
        parts.extend(std::env::split_paths(&path));
        if let Ok(joined) = std::env::join_paths(parts) {
            std::env::set_var("PATH", joined);
        }
    }
    if let Err(e) = shell::run(
        Morpheus::new(),
        shell::Options {
            mount: shell::Mount::Window { app_id: String::from(APP_ID), title: String::from("Morpheus"), width: 1040, height: 740 },
            argb: false,
            tag: "morpheus",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    ) {
        eprintln!("[morpheus] {e}");
        std::process::exit(1);
    }
}
