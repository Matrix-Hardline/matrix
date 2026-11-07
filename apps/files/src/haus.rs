//! Matrix Dateien in the window house (R217, the owner 02.11.2026: on the
//! duo, «move downloaded files to a USB stick»).
//!
//! Places on the left (Downloads, Musik, Videos, Bilder, Dokumente, the
//! personal folder, then every USB stick — label and size; a tap mounts),
//! the folder on the right: a row per entry (symbol or thumbnail, size,
//! age). A tap on a folder opens it, a tap on a file selects it (several
//! may be), a long press or the row's check key selects too. The bar at the
//! bottom acts on the selection: «Verschieben nach …» / «Kopieren nach …»
//! carry it along — pick a place (or a stick) on the left, open a folder,
//! «Hierher» — and the transfer runs on the loom with progress
//! («Kopiert 120 von 340 MB»). «Auswerfen» unmounts the stick and powers
//! it off: «Kann entfernt werden». A narrow window shows the list alone,
//! with «‹ Orte» back to the places.
//!
//! The kit does the work: `mk::volumes` (sticks), `mk::fileops` (copy and
//! move with progress), `matrixkit_image::thumbs` (pictures, made off the
//! frame), `fam::lay_nav`, `fam::MediaRow`, `fam::lay_pill`, `mk::touch::Press`.

use matrixkit_image::ImageAtlas;
use matrixkit_scene::scene as fam;
use matrixkit_scene::symbol;
use matrixkit_shell::{self as shell, Loom, Refresh, Scene, SceneBuilder, Source, Text};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::fileops::{self, Mode as Op, Progress, Summary};
use matrixkit_theme::scene::{Keep, Rect};
use matrixkit_theme::volumes::{self, Volume};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const APP_ID: &str = "matrix-files";
const CHROME_BASE: usize = 500;
const ROOT_BASE: usize = 700;
/// Sidebar entries: places first, then the volumes.
const SIDE_BASE: usize = 100;
const PART_BASE: usize = 170;
const BACK: usize = 180;
const UP: usize = 181;
const SORT: usize = 182;
const EJECT: usize = 183;
// The bar.
const ACT_MOVE: usize = 200;
const ACT_COPY: usize = 201;
const ACT_DELETE: usize = 202;
const ACT_RENAME: usize = 203;
const ACT_OPEN: usize = 204;
const ACT_CLEAR: usize = 205;
const ACT_ALL: usize = 206;
const HERE: usize = 210;
const CARRY_CANCEL: usize = 211;
const JOB_CANCEL: usize = 212;
const NEW_FOLDER: usize = 214;
const FIELD: usize = 215;
const FIELD_OK: usize = 216;
const FIELD_CANCEL: usize = 217;
const CONFIRM_DELETE: usize = 218;
const CONFIRM_CANCEL: usize = 219;
/// A row (tap: open a folder / select a file), its check key.
const ROW_BASE: usize = 10_000;
const CHECK_BASE: usize = 60_000;
const ROWS_MAX: usize = 50_000;

/// Below this width the places and the folder take turns.
const NARROW: f32 = 520.0;
const SIDE_W: f32 = 200.0;
/// A row's picture tile (the row is this plus its gap).
const TILE: f32 = 52.0;
/// Thumbnails on disk (twice the tile for sharpness).
const THUMB_PX: u32 = 96;
/// Poll the sticks every this many beats (the beat is 500 ms).
const POLL_BEATS: u32 = 4;

/// One entry of a folder.
#[derive(Debug, Clone)]
struct Entry {
    name: String,
    path: PathBuf,
    folder: bool,
    size: u64,
    changed: Option<u64>,
}

impl Entry {
    fn ext(&self) -> String {
        self.path.extension().map(|x| x.to_string_lossy().to_ascii_lowercase()).unwrap_or_default()
    }

    /// Still being written (a download in progress).
    fn partial(&self) -> bool {
        !self.folder && fileops::shown(&self.name).partial
    }

    /// The kind in words (the iced version's «Art»).
    fn kind(&self) -> &'static str {
        if self.folder {
            return "Ordner";
        }
        if self.partial() {
            return "Wird geladen";
        }
        match self.ext().as_str() {
            "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "svg" | "heic" => "Bild",
            "mp4" | "mkv" | "webm" | "mov" | "avi" | "m4v" => "Film",
            "mp3" | "flac" | "ogg" | "wav" | "m4a" | "opus" | "aac" => "Musik",
            "pdf" => "PDF-Dokument",
            "txt" | "md" => "Text",
            "zip" | "tar" | "gz" | "xz" | "zst" | "7z" => "Archiv",
            "iso" | "img" => "Abbild",
            "appimage" => "Programm",
            _ => "Dokument",
        }
    }

    fn glyph(&self) -> char {
        if self.folder {
            return symbol::FOLDER;
        }
        match self.kind() {
            "Bild" => symbol::IMAGE,
            "Film" => symbol::MOVIE,
            "Musik" => symbol::MUSIC_NOTE,
            "PDF-Dokument" => symbol::PDF,
            "Archiv" | "Abbild" => symbol::STORAGE,
            "Wird geladen" => symbol::DOWNLOAD,
            _ => symbol::FILE,
        }
    }

    /// A picture or a (finished) video the thumbnail family can shrink.
    fn picture(&self) -> bool {
        matches!(self.ext().as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp" | "gif") || self.video()
    }

    fn video(&self) -> bool {
        !self.folder && matrixkit_image::thumbs::is_video(&self.path)
    }
}

/// Read a folder (off the frame): entries without dot files.
fn read_dir(path: &Path) -> Result<Vec<Entry>, String> {
    let dir = std::fs::read_dir(path).map_err(|e| fileops::say(&e))?;
    let mut out = Vec::new();
    for e in dir.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        // Follow links (a link to a folder opens like one).
        let Ok(meta) = std::fs::metadata(e.path()) else { continue };
        out.push(Entry {
            name,
            path: e.path(),
            folder: meta.is_dir(),
            size: meta.len(),
            changed: meta.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs()),
        });
    }
    Ok(out)
}

fn sort(entries: &mut [Entry], newest: bool) {
    entries.sort_by(|a, b| {
        b.folder.cmp(&a.folder).then_with(|| if newest { b.changed.cmp(&a.changed) } else { a.name.to_lowercase().cmp(&b.name.to_lowercase()) })
    });
}

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from("/")))
}

