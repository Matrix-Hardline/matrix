//! MORPHEUS, THE COMPANION (06.11.2026, the owner: «the iTunes of Matrix»):
//! a sidebar with the connected duo on top and five pages — overview,
//! files, photos, backups, software (the start/install/recovery ways).
//! The work runs on threads (`companion`); results come back through the
//! inbox and are taken on the next beat.

use super::*;
use crate::companion::{self as co, Entry, Hello, Info, Photo};

pub const SIDE_W: f32 = 252.0;

const NAV_BASE: usize = 300;
const OV_PAIR: usize = 320;
const OV_START: usize = 321;
const OV_BACKUPS: usize = 322;
const OV_ACCESS: usize = 323;
const F_UP_DUO: usize = 330;
const F_UP_LAP: usize = 331;
const F_TO_LAP: usize = 332;
const F_TO_DUO: usize = 333;
const F_RELOAD: usize = 334;
const P_LOAD: usize = 340;
const P_ALL: usize = 341;
const P_BACK: usize = 342;
const P_SAVE: usize = 343;
const B_NOW: usize = 350;
const B_RESTORE: usize = 4000;
const DUO_ROW: usize = 5000;
const LAP_ROW: usize = 6000;
const PHOTO: usize = 7000;
const MAX_ROWS: usize = 999;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Overview,
    Files,
    Photos,
    Backups,
    Software,
}

const PAGES: [Page; 5] = [Page::Overview, Page::Files, Page::Photos, Page::Backups, Page::Software];

impl Page {
    fn title(self) -> &'static str {
        match self {
            Page::Overview => l("Übersicht", "Overview"),
            Page::Files => l("Dateien", "Files"),
            Page::Photos => l("Fotos", "Photos"),
            Page::Backups => l("Sicherungen", "Backups"),
            Page::Software => l("Software", "Software"),
        }
    }

    fn glyph(self) -> char {
        match self {
            Page::Overview => symbol::SMARTPHONE,
            Page::Files => symbol::FOLDER,
            Page::Photos => symbol::IMAGE,
            Page::Backups => symbol::SAVE,
            Page::Software => symbol::DOWNLOAD,
        }
    }
}

/// What a worker thread brings back.
pub enum Deliver {
    Info(Result<Info, String>),
    DuoList(Result<(String, Vec<Entry>), String>),
    Photos(Result<Vec<Photo>, String>),
    Opened(Result<PathBuf, String>),
    Paired(Result<Hello, String>),
    Done(Result<String, String>),
    /// «Zugriff erlauben»: the udev rule installed (or why not).
    Access(Result<(), String>),
}

/// The udev rule newer Linux releases bring (systemd >= 258 tags adb and
/// fastboot interfaces `uaccess`); on older ones Morpheus installs it once.
/// The Duo's own ids as a fallback (Android 045e:0c26/0c29, bootloader 0c2f).
const UDEV_RULE: &str = "# Morpheus (Matrix für das Surface Duo): Android-Geräte am USB für die angemeldete Person — wie systemd >= 258.
SUBSYSTEM==\"usb\", ENV{DEVTYPE}==\"usb_device\", ENV{ID_USB_INTERFACES}==\"*:ff4201:*\", TAG+=\"uaccess\"
SUBSYSTEM==\"usb\", ENV{DEVTYPE}==\"usb_device\", ENV{ID_USB_INTERFACES}==\"*:ff4203:*\", TAG+=\"uaccess\"
SUBSYSTEM==\"usb\", ATTR{idVendor}==\"045e\", ATTR{idProduct}==\"0c26|0c29|0c2f\", MODE=\"0666\", TAG+=\"uaccess\"
";

/// Install the rule with pkexec (Linux asks for the password), reload udev
/// and let adb start afresh. Blocking — runs on a worker thread.
fn allow_access() -> Result<(), String> {
    let script = "set -e; cat > /etc/udev/rules.d/51-morpheus-duo.rules; chmod 644 /etc/udev/rules.d/51-morpheus-duo.rules; udevadm control --reload-rules; udevadm trigger --subsystem-match=usb --action=change";
    let mut child = std::process::Command::new("pkexec")
        .args(["sh", "-c", script])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("{}: {e}", l("pkexec fehlt — als root ausführen", "pkexec missing — run as root")))?;
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(UDEV_RULE.as_bytes()).map_err(|e| e.to_string())?;
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(String::from(l("Zugriff nicht eingerichtet (Passwort abgebrochen?)", "Access not set up (password cancelled?)")));
    }
    let _ = std::process::Command::new("adb").arg("kill-server").status();
    Ok(())
}

#[derive(Default)]
pub struct Files {
    pub duo_dir: String,
    pub duo: Vec<Entry>,
    pub laptop_dir: PathBuf,
    pub laptop: Vec<Entry>,
    pub sel_duo: Option<usize>,
    pub sel_laptop: Option<usize>,
    pub roll_duo: f32,
    pub roll_laptop: f32,
    pub max_duo: f32,
    pub max_laptop: f32,
    pub loaded: bool,
}

#[derive(Default)]
pub struct Photos {
    pub items: Vec<Photo>,
    pub roll: f32,
    pub max: f32,
    pub open: Option<usize>,
    pub full: Option<PathBuf>,
    pub loaded: bool,
}

/// The laptop's pictures folder (xdg), for photos from the duo.
fn pictures_dir() -> PathBuf {
    let xdg = std::process::Command::new("xdg-user-dir").arg("PICTURES").output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).filter(|s| !s.is_empty());
    xdg.map(PathBuf::from).unwrap_or_else(|| home().join("Bilder")).join("Surface Duo")
}

/// The full backups (all partitions) of one duo — shown, never restored by Morpheus.
fn full_backups(serial: &str) -> Vec<(String, u64)> {
    let base = home().join("Sicherungen").join(format!("surface-duo-{serial}"));
    let mut out: Vec<(String, u64)> = std::fs::read_dir(base)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("vollsicherung-"))
        .map(|e| {
            let size = std::fs::read_dir(e.path().join("partitionen")).into_iter().flatten().flatten().filter_map(|f| f.metadata().ok()).map(|m| m.len()).sum();
            (e.file_name().to_string_lossy().trim_start_matches("vollsicherung-").to_string(), size)
        })
        .collect();
    out.sort_by(|a, b| b.0.cmp(&a.0));
    out
}

impl Morpheus {
    /// The duo's serial, from whatever knows it (Matrix, recovery).
    pub fn serial(&self) -> String {
        self.info
            .as_ref()
            .map(|i| i.serial.clone())
            .or_else(|| self.hello.as_ref().map(|h| h.serial.clone()))
            .or_else(|| self.device.as_ref().map(|d| d.serial.clone()))
            .unwrap_or_default()
    }

