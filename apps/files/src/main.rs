//! Matrix Dateien — the file manager, app #25 (round 29).
//!
//! The file-manager reference extract, read live on the reference system:
//!
//! * **Anatomy**: sidebar (FAVORITEN / ORTE = favourites / places) | content with ‹ › history,
//!   sortable column head (name / modified date / size / kind, chevron
//!   shows direction), rows with zebra stripes and an accent pill across
//!   the FULL row on selection — exactly the reference file manager's dark look.
//! * **Path bar below**: breadcrumbs, clickable — and it follows the SELECTION
//!   (reference detail: the chosen object hangs at the end).
//! * **Interaction**: single click selects, double click opens (folders enter,
//!   files via the system opener), Enter renames (the reference quirk!),
//!   ↑/↓ wander, Esc closes menu/edit, Ctrl+R refreshes.
//! * **Context menu** (right click, copied from the real reference):
//!   open | rename / duplicate / copy path | move to trash (red).
//!   Deleting goes to the trash (gio trash), never hard.

mod haus;
mod iphone;
mod mac_parity;

use matrixkit_widgets::iced;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

use matrixkit_widgets::iced::widget::{button, column, container, mouse_area, row, Space};
use matrixkit_widgets::iced::{Alignment, Element, Length, Subscription, Task};
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use mkw::color;
use std::time::Duration;

/// Album card symbol: between XLARGE and HERO — a deliberate special size.
const ALBUM_SYMBOL: f32 = 44.0;

const APP_ID: &str = "matrix-files";
/// Double-click window: the kit token (R66; R33 empirics confirmed).
const DOUBLE_CLICK_MS: u128 = mk::input::DOUBLE_CLICK_MS;

fn main() -> iced::Result {
    // R217 (owner 02.11.2026: files onto the USB stick, for the Duo): the
    // house is the default. `--iced` opens the previous version as its own
    // instance (another app id); a house that cannot start falls back.
    if std::env::args().any(|a| a == "--iced") {
        return mkw::app::window::<App>("matrix-files-iced", 920.0, 640.0);
    }
    match haus::run() {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("[dateien] Fensterhaus-Fehler ({e}) — Rückfall auf iced");
            mkw::app::window::<App>(APP_ID, 920.0, 640.0)
        }
    }
}

impl mkw::app::Program for App {
    type Msg = Msg;
    fn new() -> (Self, Task<Msg>) {
        App::new()
    }
    fn update(&mut self, msg: Msg) -> Task<Msg> {
        App::update(self, msg)
    }
    fn view(&self) -> Element<'_, Msg> {
        App::view(self)
    }
    fn title(&self) -> String {
            self.path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| String::from("Matrix Dateien"))
        }
    fn subscription(&self) -> iced::Subscription<Msg> {
        App::subscription(self)
    }
}

// ---------------------------------------------------------------- entries

#[derive(Debug, Clone)]
struct Entry {
    name: String,
    path: PathBuf,
    folder: bool,
    size: u64,
    changed: Option<SystemTime>,
}

impl Entry {
    fn hidden(&self) -> bool {
        self.name.starts_with('.')
    }

    /// Display name: AppImages do not show their extension — they are
    /// programs, not files (the truth stays in `name`).
    fn display_name(&self) -> &str {
        let n = self.name.as_str();
        if n.len() > 9 && n[n.len() - 9..].eq_ignore_ascii_case(".appimage") {
            &n[..n.len() - 9]
        } else {
            n
        }
    }

    /// The "Art" (kind) column of the reference: human-readable type names.
    fn art(&self) -> &'static str {
        if self.folder {
            return "Ordner";
        }
        let extension = self
            .name
            .rsplit('.')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        match extension.as_str() {
            "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "svg" | "heic" => "Bild",
            "mp4" | "mkv" | "webm" | "mov" | "avi" => "Film",
            "mp3" | "flac" | "ogg" | "wav" | "m4a" | "opus" => "Musik",
            "pdf" => "PDF-Dokument",
            "txt" | "md" => "Text",
            "rs" => "Rust-Quelltext",
            "toml" | "json" | "kdl" | "yaml" | "yml" | "conf" | "ini" => "Konfiguration",
            "sh" | "bash" => "Shell-Skript",
            "py" => "Python-Skript",
            "zip" | "tar" | "gz" | "xz" | "zst" | "7z" => "Archiv",
            "log" => "Protokolldatei",
            "desktop" => "App-Verknüpfung",
            "iso" | "img" => "Abbild",
            "" => "Dokument",
            _ => "Dokument",
        }
    }

    /// The row symbol for the kind.
    fn glyph(&self) -> char {
        if self.folder {
            return mkw::symbol::FOLDER;
        }
        match self.art() {
            "Bild" => mkw::symbol::IMAGE,
            "Film" => mkw::symbol::MOVIE,
            "Musik" => mkw::symbol::MUSIC_NOTE,
            "PDF-Dokument" => mkw::symbol::PDF,
            "Rust-Quelltext" | "Konfiguration" | "Shell-Skript" | "Python-Skript" => {
                mkw::symbol::CODE
            }
            _ => mkw::symbol::FILE,
        }
    }
}

/// Sizes like the reference file manager — de_CH empirics (R35): Switzerland
/// separates with a POINT ("610.7 MB"), not a German comma.
fn size_text(e: &Entry) -> String {
    if e.folder {
        return String::from("—");
    }
    // R65b: the ONE number language of the kit.
    mk::format::bytes(e.size)
}

/// Date like the reference: "18.06.2026, 11:28".
/// Load the peek for an entry (R98): images as full preview
/// (decoding is done lazily by the renderer), readable files as a
/// text head, everything else as icon + facts. Deliberately synchronous —
/// only the text head is read, never the whole file.
fn peek_load(e: &Entry) -> Peek {
    let subtitle = if e.folder {
        e.art().to_string()
    } else {
        format!("{} · {} · {}", e.art(), mk::format::bytes(e.size), date_text(e.changed))
    };
    let extension = e
        .path
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| x.to_ascii_lowercase())
        .unwrap_or_default();
    let image = matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp")
        .then(|| iced::widget::image::Handle::from_path(&e.path));
    let text = if image.is_none() && !e.folder {
        // Readable = the first 16 KB are valid UTF-8 (covers code,
        // config, markdown, logs — without extension guessing).
        std::fs::File::open(&e.path)
            .ok()
            .and_then(|mut f| {
                use std::io::Read;
                let mut puffer = vec![0u8; 16 * 1024];
                let n = f.read(&mut puffer).ok()?;
                puffer.truncate(n);
                String::from_utf8(puffer).ok()
            })
            .filter(|t| !t.trim().is_empty())
            .map(|t| t.lines().take(60).collect::<Vec<_>>().join("\n"))
    } else {
        None
    };
    Peek {
        name: e.name.clone(),
        subtitle,
        image,
        text,
        glyph: if e.folder { mkw::symbol::FOLDER } else { mkw::symbol::CONTENT_COPY },
    }
}

fn date_text(time: Option<SystemTime>) -> String {
    match time {
        Some(t) => chrono::DateTime::<chrono::Local>::from(t)
            .format("%d.%m.%Y, %H:%M")
            .to_string(),
        None => String::from("—"),
    }
}

// ------------------------------------------------------------- Sorting

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Column {
    Name,
    Date,
    Size,
    Art,
}

fn sort_all(entries: &mut [Entry], column: Column, descending: bool) {
    entries.sort_by(|a, b| {
        let ord = match column {
            Column::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            Column::Date => a.changed.cmp(&b.changed),
            Column::Size => a.size.cmp(&b.size),
            Column::Art => a
                .art()
                .cmp(b.art())
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        };
        if descending { ord.reverse() } else { ord }
    });
}

fn read(path: &Path, hidden: bool, column: Column, descending: bool) -> Vec<Entry> {
    let mut out = Vec::new();
    if let Ok(dir) = std::fs::read_dir(path) {
        for e in dir.flatten() {
            let meta = match e.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let entry = Entry {
                name: e.file_name().to_string_lossy().to_string(),
                path: e.path(),
                folder: meta.is_dir(),
                size: meta.len(),
                changed: meta.modified().ok(),
            };
            if !hidden && entry.hidden() {
                continue;
            }
            out.push(entry);
        }
    }
    sort_all(&mut out, column, descending);
    out
}

// -------------------------------------------------------------- Sidebar

/// Favorites like the reference: (title, path relative to $HOME, glyph).
const FAVORITES: &[(&str, &str, char)] = &[
    // User doctrine 2026-07-19: it is called Leinwand — the folder too
    // (XDG desktop folder = ~/Leinwand; legacy falls back to the old
    // folder name, see favorit_pfad).
    ("Leinwand", "Leinwand", mkw::symbol::CANVAS),
    ("Dokumente", "Dokumente", mkw::symbol::FILE),
    ("Downloads", "Downloads", mkw::symbol::DOWNLOAD),
    ("Musik", "Musik", mkw::symbol::MUSIC_NOTE),
];

// ------------------------------------------------------------------ Gallery
//
// Round 40 (user): images and videos no longer appear separately —
// ONE entry "Gallery". Entering it (or the folders) shows no file
// list anymore but a photo app: albums (= subfolders, new ones via
// "Neues Album"), thumbnail grid, filter Alle/Bilder/Videos (all/images/videos).

fn images_wurzel() -> PathBuf {
    home().join("Bilder")
}

/// R185: THE Hardline folder — everything Hardline lives here, and
/// only here. Opening it asks for the account password first.
fn hardline_dir() -> PathBuf {
    home().join("Hardline")
}

/// R187: whether the sidebar shows it. Switching the SERVICE off in
/// the dock must NOT make the folder vanish — hiding it is its own
/// decision, made in Einstellungen → Hardline.
fn hardline_visible() -> bool {
    std::fs::read_to_string(home().join(".config/matrix/hardline-folder"))
        .map(|raw| raw.trim() != "aus")
        .unwrap_or(true)
}
/// Claude's folder — everything the assistant keeps (settings,
/// memory, sessions) lives here as ordinary visible files, same idea
/// as Hardline: one folder, no dot-dir mystery. The sidebar shows it
/// as soon as the folder exists; no config toggle needed.
fn claude_dir() -> PathBuf {
    home().join("Claude")
}

/// A sync service's folder (Cloud-Ordner, Nextcloud, Dropbox, OneDrive, Google
/// Drive) — the first one that exists, with its name; visible in the
/// sidebar as soon as it exists, like the other services.
fn cloud_dir() -> Option<(PathBuf, &'static str)> {
    ["Cloud-Ordner", "Nextcloud", "Dropbox", "OneDrive", "Google Drive"]
        .into_iter()
        .map(|name| (home().join(name), name))
        .find(|(dir, _)| dir.is_dir())
}