fn dir_stamp(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// A place of the sidebar.
struct Place {
    title: &'static str,
    glyph: char,
    path: PathBuf,
}

fn places() -> Vec<Place> {
    let h = home();
    let mut out = vec![Place { title: "Downloads", glyph: symbol::DOWNLOAD, path: h.join("Downloads") }];
    for (title, rel, glyph) in [("Musik", "Musik", symbol::MUSIC_NOTE), ("Videos", "Videos", symbol::MOVIE), ("Bilder", "Bilder", symbol::IMAGE), ("Dokumente", "Dokumente", symbol::FILE)] {
        let path = h.join(rel);
        if path.is_dir() {
            out.push(Place { title, glyph, path });
        }
    }
    out.push(Place { title: "Persönlicher Ordner", glyph: symbol::HOME, path: h });
    out
}

/// What comes home from the loom.
enum Work {
    /// (sequence, folder, entries or why not, room (free, total)).
    Listed(u64, PathBuf, Result<Vec<Entry>, String>, Option<(u64, u64)>),
    /// The sticks now, and the shown folder's change time.
    Poll(Vec<Volume>, Option<SystemTime>),
    Mounted(String, Result<PathBuf, String>),
    Ejected(String, String, Result<(), String>),
    Progress(Progress),
    Done(Op, PathBuf, Result<Summary, String>),
    Thumb(PathBuf, Option<PathBuf>),
    Deleted(usize, Result<(), String>),
}

/// Where a press landed: the places, the folder list, or elsewhere.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Zone {
    Side,
    List,
    Other,
}

type Press = mk::touch::Press<(Zone, Option<usize>)>;

/// The field at the bottom names something.
#[derive(Clone, PartialEq, Debug)]
enum Naming {
    Rename(PathBuf),
    NewFolder,
}

/// A transfer on the loom.
struct Job {
    op: Op,
    progress: Progress,
    cancel: Arc<AtomicBool>,
    /// Set by the loom when the transfer returned (closing waits for it).
    ended: Arc<AtomicBool>,
    /// The volume it writes to (no ejecting meanwhile).
    dest: PathBuf,
}

struct Files {
    palette: mk::Palette,
    chrome: fam::Chrome,
    root: fam::RootPane,
    loom: Option<Loom<Work>>,
    places: Vec<Place>,
    volumes: Vec<Volume>,
    path: PathBuf,
    entries: Vec<Entry>,
    listing: Option<String>,
    seq: u64,
    loading: bool,
    stamp: Option<SystemTime>,
    room: Option<(u64, u64)>,
    newest: bool,
    selected: BTreeSet<PathBuf>,
    /// «Verschieben/Kopieren nach …»: what travels, waiting for «Hierher».
    carry: Option<(Op, Vec<PathBuf>)>,
    job: Option<Job>,
    confirm_delete: bool,
    deleting: bool,
    naming: Option<Naming>,
    field: String,
    cursor: usize,
    anchor: Option<usize>,
    blink: bool,
    mounting: Option<String>,
    ejecting: Option<String>,
    thumbs: HashMap<PathBuf, PathBuf>,
    asked: HashSet<PathBuf>,
    want: Vec<PathBuf>,
    receipt: mk::Flash,
    narrow: bool,
    menu: bool,
    side_roll: f32,
    side_max: f32,
    roll: f32,
    roll_max: f32,
    side_rect: Rect,
    list_rect: Rect,
    press: Option<Press>,
    beats: u32,
    polling: bool,
}

fn zero() -> Rect {
    Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 }
}

impl Files {
    fn new(start: Option<PathBuf>) -> Files {
        let places = places();
        let path = start.unwrap_or_else(|| places[0].path.clone());
        Files {
            palette: mk::Palette::load().unwrap_or_default(),
            chrome: fam::Chrome::new("Matrix Dateien", CHROME_BASE),
            root: fam::RootPane::new(APP_ID, &[]),
            loom: None,
            places,
            volumes: Vec::new(),
            path,
            entries: Vec::new(),
            listing: None,
            seq: 0,
            loading: true,
            stamp: None,
            room: None,
            newest: true,
            selected: BTreeSet::new(),
            carry: None,
            job: None,
            confirm_delete: false,
            deleting: false,
            naming: None,
            field: String::new(),
            cursor: 0,
            anchor: None,
            blink: true,
            mounting: None,
            ejecting: None,
            thumbs: HashMap::new(),
            asked: HashSet::new(),
            want: Vec::new(),
            receipt: mk::Flash::new(5.0),
            narrow: false,
            menu: false,
            side_roll: 0.0,
            side_max: 0.0,
            roll: 0.0,
            roll_max: 0.0,
            side_rect: zero(),
            list_rect: zero(),
            press: None,
            beats: 0,
            polling: false,
        }
    }

    fn note(&mut self, text: impl Into<String>) {
        let text = text.into();
        eprintln!("[dateien] {text}");
        self.receipt.set(text);
    }

    // ── Reading ────────────────────────────────────────────────────────

    /// List the shown folder on the loom.
    fn relist(&mut self) {
        self.seq += 1;
        self.loading = true;
        let (seq, path) = (self.seq, self.path.clone());
        if let Some(loom) = &self.loom {
            loom.spin(move || {
                let listed = read_dir(&path);
                let room = volumes::space(&path);
                Work::Listed(seq, path, listed, room)
            });
        }
    }

    fn go(&mut self, path: PathBuf) {
        if path != self.path {
            self.roll = 0.0;
            self.selected.clear();
            self.confirm_delete = false;
            self.naming = None;
        }
        self.path = path;
        self.entries.clear();
        self.listing = None;
        self.menu = false;
        self.relist();
    }

    fn poll(&mut self) {
        if self.polling {
            return;
        }
        let Some(loom) = &self.loom else { return };
        self.polling = true;
        let path = self.path.clone();
        loom.spin(move || Work::Poll(volumes::list(), dir_stamp(&path)));
    }

    /// The volume the shown folder lies on (when it is a stick).
    fn volume_here(&self) -> Option<&Volume> {
        self.volumes.iter().filter(|v| v.mount.as_ref().is_some_and(|m| self.path.starts_with(m))).max_by_key(|v| v.mount.as_ref().map(|m| m.as_os_str().len()))
    }

    /// The sidebar entry the shown folder belongs to (longest match).
    fn side_selected(&self) -> Option<usize> {
        let mut best: Option<(usize, usize)> = None;
        for (i, root) in self.side_roots().into_iter().enumerate() {
            if let Some(root) = root {
                if self.path.starts_with(&root) {
                    let len = root.as_os_str().len();
                    if best.is_none_or(|(_, l)| len > l) {
                        best = Some((i, len));
                    }
                }
            }
        }
        best.map(|(i, _)| i)
    }

    /// Each sidebar entry's folder (None: a stick not mounted yet).
    fn side_roots(&self) -> Vec<Option<PathBuf>> {
        self.places.iter().map(|p| Some(p.path.clone())).chain(self.volumes.iter().map(|v| v.mount.clone())).collect()
    }