    fn account(&self) -> Option<String> {
        self.hello.as_ref().filter(|h| h.paired).map(|h| h.account.clone())
    }

    /// Run `f` on a worker thread; its result lands in the inbox.
    fn work(&mut self, what: &str, f: impl FnOnce() -> Deliver + Send + 'static) {
        self.job = Some(what.to_string());
        self.note = None;
        let inbox = self.inbox.clone();
        std::thread::spawn(move || {
            let d = f();
            if let Ok(mut i) = inbox.lock() {
                i.push(d);
            }
        });
    }

    /// A new hello from the watcher: a paired duo loads its info and home folder.
    pub fn greet(&mut self, hello: Option<Hello>) {
        let paired_now = hello.as_ref().is_some_and(|h| h.paired);
        let was = self.hello.as_ref().is_some_and(|h| h.paired);
        self.hello = hello;
        if !paired_now {
            self.info = None;
            self.files = Files { laptop_dir: self.files.laptop_dir.clone(), ..Files::default() };
            self.photos = Photos::default();
            return;
        }
        if !was {
            self.load_info();
            self.saved = backups_of(&self.serial());
        }
    }

    fn load_info(&mut self) {
        if let Some(acc) = self.account() {
            self.work(l("Lese das Duo …", "Reading the duo …"), move || Deliver::Info(co::info(&acc)));
        }
    }

    fn load_duo(&mut self, dir: String) {
        if let Some(acc) = self.account() {
            self.work(l("Lese den Ordner …", "Reading the folder …"), move || Deliver::DuoList(co::list_duo(&acc, &dir)));
        }
    }

    fn load_laptop(&mut self) {
        if self.files.laptop_dir.as_os_str().is_empty() {
            self.files.laptop_dir = home();
        }
        self.files.laptop = co::list_laptop(&self.files.laptop_dir);
        self.files.sel_laptop = None;
        self.files.roll_laptop = 0.0;
    }

    fn load_photos(&mut self) {
        if let Some(acc) = self.account() {
            let serial = self.serial();
            self.work(l("Lade die Vorschaubilder …", "Loading the previews …"), move || Deliver::Photos(co::photos(&acc, &serial)));
        }
    }

    /// Take what the workers brought.
    pub fn take_inbox(&mut self) -> bool {
        let items: Vec<Deliver> = match self.inbox.lock() {
            Ok(mut i) => i.drain(..).collect(),
            Err(_) => return false,
        };
        let moved = !items.is_empty();
        for d in items {
            self.job = None;
            match d {
                Deliver::Info(Ok(i)) => self.info = Some(i),
                Deliver::Info(Err(e)) => self.note = Some((false, e)),
                Deliver::DuoList(Ok((dir, entries))) => {
                    self.files.duo_dir = dir;
                    self.files.duo = entries;
                    self.files.sel_duo = None;
                    self.files.roll_duo = 0.0;
                    self.files.loaded = true;
                }
                Deliver::DuoList(Err(e)) => self.note = Some((false, e)),
                Deliver::Photos(Ok(p)) => {
                    self.photos.items = p;
                    self.photos.loaded = true;
                    self.photos.roll = 0.0;
                }
                Deliver::Photos(Err(e)) => self.note = Some((false, e)),
                Deliver::Opened(Ok(p)) => self.photos.full = Some(p),
                Deliver::Opened(Err(e)) => self.note = Some((false, e)),
                Deliver::Paired(Ok(h)) => {
                    self.pairing = false;
                    mk::feel::confirm();
                    self.greet(Some(h));
                }
                Deliver::Paired(Err(e)) => {
                    self.pairing = false;
                    self.note = Some((false, e));
                }
                Deliver::Access(r) => {
                    self.access_busy = false;
                    self.note = Some(match r {
                        Ok(()) => (true, String::from(l("Zugriff eingerichtet — jetzt das Kabel kurz aus- und wieder einstecken.", "Access set up — now unplug the cable and plug it in again."))),
                        Err(e) => (false, e),
                    });
                }
                Deliver::Done(r) => {
                    let ok = r.is_ok();
                    self.note = Some((ok, r.unwrap_or_else(|e| e)));
                    if ok {
                        mk::feel::confirm();
                    }
                    // Both sides may have changed.
                    let dir = self.files.duo_dir.clone();
                    if self.page == Page::Files {
                        self.load_laptop();
                        self.load_duo(dir);
                    }
                }
            }
        }
        moved
    }

    // ------------------------------------------------------------- deeds