fn videos_wurzel() -> PathBuf {
    home().join("Videos")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Gallery {
    None,
    /// ~/Bilder OR ~/Videos: the combined gallery root.
    Wurzel,
    /// A subfolder inside — an album.
    Album,
}

fn gallery_from(path: &Path) -> Gallery {
    let (b, v) = (images_wurzel(), videos_wurzel());
    if path == b || path == v {
        Gallery::Wurzel
    } else if path.starts_with(&b) || path.starts_with(&v) {
        Gallery::Album
    } else {
        Gallery::None
    }
}

/// Squashfs offset of a type-2 AppImage: the embedded filesystem
/// starts right behind the ELF section headers.
fn appimage_offset(path: &std::path::Path) -> Option<u64> {
    use std::io::Read;
    let mut f = std::fs::File::open(path).ok()?;
    let mut header = [0u8; 64];
    f.read_exact(&mut header).ok()?;
    if &header[0..4] != b"\x7fELF" {
        return None;
    }
    let shoff = u64::from_le_bytes(header[0x28..0x30].try_into().ok()?);
    let shentsize = u16::from_le_bytes(header[0x3a..0x3c].try_into().ok()?) as u64;
    let shnum = u16::from_le_bytes(header[0x3c..0x3e].try_into().ok()?) as u64;
    (shoff > 0).then_some(shoff + shentsize * shnum)
}

/// Pull the embedded app icon (.DirIcon) — unsquashfs reads the
/// squashfs at the offset WITHOUT ever executing the AppImage.
/// Symlink chains (usual: .DirIcon → app.png) are chased.
fn appimage_icon(path: &std::path::Path) -> Option<image::DynamicImage> {
    let offset = appimage_offset(path)?;
    let tmp = std::env::temp_dir().join(format!(
        "matrix-appicon-{}-{}",
        std::process::id(),
        path.file_name()?.to_string_lossy()
    ));
    let _ = std::fs::remove_dir_all(&tmp);
    let run = |target: &str| {
        std::process::Command::new("unsquashfs")
            .args(["-o", &offset.to_string(), "-f", "-no-xattrs", "-d"])
            .arg(&tmp)
            .arg(path)
            .arg(target)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|st| st.success())
            .unwrap_or(false)
    };
    if !run(".DirIcon") {
        return None;
    }
    let mut icon = tmp.join(".DirIcon");
    for _ in 0..3 {
        let Ok(meta) = std::fs::symlink_metadata(&icon) else { break };
        if !meta.file_type().is_symlink() {
            break;
        }
        let target = std::fs::read_link(&icon).ok()?;
        let rel = target.to_string_lossy().trim_start_matches("./").to_string();
        if !run(&rel) {
            let _ = std::fs::remove_dir_all(&tmp);
            return None;
        }
        icon = tmp.join(rel);
    }
    let image = image::open(&icon).ok();
    let _ = std::fs::remove_dir_all(&tmp);
    image
}

/// Gallery thumbs in the background — the image-tile family recipe (R44):
/// 16:9 crop, 448x252, baked corners — exactly like the wallpaper
/// section of Matrix Einstellungen.
fn thumbs_generate(paths: Vec<PathBuf>) -> Vec<(PathBuf, u32, u32, Vec<u8>)> {
    paths
        .into_iter()
        .filter_map(|path| {
            if is_appimage(&path) {
                // App icon: square, no gallery crop.
                let icon = appimage_icon(&path)?.thumbnail(64, 64).to_rgba8();
                let (w, h) = icon.dimensions();
                return Some((path, w, h, icon.into_raw()));
            }
            let image = image::open(&path).ok()?;
            let (w, h, raw) = mkw::image::tile_thumb(image);
            Some((path, w, h, raw))
        })
        .collect()
}

/// The programs folder (R48): the /Applications of Matrix. Drop an
/// AppImage in = installed (the start menu lists it), delete =
/// uninstalled. Kit apps meanwhile live sealed in the image — exactly
/// the reference split /System/Applications vs. /Applications.
fn programs() -> PathBuf {
    home().join("Programme")
}

/// Start an AppImage: ensure the exec bit, spawn directly, reap the
/// child in a thread (zombie lesson 2026-07-07).
fn appimage_start(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mut permissions = meta.permissions();
        if permissions.mode() & 0o111 == 0 {
            permissions.set_mode(permissions.mode() | 0o755);
            let _ = std::fs::set_permissions(path, permissions);
        }
    }
    if let Ok(mut child) = std::process::Command::new(path).spawn() {
        std::thread::spawn(move || {
            let _ = child.wait();
        });
    }
}

fn is_appimage(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("appimage"))
}

/// The trash (freedesktop Trash) — reference extract R35.
fn trash() -> PathBuf {
    home().join(".local/share/Trash/files")
}

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from("/")))
}

/// Mounted volumes (USB sticks etc.) under /run/media/$USER.
fn drives() -> Vec<(String, PathBuf)> {
    let user = std::env::var("USER").unwrap_or_default();
    let base = PathBuf::from("/run/media").join(user);
    let mut out = Vec::new();
    if let Ok(dir) = std::fs::read_dir(&base) {
        for e in dir.flatten() {
            if e.path().is_dir() {
                out.push((e.file_name().to_string_lossy().to_string(), e.path()));
            }
        }
    }
    out.sort();
    out
}

/// Free space of the filesystem ("234,5 GB verfügbar") — via df, which
/// speaks every filesystem.
fn free_text(path: &Path) -> String {
    let out = std::process::Command::new("df")
        .args(["-h", "--output=avail"])
        .arg(path)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    match out.lines().last() {
        Some(z) if !z.is_empty() => format!("{} verfügbar", z.trim()),
        _ => String::new(),
    }
}

/// Find a free "Name Kopie[.ext]" path next to the original.
fn copy_path(orig: &Path) -> PathBuf {
    let stem = orig.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let extension = orig.extension().map(|s| format!(".{}", s.to_string_lossy())).unwrap_or_default();
    let parent = orig.parent().unwrap_or(Path::new("/"));
    for n in 0..99 {
        let name = if n == 0 {
            format!("{stem} Kopie{extension}")
        } else {
            format!("{stem} Kopie {n}{extension}")
        };
        let p = parent.join(&name);
        if !p.exists() {
            return p;
        }
    }
    parent.join(format!("{stem} Kopie 99{extension}"))
}

// ------------------------------------------------------------------- App

/// A drag target (R68): folders (incl. sidebar places, trash) or
/// the programs folder (accepts only AppImages — installation).
#[derive(Clone, Debug, PartialEq)]
enum DragTarget {
    Folder(PathBuf),
    Programs,
}

/// The running drag operation (R68, drag-and-drop grammar).
struct Drag {
    path: PathBuf,
    name: String,
    glyph: char,
    is_appimage: bool,
    /// R88: the pressed entry (for the set collapsing on click without
    /// a drag) and the remaining paths of the multi-selection.
    index: usize,
    further: Vec<PathBuf>,
    start: iced::Point,
    /// Only after the 4 pt threshold does the press become a drag.
    active: bool,
    target: Option<DragTarget>,
    /// For spring-loading: since when does the drag hover over the target?
    floats_since: Option<Instant>,
}

/// The peek (R98, Quick Look extract): the fleeting large preview on
/// the space bar. No window, no mode — a card that comes and goes.
/// Images fully, text as a head, everything else as icon + facts.
struct Peek {
    name: String,
    subtitle: String,
    /// Image preview (iced decodes the path lazily in the renderer).
    image: Option<iced::widget::image::Handle>,
    /// Text head (first ~60 lines) for readable files.
    text: Option<String>,
    /// Fallback glyph when neither image nor text.
    glyph: char,
}

/// An info sheet: details on an entry — since R90 as a floating card
/// above the view (the reference info window, without a window).
struct InfoSheet {
    path: PathBuf,
    name: String,
    folder: bool,
    art: String,
    size_text: String,
    changed_text: String,
    ort: String,
    wherefroms: Option<String>,
}

struct App {
    frame: mkw::Frame,
    path: PathBuf,
    entries: Vec<Entry>,
    selection: Option<usize>,
    /// Finder extract (R88): the selection is a SET — `auswahl` stays
    /// the anchor (arrows, rename, info), `markiert` carries all chosen
    /// entries. Invariant: anchor ∈ markiert.
    marked: std::collections::BTreeSet<usize>,
    /// Live state of the modifiers (Ctrl = add, Shift = range).
    mods: iced::keyboard::Modifiers,
    last_click: Option<(usize, Instant)>,
    drag: Option<Drag>,
    history: Vec<PathBuf>,
    history_pos: usize,
    column: Column,
    descending: bool,
    search: String,
    hidden: bool,
    /// Inline rename: (index, edit text).
    rename: Option<(usize, String)>,
    /// Reference extract (R33): extension changed → only the second Enter counts.
    extension_confirm: bool,
    /// WarnOnEmptyTrash (R35): emptying wants a second, red click.
    clear_confirm: bool,
    /// New folder in edit (name).
    new_folder: Option<String>,
    /// R185: the password gate in front of ~/Hardline. Some = the
    /// prompt is open (typed text); unlocked lasts for this run.
    hardline_gate: Option<String>,
    hardline_unlocked: bool,
    /// Context menu on entry i (root overlay).
    menu: Option<usize>,
    free: String,
    places: Vec<(String, PathBuf)>,
    /// Gallery: finished thumbnails (path → handle) + filter level.
    thumbs: std::collections::HashMap<PathBuf, iced::widget::image::Handle>,
    /// 0 = all, 1 = images, 2 = videos.
    gfilter: usize,
    /// R90: the info sheet as an overlay card (None = closed).
    info: Option<InfoSheet>,
    /// R98: the peek — space-bar preview (None = closed).
    peek: Option<Peek>,
    /// R149: attached iOS devices — sidebar places that mount on click.
    iphones: Vec<iphone::Device>,
    /// What to say when a device refused to open (locked, untrusted).
    device_note: Option<String>,
}