    /// The shown folder's name for the head.
    fn title(&self) -> String {
        if let Some(v) = self.volume_here() {
            if v.mount.as_deref() == Some(self.path.as_path()) {
                return v.name();
            }
        }
        if let Some(p) = self.places.iter().find(|p| p.path == self.path) {
            return p.title.to_string();
        }
        self.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| String::from("/"))
    }

    fn selection(&self) -> Vec<PathBuf> {
        self.selected.iter().cloned().collect()
    }

    fn selected_bytes(&self) -> u64 {
        self.entries.iter().filter(|e| self.selected.contains(&e.path) && !e.folder).map(|e| e.size).sum()
    }

    // ── Acting ─────────────────────────────────────────────────────────

    fn tap_side(&mut self, i: usize) {
        let n = self.places.len();
        if i < n {
            let path = self.places[i].path.clone();
            self.go(path);
            return;
        }
        let Some(v) = self.volumes.get(i - n).cloned() else { return };
        if let Some(m) = v.mount.clone() {
            self.go(m);
            return;
        }
        if self.mounting.is_some() {
            return;
        }
        self.mounting = Some(v.device.clone());
        self.note(format!("«{}» wird eingehängt …", v.name()));
        if let Some(loom) = &self.loom {
            let dev = v.device.clone();
            loom.spin(move || {
                let r = volumes::mount(&dev);
                Work::Mounted(dev, r)
            });
        }
    }

    fn tap_row(&mut self, i: usize) {
        let Some(e) = self.entries.get(i).cloned() else { return };
        if e.folder && self.selected.is_empty() {
            self.go(e.path);
            return;
        }
        // Carrying or transferring: files are not chosen now.
        if self.carry.is_some() || self.job.is_some() {
            return;
        }
        // While something is selected a tap adds or removes (folders too).
        self.toggle(&e.path);
    }

    fn toggle(&mut self, path: &Path) {
        if !self.selected.remove(path) {
            self.selected.insert(path.to_path_buf());
            mk::feel::select();
        }
        self.confirm_delete = false;
    }

    fn open_file(&mut self, path: &Path) {
        // Round 109: the kit's opener — videos in the Player, Markdown in
        // Markdown, else gio (the Duo has no xdg-open: no video opened).
        let started = mk::open::open(&path.to_string_lossy());
        if !started {
            self.note("Keine App zum Öffnen gefunden");
        }
    }

    fn start_carry(&mut self, op: Op) {
        let items = self.selection();
        if items.is_empty() {
            return;
        }
        self.carry = Some((op, items));
        self.selected.clear();
        self.confirm_delete = false;
        mk::feel::tap();
    }

    /// «Hierher»: the carried items into the shown folder.
    fn drop_here(&mut self) {
        let Some((op, items)) = self.carry.take() else { return };
        if op == Op::Move && items.iter().all(|i| i.parent() == Some(self.path.as_path())) {
            self.note("Die Dateien liegen schon hier");
            self.carry = Some((op, items));
            return;
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let ended = Arc::new(AtomicBool::new(false));
        let dest = self.path.clone();
        self.job = Some(Job { op, progress: Progress { count: items.len(), ..Default::default() }, cancel: Arc::clone(&cancel), ended: Arc::clone(&ended), dest: dest.clone() });
        eprintln!("[dateien] {} {} Objekt(e) nach {}", if op == Op::Move { "verschiebe" } else { "kopiere" }, items.len(), dest.display());
        if let Some(loom) = &self.loom {
            let feeder = loom.clone();
            loom.spin(move || {
                let mut pace = mk::time::Pace::new(std::time::Duration::from_millis(120));
                let result = fileops::transfer(&items, &dest, op, &cancel, |p| {
                    if pace.ready() || p.done == p.total {
                        feeder.feed(Work::Progress(p.clone()));
                    }
                });
                ended.store(true, Ordering::SeqCst);
                Work::Done(op, dest, result)
            });
        }
    }

    /// On a stick: delete for good (a trash there eats the stick's room
    /// unseen); elsewhere: into the trash (gio), like the iced version.
    fn on_stick(&self) -> bool {
        self.volume_here().is_some()
    }

    fn delete(&mut self) {
        let items = self.selection();
        if items.is_empty() {
            return;
        }
        self.deleting = true;
        self.confirm_delete = false;
        let hard = self.on_stick();
        if let Some(loom) = &self.loom {
            loom.spin(move || {
                let n = items.len();
                for p in &items {
                    let r = if hard {
                        let gone = if p.is_dir() { std::fs::remove_dir_all(p) } else { std::fs::remove_file(p) };
                        gone.map_err(|e| fileops::say(&e))
                    } else if mk::command::silent("gio", &["trash", &p.to_string_lossy()]) {
                        Ok(())
                    } else {
                        Err(String::from("Papierkorb nicht möglich"))
                    };
                    if let Err(e) = r {
                        let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                        return Work::Deleted(n, Err(format!("«{name}»: {e}")));
                    }
                }
                Work::Deleted(n, Ok(()))
            });
        }
    }

    fn eject(&mut self) {
        let Some(v) = self.volume_here().cloned() else { return };
        if self.job.as_ref().is_some_and(|j| v.mount.as_ref().is_some_and(|m| j.dest.starts_with(m))) {
            self.note("Erst warten, bis die Übertragung fertig ist");
            return;
        }
        if self.ejecting.is_some() {
            return;
        }
        self.ejecting = Some(v.disk.clone());
        self.note(format!("«{}» wird ausgeworfen …", v.name()));
        // Leave the stick: an open folder on it would keep it busy.
        let away = self.places[0].path.clone();
        self.go(away);
        if let Some(loom) = &self.loom {
            let (disk, name) = (v.disk.clone(), v.name());
            loom.spin(move || {
                let r = volumes::eject(&disk);
                Work::Ejected(disk, name, r)
            });
        }
    }

    fn begin_naming(&mut self, what: Naming) {
        self.field = match &what {
            Naming::Rename(p) => p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
            Naming::NewFolder => String::new(),
        };
        // The cursor before the extension (renaming keeps it).
        self.cursor = match &what {
            Naming::Rename(_) => fileops::split_name(&self.field).0.chars().count(),
            Naming::NewFolder => 0,
        };
        self.anchor = None;
        self.naming = Some(what);
    }

    fn finish_naming(&mut self) {
        let Some(what) = self.naming.take() else { return };
        let name = self.field.trim().to_string();
        if name.is_empty() || name.contains('/') || name == "." || name == ".." {
            self.note("Ungültiger Name");
            return;
        }
        let target = self.path.join(&name);
        if target.symlink_metadata().is_ok() {
            self.note(format!("«{name}» gibt es hier schon"));
            return;
        }
        let r = match &what {
            Naming::Rename(from) => std::fs::rename(from, &target),
            Naming::NewFolder => std::fs::create_dir(&target),
        };
        match r {
            Ok(()) => {
                self.selected.clear();
                self.note(match what {
                    Naming::Rename(_) => format!("Umbenannt in «{name}»"),
                    Naming::NewFolder => format!("Ordner «{name}» angelegt"),
                });
                self.relist();
            }
            Err(e) => self.note(fileops::say(&e)),
        }
    }

    /// A deferred tap lands (on release, the finger stayed put).
    fn act(&mut self, role: usize) -> Refresh {
        if role != ACT_DELETE && role != CONFIRM_DELETE {
            self.confirm_delete = false;
        }
        match role {
            r if (SIDE_BASE..PART_BASE).contains(&r) => self.tap_side(r - SIDE_BASE),
            BACK => self.menu = true,
            UP => {
                if let Some(parent) = self.path.parent().map(Path::to_path_buf) {
                    self.go(parent);
                }
            }
            SORT => {
                self.newest = !self.newest;
                sort(&mut self.entries, self.newest);
                self.note(if self.newest { "Neueste zuerst" } else { "Nach Name" });
            }
            EJECT => self.eject(),
            ACT_MOVE => self.start_carry(Op::Move),
            ACT_COPY => self.start_carry(Op::Copy),
            ACT_DELETE => self.confirm_delete = true,
            CONFIRM_DELETE => self.delete(),
            CONFIRM_CANCEL => self.confirm_delete = false,
            ACT_RENAME => {
                if let Some(p) = self.selected.iter().next().cloned() {
                    self.begin_naming(Naming::Rename(p));
                }
            }
            ACT_OPEN => {
                if let Some(p) = self.selected.iter().next().cloned() {
                    self.open_file(&p);
                    return Refresh::Calm;
                }
            }
            ACT_CLEAR => self.selected.clear(),
            ACT_ALL => {
                self.selected = self.entries.iter().map(|e| e.path.clone()).collect();
            }
            HERE => self.drop_here(),
            CARRY_CANCEL => self.carry = None,
            JOB_CANCEL => {
                if let Some(j) = &self.job {
                    j.cancel.store(true, Ordering::Relaxed);
                    self.note("Wird abgebrochen …");
                }
            }
            NEW_FOLDER => self.begin_naming(Naming::NewFolder),
            FIELD_OK => self.finish_naming(),
            FIELD_CANCEL => self.naming = None,
            r if (ROW_BASE..ROW_BASE + ROWS_MAX).contains(&r) => self.tap_row(r - ROW_BASE),
            r if (CHECK_BASE..CHECK_BASE + ROWS_MAX).contains(&r) => {
                if self.carry.is_some() || self.job.is_some() {
                    return Refresh::Calm;
                }
                if let Some(p) = self.entries.get(r - CHECK_BASE).map(|e| e.path.clone()) {
                    self.toggle(&p);
                }
            }
            _ => return Refresh::Calm,
        }
        Refresh::Dirty
    }

    // ── Drawing ────────────────────────────────────────────────────────

    fn lay_side(&mut self, engine: &mut TextEngine, b: &mut SceneBuilder, area: Rect) {
        let titles: Vec<String> = self
            .places
            .iter()
            .map(|p| p.title.to_string())
            .chain(self.volumes.iter().map(|v| {
                let size = mk::format::bytes(v.size);
                if self.mounting.as_deref() == Some(v.device.as_str()) {
                    format!("{} · hängt ein …", v.name())
                } else {
                    format!("{} · {size}", v.name())
                }
            }))
            .collect();
        let entries: Vec<fam::NavEntry> = titles
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let n = self.places.len();
                let (glyph, dot) = if i < n { (self.places[i].glyph, false) } else { (symbol::USB, !self.volumes[i - n].mounted()) };
                fam::NavEntry { title: t, glyph, parts: &[], dot }
            })
            .collect();
        let selected = if self.menu { None } else { self.side_selected() };
        let height = fam::lay_nav(b, engine, &entries, selected, None, false, area, self.side_roll, SIDE_BASE, PART_BASE, self.palette);
        self.side_max = (height - area.h).max(0.0);
    }

    /// The head of the folder: (‹ Orte) ↑ name, a line of facts, sort, eject.
    fn lay_head(&mut self, engine: &mut TextEngine, b: &mut SceneBuilder, page: Rect) -> f32 {
        let p = self.palette;
        let (m, s) = (mk::spacing::M, mk::spacing::S);
        let mut x = page.x + m;
        let y = page.y + s;
        let key = 40.0;
        if self.narrow {
            let (bw, _) = fam::lay_pill(b, engine, Some('\u{e5cb}'), "Orte", x, y, None, Some(BACK), fam::PillTone::Quiet, p);
            x += bw + s;
        }
        if self.path.parent().is_some() {
            fam::lay_round_icon(b, engine, symbol::ARROW_UPWARD, x, y, key, false, Some(UP), None, p);
            x += key + s;
        }
        // Right side: sort, and «Auswerfen» on a stick.
        let mut right = page.x + page.w - m;
        right -= key;
        fam::lay_round_icon(b, engine, symbol::SWAP_VERT, right, y, key, self.newest, Some(SORT), None, p);
        if self.volume_here().is_some() {
            let text = if self.ejecting.is_some() { "Wirft aus …" } else { "Auswerfen" };
            let w = fam::pill_width(engine, Some(symbol::USB), text);
            right -= w + s;
            fam::lay_pill(b, engine, Some(symbol::USB), text, right, y, Some(w), Some(EJECT), fam::PillTone::Quiet, p);
        }
        let title = self.title();
        let st = mk::typo::HEADLINE;
        let chars = (((right - s - x) / (st.px() * 0.55)) as usize).max(6);
        let (_, th) = engine.measure_styled(&title, st.px(), st.weight);
        b.styled(engine, mk::elide(&title, chars), st, p.on_surface, x, y + (key - th) / 2.0, None, None);
        // Facts: how many, the room left, the order.
        let count = match (&self.listing, self.loading && self.entries.is_empty()) {
            (Some(_), _) => String::from("nicht lesbar"),
            (None, true) => String::from("lädt …"),
            (None, false) => match self.entries.len() {
                1 => String::from("1 Objekt"),
                n => format!("{n} Objekte"),
            },
        };
        let free = self.room.map(|(f, _)| format!("{} frei", mk::format::bytes(f))).unwrap_or_default();
        let order = if self.newest { "neueste zuerst" } else { "nach Name" };
        let facts = fam::dot_join(&[&count, &free, order]);
        let cap = mk::typo::CAPTION;
        let (_, ch) = b.styled(engine, facts, cap, p.on_surface_variant, page.x + m, y + key + 4.0, None, None);
        y + key + 4.0 + ch + s - page.y
    }

    fn lay_rows(&mut self, engine: &mut TextEngine, b: &mut SceneBuilder, list: Rect) {
        let p = self.palette;
        let m = mk::spacing::M;
        let clip = Some(list);
        let row_h = TILE + mk::spacing::S;
        let top = list.y + mk::spacing::XS;
        if self.entries.is_empty() {
            let text = match &self.listing {
                Some(e) => format!("Ordner nicht lesbar — {e}"),
                None if self.loading => String::from("Lädt …"),
                None => String::from("Leer"),
            };
            b.styled(engine, text, mk::typo::CALLOUT, p.on_surface_variant, list.x + m, top + m, None, clip);
            self.roll_max = 0.0;
            return;
        }
        let symfam = matrixkit_scene::symbol_family();
        let gpx = mk::typo::symbol_px(24.0);
        let mut want = Vec::new();
        for (i, e) in self.entries.iter().enumerate().take(ROWS_MAX) {
            let y = top + i as f32 * row_h - self.roll;
            if y + row_h < list.y || y > list.y + list.h {
                continue;
            }
            let thumb = if e.picture() { self.thumbs.get(&e.path) } else { None };
            if e.picture() && thumb.is_none() && !self.asked.contains(&e.path) {
                want.push(e.path.clone());
            }
            let size = if e.folder { String::new() } else { mk::format::bytes(e.size) };
            let age = e.changed.map(mk::format::ago).unwrap_or_default();
            // The readable name (no video id, no «.part»); the real name
            // stays in the path and in «Umbenennen».
            let shown = if e.folder { None } else { Some(fileops::shown(&e.name)) };
            let ext = shown.as_ref().map(|s| s.ext.clone()).unwrap_or_default();
            let meta = fam::dot_join(&[e.kind(), &ext, &size, &age]);
            let title = shown.map(|s| s.title).unwrap_or_else(|| e.name.clone());
            let on = self.selected.contains(&e.path);
            let carried = self.carry.as_ref().is_some_and(|(_, items)| items.contains(&e.path));
            let row = fam::MediaRow {
                thumb: thumb.map(PathBuf::as_path),
                thumb_w: TILE,
                thumb_h: TILE,
                picked: on || carried,
                trailing: Some((if on { symbol::CHECK_BOX } else { symbol::CHECK_BOX_BLANK }, on, CHECK_BASE + i)),
                ..fam::MediaRow::new(&title, &meta)
            };
            let x = list.x + m;
            fam::lay_media_row(b, engine, &row, x, y, list.w - 2.0 * m, Some(ROW_BASE + i), clip, p);
            if thumb.is_some() && e.video() {
                // A small play badge marks a film's frame.
                let d = 20.0;
                let (bx, by) = (x + TILE - d - 3.0, y + TILE - d - 3.0);
                b.quad(bx, by, d, d, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.6 }, d / 2.0, Some(ROW_BASE + i), clip);
                let g = symbol::PLAY_ARROW.to_string();
                let gp = mk::typo::symbol_px(14.0);
                let (gw, gh) = engine.measure_family(symfam, &g, gp, mk::typo::Weight::Normal);
                b.text(engine, g, gp, mk::typo::Weight::Normal, symfam, mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, bx + (d - gw) / 2.0, by + (d - gh) / 2.0, Some(ROW_BASE + i), clip);
            }
            if thumb.is_none() {
                let g = e.glyph().to_string();
                let (gw, gh) = engine.measure_family(symfam, &g, gpx, mk::typo::Weight::Normal);
                let ink = if e.folder || e.partial() { p.primary } else { p.on_surface_variant };
                b.text(engine, g, gpx, mk::typo::Weight::Normal, symfam, ink, x + (TILE - gw) / 2.0, y + (TILE - gh) / 2.0, Some(ROW_BASE + i), clip);
            }
        }
        let total = mk::spacing::XS + self.entries.len() as f32 * row_h + mk::spacing::M;
        self.roll_max = (total - list.h).max(0.0);
        self.roll = self.roll.min(self.roll_max);
        if self.roll_max > 0.0 {
            // A thin scroll hint at the list's right edge.
            let th = (list.h * list.h / total).max(28.0);
            let ty = list.y + (list.h - th) * (self.roll / self.roll_max);
            b.quad(list.x + list.w - 4.0, ty, 3.0, th, p.on_surface_variant.with_alpha(0.35), 1.5, None, None);
        }
        self.want = want;
    }

    /// A red pill (delete for good).
    fn lay_danger(&self, engine: &mut TextEngine, b: &mut SceneBuilder, text: &str, x: f32, y: f32, role: usize) -> f32 {
        let p = self.palette;
        let st = mk::typo::CALLOUT;
        let w = fam::pill_width(engine, None, text);
        let ink = if p.is_light { mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } } else { p.surface };
        b.quad(x, y, w, 40.0, p.error, 20.0, Some(role), None);
        let (_, th) = engine.measure_styled(text, st.px(), st.weight);
        b.styled(engine, text, st, ink, x + mk::spacing::M, y + (40.0 - th) / 2.0, Some(role), None);
        w
    }

    /// Pills in a row that wraps; returns the height used.
    fn lay_flow(&self, engine: &mut TextEngine, b: &mut SceneBuilder, pills: &[(Option<char>, String, usize, u8)], x0: f32, y0: f32, w: f32) -> f32 {
        let s = mk::spacing::S;
        let (mut x, mut y) = (x0, y0);
        for (glyph, text, role, tone) in pills {
            let pw = if *tone == 2 { fam::pill_width(engine, None, text) } else { fam::pill_width(engine, *glyph, text) };
            if x > x0 && x + pw > x0 + w {
                x = x0;
                y += 40.0 + s;
            }
            match tone {
                2 => {
                    self.lay_danger(engine, b, text, x, y, *role);
                }
                t => {
                    let tone = if *t == 1 { fam::PillTone::Accent } else { fam::PillTone::Quiet };
                    fam::lay_pill(b, engine, *glyph, text, x, y, None, Some(*role), tone, self.palette);
                }
            }
            x += pw + s;
        }
        y + 40.0 - y0
    }

    /// The field at the bottom (rename, new folder) — the kit's input field.
    fn lay_field(&self, engine: &mut TextEngine, b: &mut SceneBuilder, x: f32, y: f32, w: f32) -> f32 {
        let p = self.palette;
        let spec = fam::FieldSpec {
            value: &self.field,
            cursor: self.cursor,
            anchor: self.anchor,
            focused: true,
            blink: self.blink,
            placeholder: if self.naming == Some(Naming::NewFolder) { "Name des neuen Ordners …" } else { "Neuer Name …" },
            leading: Some(if self.naming == Some(Naming::NewFolder) { symbol::NEW_FOLDER } else { symbol::EDIT }),
        };
        let field = fam::input_field(engine, &spec, w, p);
        let (first, _, field_h) = b.node(&field.node, x, y, Some(FIELD), None);
        let ground = Rect::of(&b.scene.placed[first]);
        if let Some(glyph) = &field.leading {
            let gq = b.scene.placed[first + 2];
            b.text_on(
                first + 2,
                Text {
                    s: glyph.clone(),
                    px: mk::typo::symbol_px(mk::font_size::MEDIUM),
                    weight: mk::typo::Weight::Normal,
                    family: matrixkit_scene::symbol_family(),
                    color: p.on_surface_variant,
                    x: gq.x,
                    y: gq.y,
                    home: ground,
                    anchor: 0,
                    clip: None,
                },
            );
        }
        let slot = first + if field.leading.is_some() { 6 } else { 4 };
        let aq = b.scene.placed[slot];
        b.text_on(
            slot,
            Text {
                s: field.label.text.clone(),
                px: field.label.px,
                weight: field.label.weight,
                family: field.label.family,
                color: field.label.color,
                x: aq.x,
                y: aq.y,
                home: ground,
                anchor: 0,
                clip: None,
            },
        );
        field_h
    }

    /// What the bar at the bottom says, laid at `y` (measuring pass when
    /// `b` is a scratch builder). Returns its height (0: no bar).
    fn lay_bar(&self, engine: &mut TextEngine, b: &mut SceneBuilder, page: Rect, y: f32) -> f32 {
        let p = self.palette;
        let (m, s) = (mk::spacing::M, mk::spacing::S);
        let (x, w) = (page.x + m, page.w - 2.0 * m);
        let cap = mk::typo::CAPTION;
        let mut cy = y + s;
        let line = |b: &mut SceneBuilder, engine: &mut TextEngine, text: &str, cy: &mut f32| {
            for part in mk::wrap_words(text, (w / 6.5) as usize, 2) {
                let (_, h) = b.styled(engine, part, cap, p.on_surface_variant, x, *cy, None, None);
                *cy += h;
            }
            *cy += s;
        };
        if let Some(job) = &self.job {
            let pr = &job.progress;
            let verb = if job.op == Op::Move { "Verschiebt" } else { "Kopiert" };
            let head = if pr.total > 0 {
                format!("{verb} {}", mk::format::bytes_paar(pr.done, pr.total).replacen(" / ", " von ", 1))
            } else {
                format!("{verb} …")
            };
            let (_, hh) = b.styled(engine, head, mk::typo::CALLOUT, p.on_surface, x, cy, None, None);
            cy += hh + 4.0;
            let file = if pr.count > 1 { format!("{} ({} von {})", pr.file, pr.index + 1, pr.count) } else { pr.file.clone() };
            line(b, engine, &file, &mut cy);
            // The bar of bytes.
            let frac = if pr.total > 0 { (pr.done as f32 / pr.total as f32).clamp(0.0, 1.0) } else { 0.0 };
            b.quad(x, cy, w, 6.0, p.surface_container_high, 3.0, None, None);
            b.quad(x, cy, (w * frac).max(6.0), 6.0, p.primary, 3.0, None, None);
            cy += 6.0 + s;
            cy += self.lay_flow(engine, b, &[(Some(symbol::CLOSE), String::from("Abbrechen"), JOB_CANCEL, 0)], x, cy, w) + s;
            return cy - y;
        }
        if self.naming.is_some() {
            let fh = self.lay_field(engine, b, x, cy, w);
            cy += fh + s;
            let ok = if self.naming == Some(Naming::NewFolder) { "Anlegen" } else { "Umbenennen" };
            cy += self.lay_flow(engine, b, &[(Some(symbol::CHECK), ok.to_string(), FIELD_OK, 1), (Some(symbol::CLOSE), String::from("Abbrechen"), FIELD_CANCEL, 0)], x, cy, w) + s;
            return cy - y;
        }
        if self.confirm_delete {
            let n = self.selected.len();
            let what = if n == 1 { String::from("1 Objekt") } else { format!("{n} Objekte") };
            let ask = if self.on_stick() { format!("{what} endgültig vom Stick löschen?") } else { format!("{what} in den Papierkorb legen?") };
            line(b, engine, &ask, &mut cy);
            let yes = if self.on_stick() { "Endgültig löschen" } else { "In den Papierkorb" };
            cy += self.lay_flow(engine, b, &[(None, yes.to_string(), CONFIRM_DELETE, 2), (Some(symbol::CLOSE), String::from("Abbrechen"), CONFIRM_CANCEL, 0)], x, cy, w) + s;
            return cy - y;
        }
        if let Some((op, items)) = &self.carry {
            let n = items.len();
            let what = if n == 1 { items[0].file_name().map(|f| format!("«{}»", f.to_string_lossy())).unwrap_or_default() } else { format!("{n} Objekte") };
            let verb = if *op == Op::Move { "verschieben" } else { "kopieren" };
            let pick = if self.narrow { "über «Orte» ein Ziel wählen" } else { "links ein Ziel wählen" };
            line(b, engine, &format!("{what} {verb} — {pick} (auch den Stick), Ordner öffnen, dann «Hierher»"), &mut cy);
            let here = if *op == Op::Move { "Hierher verschieben" } else { "Hierher kopieren" };
            let pills = [
                (Some(symbol::CHECK), here.to_string(), HERE, 1),
                (Some(symbol::NEW_FOLDER), String::from("Neuer Ordner"), NEW_FOLDER, 0),
                (Some(symbol::CLOSE), String::from("Abbrechen"), CARRY_CANCEL, 0),
            ];
            cy += self.lay_flow(engine, b, &pills, x, cy, w) + s;
            return cy - y;
        }
        if !self.selected.is_empty() {
            let n = self.selected.len();
            let bytes = self.selected_bytes();
            let head = fam::dot_join(&[&format!("{n} ausgewählt"), &if bytes > 0 { mk::format::bytes(bytes) } else { String::new() }]);
            // The head: what is chosen, «alle» and «✕» as round keys at its end.
            let key = 36.0;
            let (_, hh) = engine.measure_styled(&head, cap.px(), cap.weight);
            b.styled(engine, head, cap, p.on_surface_variant, x, cy + (key - hh) / 2.0, None, None);
            fam::lay_round_icon(b, engine, symbol::CLOSE, x + w - key, cy, key, false, Some(ACT_CLEAR), None, p);
            if n < self.entries.len() {
                fam::lay_round_icon(b, engine, symbol::CHECK_BOX, x + w - 2.0 * key - s, cy, key, false, Some(ACT_ALL), None, p);
            }
            cy += key + s;
            let mut pills = vec![
                (Some(symbol::ARROW_FORWARD), String::from("Verschieben nach …"), ACT_MOVE, 1),
                (Some(symbol::CONTENT_COPY), String::from("Kopieren nach …"), ACT_COPY, 0),
            ];
            if n == 1 {
                let one = self.selected.iter().next().cloned().unwrap_or_default();
                if one.is_file() {
                    pills.push((Some('\u{e89e}'), String::from("Öffnen"), ACT_OPEN, 0));
                }
                pills.push((Some(symbol::EDIT), String::from("Umbenennen"), ACT_RENAME, 0));
            }
            pills.push((Some(symbol::DELETE), if self.deleting { String::from("Löscht …") } else { String::from("Löschen") }, ACT_DELETE, 0));
            cy += self.lay_flow(engine, b, &pills, x, cy, w) + s;
            return cy - y;
        }
        0.0
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
            self.list_rect = zero();
            self.lay_side(engine, &mut b, body);
        } else {
            let (side, page) = if narrow { (zero(), body) } else { (Rect { x: 0.0, y: top, w: SIDE_W, h: body.h }, Rect { x: SIDE_W, y: top, w: wf - SIDE_W, h: body.h }) };
            self.side_rect = side;
            if !narrow {
                b.quad(side.x, side.y, side.w, side.h, p.surface_container.with_alpha(0.55), 0.0, None, None);
                self.lay_side(engine, &mut b, side);
            }
            let head_h = self.lay_head(engine, &mut b, page);
            // The bar: measured first on a scratch builder, then laid.
            let bar_h = {
                let mut scratch = SceneBuilder::new();
                self.lay_bar(engine, &mut scratch, page, 0.0)
            };
            let list = Rect { x: page.x, y: page.y + head_h, w: page.w, h: (page.h - head_h - bar_h).max(0.0) };
            self.list_rect = list;
            self.lay_rows(engine, &mut b, list);
            if bar_h > 0.0 {
                let by = page.y + page.h - bar_h;
                b.quad(page.x, by, page.w, bar_h, p.surface_container, 0.0, None, None);
                b.quad(page.x, by, page.w, 1.0, p.outline.with_alpha(0.25), 0.0, None, None);
                self.lay_bar(engine, &mut b, page, by);
            }
            // The receipt of the last deed, above the bar.
            if let Some(note) = self.receipt.text() {
                let note = note.to_string();
                let wn = fam::pill_width(engine, Some(symbol::CHECK), &note).min(page.w - 2.0 * mk::spacing::M);
                let x = page.x + (page.w - wn) / 2.0;
                fam::lay_pill(&mut b, engine, Some(symbol::CHECK), &note, x, page.y + page.h - bar_h - 52.0, Some(wn), None, fam::PillTone::Accent, p);
            }
        }
        // A receipt also shows on the narrow places list.
        if narrow && self.menu {
            if let Some(note) = self.receipt.text() {
                let note = note.to_string();
                let wn = fam::pill_width(engine, Some(symbol::CHECK), &note).min(wf - 2.0 * mk::spacing::M);
                fam::lay_pill(&mut b, engine, Some(symbol::CHECK), &note, (wf - wn) / 2.0, hf - 56.0, Some(wn), None, fam::PillTone::Accent, p);
            }
        }
        let mut scene = b.finish();
        if self.root.visible() {
            let mut rb = SceneBuilder::new();
            rb.scene = scene;
            let about = fam::RootAbout {
                name: "Matrix Dateien",
                version: env!("CARGO_PKG_VERSION"),
                description: "Ordner durchsehen, Dateien verschieben und kopieren — auch auf den USB-Stick, mit Fortschritt und sicherem Auswerfen.",
            };
            self.root.lay(engine, &mut rb, &about, wf, hf, p, ROOT_BASE);
            scene = rb.finish();
        }
        scene
    }

    /// Thumbnails the last frame wanted, made on the loom.
    fn ask_thumbs(&mut self) {
        let want = std::mem::take(&mut self.want);
        let Some(loom) = &self.loom else { return };
        for src in want {
            if !self.asked.insert(src.clone()) {
                continue;
            }
            loom.spin(move || {
                let t = matrixkit_image::thumbs::ensure_any(&src, THUMB_PX, THUMB_PX);
                Work::Thumb(src, t)
            });
        }
    }
}