    pub fn strike_page(&mut self, role: usize) -> Option<Refresh> {
        let busy = self.job.is_some() || self.running;
        if (NAV_BASE..NAV_BASE + PAGES.len()).contains(&role) {
            mk::feel::tap();
            self.page = PAGES[role - NAV_BASE];
            self.note = None;
            match self.page {
                Page::Files if !self.files.loaded && !busy => {
                    self.load_laptop();
                    self.load_duo(String::new());
                }
                Page::Photos if !self.photos.loaded && !busy => self.load_photos(),
                Page::Backups => self.saved = backups_of(&self.serial()),
                _ => {}
            }
            return Some(Refresh::Dirty);
        }
        let r = match role {
            OV_PAIR if !self.pairing => {
                mk::feel::tap();
                self.pairing = true;
                self.note = None;
                let inbox = self.inbox.clone();
                std::thread::spawn(move || {
                    let r = co::pair();
                    if let Ok(mut i) = inbox.lock() {
                        i.push(Deliver::Paired(r));
                    }
                });
                Refresh::Dirty
            }
            OV_START => {
                self.page = Page::Software;
                self.choose(Way::Start);
                Refresh::Dirty
            }
            OV_ACCESS if !self.access_busy => {
                self.access_busy = true;
                let inbox = self.inbox.clone();
                std::thread::spawn(move || {
                    let r = allow_access();
                    if let Ok(mut i) = inbox.lock() {
                        i.push(Deliver::Access(r));
                    }
                });
                Refresh::Dirty
            }
            OV_BACKUPS => {
                self.page = Page::Backups;
                self.saved = backups_of(&self.serial());
                Refresh::Dirty
            }
            F_RELOAD if !busy => {
                self.load_laptop();
                let dir = self.files.duo_dir.clone();
                self.load_duo(dir);
                Refresh::Dirty
            }
            F_UP_DUO if !busy => {
                let up = self.files.duo_dir.rsplit_once('/').map(|(a, _)| a.to_string()).unwrap_or_default();
                self.load_duo(up);
                Refresh::Dirty
            }
            F_UP_LAP => {
                if let Some(parent) = self.files.laptop_dir.parent() {
                    self.files.laptop_dir = parent.to_path_buf();
                    self.load_laptop();
                }
                Refresh::Dirty
            }
            r if (DUO_ROW..DUO_ROW + MAX_ROWS).contains(&r) && !busy => {
                let k = r - DUO_ROW;
                let Some(e) = self.files.duo.get(k).cloned() else { return Some(Refresh::Calm) };
                if e.dir && self.files.sel_duo == Some(k) {
                    let dir = if self.files.duo_dir.is_empty() { e.name } else { format!("{}/{}", self.files.duo_dir, e.name) };
                    self.load_duo(dir);
                } else {
                    mk::feel::tap();
                    self.files.sel_duo = Some(k);
                }
                Refresh::Dirty
            }
            r if (LAP_ROW..LAP_ROW + MAX_ROWS).contains(&r) => {
                let k = r - LAP_ROW;
                let Some(e) = self.files.laptop.get(k).cloned() else { return Some(Refresh::Calm) };
                if e.dir && self.files.sel_laptop == Some(k) {
                    self.files.laptop_dir = self.files.laptop_dir.join(&e.name);
                    self.load_laptop();
                } else {
                    mk::feel::tap();
                    self.files.sel_laptop = Some(k);
                }
                Refresh::Dirty
            }
            F_TO_LAP if !busy => {
                let (Some(acc), Some(e)) = (self.account(), self.files.sel_duo.and_then(|k| self.files.duo.get(k).cloned())) else { return Some(Refresh::Calm) };
                let (dir, into) = (self.files.duo_dir.clone(), self.files.laptop_dir.clone());
                self.work(l("Kopiere auf den Laptop …", "Copying to the laptop …"), move || {
                    Deliver::Done(co::copy_to_laptop(&acc, &dir, &e, &into).map(|p| format!("{} {}", l("Auf den Laptop kopiert:", "Copied to the laptop:"), p.display())))
                });
                Refresh::Dirty
            }
            F_TO_DUO if !busy => {
                let (Some(acc), Some(e)) = (self.account(), self.files.sel_laptop.and_then(|k| self.files.laptop.get(k).cloned())) else { return Some(Refresh::Calm) };
                let (local, dir) = (self.files.laptop_dir.join(&e.name), self.files.duo_dir.clone());
                self.work(l("Kopiere aufs Duo …", "Copying to the duo …"), move || {
                    Deliver::Done(co::copy_to_duo(&acc, &local, &dir).map(|n| format!("{} {n}", l("Aufs Duo kopiert:", "Copied to the duo:"))))
                });
                Refresh::Dirty
            }
            P_LOAD if !busy => {
                self.load_photos();
                Refresh::Dirty
            }
            r if (PHOTO..PHOTO + MAX_ROWS * 4).contains(&r) && !busy => {
                let k = r - PHOTO;
                let (Some(acc), Some(ph)) = (self.account(), self.photos.items.get(k).cloned()) else { return Some(Refresh::Calm) };
                mk::feel::tap();
                self.photos.open = Some(k);
                self.photos.full = None;
                let serial = self.serial();
                self.work(l("Lade das Foto …", "Loading the photo …"), move || Deliver::Opened(co::open_photo(&acc, &serial, &ph)));
                Refresh::Dirty
            }
            P_BACK => {
                self.photos.open = None;
                self.photos.full = None;
                Refresh::Dirty
            }
            P_SAVE if !busy => {
                let (Some(acc), Some(ph)) = (self.account(), self.photos.open.and_then(|k| self.photos.items.get(k).cloned())) else { return Some(Refresh::Calm) };
                let into = pictures_dir();
                self.work(l("Speichere das Foto …", "Saving the photo …"), move || {
                    let _ = std::fs::create_dir_all(&into);
                    let (dir, name) = ph.path.rsplit_once('/').map(|(d, n)| (d.to_string(), n.to_string())).unwrap_or((String::new(), ph.path.clone()));
                    let e = Entry { name, dir: false, size: ph.size };
                    Deliver::Done(co::copy_to_laptop(&acc, &dir, &e, &into).map(|p| format!("{} {}", l("Gespeichert:", "Saved:"), p.display())))
                });
                Refresh::Dirty
            }
            P_ALL if !busy => {
                let Some(acc) = self.account() else { return Some(Refresh::Calm) };
                let items = self.photos.items.clone();
                let into = pictures_dir();
                self.work(l("Hole alle Fotos …", "Fetching all photos …"), move || {
                    let _ = std::fs::create_dir_all(&into);
                    let (mut got, mut skipped) = (0, 0);
                    for ph in items {
                        let (dir, name) = ph.path.rsplit_once('/').map(|(d, n)| (d.to_string(), n.to_string())).unwrap_or((String::new(), ph.path.clone()));
                        // Already here with the same size: not again.
                        if std::fs::metadata(into.join(&name)).map(|m| m.len() == ph.size).unwrap_or(false) {
                            skipped += 1;
                            continue;
                        }
                        if co::copy_to_laptop(&acc, &dir, &Entry { name, dir: false, size: ph.size }, &into).is_ok() {
                            got += 1;
                        }
                    }
                    Deliver::Done(Ok(format!("{got} {} {} ({skipped} {})", l("Fotos nach", "photos to"), into.display(), l("schon da", "already there"))))
                });
                Refresh::Dirty
            }
            B_NOW if !busy => self.backup_now(),
            r if (B_RESTORE..B_RESTORE + 50).contains(&r) && !busy => match self.saved.get(r - B_RESTORE) {
                Some(s) => {
                    let dir = s.dir.clone();
                    self.restore(dir)
                }
                None => Refresh::Calm,
            },
            _ => return None,
        };
        Some(r)
    }

    /// «Jetzt sichern»: in Recovery the live way; with Matrix running and paired, over the cable.
    fn backup_now(&mut self) -> Refresh {
        if self.device.as_ref().is_some_and(|d| d.matrix) {
            return self.recover(Task::Backup);
        }
        let (Some(acc), Some(script)) = (self.account(), find_script("morpheus-recovery.sh")) else { return Refresh::Calm };
        mk::feel::tap();
        self.running = true;
        self.outcome = None;
        self.log.clear();
        self.task = Some(Task::Backup);
        self.spawn(format!("exec bash '{}' sichern-betrieb '{acc}' 2>&1", script.display()), vec![]);
        Refresh::Dirty
    }

    /// Restore needs Matrix Recovery: in it, straight away; from a running
    /// Matrix, the duo goes to its bootloader, the live system starts, and
    /// the restore waits until Recovery is there.
    fn restore(&mut self, dir: PathBuf) -> Refresh {
        if self.device.as_ref().is_some_and(|d| d.matrix) {
            return self.recover(Task::Restore(dir));
        }
        let Some(acc) = self.account() else { return Refresh::Calm };
        mk::feel::tap();
        self.pending_restore = Some(dir);
        co::to_bootloader(&acc);
        self.hello = None;
        self.info = None;
        self.way = Way::Install;
        self.note = Some((true, String::from(l("Das Duo startet in Matrix Recovery — danach wird wiederhergestellt (Bestätigung auf dem Duo).", "The duo starts into Matrix Recovery — then the restore follows (confirm on the duo)."))));
        self.go()
    }