#[derive(Debug, Clone)]
enum Msg {
    Frame(mkw::FrameMsg),
    /// An Apple device appeared or left the bus (udev told us).
    Devices,
    /// Open device i from the sidebar — mounting it first if needed.
    IPhoneOpen(usize),
    /// Eject the device the current folder lies on.
    IPhoneEject,
    Key(mkw::Key),
    Tick,
    Goto(PathBuf),
    Back,
    Forward,
    Up,
    Click(usize),
    Right(usize),
    Install(usize),
    MenuClose,
    Search(String),
    SearchClear,
    SortNow(Column),
    Concealed,
    RenameStart,
    RenameInput(String),
    RenameDone,
    NewFolderStart,
    NewFolderInput(String),
    NewFolderDone,
    GatePassword(String),
    GateConfirm,
    GateCancel,
    Open(usize),
    TrashClear,
    Delete(usize),
    GFilter(usize),
    ThumbsLoaded(Vec<(PathBuf, u32, u32, Vec<u8>)>),
    Duplicate(usize),
    CopyPath(usize),
    Share(usize),
    /// R68: drag grammar — enter/leave target, release, spring pulse.
    DragOver(Option<DragTarget>),
    DragDrop,
    DragPulse,
    /// R90: open/close the info sheet (overlay card).
    InfoOpen(usize),
    InfoClose,
    /// R98: the peek — the space bar opens/closes the preview.
    ViewToggle,
    /// R88 multi-selection: modifier state, Ctrl+A, Del.
    Modifiers(iced::keyboard::Modifiers),
    SelectAll,
    DeleteSelection,
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        let _ = std::fs::create_dir_all(programs());
        let path = home();
        let column = Column::Name;
        let entries = read(&path, false, column, false);
        let free = free_text(&path);
        let app = Self {
            frame: mkw::Frame::new(APP_ID, &[]),
            history: vec![path.clone()],
            history_pos: 0,
            path,
            entries,
            selection: None,
            marked: std::collections::BTreeSet::new(),
            mods: iced::keyboard::Modifiers::default(),
            last_click: None,
            drag: None,
            column,
            descending: false,
            search: String::new(),
            hidden: false,
            rename: None,
            extension_confirm: false,
            clear_confirm: false,
            new_folder: None,
            hardline_gate: None,
            hardline_unlocked: false,
            menu: None,
            free,
            places: drives(),
            thumbs: std::collections::HashMap::new(),
            // The programs folder (R48) exists from the first start on.
            gfilter: 0,
            info: None,
            peek: None,
            iphones: iphone::devices(),
            device_note: None,
        };
        (app, Task::none())
    }

    fn new_load(&mut self) {
        self.entries = if gallery_from(&self.path) == Gallery::Wurzel {
            // The gallery root unites ~/Bilder and ~/Videos.
            let mut all = read(&images_wurzel(), self.hidden, self.column, self.descending);
            all.extend(read(&videos_wurzel(), self.hidden, self.column, self.descending));
            sort_all(&mut all, self.column, self.descending);
            all
        } else {
            read(&self.path, self.hidden, self.column, self.descending)
        };
        let n = self.entries.len();
        self.marked.retain(|&i| i < n);
        if self.selection.is_some_and(|i| i >= n) {
            self.selection = self.marked.iter().next().copied();
        }
    }

    /// R88: set the selection to exactly one entry (anchor = set).
    fn select(&mut self, i: Option<usize>) {
        self.selection = i;
        self.marked.clear();
        if let Some(i) = i {
            self.marked.insert(i);
        }
    }

    /// The rows after the search filter — the view works ONLY on this list.
    fn visible_entries(&self) -> Vec<usize> {
        let s = mac_parity::fold(&self.search);
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| mac_parity::matches(&e.name, &s))
            .map(|(i, _)| i)
            .collect()
    }

    /// Generate missing gallery thumbs in the background (max 150 per load).
    fn thumbs_reload(&self) -> Task<Msg> {
        let gallery = gallery_from(&self.path) != Gallery::None;
        let missing: Vec<PathBuf> = self
            .entries
            .iter()
            .filter(|e| {
                !e.folder
                    && !self.thumbs.contains_key(&e.path)
                    && ((gallery && e.art() == "Bild") || is_appimage(&e.path))
            })
            .map(|e| e.path.clone())
            .take(150)
            .collect();
        if missing.is_empty() {
            return Task::none();
        }
        Task::perform(async move { thumbs_generate(missing) }, Msg::ThumbsLoaded)
    }

    fn go_to(&mut self, target: PathBuf) {
        if !target.is_dir() {
            return;
        }
        // R185: the Hardline folder opens only after the password —
        // and the guard sits HERE, so double-click from Zuhause (home), the
        // path bar and the sidebar all pass the same door.
        if target.starts_with(hardline_dir()) && !self.hardline_unlocked {
            self.hardline_gate = Some(String::new());
            return;
        }
        // A folder change closes the peek — it belongs to the old view.
        self.peek = None;
        // History like a browser: everything behind the position expires.
        self.history.truncate(self.history_pos + 1);
        self.history.push(target.clone());
        self.history_pos = self.history.len() - 1;
        self.path = target;
        self.select(None);
        self.search.clear();
        self.menu = None;
        self.rename = None;
        self.new_folder = None;
        self.free = free_text(&self.path);
        self.clear_confirm = false;
        self.new_load();
    }

    fn open(&mut self, i: usize) {
        let Some(e) = self.entries.get(i).cloned() else { return };
        if e.folder {
            self.go_to(e.path);
        } else if is_appimage(&e.path) {
            // Start programs like the reference: a double click suffices (R48).
            appimage_start(&e.path);
        } else {
            // R82c: Matrix extensions open their app DIRECTLY — generic xdg
            // detection classifies .md via file(1) as text/plain and landed
            // in a foreign editor. Own culture, deterministic, dev-slot-true
            // (PATH); everything else keeps going through xdg-open.
            let extension = e
                .path
                .extension()
                .map(|x| x.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            let own = match extension.as_str() {
                "md" | "markdown" => Some("matrix-markdown"),
                _ => None,
            };
            let mut kdo = match own {
                Some(app) => {
                    let mut k = std::process::Command::new("sh");
                    k.arg("-c").arg(format!(
                        "PATH=$HOME/.local/bin:$PATH exec {app} \"$1\"",
                    ));
                    k.arg(app).arg(&e.path);
                    k
                }
                None => {
                    // Round 178: the kit's opener (gio first — xdg-open is missing on the Duo).
                    mk::open::open(&e.path.to_string_lossy());
                    return;
                }
            };
            // Reap the child in a thread (zombie lesson 2026-07-07).
            if let Ok(mut child) = kdo.spawn() {
                std::thread::spawn(move || {
                    let _ = child.wait();
                });
            }
        }
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Frame(m) => {
                // R68: the window-wide mouse of the frame drives the drag —
                // only beyond the 4 pt threshold does the press become a drag.
                if let mkw::FrameMsg::Mouse(pt) = &m {
                    if let Some(z) = &mut self.drag {
                        if !z.active {
                            let dx = pt.x - z.start.x;
                            let dy = pt.y - z.start.y;
                            if (dx * dx + dy * dy).sqrt() > mk::input::DRAG_THRESHOLD {
                                z.active = true;
                            }
                        }
                    }
                }
                return self.frame.update(m).map(Msg::Frame);
            }
            Msg::Tick => {
                self.frame.palette_changed();
                self.places = drives();
                // The slow belt behind the udev listener: it also catches
                // the moment the user taps "Trust" on the device.
                self.iphones = iphone::devices();
            }
            Msg::Devices => {
                self.iphones = iphone::devices();
                if self.iphones.is_empty() {
                    self.device_note = None;
                }
            }
            Msg::IPhoneOpen(i) => {
                if let Some(dev) = self.iphones.get(i).cloned() {
                    match iphone::mount(&dev) {
                        Ok(mp) => {
                            self.device_note = None;
                            return self.update(Msg::Goto(mp));
                        }
                        Err(note) => self.device_note = Some(note),
                    }
                }
            }
            Msg::IPhoneEject => {
                if let Some(mp) = iphone::inside_device(&self.path) {
                    match iphone::eject(&mp) {
                        Ok(()) => {
                            self.iphones = iphone::devices();
                            return self.update(Msg::Goto(home()));
                        }
                        Err(note) => self.device_note = Some(note),
                    }
                }
            }
            Msg::Key(t) => {
                if self.frame.handle_key(t) {
                    return Task::none();
                }
                match t {
                    mkw::Key::Escape => {
                        self.info = None;
                        self.peek = None;
                        self.drag = None;
                        self.menu = None;
                        self.rename = None;
                        self.new_folder = None;
                        self.extension_confirm = false;
                        self.clear_confirm = false;
                    }
                    // ↑/↓ wander through the visible rows (round 15).
                    mkw::Key::Prev | mkw::Key::Next
                        if self.rename.is_none() && self.new_folder.is_none() =>
                    {
                        let view_slice = self.visible_entries();
                        if view_slice.is_empty() {
                            return Task::none();
                        }
                        let pos = self
                            .selection
                            .and_then(|a| view_slice.iter().position(|&i| i == a));
                        let new = match (t, pos) {
                            (mkw::Key::Next, Some(p)) if p + 1 < view_slice.len() => p + 1,
                            (mkw::Key::Next, None) => 0,
                            (mkw::Key::Prev, Some(p)) if p > 0 => p - 1,
                            (mkw::Key::Prev, None) => view_slice.len() - 1,
                            (_, Some(p)) => p,
                            _ => 0,
                        };
                        self.select(Some(view_slice[new]));
                        // An open peek pages along (Quick Look grammar:
                        // the arrows wander, the preview follows).
                        if self.peek.is_some() {
                            self.peek =
                                self.selection.and_then(|i| self.entries.get(i)).map(peek_load);
                        }
                    }
                    // Space: the peek — not while typing.
                    mkw::Key::Peek
                        if self.rename.is_none()
                            && self.new_folder.is_none()
                            && self.search.is_empty() =>
                    {
                        return self.update(Msg::ViewToggle);
                    }
                    // Enter renames — the reference quirk (only with a selection,
                    // and not while an edit is running).
                    mkw::Key::Activate
                        if self.rename.is_none()
                            && self.new_folder.is_none()
                            && self.selection.is_some() =>
                    {
                        return self.update(Msg::RenameStart);
                    }
                    mkw::Key::Refresh => {
                        self.new_load();
                        self.free = free_text(&self.path);
                    }
                    _ => {}
                }
            }
            Msg::InfoOpen(i) => {
                if let Some(e) = self.entries.get(i) {
                    self.info = Some(InfoSheet {
                        path: e.path.clone(),
                        name: e.name.clone(),
                        folder: e.folder,
                        art: e.art().to_string(),
                        size_text: if e.folder {
                            String::from("\u{2014}")
                        } else {
                            mk::format::bytes(e.size)
                        },
                        changed_text: date_text(e.changed),
                        ort: e.path.display().to_string(),
                        wherefroms: mac_parity::wherefroms(&e.path),
                    });
                }
                self.menu = None;
            }
            Msg::InfoClose => self.info = None,
            Msg::ViewToggle => {
                if self.peek.is_some() {
                    self.peek = None;
                } else {
                    self.peek =
                        self.selection.and_then(|i| self.entries.get(i)).map(peek_load);
                }
            }
            Msg::GatePassword(text) => self.hardline_gate = Some(text),
            Msg::GateCancel => self.hardline_gate = None,
            Msg::GateConfirm => {
                let typed = self.hardline_gate.clone().unwrap_or_default();
                if mk::permissions::password_verify(&typed) {
                    self.hardline_unlocked = true;
                    self.hardline_gate = None;
                    self.go_to(hardline_dir());
                    return self.thumbs_reload();
                }
                // Wrong password: the field empties, the door stays.
                self.hardline_gate = Some(String::new());
            }
            Msg::Goto(p) => {
                self.go_to(p);
                return self.thumbs_reload();
            }
            Msg::Back => {
                if self.history_pos > 0 {
                    self.history_pos -= 1;
                    self.path = self.history[self.history_pos].clone();
                    self.select(None);
                    self.new_load();
                    self.free = free_text(&self.path);
                }
            }
            Msg::Forward => {
                if self.history_pos + 1 < self.history.len() {
                    self.history_pos += 1;
                    self.path = self.history[self.history_pos].clone();
                    self.select(None);
                    self.new_load();
                    self.free = free_text(&self.path);
                }
            }
            Msg::Up => {
                if let Some(parent) = self.path.parent() {
                    self.go_to(parent.to_path_buf());
                    return self.thumbs_reload();
                }
            }
            Msg::Click(i) => {
                let now = Instant::now();
                let double = self
                    .last_click
                    .is_some_and(|(li, lz)| li == i && now.duration_since(lz).as_millis() < DOUBLE_CLICK_MS);
                self.last_click = Some((i, now));
                self.menu = None;
                if double {
                    self.drag = None;
                    self.open(i);
                } else {
                    // R88 Finder grammar: Ctrl toggles, Shift spans the range
                    // from the anchor, pressing on an existing set leaves it
                    // standing (group drag).
                    if self.mods.control() {
                        if self.marked.contains(&i) {
                            self.marked.remove(&i);
                            self.selection = self.marked.iter().next().copied();
                        } else {
                            self.marked.insert(i);
                            self.selection = Some(i);
                        }
                    } else if self.mods.shift() {
                        let view_slice = self.visible_entries();
                        let anchor = self.selection.and_then(|a| view_slice.iter().position(|&x| x == a));
                        let target = view_slice.iter().position(|&x| x == i);
                        if let (Some(a), Some(z)) = (anchor, target) {
                            for &x in &view_slice[a.min(z)..=a.max(z)] {
                                self.marked.insert(x);
                            }
                        } else {
                            self.select(Some(i));
                        }
                    } else if self.marked.contains(&i) && self.marked.len() > 1 {
                        self.selection = Some(i);
                    } else {
                        self.select(Some(i));
                    }
                    // R68: every press is a drag candidate — with a set the
                    // drag carries all chosen paths (R88).
                    if self.marked.contains(&i) {
                        if let Some(e) = self.entries.get(i) {
                            let further: Vec<PathBuf> = self
                                .marked
                                .iter()
                                .filter(|&&x| x != i)
                                .filter_map(|&x| self.entries.get(x).map(|e| e.path.clone()))
                                .collect();
                            self.drag = Some(Drag {
                                path: e.path.clone(),
                                name: e.name.clone(),
                                glyph: e.glyph(),
                                is_appimage: !e.folder && is_appimage(&e.path),
                                index: i,
                                further,
                                start: self.frame.ghost.mouse,
                                active: false,
                                target: None,
                                floats_since: None,
                            });
                        }
                    } else {
                        self.drag = None;
                    }
                }
            }
            Msg::Right(i) => {
                // A right click on the unselected selects it — on a set it
                // leaves it standing (the menu then hits all).
                if !self.marked.contains(&i) {
                    self.select(Some(i));
                } else {
                    self.selection = Some(i);
                }
                self.menu = Some(i);
            }
            Msg::MenuClose => self.menu = None,
            // R48: "In Programme installieren" (install into Programs) — the reference gesture as a click.
            Msg::Install(i) => {
                self.menu = None;
                if let Some(e) = self.entries.get(i).cloned() {
                    let target = programs().join(e.path.file_name().unwrap_or_default());
                    let _ = std::fs::create_dir_all(programs());
                    // rename first (same disk), else copy+delete.
                    if std::fs::rename(&e.path, &target).is_err() {
                        if mac_parity::copy_reflink(&e.path, &target).is_ok() {
                            let _ = std::fs::remove_file(&e.path);
                        }
                    }
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = std::fs::metadata(&target) {
                        let mut permissions = meta.permissions();
                        permissions.set_mode(permissions.mode() | 0o755);
                        let _ = std::fs::set_permissions(&target, permissions);
                    }
                    // Reference moment: jump there and show the result.
                    self.go_to(programs());
                }
            }
            Msg::DragOver(target) => {
                if let Some(z) = &mut self.drag {
                    if z.active {
                        // Programme (Programs) accepts only AppImages; nothing drags onto itself.
                        let target = match target {
                            Some(DragTarget::Programs) if !z.is_appimage => None,
                            Some(DragTarget::Folder(p)) if p == z.path => None,
                            other => other,
                        };
                        if target != z.target {
                            z.floats_since = target.is_some().then(Instant::now);
                            z.target = target;
                        }
                    }
                }
            }
            Msg::DragPulse => {
                // Spring-loading (0.5 s, reference empirics): the folder opens
                // under the hovering drag — the drag lives on.
                let spring = self.drag.as_ref().and_then(|z| {
                    if !z.active {
                        return None;
                    }
                    match (&z.target, z.floats_since) {
                        (Some(DragTarget::Folder(p)), Some(since))
                            if since.elapsed().as_millis() as u64
                                >= mk::input::SPRING_LOAD_MS =>
                        {
                            Some(p.clone())
                        }
                        _ => None,
                    }
                });
                if let Some(path) = spring {
                    if let Some(z) = &mut self.drag {
                        z.target = None;
                        z.floats_since = None;
                    }
                    self.go_to(path);
                    return self.thumbs_reload();
                }
            }
            Msg::DragDrop => {
                if let Some(z) = self.drag.take() {
                    if z.active {
                        match z.target {
                            Some(DragTarget::Programs) => {
                                // Installation by drag — the same mechanism
                                // like the context menu (R48).
                                let target = programs()
                                    .join(z.path.file_name().unwrap_or_default());
                                let _ = std::fs::create_dir_all(programs());
                                if std::fs::rename(&z.path, &target).is_err() {
                                    if mac_parity::copy_reflink(&z.path, &target).is_ok() {
                                        let _ = std::fs::remove_file(&z.path);
                                    }
                                }
                                use std::os::unix::fs::PermissionsExt;
                                if let Ok(meta) = std::fs::metadata(&target) {
                                    let mut permissions = meta.permissions();
                                    permissions.set_mode(permissions.mode() | 0o755);
                                    let _ = std::fs::set_permissions(&target, permissions);
                                }
                                self.go_to(programs());
                                return self.thumbs_reload();
                            }
                            Some(DragTarget::Folder(folder)) => {
                                // R88: the group drag takes the whole set along.
                                let mut all = vec![z.path.clone()];
                                all.extend(z.further.iter().cloned());
                                for path in all {
                                    let Some(name) = path.file_name() else { continue };
                                    let target = folder.join(name);
                                    // rename first (same disk), else
                                    // copy+delete (files only).
                                    if std::fs::rename(&path, &target).is_err()
                                        && !path.is_dir()
                                        && mac_parity::copy_reflink(&path, &target).is_ok()
                                    {
                                        let _ = std::fs::remove_file(&path);
                                    }
                                }
                                self.new_load();
                                self.select(None);
                            }
                            None => {}
                        }
                    } else if !self.mods.control()
                        && !self.mods.shift()
                        && self.marked.len() > 1
                    {
                        // Click without drag on the multi-selection: the Finder
                        // rule — it collapses onto the entry.
                        self.select(Some(z.index));
                    }
                }
            }
            Msg::Search(s) => {
                self.search = s;
                self.select(None);
            }
            Msg::SearchClear => self.search.clear(),
            Msg::SortNow(sp) => {
                if self.column == sp {
                    self.descending = !self.descending;
                } else {
                    self.column = sp;
                    self.descending = false;
                }
                self.new_load();
            }
            Msg::Concealed => {
                self.hidden = !self.hidden;
                self.new_load();
            }
            Msg::RenameStart => {
                if let Some(i) = self.selection {
                    if let Some(e) = self.entries.get(i) {
                        self.rename = Some((i, e.name.clone()));
                        self.menu = None;
                    }
                }
            }
            Msg::RenameInput(s) => {
                if let Some((_, text)) = &mut self.rename {
                    *text = s;
                }
                self.extension_confirm = false;
            }
            Msg::RenameDone => {
                // File manager reference extract (R33, FXEnableExtensionChangeWarning): whoever
                // changes the EXTENSION, it may not be meant — the first
                // Enter warns, only the second renames.
                if let Some((i, text)) = self.rename.clone() {
                    let new = text.trim().to_string();
                    let extension = |n: &str| n.rsplit_once('.').map(|(_, e)| e.to_lowercase());
                    let old_extension = self.entries.get(i).and_then(|e| extension(&e.name));
                    if !self.extension_confirm
                        && self.entries.get(i).is_some_and(|e| e.name != new)
                        && old_extension != extension(&new)
                    {
                        self.extension_confirm = true;
                        return Task::none();
                    }
                }
                self.extension_confirm = false;
                if let Some((i, new)) = self.rename.take() {
                    let new = new.trim();
                    if let Some(e) = self.entries.get(i) {
                        if !new.is_empty() && new != e.name && !new.contains('/') {
                            let target = self.path.join(new);
                            if !target.exists() {
                                let _ = std::fs::rename(&e.path, &target);
                            }
                        }
                    }
                    self.new_load();
                }
            }
            Msg::NewFolderStart => {
                self.new_folder = Some(String::from("Neuer Ordner"));
            }
            Msg::NewFolderInput(s) => self.new_folder = Some(s),
            Msg::NewFolderDone => {
                if let Some(name) = self.new_folder.take() {
                    let name = name.trim();
                    if !name.is_empty() && !name.contains('/') {
                        let _ = std::fs::create_dir(self.path.join(name));
                        self.new_load();
                    }
                }
            }
            Msg::Open(i) => {
                self.menu = None;
                self.open(i);
                return self.thumbs_reload();
            }
            Msg::TrashClear => {
                // WarnOnEmptyTrash (reference default ON): warn first, then
                // empty — and the trash sound (10) seals it.
                if !self.clear_confirm {
                    self.clear_confirm = true;
                } else {
                    self.clear_confirm = false;
                    let _ = mk::command::silent("gio", &["trash", "--empty"]);
                    std::thread::spawn(|| {
                        mk::feedback::now("papierkorb", "10-papierkorb.wav");
                    });
                    self.new_load();
                }
            }
            Msg::Modifiers(m) => self.mods = m,
            Msg::SelectAll => {
                if self.rename.is_none() && self.new_folder.is_none() {
                    let view_slice = self.visible_entries();
                    self.marked = view_slice.iter().copied().collect();
                    if self.selection.is_none() {
                        self.selection = view_slice.first().copied();
                    }
                }
            }
            Msg::DeleteSelection => {
                if self.rename.is_none()
                    && self.new_folder.is_none()
                    && !self.marked.is_empty()
                {
                    for i in self.marked.clone() {
                        if let Some(e) = self.entries.get(i) {
                            let _ = mk::command::silent("gio", &["trash", &e.path.to_string_lossy()]);
                        }
                    }
                    std::thread::spawn(|| {
                        mk::feedback::now("papierkorb", "15-wurf.wav");
                    });
                    self.menu = None;
                    self.select(None);
                    self.new_load();
                }
            }
            Msg::Delete(i) => {
                // Finder rule (R88): if the menu hits a multi-selection,
                // the whole set wanders into the trash.
                if self.marked.len() > 1 && self.marked.contains(&i) {
                    return self.update(Msg::DeleteSelection);
                }
                // Into the trash, never hard delete (gio speaks Trash).
                if let Some(e) = self.entries.get(i) {
                    let _ = mk::command::silent("gio", &["trash", &e.path.to_string_lossy()]);
                    // The reference plop (R37) — if the file is missing (old
                    // image), it simply stays silent.
                    std::thread::spawn(|| {
                        mk::feedback::now("papierkorb", "15-wurf.wav");
                    });
                }
                self.menu = None;
                self.select(None);
                self.new_load();
            }
            Msg::Duplicate(i) => {
                // R88: a multi-selection duplicates as a whole.
                let targets: Vec<usize> = if self.marked.len() > 1 && self.marked.contains(&i) {
                    self.marked.iter().copied().collect()
                } else {
                    vec![i]
                };
                for i in targets {
                    if let Some(e) = self.entries.get(i) {
                        let target = copy_path(&e.path);
                        let _ = mk::command::silent(
                            "cp",
                            &["-a", &e.path.to_string_lossy(), &target.to_string_lossy()],
                        );
                    }
                }
                self.menu = None;
                self.new_load();
            }
            Msg::GFilter(level) => self.gfilter = level.min(2),
            Msg::ThumbsLoaded(done) => {
                for (path, w, h, rgba) in done {
                    self.thumbs
                        .insert(path, iced::widget::image::Handle::from_rgba(w, h, rgba));
                }
                // Evaporation against RAM hunger: each tile weighs ~450 KB
                // (448x252 RGBA). While wandering through albums keep only
                // what the current folder shows — otherwise the map grows
                // without limit.
                if self.thumbs.len() > 240 {
                    let current: std::collections::HashSet<PathBuf> =
                        self.entries.iter().map(|e| e.path.clone()).collect();
                    self.thumbs.retain(|k, _| current.contains(k));
                }
            }
            // R171: sharing here means — Decimen shows the content as a
            // code on the screen. Nothing leaves the device over
            // a network; the other person's camera is the whole channel.
            Msg::Share(i) => {
                if let Some(e) = self.entries.get(i) {
                    if !e.folder {
                        let _ = std::process::Command::new("matrix-decimen")
                            .arg(e.path.as_os_str())
                            .spawn()
                            .map(|mut child| {
                                std::thread::spawn(move || {
                                    let _ = child.wait();
                                });
                            });
                    }
                }
                self.menu = None;
            }
            Msg::CopyPath(i) => {
                if let Some(e) = self.entries.get(i) {
                    let _ = std::process::Command::new("wl-copy")
                        .arg("--")
                        .arg(e.path.as_os_str())
                        .spawn()
                        .map(|mut child| {
                            std::thread::spawn(move || {
                                let _ = child.wait();
                            })
                        });
                }
                self.menu = None;
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Msg> {
        let mut all = vec![
            self.frame.subscription().map(Msg::Frame),
            mkw::tick("files", Duration::from_secs(4)).map(|_| Msg::Tick),
            // R149/R135: the bridge listens instead of polling — udev
            // speaks the moment an Apple device joins or leaves the bus.
            mkw::events::watch(
                "iphone",
                "udevadm",
                &["monitor", "--udev", "--property", "--subsystem-match=usb"],
                iphone::is_ios_event,
            )
            .map(|_| Msg::Devices),
            mkw::key_subscription(Msg::Key),
            // R88 multi-selection: modifier state + Ctrl+A + Del.
            iced::event::listen_with(|event, _, _| match event {
                iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(m)) => {
                    Some(Msg::Modifiers(m))
                }
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key, modifiers, ..
                }) => match key {
                    iced::keyboard::Key::Character(ref c)
                        if c.as_str() == "a" && modifiers.control() =>
                    {
                        Some(Msg::SelectAll)
                    }
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete) => {
                        Some(Msg::DeleteSelection)
                    }
                    _ => None,
                },
                _ => None,
            }),
        ];
        // R68: only during a drag — release ear and spring pulse.
        if self.drag.is_some() {
            all.push(iced::event::listen_with(|event, _, _| {
                matches!(
                    event,
                    iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
                        iced::mouse::Button::Left
                    )) | iced::Event::Touch(iced::touch::Event::FingerLifted { .. })
                )
                .then_some(Msg::DragDrop)
            }));
        }
        if self.drag.as_ref().is_some_and(|z| z.active) {
            all.push(
                mkw::tick("files-drag", Duration::from_millis(100)).map(|_| Msg::DragPulse),
            );
        }
        Subscription::batch(all)
    }

    // -------------------------------------------------------- Sidebar

    fn sidebar_entry<'a>(
        &self,
        glyph: char,
        title: &'a str,
        target: PathBuf,
    ) -> Element<'a, Msg> {
        // The SAME family as the Hilfe sidebar (user, 2026-07-14): the
        // kit block sidebar_eintrag — identical height, palette pill
        // (primary_container) for the active one, same hover, same magnifier.
        // R68: sidebar places are drag targets; Programme takes AppImages.
        let drag_target = if target == programs() {
            DragTarget::Programs
        } else {
            DragTarget::Folder(target.clone())
        };
        let drag_hier = self.drag.as_ref().is_some_and(|z| {
            z.active && z.target.as_ref() == Some(&drag_target)
        });
        mouse_area(mkw::sidebar_entry(
            mkw::SidebarItem { glyph: glyph, title, count: None },
            self.path == target || drag_hier,
            false,
            Msg::Goto(target),
            self.frame.palette,
        ))
        .on_enter(Msg::DragOver(Some(drag_target)))
        .on_exit(Msg::DragOver(None))
        .into()
    }

    fn sidebar(&self) -> Element<'_, Msg> {
        let p = self.frame.palette;
        let home = home();
        let mut column = column![
            mkw::txt("FAVORITEN", mk::typo::LABEL, p.on_surface_variant),
        ]
        .spacing(2);
        for (title, rel, glyph) in FAVORITES {
            // Leinwand rename (2026-07-19): legacy still has the folder under
            // its earlier name — point there until migrated.
            let mut target = home.join(rel);
            if *rel == "Leinwand" && !target.exists() {
                let old = home.join("Schreibtisch");
                if old.exists() {
                    target = old;
                }
            }
            column = column.push(self.sidebar_entry(*glyph, title, target));
        }
        // Gallery instead of Bilder+Videos (R40) — active in the WHOLE gallery area.
        column = column.push(mkw::sidebar_entry(
            mkw::SidebarItem { glyph: mkw::symbol::IMAGE, title: "Galerie", count: None },
            gallery_from(&self.path) != Gallery::None,
            false,
            Msg::Goto(images_wurzel()),
            self.frame.palette,
        ));
        // Dienste (services) — the folders that belong to a running service, not
        // to the user's own filing: Hardline (the net) and Claude (the
        // assistant). Their own section keeps Favoriten (favourites) personal.
        if hardline_visible() || claude_dir().is_dir() || cloud_dir().is_some() {
            column = column.push(Space::new().height(mk::spacing::M));
            column = column.push(mkw::txt("DIENSTE", mk::typo::LABEL, p.on_surface_variant));
        }
        // R185: Hardline — ONE folder for everything from the net,
        // behind the password gate (the entry itself opens the door).
        if hardline_visible() {
            column = column.push(mkw::sidebar_entry(
                mkw::SidebarItem { glyph: mkw::symbol::KEY, title: "Hardline", count: None },
                self.path.starts_with(hardline_dir()) || self.hardline_gate.is_some(),
                false,
                Msg::Goto(hardline_dir()),
                self.frame.palette,
            ));
        }
        // Claude — one folder for everything the assistant stores,
        // shown whenever it exists (no gate: these are the user's
        // own notes and settings, not net traffic like Hardline).
        if claude_dir().is_dir() {
            column = column.push(mkw::sidebar_entry(
                mkw::SidebarItem { glyph: mkw::symbol::SPARK, title: "Claude", count: None },
                self.path.starts_with(claude_dir()),
                false,
                Msg::Goto(claude_dir()),
                self.frame.palette,
            ));
        }
        // The sync service's folder, third of the Dienste (services).
        if let Some((dir, name)) = cloud_dir() {
            column = column.push(mkw::sidebar_entry(
                mkw::SidebarItem { glyph: mkw::symbol::CLOUD, title: name, count: None },
                self.path.starts_with(&dir),
                false,
                Msg::Goto(dir),
                self.frame.palette,
            ));
        }
        column = column.push(Space::new().height(mk::spacing::M));
        column = column.push(mkw::txt("ORTE", mk::typo::LABEL, p.on_surface_variant));
        column = column.push(self.sidebar_entry(mkw::symbol::HOME, "Zuhause", home.clone()));
        column = column.push(self.sidebar_entry(mkw::symbol::APPS, "Programme", programs()));
        column = column.push(self.sidebar_entry(mkw::symbol::STORAGE, "Matrix", PathBuf::from("/")));
        column = column.push(self.sidebar_entry(mkw::symbol::DELETE, "Papierkorb", trash()));
        for (name, path) in &self.places {
            column = column.push(self.sidebar_entry(mkw::symbol::USB, name.as_str(), path.clone()));
        }
        // R149: iOS devices are places too — one click mounts and opens.
        for (i, dev) in self.iphones.iter().enumerate() {
            let mp = dev.mount_point();
            column = column.push(mkw::sidebar_entry(
                mkw::SidebarItem {
                    glyph: mkw::symbol::SMARTPHONE,
                    title: dev.name.as_str(),
                    count: None,
                },
                self.path.starts_with(&mp),
                false,
                Msg::IPhoneOpen(i),
                self.frame.palette,
            ));
        }
        // The SAME sidebar ground as Matrix Hilfe (family block).
        let _ = p;
        mkw::sidebar_area(column.into(), mkw::ui::SIDEBAR_WIDTH, self.frame.palette)
    }

    // ------------------------------------------------------------- Content

    fn toolbar(&self) -> Element<'_, Msg> {
        let p = self.frame.palette;
        let title = self
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("Matrix"));
        let mut right: Vec<Element<'_, Msg>> = Vec::new();
        if self.path == trash() && !self.entries.is_empty() {
            right.push(mkw::magnifier(mkw::button(
                if self.clear_confirm { "Wirklich leeren?" } else { "Leeren …" },
                mkw::buttons::Style::Tinted,
                mkw::buttons::Role::Destructive,
                mkw::buttons::Size::Small,
                p,
                Some(Msg::TrashClear),
            )));
        }
        if iphone::inside_device(&self.path).is_some() {
            right.push(mkw::magnifier(mkw::button(
                "Auswerfen",
                mkw::buttons::Style::Tinted,
                mkw::buttons::Role::Normal,
                mkw::buttons::Size::Small,
                p,
                Some(Msg::IPhoneEject),
            )));
        }
        right.push(mkw::ui::tool_button(mkw::symbol::NEW_FOLDER, Some(Msg::NewFolderStart), p));
        right.push(mkw::ui::tool_button(mkw::symbol::VISIBILITY_OFF, Some(Msg::Concealed), p));
        // The toolbar family (mkw::ui) — Dateien was its firstborn.
        mkw::ui::toolbar(
            title,
            (self.history_pos > 0).then_some(Msg::Back),
            (self.history_pos + 1 < self.history.len()).then_some(Msg::Forward),
            vec![mkw::ui::tool_button(
                mkw::symbol::ARROW_UPWARD,
                self.path.parent().map(|_| Msg::Up),
                p,
            )],
            right,
            &self.search,
            Msg::Search,
            Msg::SearchClear,
            p,
        )
    }

    fn column_header(&self) -> Element<'_, Msg> {
        let p = self.frame.palette;
        let header = |title: &'static str, sp: Column, width: Option<f32>| -> Element<'_, Msg> {
            let active = self.column == sp;
            let mut line = row![mkw::txt(
                title,
                mk::typo::CAPTION,
                if active { p.on_surface } else { p.on_surface_variant },
            )]
            .spacing(2)
            .align_y(Alignment::Center);
            if active {
                line = line.push(mkw::symbol::<Msg>(
                    if self.descending { mkw::symbol::ARROW_DOWNWARD } else { mkw::symbol::ARROW_UPWARD },
                    11.0,
                    p.on_surface_variant,
                ));
            }
            let b = button(line)
                .padding(iced::Padding { left: 4.0, right: 4.0, top: 2.0, bottom: 2.0 })
                .style(move |_, status| mkw::panel::button_style(p, status, mk::radius::SMALL))
                .on_press(Msg::SortNow(sp));
            match width {
                Some(w) => container(b).width(Length::Fixed(w)).into(),
                None => container(b).width(Length::Fill).into(),
            }
        };
        container(
            row![
                Space::new().width(Length::Fixed(30.0)),
                header("Name", Column::Name, None),
                header("Änderungsdatum", Column::Date, Some(160.0)),
                header("Größe", Column::Size, Some(84.0)),
                header("Art", Column::Art, Some(120.0)),
            ]
            .align_y(Alignment::Center),
        )
        .padding(iced::Padding { bottom: 2.0, ..iced::Padding::ZERO })
        .into()
    }

    fn line(&self, i: usize, zebra: bool) -> Element<'_, Msg> {
        let p = self.frame.palette;
        let e = &self.entries[i];
        let selected = self.marked.contains(&i);
        let (front, beside) = if selected {
            (p.on_primary, p.on_primary)
        } else {
            (p.on_surface, p.on_surface_variant)
        };

        // Name — or the inline rename field (Enter confirms).
        let name: Element<'_, Msg> = match &self.rename {
            Some((ui, text)) if *ui == i => mkw::input_field(
                "Name",
                text,
                Msg::RenameInput,
                Some(Msg::RenameDone),
                false,
                p,
            ),
            _ => mkw::txt(e.display_name(), mk::typo::CAPTION, front).into(),
        };

        // AppImages show their embedded icon (once extracted).
        let image_icon: Element<'_, Msg> = match self.thumbs.get(&e.path) {
            Some(h) if is_appimage(&e.path) => iced::widget::image(h.clone())
                .width(Length::Fixed(20.0))
                .height(Length::Fixed(20.0))
                .into(),
            _ => mkw::symbol::<Msg>(e.glyph(), mk::icon_size::SMALL, if selected { front } else { p.primary }),
        };
        let content = row![
            container(image_icon).width(Length::Fixed(30.0)),
            container(name).width(Length::Fill),
            container(mkw::txt(date_text(e.changed), mk::typo::CAPTION, beside))
                .width(Length::Fixed(160.0)),
            container(
                container(mkw::txt(size_text(e), mk::typo::CAPTION, beside))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
                    .padding(iced::Padding { right: mk::spacing::M, ..iced::Padding::ZERO })
            )
            .width(Length::Fixed(84.0)),
            container(mkw::txt(e.art(), mk::typo::CAPTION, beside)).width(Length::Fixed(120.0)),
        ]
        .align_y(Alignment::Center);

        // R68: if a drag hovers over this folder, the ring lights up.
        let drag_hier = self.drag.as_ref().is_some_and(|z| {
            z.active && z.target == Some(DragTarget::Folder(e.path.clone()))
        });
        // Reference look: selection = accent pill full row; else zebra.
        let area = container(content)
            .width(Length::Fill)
            .padding(iced::Padding { left: 6.0, right: 6.0, top: 4.0, bottom: 4.0 })
            .style(move |_| container::Style {
                background: if selected {
                    Some(color(p.primary).into())
                } else if drag_hier {
                    Some(color(p.primary.over(p.surface_container_high, 0.18)).into())
                } else if zebra {
                    Some(color(p.on_surface.over(p.surface_container_high, 0.04)).into())
                } else {
                    None
                },
                border: iced::Border {
                    radius: mk::radius::SMALL.into(),
                    width: if drag_hier { 2.0 } else { 0.0 },
                    color: color(p.primary),
                },
                ..Default::default()
            });

        let drag_target = e
            .folder
            .then(|| DragTarget::Folder(e.path.clone()));
        // R168: right click AND long press open the row menu — the
        // press_area lifts the row while the touch charges.
        mkw::press_area(
            mouse_area(area)
                .on_press(Msg::Click(i))
                .on_enter(Msg::DragOver(drag_target))
                .on_exit(Msg::DragOver(None)),
            move |_| Msg::Right(i),
        )
    }

    fn path_bar(&self) -> Element<'_, Msg> {
        let p = self.frame.palette;
        let mut line = row![].spacing(2).align_y(Alignment::Center);
        let mut so_far = PathBuf::from("/");
        let mut segmente: Vec<(String, PathBuf)> = vec![(String::from("Matrix"), so_far.clone())];
        for part in self.path.components() {
            if let std::path::Component::Normal(n) = part {
                so_far = so_far.join(n);
                segmente.push((n.to_string_lossy().to_string(), so_far.clone()));
            }
        }
        let n = segmente.len();
        for (idx, (name, target)) in segmente.into_iter().enumerate() {
            if idx > 0 {
                line = line.push(mkw::txt("›", mk::typo::LABEL, p.outline));
            }
            let last = idx + 1 == n;
            line = line.push(
                button(mkw::txt(
                    name,
                    mk::typo::LABEL,
                    if last { p.on_surface } else { p.on_surface_variant },
                ))
                .padding(iced::Padding { left: 4.0, right: 4.0, top: 1.0, bottom: 1.0 })
                .style(move |_, status| mkw::panel::button_style(p, status, mk::radius::SMALL))
                .on_press(Msg::Goto(target)),
            );
        }
        // Reference detail: the selection hangs at the end of the path bar.
        if let Some(e) = self.selection.and_then(|i| self.entries.get(i)) {
            line = line.push(mkw::txt("›", mk::typo::LABEL, p.outline));
            line = line.push(mkw::txt(&e.name, mk::typo::LABEL, p.on_surface));
        }
        container(line).width(Length::Fill).into()
    }

    /// Context menu as a root overlay — the reference entries in the ONE
    /// menu language of the MenuFamily.
    fn context(&self, i: usize) -> Element<'_, Msg> {
        let p = self.frame.palette;
        let folder = self.entries.get(i).is_some_and(|e| e.folder);
        let installable = self
            .entries
            .get(i)
            .is_some_and(|e| !e.folder && is_appimage(&e.path) && !e.path.starts_with(programs()));
        let mut entries: Vec<mkw::ui::MenuItem<Msg>> = vec![
            mkw::ui::MenuItem::Item {
                glyph: Some(if folder { mkw::symbol::FOLDER } else { mkw::symbol::PLAY_ARROW }),
                title: String::from("Öffnen"),
                color: None,
                msg: Msg::Open(i),
            },
            mkw::ui::MenuItem::Separator,
            mkw::ui::MenuItem::Item {
                glyph: None,
                title: String::from("Umbenennen"),
                color: None,
                msg: Msg::RenameStart,
            },
            mkw::ui::MenuItem::Item {
                glyph: None,
                title: String::from("Duplizieren"),
                color: None,
                msg: Msg::Duplicate(i),
            },
            mkw::ui::MenuItem::Item {
                glyph: None,
                title: String::from("Pfad kopieren"),
                color: None,
                msg: Msg::CopyPath(i),
            },
            mkw::ui::MenuItem::Item {
                glyph: Some(mkw::symbol::SHARE),
                title: String::from("Teilen \u{2026}"),
                color: None,
                msg: Msg::Share(i),
            },
            mkw::ui::MenuItem::Separator,
            mkw::ui::MenuItem::Item {
                glyph: Some(mkw::symbol::HELP),
                title: String::from("Informationen"),
                color: None,
                msg: Msg::InfoOpen(i),
            },
            mkw::ui::MenuItem::Separator,
            mkw::ui::MenuItem::Item {
                glyph: Some(mkw::symbol::DELETE),
                title: String::from("In den Papierkorb legen"),
                color: Some(p.error),
                msg: Msg::Delete(i),
            },
        ];
        if installable {
            entries.insert(
                1,
                mkw::ui::MenuItem::Item {
                    glyph: Some(mkw::symbol::APPS),
                    title: String::from("In Programme installieren"),
                    color: None,
                    msg: Msg::Install(i),
                },
            );
        }
        mouse_area(
            container(mkw::ui::menu_family(None, entries, p))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Msg::MenuClose)
        .on_right_press(Msg::MenuClose)
        .into()
    }

    /// An album card (folder in the gallery): large folder, name.
    /// R86: also a drag target — media tiles can be dragged in
    /// (same grammar as sidebar places; the tiles have long been drag
    /// sources via Msg::Klick, only the target was missing).
    fn album_card(&self, i: usize) -> Element<'_, Msg> {
        let p = self.frame.palette;
        let e = &self.entries[i];
        let drag_target = DragTarget::Folder(e.path.clone());
        let drag_hier = self
            .drag
            .as_ref()
            .is_some_and(|z| z.active && z.target.as_ref() == Some(&drag_target));
        let selected = self.marked.contains(&i) || drag_hier;
        let content = column![
            mkw::symbol::<Msg>(mkw::symbol::FOLDER, ALBUM_SYMBOL, p.primary),
            mkw::txt(&e.name, mk::typo::CAPTION, p.on_surface),
        ]
        .spacing(mk::spacing::XXS)
        .align_x(Alignment::Center);
        let chain = mouse_area(
            container(content)
                .width(Length::Fixed(110.0))
                .padding(mk::spacing::S)
                .style(move |_| container::Style {
                    background: Some(color(if selected {
                        p.primary.over(p.surface_container_high, 0.14)
                    } else {
                        p.surface_container_high
                    }).into()),
                    border: iced::Border {
                        radius: mk::radius::NORMAL.into(),
                        width: if selected { 1.5 } else { 0.0 },
                        color: color(p.primary),
                    },
                    ..Default::default()
                }),
        )
        .on_press(Msg::Click(i))
        .on_enter(Msg::DragOver(Some(drag_target)))
        .on_exit(Msg::DragOver(None));
        mkw::press_area(chain, move |_| Msg::Right(i))
    }

    /// A media tile: image thumb (once loaded) or movie card.
    fn media_tile(&self, i: usize) -> Element<'_, Msg> {
        let p = self.frame.palette;
        let e = &self.entries[i];
        let selected = self.marked.contains(&i);
        // Image-tile family (mkw::ui): the same anatomy as the wallpaper
        // section of the settings — 112x63, baked corners.
        let tile = mkw::ui::image_tile(
            self.thumbs.get(&e.path).cloned(),
            if e.art() == "Film" { mkw::symbol::MOVIE } else { mkw::symbol::IMAGE },
            e.name.clone(),
            selected,
            None,
            p,
        );
        mkw::press_area(
            mouse_area(container(tile).padding(2)).on_press(Msg::Click(i)),
            move |_| Msg::Right(i),
        )
    }

    /// The gallery view (R40): albums on top, media grid below —
    /// no file list. Filter: Alle / Bilder / Videos.
    fn gallery_view(&self) -> Element<'_, Msg> {
        let p = self.frame.palette;
        let view_slice = self.visible_entries();

        // Head: filter segments + Neues Album (new album).
        let mut header = row![
            container(mkw::segments(
                &["Alle", "Bilder", "Videos"],
                self.gfilter,
                Msg::GFilter,
                p,
            ))
            .width(Length::Fixed(260.0)),
            Space::new().width(Length::Fill),
            mkw::button(
                "Neues Album",
                mkw::buttons::Style::Tinted,
                mkw::buttons::Role::Normal,
                mkw::buttons::Size::Small,
                p,
                Some(Msg::NewFolderStart),
            ),
        ]
        .spacing(mk::spacing::S)
        .align_y(Alignment::Center);
        let _ = &mut header;

        let mut column = column![header].spacing(mk::spacing::M);

        // R87: the grid breathes with the window — the column count
        // comes from the real width instead of a fixed five.
        let window_b = if self.frame.size.width > 0.0 {
            self.frame.size.width
        } else {
            920.0
        };
        let area = window_b - mkw::ui::SIDEBAR_WIDTH
            - 2.0 * mk::spacing::L
            - mk::spacing::S;

        // Create/rename album — inline inputs of the gallery.
        if let Some(name) = &self.new_folder {
            column = column.push(
                row![
                    mkw::txt("Neues Album:", mk::typo::CAPTION, p.on_surface_variant),
                    container(mkw::input_field(
                        "Albumname",
                        name,
                        Msg::NewFolderInput,
                        Some(Msg::NewFolderDone),
                        false,
                        p,
                    ))
                    .width(Length::Fixed(240.0)),
                ]
                .spacing(mk::spacing::S)
                .align_y(Alignment::Center),
            );
        }
        if let Some((i, text)) = &self.rename {
            if self.entries.get(*i).is_some() {
                column = column.push(
                    row![
                        mkw::txt("Umbenennen:", mk::typo::CAPTION, p.on_surface_variant),
                        container(mkw::input_field(
                            "Name",
                            text,
                            Msg::RenameInput,
                            Some(Msg::RenameDone),
                            false,
                            p,
                        ))
                        .width(Length::Fixed(240.0)),
                    ]
                    .spacing(mk::spacing::S)
                    .align_y(Alignment::Center),
                );
            }
        }

        // Albums (folders) as a card row.
        let albums: Vec<usize> = view_slice
            .iter()
            .copied()
            .filter(|&i| self.entries[i].folder)
            .collect();
        if !albums.is_empty() {
            column = column.push(mkw::txt("ALBEN", mk::typo::LABEL, p.on_surface_variant));
            column = column.push(mkw::ui::raster(
                albums.iter().map(|&i| self.album_card(i)).collect(),
                area,
                110.0 + mk::spacing::S,
            ));
        }

        // Media by filter.
        let media: Vec<usize> = view_slice
            .iter()
            .copied()
            .filter(|&i| {
                let e = &self.entries[i];
                if e.folder {
                    return false;
                }
                match self.gfilter {
                    1 => e.art() == "Bild",
                    2 => e.art() == "Film",
                    _ => e.art() == "Bild" || e.art() == "Film",
                }
            })
            .collect();
        if !media.is_empty() {
            column = column.push(mkw::txt(
                match self.gfilter {
                    1 => "BILDER",
                    2 => "VIDEOS",
                    _ => "MEDIEN",
                },
                mk::typo::LABEL,
                p.on_surface_variant,
            ));
            column = column.push(mkw::ui::raster(
                media.iter().map(|&i| self.media_tile(i)).collect(),
                area,
                mkw::ui::IMAGE_TILE_W + 4.0 + mk::spacing::S,
            ));
        } else if albums.is_empty() {
            column = column.push(
                container(mkw::txt(
                    "Noch keine Bilder oder Videos — lege welche in die Galerie.",
                    mk::typo::CAPTION,
                    p.on_surface_variant,
                ))
                .center_x(Length::Fill)
                .padding(mk::spacing::XL),
            );
        }

        column.into()
    }

    /// The info sheet (R90): details as a floating card above the
    /// view — until R89 it lived in the band, the band is history.
    fn info_card<'a>(&'a self, sheet: &'a InfoSheet) -> Element<'a, Msg> {
        let p = self.frame.palette;
        let image: Element<'_, Msg> = match self.thumbs.get(&sheet.path) {
            Some(h) => iced::widget::image(h.clone())
                .width(Length::Fixed(96.0))
                .height(Length::Fixed(96.0))
                .content_fit(iced::ContentFit::Contain)
                .into(),
            None => mkw::symbol::<Msg>(
                if sheet.folder { mkw::symbol::FOLDER } else { mkw::symbol::CONTENT_COPY },
                mk::icon_size::HERO,
                p.primary,
            ),
        };
        let line = |k: &'static str, w: &'a str| -> Element<'a, Msg> {
            mkw::row_value(k, None, w, p)
        };
        let core = column![
            row![
                mkw::txt("Informationen", mk::typo::HEADLINE, p.on_surface_variant),
                Space::new().width(Length::Fill),
                mkw::ui::tool_button(mkw::symbol::CLOSE, Some(Msg::InfoClose), p),
            ]
            .spacing(mk::spacing::S)
            .align_y(Alignment::Center),
            Space::new().height(mk::spacing::M),
            container(image).center_x(Length::Fill),
            Space::new().height(mk::spacing::S),
            container(mkw::txt(&sheet.name, mk::typo::SUBTITLE, p.on_surface)).center_x(Length::Fill),
            Space::new().height(mk::spacing::M),
            mkw::section(
                "",
                vec![
                    line("Art", &sheet.art),
                    line("Größe", &sheet.size_text),
                    line("Geändert", &sheet.changed_text),
                    line("Ort", &sheet.ort),
                ]
                .into_iter()
                .chain(sheet.wherefroms.as_deref().map(|w| line("Woher", w)))
                .collect::<Vec<_>>(),
                p,
            ),
        ]
        .padding(mk::spacing::L);
        container(core)
            .width(Length::Fixed(420.0))
            .style(move |_| container::Style {
                background: Some(color(p.surface_container_high).into()),
                border: iced::Border {
                    radius: mk::radius::NORMAL.into(),
                    width: 1.0,
                    color: color(p.outline.with_alpha(0.3)),
                },
                shadow: iced::Shadow {
                    color: iced::Color { a: 0.35, ..iced::Color::BLACK },
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 32.0,
                },
                ..Default::default()
            })
            .into()
    }

    /// The peek card (R98): generous, fleeting, chromeless — only
    /// content, name and facts. Space/Esc/veil close.
    fn peek_card<'a>(&'a self, b: &'a Peek) -> Element<'a, Msg> {
        let p = self.frame.palette;
        let content: Element<'_, Msg> = if let Some(h) = &b.image {
            iced::widget::image(h.clone())
                .width(Length::Fixed(640.0))
                .height(Length::Fixed(420.0))
                .content_fit(iced::ContentFit::Contain)
                .into()
        } else if let Some(t) = &b.text {
            container(
                self.frame.scroll_area(
                    mkw::txt(t, mk::typo::CAPTION, p.on_surface).into(),
                    Msg::Frame,
                ),
            )
            .width(Length::Fixed(640.0))
            .height(Length::Fixed(420.0))
            .padding(mk::spacing::M)
            .style(move |_| container::Style {
                background: Some(color(p.surface_container).into()),
                border: iced::Border { radius: mk::radius::NORMAL.into(), ..Default::default() },
                ..Default::default()
            })
            .into()
        } else {
            container(mkw::symbol::<Msg>(b.glyph, mk::icon_size::HERO, p.primary))
                .width(Length::Fixed(640.0))
                .height(Length::Fixed(280.0))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        };
        let core = column![
            content,
            Space::new().height(mk::spacing::S),
            container(mkw::txt(&b.name, mk::typo::SUBTITLE, p.on_surface)).center_x(Length::Fill),
            container(mkw::txt(&b.subtitle, mk::typo::CAPTION, p.on_surface_variant))
                .center_x(Length::Fill),
        ]
        .padding(mk::spacing::L);
        container(core)
            .style(move |_| container::Style {
                background: Some(color(p.surface_container_high).into()),
                border: iced::Border {
                    radius: mk::radius::NORMAL.into(),
                    width: 1.0,
                    color: color(p.outline.with_alpha(0.3)),
                },
                shadow: iced::Shadow {
                    color: iced::Color { a: 0.35, ..iced::Color::BLACK },
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 32.0,
                },
                ..Default::default()
            })
            .into()
    }

    fn view(&self) -> Element<'_, Msg> {
        let p = self.frame.palette;

        // Rows (filtered), zebra over the VISIBLE position.
        let view_slice = self.visible_entries();
        let mut list = column![].spacing(1);
        // "Neuer Ordner" (new folder) edit as the first row.
        if let Some(name) = &self.new_folder {
            list = list.push(
                row![
                    container(mkw::symbol::<Msg>(mkw::symbol::FOLDER, mk::icon_size::SMALL, p.primary))
                        .width(Length::Fixed(30.0)),
                    mkw::input_field(
                        "Neuer Ordner",
                        name,
                        Msg::NewFolderInput,
                        Some(Msg::NewFolderDone),
                        false,
                        p,
                    ),
                ]
                .align_y(Alignment::Center),
            );
        }
        for (pos, i) in view_slice.iter().enumerate() {
            list = list.push(self.line(*i, pos % 2 == 1));
        }
        if view_slice.is_empty() && self.new_folder.is_none() {
            list = list.push(
                container(mkw::txt(
                    if self.search.is_empty() { "Dieser Ordner ist leer" } else { "Keine Treffer" },
                    mk::typo::CAPTION,
                    p.on_surface_variant,
                ))
                .center_x(Length::Fill)
                .padding(mk::spacing::XL),
            );
        }

        // Gallery area (R40): photo app instead of file list.
        // R149: a device that refused to open says so where the files
        // would be — the sidebar keeps the device, the body the reason.
        let content = if let Some(pw) = &self.hardline_gate {
            // The gate IS the page while it is open — no peeking past.
            let card = column![
                mkw::txt("Hardline ist geschützt", mk::typo::SUBTITLE, p.on_surface),
                mkw::txt(
                    "In diesem Ordner liegt alles, was dieses Gerät im Hardline-Netz besitzt — deine Seite, das Regal, die Schlüssel. Dein Passwort öffnet ihn.",
                    mk::typo::BODY,
                    p.on_surface_variant,
                ),
                Space::new().height(mk::spacing::S),
                row![
                    container(mkw::input_field(
                        "Passwort",
                        pw,
                        Msg::GatePassword,
                        Some(Msg::GateConfirm),
                        true,
                        p,
                    ))
                    .width(Length::Fixed(280.0)),
                    Space::new().width(mk::spacing::S),
                    mkw::text_button(mkw::txt("Öffnen", mk::typo::BODY, p.primary), Msg::GateConfirm),
                    Space::new().width(mk::spacing::S),
                    mkw::text_button(mkw::txt("Abbrechen", mk::typo::BODY, p.on_surface_variant), Msg::GateCancel),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(mk::spacing::XS);
            column![
                self.toolbar(),
                container(mkw::tile_chrome(card.into(), p))
                    .center_x(Length::Fill)
                    .padding(mk::spacing::XL),
                self.path_bar(),
            ]
        } else if let Some(note) = &self.device_note {
            column![
                self.toolbar(),
                mkw::empty_state(mkw::symbol::SMARTPHONE, "iPhone nicht bereit", note, p),
                self.path_bar(),
            ]
        } else if gallery_from(&self.path) != Gallery::None {
            column![
                self.toolbar(),
                Space::new().height(mk::spacing::S),
                self.frame.scroll_area(self.gallery_view(), Msg::Frame),
                Space::new().height(mk::spacing::XS),
                self.path_bar(),
            ]
        } else {
            column![
                self.toolbar(),
                Space::new().height(mk::spacing::S),
                self.column_header(),
                self.frame.scroll_area(list.into(), Msg::Frame),
                Space::new().height(mk::spacing::XS),
                self.path_bar(),
            ]
        };

        // The SidebarFamily anatomy as in Matrix Hilfe: dark bar bare
        // at the edge, on the right the SAME bright detail card (family
        // block detail_card) — the footer lives in the card.
        let detail = column![
            content,
            mkw::footer(
                if self.clear_confirm {
                    String::from("Papierkorb unwiderruflich leeren? Noch ein Klick bestätigt — Esc bricht ab")
                } else if self.extension_confirm {
                    String::from("Dateiendung geändert — Enter bestätigt, Esc bricht ab")
                } else if self.marked.len() > 1 {
                    // R88: the footer counts the selection — like the reference.
                    format!(
                        "{} von {} ausgewählt · {}",
                        self.marked.len(),
                        self.visible_entries().len(),
                        if self.free.is_empty() { String::from("—") } else { self.free.clone() }
                    )
                } else {
                    format!(
                        "{} Objekte · {}",
                        self.visible_entries().len(),
                        if self.free.is_empty() { String::from("—") } else { self.free.clone() }
                    )
                },
                p,
            ),
        ]
        .spacing(mk::spacing::S);

        let card = row![
            self.sidebar(),
            mkw::ui::detail_card(detail.into(), p),
        ]
        .height(Length::Fill);

        // R90: the info sheet floats as a card above the view — the
        // veil behind it closes on click (like the context menu).
        let card: Element<'_, Msg> = if let Some(sheet) = &self.info {
            iced::widget::stack![
                card,
                mouse_area(
                    container(Space::new().width(Length::Fill).height(Length::Fill)).style(
                        |_| container::Style {
                            background: Some(iced::Color { a: 0.35, ..iced::Color::BLACK }.into()),
                            ..Default::default()
                        }
                    )
                )
                .on_press(Msg::InfoClose),
                container(self.info_card(sheet))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
            ]
            .into()
        } else {
            card.into()
        };

        // R98: the peek — the same veil grammar as the info sheet,
        // but more fleeting (Space/Esc/click close).
        let card: Element<'_, Msg> = if let Some(b) = &self.peek {
            iced::widget::stack![
                card,
                mouse_area(
                    container(Space::new().width(Length::Fill).height(Length::Fill)).style(
                        |_| container::Style {
                            background: Some(iced::Color { a: 0.35, ..iced::Color::BLACK }.into()),
                            ..Default::default()
                        }
                    )
                )
                .on_press(Msg::ViewToggle),
                container(self.peek_card(b))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
            ]
            .into()
        } else {
            card
        };

        // R68: the drag ghost floats at ~0.7 opacity under the pointer.
        let card: Element<'_, Msg> = if let Some(z) = self.drag.as_ref().filter(|z| z.active) {
            let mouse = self.frame.ghost.mouse;
            let ghost = container(
                container(
                    row![
                        mkw::symbol::<Msg>(z.glyph, mk::icon_size::SMALL, p.primary),
                        // R88: the group drag shows the object count instead of the name.
                        if z.further.is_empty() {
                            mkw::txt(&z.name, mk::typo::CAPTION, p.on_surface)
                        } else {
                            mkw::txt(
                                format!("{} Objekte", z.further.len() + 1),
                                mk::typo::CAPTION,
                                p.on_surface,
                            )
                        },
                    ]
                    .spacing(mk::spacing::XS)
                    .align_y(Alignment::Center),
                )
                .padding(iced::Padding {
                    top: 4.0,
                    right: mk::spacing::S,
                    bottom: 4.0,
                    left: mk::spacing::S,
                })
                .style(move |_| container::Style {
                    background: Some(
                        color(p.surface_container_high.with_alpha(mk::input::DRAG_GHOST_ALPHA))
                            .into(),
                    ),
                    border: iced::Border {
                        radius: mk::radius::SMALL.into(),
                        width: 1.0,
                        color: color(p.outline.with_alpha(0.25)),
                    },
                    ..Default::default()
                }),
            )
            .padding(iced::Padding {
                left: (mouse.x - 10.0).max(0.0),
                top: (mouse.y - 52.0).max(0.0),
                ..iced::Padding::ZERO
            });
            iced::widget::stack![card, ghost].into()
        } else {
            card.into()
        };
        let root = self.menu.map(|i| self.context(i));
        self.frame.shell("Matrix Dateien", card, root, Msg::Frame)
    }
}