impl shell::Client for Files {
    type Card = ();
    type Work = Work;

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn warp(&mut self, loom: &Loom<Work>) {
        self.loom = Some(loom.clone());
        self.relist();
        self.poll();
    }

    fn woven(&mut self, work: Work) -> Refresh {
        match work {
            Work::Listed(seq, path, listed, room) => {
                if seq != self.seq || path != self.path {
                    return Refresh::Calm;
                }
                self.loading = false;
                self.room = room;
                self.stamp = dir_stamp(&path);
                match listed {
                    Ok(mut entries) => {
                        sort(&mut entries, self.newest);
                        // Keep only selections that still exist here.
                        let here: HashSet<&PathBuf> = entries.iter().map(|e| &e.path).collect();
                        self.selected.retain(|p| here.contains(p));
                        self.entries = entries;
                        self.listing = None;
                    }
                    Err(e) => {
                        self.entries.clear();
                        self.listing = Some(e);
                    }
                }
            }
            Work::Poll(found, stamp) => {
                self.polling = false;
                // A stick that came: say so; one that went while open: leave.
                for v in &found {
                    if !self.volumes.iter().any(|o| o.device == v.device) && self.beats > 0 {
                        self.note(format!("«{}» erkannt — links antippen", v.name()));
                    }
                }
                let gone_here = self.volume_here().is_some_and(|v| !found.iter().any(|f| f.device == v.device && f.mounted()));
                self.volumes = found;
                if gone_here && self.ejecting.is_none() {
                    self.note("Der Stick ist weg");
                    let away = self.places[0].path.clone();
                    self.go(away);
                } else if stamp.is_some() && stamp != self.stamp && !self.loading && self.job.is_none() {
                    self.relist();
                }
            }
            Work::Mounted(dev, r) => {
                self.mounting = None;
                match r {
                    Ok(path) => {
                        eprintln!("[dateien] {dev} eingehängt: {}", path.display());
                        self.receipt.clear();
                        self.go(path);
                    }
                    Err(e) => self.note(format!("Einhängen ging nicht: {e}")),
                }
                self.poll();
            }
            Work::Ejected(_disk, name, r) => {
                self.ejecting = None;
                match r {
                    Ok(()) => {
                        mk::feel::confirm();
                        self.note(format!("«{name}» kann entfernt werden"));
                    }
                    Err(e) => self.note(format!("Auswerfen ging nicht: {e}")),
                }
                self.poll();
            }
            Work::Progress(pr) => {
                if let Some(j) = &mut self.job {
                    j.progress = pr;
                }
            }
            Work::Done(op, dest, result) => {
                self.job = None;
                match result {
                    Ok(sum) => {
                        let n = sum.arrived.len();
                        let what = if n == 1 { String::from("1 Objekt") } else { format!("{n} Objekte") };
                        let verb = if op == Op::Move { "verschoben" } else { "kopiert" };
                        let place = dest.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
                        let skipped = if sum.skipped > 0 { format!(" ({} Verknüpfungen ausgelassen)", sum.skipped) } else { String::new() };
                        mk::feel::confirm();
                        self.note(format!("{what} {verb} nach «{place}»{skipped}"));
                    }
                    Err(e) => self.note(format!("Nicht fertig: {e}")),
                }
                self.relist();
            }
            Work::Thumb(src, t) => {
                if let Some(t) = t {
                    self.thumbs.insert(src, t);
                }
            }
            Work::Deleted(n, r) => {
                self.deleting = false;
                match r {
                    Ok(()) => {
                        self.selected.clear();
                        self.note(if n == 1 { String::from("1 Objekt gelöscht") } else { format!("{n} Objekte gelöscht") });
                    }
                    Err(e) => self.note(format!("Löschen ging nicht: {e}")),
                }
                self.relist();
            }
        }
        Refresh::Dirty
    }