    /// After Recovery came up: a restore that was waiting for it.
    pub fn resume_restore(&mut self) {
        if let Some(dir) = self.pending_restore.take() {
            if !self.running {
                self.recover(Task::Restore(dir));
            }
        }
    }

    pub fn axis_page(&mut self, x: f64, value: f64, w: f32) -> Refresh {
        let d = value as f32 * 1.2;
        match self.page {
            Page::Files => {
                let mid = SIDE_W + (w - SIDE_W) / 2.0;
                if (x as f32) < mid {
                    self.files.roll_duo = (self.files.roll_duo + d).clamp(0.0, self.files.max_duo);
                } else {
                    self.files.roll_laptop = (self.files.roll_laptop + d).clamp(0.0, self.files.max_laptop);
                }
                Refresh::Dirty
            }
            Page::Photos if self.photos.open.is_none() => {
                self.photos.roll = (self.photos.roll + d).clamp(0.0, self.photos.max);
                Refresh::Dirty
            }
            _ => Refresh::Calm,
        }
    }

    /// Files dropped on the window (from the laptop's file manager): onto the duo, into its open folder.
    pub fn dropped_files(&mut self, paths: Vec<PathBuf>) -> Refresh {
        let Some(acc) = self.account() else { return Refresh::Calm };
        if self.job.is_some() || paths.is_empty() {
            return Refresh::Calm;
        }
        self.page = Page::Files;
        let dir = self.files.duo_dir.clone();
        self.work(l("Kopiere aufs Duo …", "Copying to the duo …"), move || {
            let (mut ok, mut bad) = (0, 0);
            for p in paths {
                if co::copy_to_duo(&acc, &p, &dir).is_ok() {
                    ok += 1;
                } else {
                    bad += 1;
                }
            }
            if bad == 0 {
                Deliver::Done(Ok(format!("{ok} {}", l("aufs Duo kopiert", "copied to the duo"))))
            } else {
                Deliver::Done(Err(format!("{ok} {}, {bad} {}", l("kopiert", "copied"), l("nicht", "failed"))))
            }
        });
        Refresh::Dirty
    }

    // ------------------------------------------------------------- the frame

    /// The duo at the top of the sidebar: a disc, its name, its state.
    pub fn lay_side(&self, b: &mut SceneBuilder, engine: &mut TextEngine, top: f32, hf: f32) {
        let p = self.palette;
        b.quad(0.0, top, SIDE_W, hf - top, p.on_surface.over(p.surface, 0.035), 0.0, None, None);
        let m = mk::spacing::M;
        let mut y = top + m;
        let disc = 52.0;
        let on = self.link != Link::None;
        b.quad(m, y, disc, disc, if on { p.primary_container } else { p.surface_container_high }, disc / 2.0, None, None);
        let g = symbol::SMARTPHONE.to_string();
        let gpx = mk::typo::symbol_px(28.0);
        let (gw, gh) = engine.measure_family(matrixkit_scene::symbol_family(), &g, gpx, mk::typo::Weight::Normal);
        b.text(engine, g, gpx, mk::typo::Weight::Normal, matrixkit_scene::symbol_family(), if on { p.on_primary_container } else { p.outline }, m + (disc - gw) / 2.0, y + (disc - gh) / 2.0, None, None);
        let name = self.info.as_ref().map(|i| i.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| String::from("Surface Duo"));
        let clip = Some(Rect { x: 0.0, y: top, w: SIDE_W, h: hf - top });
        b.styled(engine, &name, mk::typo::LABEL, p.on_surface, m + disc + 12.0, y + 6.0, None, clip);
        let (dot, word) = self.state_word();
        b.quad(m + disc + 12.0, y + 33.0, 8.0, 8.0, dot, 4.0, None, None);
        b.styled(engine, word, mk::typo::CAPTION, p.on_surface_variant, m + disc + 26.0, y + 27.0, None, clip);
        y += disc + m + 6.0;
        let entries: Vec<fam::NavEntry> = PAGES.iter().map(|pg| fam::NavEntry { title: pg.title(), glyph: pg.glyph(), parts: &[], dot: false }).collect();
        let selected = PAGES.iter().position(|pg| *pg == self.page);
        let area = Rect { x: 0.0, y, w: SIDE_W, h: hf - y - 60.0 };
        fam::lay_nav(b, engine, &entries, selected, None, false, area, 0.0, NAV_BASE, NAV_BASE + 50, p);
        // The language, at the foot.
        const GLOBE: char = '\u{e894}';
        let lang_label = LANGS.iter().find(|(c, _)| *c == self.lang).map(|(_, n)| n.to_string()).unwrap_or_default();
        if !self.running {
            fam::lay_pill(b, engine, Some(GLOBE), &lang_label, m, hf - 52.0, None, Some(LANG_OPEN), fam::PillTone::Quiet, p);
        }
    }

    /// The duo's state in a word (sidebar, overview).
    pub fn state_word(&self) -> (mk::Rgba, &'static str) {
        let p = self.palette;
        match self.link {
            Link::Fedora if self.device.as_ref().is_some_and(|d| d.matrix) => (p.tertiary, l("Matrix Recovery", "Matrix Recovery")),
            Link::Fedora if self.hello.as_ref().is_some_and(|h| h.paired) => (p.primary, l("Matrix · gekoppelt", "Matrix · paired")),
            Link::Fedora if self.hello.is_some() => (p.tertiary, l("Matrix · nicht gekoppelt", "Matrix · not paired")),
            Link::Fedora => (p.primary, l("Matrix läuft", "Matrix running")),
            Link::None => (p.outline, l("Nicht verbunden", "Not connected")),
            Link::Android => (p.tertiary, l("Android läuft", "Android running")),
            Link::Bootloader => (p.tertiary, l("Im Bootloader", "In the bootloader")),
            Link::Stage1 => (p.tertiary, l("Startet …", "Starting …")),
            Link::NoAccess => (p.error, l("Kein Zugriff", "No access")),
            Link::Unauthorized => (p.tertiary, l("Auf dem Duo bestätigen", "Confirm on the duo")),
        }
    }

    /// Page heading; returns the y below it.
    fn heading(&self, b: &mut SceneBuilder, engine: &mut TextEngine, title: &str, x: f32, y: f32) -> f32 {
        b.styled(engine, title, mk::typo::LARGE_TITLE, self.palette.on_surface, x, y, None, None);
        y + mk::typo::LARGE_TITLE.px() * 1.5
    }