// ------------------------------------------------------------------ Tests

#[cfg(test)]
mod tests {
    use super::*;

    fn probe(name: &str, folder: bool, size: u64) -> Entry {
        Entry {
            name: name.into(),
            path: PathBuf::from("/tmp").join(name),
            folder,
            size,
            changed: None,
        }
    }

    #[test]
    fn sizes_read_like_reference() {
        assert_eq!(size_text(&probe("a", false, 610_700_000)), "610.7 MB");
        assert_eq!(size_text(&probe("a", false, 26_000)), "26 KB");
        assert_eq!(size_text(&probe("a", false, 512)), "512 Byte");
        assert_eq!(size_text(&probe("a", true, 4096)), "—");
    }

    #[test]
    fn art_detects_kinds() {
        assert_eq!(probe("bild.PNG", false, 1).art(), "Bild");
        assert_eq!(probe("main.rs", false, 1).art(), "Rust-Quelltext");
        assert_eq!(probe("ordner", true, 0).art(), "Ordner");
        assert_eq!(probe("irgendwas.xyz", false, 1).art(), "Dokument");
    }

    #[test]
    fn sorting_and_direction() {
        let mut v = vec![probe("b", false, 2), probe("A", false, 3), probe("c", true, 1)];
        sort_all(&mut v, Column::Name, false);
        assert_eq!(v[0].name, "A");
        sort_all(&mut v, Column::Size, true);
        assert_eq!(v[0].size, 3);
    }

    #[test]
    fn copy_path_gives_way_out() {
        let p = copy_path(Path::new("/nirgends/bericht.txt"));
        assert_eq!(p, PathBuf::from("/nirgends/bericht Kopie.txt"));
    }

    #[test]
    fn hidden_am_item() {
        assert!(probe(".geheim", false, 1).hidden());
        assert!(!probe("offen", false, 1).hidden());
    }
}