    fn base(&mut self, _atlas: &mut ImageAtlas, buf: &mut [u8], _w: u32, _h: u32) {
        let p = self.palette;
        let (b, g, r) = ((p.surface.b * 255.0) as u8, (p.surface.g * 255.0) as u8, (p.surface.r * 255.0) as u8);
        for px in buf.chunks_exact_mut(4) {
            px.copy_from_slice(&[b, g, r, 0xff]);
        }
    }

    fn view(&mut self, engine: &mut TextEngine, _keep: &mut Keep<f32>, _atlas: &mut ImageAtlas, w: u32, h: u32) -> Scene {
        let scene = self.view_inner(engine, w, h);
        self.ask_thumbs();
        scene
    }

    fn animating(&self) -> bool {
        self.chrome.animating() || self.root.animating()
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        if self.chrome.glide(dt) | self.root.glide(dt) {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn beat(&mut self) -> Refresh {
        self.beats += 1;
        self.blink = !self.blink;
        let _ = self.root.beat();
        if self.beats % POLL_BEATS == 0 {
            self.poll();
        }
        if self.naming.is_some() || self.receipt.alive() || self.job.is_some() || self.mounting.is_some() || self.ejecting.is_some() {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn pulse(&mut self) -> Refresh {
        let fresh = mk::Palette::load().unwrap_or_default();
        if fresh != self.palette {
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
                    matrixkit_scene::help_open("Matrix Dateien");
                    Refresh::Calm
                }
                fam::RootDeed::Calm => Refresh::Calm,
                _ => Refresh::Dirty,
            };
        }
        match self.chrome.strike(role) {
            Some(fam::ChromeDeed::Close) => {
                eprintln!("[dateien] Ampel × ({})", source.label());
                return Refresh::Close;
            }
            Some(fam::ChromeDeed::Answer(r)) => return r,
            Some(fam::ChromeDeed::Title) => {
                self.root.toggle();
                return Refresh::Dirty;
            }
            None => {}
        }
        if role == Some(FIELD) {
            return Refresh::Calm;
        }
        // Everything else acts on release; a moving finger scrolls.
        let inside = |r: Rect| r.w > 0.0 && xf >= r.x && xf < r.x + r.w && yf >= r.y && yf < r.y + r.h;
        let zone = if inside(self.side_rect) {
            Zone::Side
        } else if inside(self.list_rect) {
            Zone::List
        } else {
            Zone::Other
        };
        let roll = match zone {
            Zone::Side => self.side_roll,
            Zone::List => self.roll,
            Zone::Other => 0.0,
        };
        self.press = Some(Press::new(xf, yf, roll, (zone, role)));
        Refresh::Calm
    }

    fn travel(&mut self, x: f64, y: f64, _source: Source) -> Refresh {
        let Some(pr) = &mut self.press else { return Refresh::Calm };
        let zone = pr.what.0;
        let max = match zone {
            Zone::Side => self.side_max,
            Zone::List => self.roll_max,
            Zone::Other => 0.0,
        };
        match pr.travel(x as f32, y as f32, max) {
            Some(r) if zone == Zone::Side => self.side_roll = r,
            Some(r) if zone == Zone::List => self.roll = r,
            _ => return Refresh::Calm,
        }
        Refresh::Dirty
    }

    fn release(&mut self, _role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        match self.press.take().and_then(Press::release) {
            Some((_, Some(r))) => self.act(r),
            _ => Refresh::Calm,
        }
    }

    /// A long press on a row selects it (the house measured the hold).
    fn hold(&mut self, role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        let Some(r) = role else { return Refresh::Calm };
        let i = if (ROW_BASE..ROW_BASE + ROWS_MAX).contains(&r) {
            r - ROW_BASE
        } else if (CHECK_BASE..CHECK_BASE + ROWS_MAX).contains(&r) {
            r - CHECK_BASE
        } else {
            return Refresh::Calm;
        };
        if self.press.as_ref().is_some_and(|p| p.scrolling()) || self.carry.is_some() || self.job.is_some() {
            return Refresh::Calm;
        }
        self.press = None;
        if let Some(p) = self.entries.get(i).map(|e| e.path.clone()) {
            self.selected.insert(p);
            self.confirm_delete = false;
            mk::feel::hold();
        }
        Refresh::Dirty
    }

    /// Right click: like a long press.
    fn secondary(&mut self, role: Option<usize>, x: f64, y: f64) -> Refresh {
        self.hold(role, x, y, Source::Pointer)
    }

    fn axis(&mut self, x: f64, _y: f64, value: f64, _keep: &mut Keep<f32>) -> Refresh {
        if self.side_rect.w > 0.0 && (x as f32) < self.side_rect.x + self.side_rect.w {
            self.side_roll = (self.side_roll + value as f32).clamp(0.0, self.side_max);
        } else {
            self.roll = (self.roll + value as f32).clamp(0.0, self.roll_max);
        }
        Refresh::Dirty
    }

    fn motion(&mut self, x: f64, y: f64) -> Refresh {
        self.chrome.motion(x, y)
    }

    fn depart(&mut self) -> Refresh {
        self.chrome.depart()
    }

    fn key(&mut self, key: shell::Key, shift: bool) -> Refresh {
        if self.root.key(&key, shift) {
            return Refresh::Dirty;
        }
        if self.naming.is_some() {
            return match fam::field_edit(&mut self.field, &mut self.cursor, &mut self.anchor, &key, shift) {
                fam::FieldDeed::Changed => Refresh::Dirty,
                fam::FieldDeed::Submitted => {
                    self.finish_naming();
                    Refresh::Dirty
                }
                fam::FieldDeed::Escaped => {
                    self.naming = None;
                    Refresh::Dirty
                }
                fam::FieldDeed::Calm => Refresh::Calm,
            };
        }
        match key {
            shell::Key::Escape => {
                if self.confirm_delete {
                    self.confirm_delete = false;
                } else if self.carry.is_some() {
                    self.carry = None;
                } else {
                    self.selected.clear();
                }
                Refresh::Dirty
            }
            shell::Key::Backspace => self.act(UP),
            shell::Key::Function(5) => {
                self.relist();
                Refresh::Dirty
            }
            _ => Refresh::Calm,
        }
    }

    fn wants_keyboard(&self) -> bool {
        self.naming.is_some()
    }

    fn wants_text(&self) -> bool {
        self.naming.is_some()
    }

    fn root(&mut self) -> Refresh {
        if self.root.open() {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn closing(&mut self) {
        // A transfer in flight stops cleanly: cancel removes the
        // half-written file (finished ones stay) — wait for that, briefly,
        // or the process would end mid-file and leave a torn copy behind.
        if let Some(j) = &self.job {
            j.cancel.store(true, Ordering::Relaxed);
            let t0 = std::time::Instant::now();
            while !j.ended.load(Ordering::SeqCst) && t0.elapsed() < std::time::Duration::from_secs(5) {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            eprintln!("[dateien] beim Schliessen abgebrochen: {}", j.progress.file);
        }
    }
}

/// The house version (default). `matrix-files <folder>` opens that folder.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    shell::prefer_gpu();
    let start = std::env::args().skip(1).filter(|a| !a.starts_with("--")).map(PathBuf::from).find(|p| p.is_dir());
    eprintln!("[dateien] Matrix Dateien im Fensterhaus, pid={}", std::process::id());
    fam::run(
        Files::new(start),
        shell::Options {
            mount: shell::Mount::Window { app_id: String::from(APP_ID), title: String::from("Matrix Dateien"), width: 900, height: 640 },
            argb: false,
            tag: "files",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}