    /// The working line or the last result, under a page's actions.
    fn lay_note(&self, b: &mut SceneBuilder, engine: &mut TextEngine, x: f32, y: f32, w: f32) -> f32 {
        let p = self.palette;
        let (text, ink) = if let Some(j) = &self.job {
            (j.clone(), p.tertiary)
        } else if let Some((ok, t)) = &self.note {
            (t.clone(), if *ok { p.primary } else { p.error })
        } else {
            return y;
        };
        let mut yy = y;
        for line in mk::wrap_words(&text, ((w / (mk::typo::CAPTION.px() * 0.52)) as usize).max(30), 3) {
            b.styled(engine, &line, mk::typo::CAPTION, ink, x, yy, None, None);
            yy += 17.0;
        }
        yy + 6.0
    }

    // ------------------------------------------------------------- pages

    pub fn lay_page(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, wf: f32, hf: f32, top: f32) {
        match self.page {
            Page::Overview => self.lay_overview(b, engine, wf, top),
            Page::Files => self.lay_files(b, engine, wf, hf, top),
            Page::Photos => self.lay_photos(b, engine, wf, hf, top),
            Page::Backups => self.lay_backups(b, engine, wf, hf, top),
            Page::Software => self.lay(b, engine, SIDE_W, wf - SIDE_W, hf, top),
        }
    }

    /// Files and photos need a paired duo running Matrix; else say why.
    fn needs_pairing(&self, b: &mut SceneBuilder, engine: &mut TextEngine, x: f32, y: f32, w: f32) -> bool {
        if self.account().is_some() {
            return false;
        }
        let (title, text) = match self.link {
            Link::Fedora if self.device.as_ref().is_some_and(|d| d.matrix) => (l("Im Recovery", "In recovery"), l("Dateien und Fotos gibt es, wenn Matrix läuft. Starte Matrix unter «Software».", "Files and photos are there when Matrix runs. Start Matrix under «Software».")),
            Link::Fedora => (l("Noch nicht gekoppelt", "Not paired yet"), l("Koppele das Duo in der Übersicht — es fragt auf seinem Bildschirm, ob es diesem Laptop vertrauen soll.", "Pair the duo in the overview — it asks on its screen whether to trust this laptop.")),
            _ => (l("Matrix läuft nicht", "Matrix is not running"), l("Dateien und Fotos gibt es, wenn das Duo in Matrix läuft und am USB-Kabel hängt.", "Files and photos are there when the duo runs Matrix and is on the USB cable.")),
        };
        fam::lay_notice(b, engine, symbol::LOCK, title, text, x, y, w, self.palette);
        true
    }

    fn lay_overview(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, wf: f32, top: f32) {
        let p = self.palette;
        let x = SIDE_W + 32.0;
        let w = wf - SIDE_W - 64.0;
        let mut y = self.heading(b, engine, Page::Overview.title(), x, top + 24.0);
        match self.link {
            Link::None => {
                fam::lay_notice(b, engine, symbol::USB, l("Kein Duo verbunden", "No duo connected"), l("Schliesse dein Surface Duo mit dem USB-Kabel an. Morpheus erkennt es in Android, im Bootloader, in Matrix und im Recovery.", "Connect your Surface Duo with the USB cable. Morpheus recognises it in Android, in the bootloader, in Matrix and in recovery."), x, y, w, self.palette);
            }
            Link::NoAccess => {
                y = fam::lay_notice(b, engine, symbol::LOCK, l("Duo gefunden — aber kein Zugriff", "Duo found — but no access"), l("Dieses Linux erlaubt Programmen noch nicht, mit Android-Geräten am USB zu sprechen (die Regel fehlt, die neuere Linux-Ausgaben mitbringen). «Zugriff erlauben» richtet sie einmal ein — dafür fragt Linux nach deinem Passwort. Danach das Kabel kurz aus- und wieder einstecken.", "This Linux does not yet let programs talk to Android devices over USB (the rule newer Linux releases bring is missing). «Allow access» sets it up once — Linux asks for your password. Then unplug the cable and plug it in again."), x, y, w, self.palette);
                let (_, ph) = fam::lay_pill(b, engine, Some(symbol::LOCK), if self.access_busy { l("Bitte Passwort eingeben …", "Please enter your password …") } else { l("Zugriff erlauben", "Allow access") }, x, y, None, (!self.access_busy).then_some(OV_ACCESS), if self.access_busy { fam::PillTone::Quiet } else { fam::PillTone::Accent }, p);
                self.lay_note(b, engine, x, y + ph + 12.0, w);
            }
            Link::Unauthorized => {
                fam::lay_notice(b, engine, symbol::SMARTPHONE, l("Auf dem Duo bestätigen", "Confirm on the duo"), l("Android fragt auf dem Duo «USB-Debugging zulassen?». Haken bei «Von diesem Computer immer zulassen» setzen und «Zulassen» tippen. Erscheint keine Frage: Kabel aus- und wieder einstecken, das Duo entsperrt lassen.", "Android asks on the duo «Allow USB debugging?». Tick «Always allow from this computer» and tap «Allow». No question? Unplug and replug the cable with the duo unlocked."), x, y, w, self.palette);
            }
            Link::Android | Link::Bootloader | Link::Stage1 => {
                let (_, word) = self.state_word();
                y = fam::lay_notice(b, engine, symbol::SMARTPHONE, word, l("Das Duo ist verbunden, Matrix läuft gerade nicht. Morpheus kann Matrix von seiner Partition starten — auf dem Duo wird dabei nichts geschrieben.", "The duo is connected but Matrix is not running. Morpheus can start Matrix from its partition — nothing is written on the duo."), x, y, w, self.palette);
                if !self.running {
                    fam::lay_pill(b, engine, Some(symbol::PLAY_ARROW), l("Matrix starten", "Start Matrix"), x, y, None, Some(OV_START), fam::PillTone::Accent, p);
                }
            }
            Link::Fedora if self.device.as_ref().is_some_and(|d| d.matrix) => {
                y = fam::lay_notice(b, engine, symbol::SHIELD, "Matrix Recovery", l("Das Duo läuft im Live-System. Sichern, Wiederherstellen, Neu installieren und Starten findest du unter «Sicherungen» und «Software».", "The duo runs the live system. Back up, restore, reinstall and start are under «Backups» and «Software»."), x, y, w, self.palette);
                fam::lay_pill(b, engine, Some(symbol::SAVE), Page::Backups.title(), x, y, None, Some(OV_BACKUPS), fam::PillTone::Accent, p);
            }
            Link::Fedora => match (&self.hello, &self.info) {
                (Some(h), _) if !h.paired => {
                    let text = format!(
                        "{} {} · {} {} · {} {}. {}",
                        h.model,
                        h.serial,
                        l("Konto", "account"),
                        h.account,
                        l("Dieser Laptop:", "This laptop:"),
                        co::laptop_name(),
                        l("Beim Koppeln fragt das Duo «Diesem Laptop vertrauen?». Danach kann Morpheus über das Kabel Dateien und Fotos zeigen und kopieren und Matrix sichern.", "When pairing, the duo asks «Trust this laptop?». Then Morpheus can show and copy files and photos over the cable and back up Matrix.")
                    );
                    y = fam::lay_notice(b, engine, symbol::LAPTOP, l("Diesem Duo vertrauen", "Trust this duo"), &text, x, y, w, self.palette);
                    let label = if self.pairing { l("Bitte auf dem Duo bestätigen …", "Please confirm on the duo …") } else { l("Koppeln", "Pair") };
                    let (_, ph) = fam::lay_pill(b, engine, Some(symbol::LAPTOP), label, x, y, None, (!self.pairing).then_some(OV_PAIR), if self.pairing { fam::PillTone::Quiet } else { fam::PillTone::Accent }, p);
                    self.lay_note(b, engine, x, y + ph + 12.0, w);
                }
                (Some(_), Some(i)) => {
                    let i = i.clone();
                    // The hero line.
                    b.styled(engine, &format!("{} · {} {}", i.model, l("Seriennummer", "Serial"), i.serial), mk::typo::BODY, p.on_surface_variant, x, y, None, None);
                    y += 34.0;
                    let gap = 14.0;
                    let tw = (w - 2.0 * gap) / 3.0;
                    let used = (i.storage_gib - i.free_gib).max(0.0);
                    let battery = if i.battery >= 0 { format!("{} %{}", i.battery, if i.charging { l(" · lädt", " · charging") } else { "" }) } else { String::from("—") };
                    let rows: [[(String, String, Option<f32>); 3]; 2] = [
                        [
                            (String::from(l("Speicher (Matrix)", "Storage (Matrix)")), format!("{:.0} / {:.0} GB", used, i.storage_gib), Some(if i.storage_gib > 0.0 { used / i.storage_gib } else { 0.0 })),
                            (String::from(l("Akku", "Battery")), battery, (i.battery >= 0).then_some(i.battery as f32 / 100.0)),
                            (String::from(l("Konto", "Account")), i.account.clone(), None),
                        ],
                        [
                            (String::from(l("Matrix-System vom", "Matrix system of")), i.system.clone(), None),
                            (String::from(l("Läuft seit", "Up for")), format!("{:.1} h", i.up_h), None),
                            (String::from("Fedora"), i.fedora.replace("Fedora Linux ", ""), None),
                        ],
                    ];
                    for row in rows.iter() {
                        let mut tx = x;
                        for (label, value, bar) in row.iter() {
                            fam::lay_fact_tile(b, engine, label, value, *bar, tx, y, tw, p);
                            tx += tw + gap;
                        }
                        y += fam::FACT_TILE_H + gap;
                    }
                    // The last backup, and the way to make one.
                    y += 6.0;
                    let last = self.saved.first().map(|s| format!("{} {} ({} MB)", l("Letzte Matrix-Sicherung:", "Last Matrix backup:"), s.when, s.mb)).unwrap_or_else(|| String::from(l("Noch keine Matrix-Sicherung auf diesem Laptop.", "No Matrix backup on this laptop yet.")));
                    b.styled(engine, &last, mk::typo::BODY, p.on_surface, x, y, None, None);
                    y += 30.0;
                    let (_, ph) = fam::lay_pill(b, engine, Some(symbol::SAVE), Page::Backups.title(), x, y, None, Some(OV_BACKUPS), fam::PillTone::Quiet, p);
                    self.lay_note(b, engine, x, y + ph + 12.0, w);
                }
                (Some(_), None) => {
                    self.lay_note(b, engine, x, y, w);
                }
                (None, _) => {
                    fam::lay_notice(b, engine, symbol::INFO, l("Matrix läuft", "Matrix running"), l("Dieses Matrix kennt die Kopplung mit Morpheus noch nicht (ältere Fassung). Starten, Installieren und Recovery gehen unter «Software».", "This Matrix does not know pairing with Morpheus yet (older version). Start, install and recovery are under «Software»."), x, y, w, self.palette);
                }
            },
        }
    }

    /// One column of the file page (duo or laptop).
    #[allow(clippy::too_many_arguments)]
    fn lay_column(&self, b: &mut SceneBuilder, engine: &mut TextEngine, title: &str, path: &str, entries: &[Entry], sel: Option<usize>, roll: f32, x: f32, y: f32, w: f32, h: f32, up: usize, row_base: usize) -> f32 {
        let p = self.palette;
        b.quad(x, y, w, h, p.surface_container, 16.0, None, None);
        b.styled(engine, title, mk::typo::LABEL, p.on_surface, x + 16.0, y + 12.0, None, None);
        let uw = fam::pill_width(engine, Some(symbol::ARROW_UPWARD), l("Hoch", "Up"));
        fam::lay_pill(b, engine, Some(symbol::ARROW_UPWARD), l("Hoch", "Up"), x + w - uw - 8.0, y + 6.0, None, Some(up), fam::PillTone::Quiet, p);
        let pclip = Some(Rect { x, y, w: w - uw - 20.0, h: 60.0 });
        b.styled(engine, if path.is_empty() { "~" } else { path }, mk::typo::CAPTION, p.on_surface_variant, x + 16.0, y + 34.0, None, pclip);
        let list = Rect { x, y: y + 58.0, w, h: h - 64.0 };
        let row = 38.0;
        let first = (roll / row).floor().max(0.0) as usize;
        let shown = (list.h / row).ceil() as usize + 1;
        for (k, e) in entries.iter().enumerate().skip(first).take(shown).take(MAX_ROWS) {
            let ry = list.y + k as f32 * row - roll;
            let on = sel == Some(k);
            b.quad(x + 6.0, ry, w - 12.0, row - 4.0, if on { p.primary_container } else { mk::Rgba { a: 0.0, ..p.surface } }, 10.0, Some(row_base + k), Some(list));
            let g = (if e.dir { symbol::FOLDER } else { symbol::FILE }).to_string();
            let gpx = mk::typo::symbol_px(20.0);
            b.text(engine, g, gpx, mk::typo::Weight::Normal, matrixkit_scene::symbol_family(), if e.dir { p.primary } else { p.on_surface_variant }, x + 16.0, ry + 7.0, None, Some(list));
            let ink = if on { p.on_primary_container } else { p.on_surface };
            let size = if e.dir { String::new() } else { mk::format::bytes(e.size) };
            let (sw, _) = engine.measure_styled(&size, mk::typo::CAPTION.px(), mk::typo::CAPTION.weight);
            let nclip = Rect { x, y: list.y.max(ry), w: w - sw - 30.0, h: row };
            b.styled(engine, &e.name, mk::typo::BODY, ink, x + 46.0, ry + 8.0, None, Some(Rect { h: nclip.h.min(list.y + list.h - nclip.y).max(0.0), ..nclip }));
            if !size.is_empty() {
                b.styled(engine, &size, mk::typo::CAPTION, p.on_surface_variant, x + w - sw - 16.0, ry + 11.0, None, Some(list));
            }
        }
        if entries.is_empty() {
            b.styled(engine, l("leer", "empty"), mk::typo::CAPTION, p.outline, x + 16.0, list.y + 8.0, None, None);
        }
        (entries.len() as f32 * row - list.h + 10.0).max(0.0)
    }

    fn lay_files(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, wf: f32, hf: f32, top: f32) {
        let p = self.palette;
        let x = SIDE_W + 24.0;
        let w = wf - SIDE_W - 48.0;
        let mut y = self.heading(b, engine, Page::Files.title(), x, top + 20.0);
        if self.needs_pairing(b, engine, x, y, w) {
            return;
        }
        let gap = 16.0;
        let cw = (w - gap) / 2.0;
        let ch = hf - y - 120.0;
        let laptop_path = self.files.laptop_dir.display().to_string().replace(&home().display().to_string(), "~");
        let duo_title = format!("Duo · {}", self.hello.as_ref().map(|h| h.account.as_str()).unwrap_or(""));
        let laptop_title = format!("{} · {}", l("Laptop", "Laptop"), co::laptop_name());
        self.files.max_duo = self.lay_column(b, engine, &duo_title, &self.files.duo_dir.clone(), &self.files.duo.clone(), self.files.sel_duo, self.files.roll_duo, x, y, cw, ch, F_UP_DUO, DUO_ROW);
        self.files.max_laptop = self.lay_column(b, engine, &laptop_title, &laptop_path, &self.files.laptop.clone(), self.files.sel_laptop, self.files.roll_laptop, x + cw + gap, y, cw, ch, F_UP_LAP, LAP_ROW);
        y += ch + 14.0;
        let idle = self.job.is_none();
        let to_lap = idle && self.files.sel_duo.is_some();
        let to_duo = idle && self.files.sel_laptop.is_some();
        let (w1, ph) = fam::lay_pill(b, engine, Some(symbol::ARROW_FORWARD), l("Auf den Laptop", "To the laptop"), x, y, None, to_lap.then_some(F_TO_LAP), if to_lap { fam::PillTone::Accent } else { fam::PillTone::Quiet }, p);
        let (w2, _) = fam::lay_pill(b, engine, Some(symbol::ARROW_BACK), l("Aufs Duo", "To the duo"), x + w1 + 10.0, y, None, to_duo.then_some(F_TO_DUO), if to_duo { fam::PillTone::Accent } else { fam::PillTone::Quiet }, p);
        fam::lay_pill(b, engine, Some(symbol::RESTART), l("Neu laden", "Reload"), x + w1 + w2 + 20.0, y, None, idle.then_some(F_RELOAD), fam::PillTone::Quiet, p);
        y += ph + 10.0;
        if self.job.is_none() && self.note.is_none() {
            b.styled(engine, l("Antippen wählt aus, ein zweites Antippen öffnet einen Ordner. Dateien vom Laptop kannst du auch ins Fenster ziehen — sie landen im offenen Ordner des Duo.", "Tap selects, a second tap opens a folder. You can also drag files from the laptop into the window — they land in the duo's open folder."), mk::typo::CAPTION, p.on_surface_variant, x, y, None, Some(Rect { x, y, w, h: 40.0 }));
        } else {
            self.lay_note(b, engine, x, y, w);
        }
    }

    fn lay_photos(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, wf: f32, hf: f32, top: f32) {
        let p = self.palette;
        let x = SIDE_W + 24.0;
        let w = wf - SIDE_W - 48.0;
        let mut y = self.heading(b, engine, Page::Photos.title(), x, top + 20.0);
        if self.needs_pairing(b, engine, x, y, w) {
            return;
        }
        let idle = self.job.is_none();
        // One photo, big.
        if let Some(k) = self.photos.open {
            let (bw, ph) = fam::lay_pill(b, engine, Some(symbol::ARROW_BACK), l("Alle Fotos", "All photos"), x, y, None, Some(P_BACK), fam::PillTone::Quiet, p);
            fam::lay_pill(b, engine, Some(symbol::DOWNLOAD), l("Auf den Laptop", "To the laptop"), x + bw + 10.0, y, None, idle.then_some(P_SAVE), fam::PillTone::Accent, p);
            if let Some(ph_item) = self.photos.items.get(k) {
                let name = ph_item.path.rsplit('/').next().unwrap_or("").to_string();
                b.styled(engine, &format!("{name} · {}", mk::format::bytes(ph_item.size)), mk::typo::CAPTION, p.on_surface_variant, x, y + ph + 8.0, None, None);
            }
            y += ph + 34.0;
            let box_h = hf - y - 60.0;
            let home_rect = Rect { x, y, w, h: box_h };
            let anchor = b.quad(x, y, w, box_h, p.surface_container, 16.0, None, None);
            match &self.photos.full {
                Some(path) => b.pict(shell::Pict { path: path.clone(), box_w: w as u32, box_h: box_h as u32, fit: matrixkit_image::Fit::Contain, x, y, radius: 16.0, home: home_rect, anchor, clip: None }),
                None => {
                    if let Some(item) = self.photos.items.get(k) {
                        b.pict(shell::Pict { path: item.preview.clone(), box_w: w as u32, box_h: box_h as u32, fit: matrixkit_image::Fit::Contain, x, y, radius: 16.0, home: home_rect, anchor, clip: None });
                    }
                }
            }
            self.lay_note(b, engine, x, hf - 48.0, w);
            return;
        }
        // The grid.
        let count = format!("{} {}", self.photos.items.len(), l("Fotos in «Bilder» auf dem Duo", "photos in «Bilder» on the duo"));
        b.styled(engine, &count, mk::typo::CAPTION, p.on_surface_variant, x, y - 8.0, None, None);
        y += 14.0;
        let (lw, ph) = fam::lay_pill(b, engine, Some(symbol::RESTART), l("Neu laden", "Reload"), x, y, None, idle.then_some(P_LOAD), fam::PillTone::Quiet, p);
        let all_ok = idle && !self.photos.items.is_empty();
        fam::lay_pill(b, engine, Some(symbol::DOWNLOAD), l("Alle auf den Laptop", "All to the laptop"), x + lw + 10.0, y, None, all_ok.then_some(P_ALL), if all_ok { fam::PillTone::Accent } else { fam::PillTone::Quiet }, p);
        y += ph + 8.0;
        y = self.lay_note(b, engine, x, y, w) + 4.0;
        let grid = Rect { x, y, w, h: hf - y - 16.0 };
        let previews: Vec<PathBuf> = self.photos.items.iter().map(|ph| ph.preview.clone()).collect();
        self.photos.max = fam::lay_pict_grid(b, &previews, grid, self.photos.roll, 150.0, 10.0, PHOTO, p);
        if self.photos.items.is_empty() && self.photos.loaded {
            b.styled(engine, l("Keine Fotos in «Bilder» auf dem Duo.", "No photos in «Bilder» on the duo."), mk::typo::BODY, p.outline, x, y + 8.0, None, None);
        }
    }

    fn lay_backups(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, wf: f32, hf: f32, top: f32) {
        let p = self.palette;
        let x = SIDE_W + 32.0;
        let w = wf - SIDE_W - 64.0;
        let mut y = self.heading(b, engine, Page::Backups.title(), x, top + 20.0);
        let serial = self.serial();
        let recovery = self.device.as_ref().is_some_and(|d| d.matrix);
        let paired = self.account().is_some();
        // Back up now.
        let can = !self.running && self.job.is_none() && (recovery || paired);
        let why = if recovery {
            l("Im Recovery: sichert Konten, Dateien, WLAN, Bluetooth und Sprache von Matrix auf diesen Laptop.", "In recovery: backs up Matrix's accounts, files, Wi-Fi, Bluetooth and language to this laptop.")
        } else if paired {
            l("Während Matrix läuft, über das Kabel: Konten, Dateien, WLAN, Bluetooth und Sprache auf diesen Laptop. Nicht das System selbst — die Sicherung passt auch auf ein neueres Matrix.", "While Matrix runs, over the cable: accounts, files, Wi-Fi, Bluetooth and language to this laptop. Not the system itself — the backup also fits a newer Matrix.")
        } else {
            l("Sichern geht, wenn das Duo gekoppelt in Matrix läuft oder im Recovery ist.", "Backing up works when the duo runs Matrix paired, or is in recovery.")
        };
        let (_, ph) = fam::lay_pill(b, engine, Some(symbol::SAVE), l("Jetzt sichern", "Back up now"), x, y, None, can.then_some(B_NOW), if can { fam::PillTone::Accent } else { fam::PillTone::Quiet }, p);
        y += ph + 8.0;
        for line in mk::wrap_words(why, ((w / (mk::typo::CAPTION.px() * 0.52)) as usize).max(30), 3) {
            b.styled(engine, &line, mk::typo::CAPTION, p.on_surface_variant, x, y, None, None);
            y += 17.0;
        }
        y = self.lay_note(b, engine, x, y + 4.0, w) + 10.0;
        // Matrix backups.
        b.styled(engine, l("Matrix-Sicherungen", "Matrix backups"), mk::typo::HEADLINE, p.on_surface, x, y, None, None);
        y += 34.0;
        if serial.is_empty() {
            b.styled(engine, l("Verbinde das Duo, um seine Sicherungen zu sehen.", "Connect the duo to see its backups."), mk::typo::BODY, p.outline, x, y, None, None);
            y += 28.0;
        } else if self.saved.is_empty() {
            b.styled(engine, l("Noch keine Matrix-Sicherung von diesem Duo auf diesem Laptop.", "No Matrix backup of this duo on this laptop yet."), mk::typo::BODY, p.outline, x, y, None, None);
            y += 28.0;
        }
        let row = 50.0;
        let restore_ok = !self.running && self.job.is_none() && (recovery || paired);
        for (k, s) in self.saved.iter().enumerate().take(8) {
            b.quad(x, y, w, row - 6.0, p.surface_container, 12.0, None, None);
            let label = l("Wiederherstellen", "Restore");
            let pw = fam::pill_width(engine, Some(symbol::DOWNLOAD), label);
            let what = format!("{} · {} MB · {}", s.when, s.mb, if s.accounts.is_empty() { "—" } else { &s.accounts });
            b.styled(engine, &what, mk::typo::BODY, p.on_surface, x + 16.0, y + 12.0, None, Some(Rect { x, y, w: w - pw - 24.0, h: row }));
            fam::lay_pill(b, engine, Some(symbol::DOWNLOAD), label, x + w - pw - 4.0, y + 2.0, None, restore_ok.then_some(B_RESTORE + k), fam::PillTone::Quiet, p);
            y += row;
        }
        if paired && !recovery && !self.saved.is_empty() {
            b.styled(engine, l("Wiederherstellen startet das Duo in Matrix Recovery; dort fragt es vor dem Ersetzen.", "Restoring starts the duo into Matrix Recovery; it asks there before replacing anything."), mk::typo::CAPTION, p.on_surface_variant, x, y + 2.0, None, None);
            y += 22.0;
        }
        // Full backups (all partitions) — shown, not restored here.
        let full = if serial.is_empty() { Vec::new() } else { full_backups(&serial) };
        if !full.is_empty() && y + 120.0 < hf {
            y += 16.0;
            b.styled(engine, l("Vollsicherungen (alle Partitionen)", "Full backups (all partitions)"), mk::typo::HEADLINE, p.on_surface, x, y, None, None);
            y += 34.0;
            for (when, size) in full.iter().take(4) {
                if y + 30.0 > hf - 80.0 {
                    break;
                }
                b.styled(engine, &format!("{when} · {}", mk::format::bytes(*size)), mk::typo::BODY, p.on_surface_variant, x + 16.0, y, None, None);
                y += 26.0;
            }
            b.styled(engine, l("Vollsicherungen liegen für den Notfall bereit; Morpheus spielt sie nicht zurück.", "Full backups are kept for emergencies; Morpheus does not restore them."), mk::typo::CAPTION, p.outline, x, y + 4.0, None, None);
            y += 26.0;
        }
        // The running task's log.
        if self.task.is_some() || !self.log.is_empty() {
            let log_h = (hf - y - 20.0).max(0.0);
            if log_h > 50.0 {
                b.quad(x, y + 8.0, w, log_h - 8.0, p.surface_container_high, 12.0, None, None);
                let n = ((log_h - 24.0) / 18.0).floor().max(0.0) as usize;
                let from = self.log.len().saturating_sub(n);
                let clip = Some(Rect { x, y: y + 8.0, w, h: log_h - 8.0 });
                for (k, line) in self.log[from..].iter().enumerate() {
                    b.text(engine, line.clone(), 12.5, mk::typo::Weight::Normal, matrixkit_text::FAMILY, p.on_surface_variant, x + 12.0, y + 18.0 + k as f32 * 18.0, None, clip);
                }
            }
        }
    }
}
