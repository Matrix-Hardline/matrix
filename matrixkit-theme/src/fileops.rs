//! COPY AND MOVE WITH PROGRESS (R217 family, 02.11.2026 — out of Matrix
//! Dateien, the owner: «move the downloads to the USB stick»; files of
//! 300 MB and more). Runs OFF the drawing thread; reports bytes as it goes.
//!
//! * COPY: in chunks (4 MiB), every chunk reported, then fsync and a size
//!   check — a stick that lied is caught before anyone trusts the copy.
//! * MOVE: a rename when source and target share the filesystem (instant);
//!   else copy + fsync + verify, and only THEN the source goes. A failed
//!   copy never deletes anything; its half-written target is removed.
//! * A NAME CLASH never overwrites: «Datei.mp4» becomes «Datei (2).mp4».

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// The chunk a copy reads and writes at once (and reports after).
pub const CHUNK: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Copy,
    Move,
}

/// How far a transfer is.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Progress {
    /// Bytes done of all files.
    pub done: u64,
    /// Bytes in all (0 when only renames are needed).
    pub total: u64,
    /// The file being worked on (its name).
    pub file: String,
    /// Which top-level item (0-based) of how many.
    pub index: usize,
    pub count: usize,
}

/// What a finished transfer did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Summary {
    /// Top-level items that arrived (their new paths).
    pub arrived: Vec<PathBuf>,
    pub bytes: u64,
    /// Symlinks inside folders that could not be recreated (e.g. FAT).
    pub skipped: usize,
}

/// Split a file name into stem and extension: «Lied.mp3» → («Lied», «.mp3»);
/// a dot file («.bashrc») and a name without a dot have no extension;
/// «Archiv.tar.gz» keeps «.tar.gz» together.
pub fn split_name(name: &str) -> (&str, &str) {
    let lower = name.to_ascii_lowercase();
    for double in [".tar.gz", ".tar.xz", ".tar.zst", ".tar.bz2"] {
        if lower.ends_with(double) && name.len() > double.len() {
            let at = name.len() - double.len();
            return (&name[..at], &name[at..]);
        }
    }
    match name.rfind('.') {
        Some(0) | None => (name, ""),
        Some(at) => (&name[..at], &name[at..]),
    }
}

/// HOW A FILE SHOWS (round 30, Files polish): the name a person reads,
/// with what the list says beside it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    /// The readable title: no extension, no video id «[xxxxxxxxxxx]», no
    /// format tag «.f136», full-width ：｜ (yt-dlp's stand-ins) back to : |.
    pub title: String,
    /// The extension in capitals («MP4»), empty for none.
    pub ext: String,
    /// Still being written (a «.part» / «.crdownload» / «.ytdl» file).
    pub partial: bool,
}

/// The readable form of a file name (the real name stays in the path).
pub fn shown(name: &str) -> Shown {
    let mut rest = name;
    let mut partial = false;
    for tail in [".part", ".crdownload", ".ytdl", ".download"] {
        if rest.len() > tail.len() && rest.to_ascii_lowercase().ends_with(tail) {
            rest = &rest[..rest.len() - tail.len()];
            partial = true;
            break;
        }
    }
    let (mut stem, ext) = split_name(rest);
    // yt-dlp's per-format tag before the extension: «.f136».
    if let Some(at) = stem.rfind(".f") {
        if stem.len() > at + 2 && stem[at + 2..].chars().all(|c| c.is_ascii_digit()) {
            stem = &stem[..at];
        }
    }
    // A trailing video id: « [6i01tOMgBDU]» (11 of A–Z a–z 0–9 - _).
    let mut title = stem.trim_end().to_string();
    if title.ends_with(']') {
        if let Some(open) = title.rfind('[') {
            let id = &title[open + 1..title.len() - 1];
            if id.len() == 11 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') && open > 0 {
                title.truncate(open);
                title = title.trim_end().to_string();
            }
        }
    }
    let title = title.replace('：', ":").replace('｜', "|").replace('？', "?").replace('／', "/").replace('＂', "\"");
    let title = if title.trim().is_empty() { name.to_string() } else { title };
    Shown { title, ext: ext.trim_start_matches('.').to_uppercase(), partial }
}

/// A free name in `dir` for `name`: the name itself if free, else
/// «stem (2).ext», «stem (3).ext» … `exists` asks the filesystem (or a
/// test). A stem that already ends in « (n)» counts on from there.
pub fn free_name(dir: &Path, name: &str, exists: impl Fn(&Path) -> bool) -> PathBuf {
    let first = dir.join(name);
    if !exists(&first) {
        return first;
    }
    let (stem, ext) = split_name(name);
    let (base, start) = match stem.strip_suffix(')').and_then(|s| s.rsplit_once(" (")) {
        Some((b, n)) if n.parse::<u32>().is_ok() => (b, n.parse::<u32>().unwrap_or(1) + 1),
        _ => (stem, 2),
    };
    let mut n = start.max(2);
    loop {
        let candidate = dir.join(format!("{base} ({n}){ext}"));
        if !exists(&candidate) || n > 100_000 {
            return candidate;
        }
        n += 1;
    }
}

/// An io error in words a person understands (German UI).
pub fn say(e: &std::io::Error) -> String {
    match e.raw_os_error() {
        Some(libc::ENOSPC) => String::from("Kein Platz mehr auf dem Ziel"),
        Some(libc::EFBIG) => String::from("Datei zu gross für dieses Dateisystem (FAT32: höchstens 4 GB)"),
        Some(libc::EROFS) => String::from("Das Ziel ist schreibgeschützt"),
        Some(libc::EACCES) | Some(libc::EPERM) => String::from("Keine Berechtigung"),
        Some(libc::EINVAL) => String::from("Name auf dem Ziel nicht erlaubt (Zeichen wie : ? * \" < > |)"),
        Some(libc::ENAMETOOLONG) => String::from("Name zu lang für das Ziel"),
        Some(libc::EIO) => String::from("Lese- oder Schreibfehler (Stick abgezogen?)"),
        Some(libc::ENOENT) => String::from("Datei nicht mehr da"),
        _ => e.to_string(),
    }
}

/// Bytes under `path` (a file, or a folder summed up; links not followed).
pub fn size_of(path: &Path) -> u64 {
    let Ok(meta) = std::fs::symlink_metadata(path) else { return 0 };
    if meta.is_dir() {
        std::fs::read_dir(path).map(|d| d.flatten().map(|e| size_of(&e.path())).sum()).unwrap_or(0)
    } else if meta.is_file() {
        meta.len()
    } else {
        0
    }
}

/// Same filesystem? (Then a move is a rename.)
fn same_fs(a: &Path, b: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (std::fs::symlink_metadata(a), std::fs::metadata(b)) {
        (Ok(x), Ok(y)) => x.dev() == y.dev(),
        _ => false,
    }
}

fn cancelled() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Interrupted, "abgebrochen")
}

/// Copy one file in chunks, then fsync and verify the size. On any error
/// the half-written target is removed.
pub fn copy_file(src: &Path, dst: &Path, cancel: &AtomicBool, mut step: impl FnMut(u64)) -> std::io::Result<u64> {
    let result = (|| {
        let mut from = std::fs::File::open(src)?;
        let meta = from.metadata()?;
        let mut to = std::fs::OpenOptions::new().write(true).create_new(true).open(dst)?;
        let mut buf = vec![0u8; CHUNK];
        let mut written = 0u64;
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err(cancelled());
            }
            let n = from.read(&mut buf)?;
            if n == 0 {
                break;
            }
            to.write_all(&buf[..n])?;
            written += n as u64;
            step(n as u64);
        }
        to.flush()?;
        to.sync_all()?;
        let landed = to.metadata()?.len();
        if landed != meta.len() || written != meta.len() {
            return Err(std::io::Error::other(format!("Grösse stimmt nicht ({landed} statt {} Byte)", meta.len())));
        }
        // The date travels along (best effort: FAT keeps 2-s steps).
        if let Ok(t) = meta.modified() {
            let _ = to.set_modified(t);
        }
        Ok(written)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(dst);
    }
    result
}

/// Copy a folder tree into `dst` (which must not exist yet).
fn copy_tree(src: &Path, dst: &Path, cancel: &AtomicBool, skipped: &mut usize, step: &mut dyn FnMut(&str, u64)) -> std::io::Result<()> {
    std::fs::create_dir(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let (from, to) = (entry.path(), dst.join(entry.file_name()));
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_tree(&from, &to, cancel, skipped, step)?;
        } else if kind.is_symlink() {
            let made = std::fs::read_link(&from).and_then(|target| std::os::unix::fs::symlink(target, &to));
            if made.is_err() {
                *skipped += 1;
            }
        } else if kind.is_file() {
            let name = entry.file_name().to_string_lossy().into_owned();
            copy_file(&from, &to, cancel, |n| step(&name, n))?;
        }
    }
    Ok(())
}

/// Is `inner` the folder `outer` or inside it?
fn inside(inner: &Path, outer: &Path) -> bool {
    let (a, b) = (inner.canonicalize().unwrap_or_else(|_| inner.to_path_buf()), outer.canonicalize().unwrap_or_else(|_| outer.to_path_buf()));
    a.starts_with(&b)
}

/// THE TRANSFER: copy or move `sources` into the folder `dest`. `report`
/// hears every chunk; `cancel` stops between chunks (the file in progress
/// is removed, finished ones stay — a move's finished sources are gone).
/// Errors come in words (`say`).
pub fn transfer(sources: &[PathBuf], dest: &Path, mode: Mode, cancel: &AtomicBool, mut report: impl FnMut(&Progress)) -> Result<Summary, String> {
    if !dest.is_dir() {
        return Err(String::from("Ziel ist kein Ordner"));
    }
    // What has to travel byte by byte (renames cost nothing).
    let copies: Vec<bool> = sources.iter().map(|s| mode == Mode::Copy || !same_fs(s, dest)).collect();
    let total: u64 = sources.iter().zip(&copies).filter(|(_, c)| **c).map(|(s, _)| size_of(s)).sum();
    if total > 0 {
        if let Some((free, _)) = crate::volumes::space(dest) {
            if free < total {
                return Err(format!("Zu wenig Platz: {} nötig, {} frei", crate::format::bytes(total), crate::format::bytes(free)));
            }
        }
    }
    let mut progress = Progress { done: 0, total, file: String::new(), index: 0, count: sources.len() };
    let mut summary = Summary::default();
    for (i, src) in sources.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err(String::from("Abgebrochen"));
        }
        let Some(name) = src.file_name().map(|n| n.to_string_lossy().into_owned()) else { continue };
        let Ok(meta) = std::fs::symlink_metadata(src) else {
            return Err(format!("«{name}»: Datei nicht mehr da"));
        };
        // Moving into the folder it already lies in: nothing to do.
        if mode == Mode::Move && src.parent() == Some(dest) {
            summary.arrived.push(src.clone());
            continue;
        }
        if meta.is_dir() && inside(dest, src) {
            return Err(format!("«{name}» kann nicht in sich selbst"));
        }
        progress.index = i;
        progress.file = name.clone();
        report(&progress);
        let target = free_name(dest, &name, |p| p.symlink_metadata().is_ok());
        if !copies[i] {
            std::fs::rename(src, &target).map_err(|e| format!("«{name}»: {}", say(&e)))?;
            summary.arrived.push(target);
            continue;
        }
        let mut skipped = 0usize;
        let mut step = |file: &str, n: u64| {
            progress.done += n;
            if progress.file != file {
                progress.file = file.to_string();
            }
            report(&progress);
        };
        let done = if meta.is_dir() {
            let made = copy_tree(src, &target, cancel, &mut skipped, &mut step);
            if made.is_err() {
                let _ = std::fs::remove_dir_all(&target);
            }
            made
        } else if meta.is_file() {
            copy_file(src, &target, cancel, |n| step(&name, n)).map(|_| ())
        } else {
            Err(std::io::Error::other("keine gewöhnliche Datei"))
        };
        summary.skipped += skipped;
        summary.bytes = progress.done;
        if let Err(e) = done {
            if e.kind() == std::io::ErrorKind::Interrupted {
                return Err(String::from("Abgebrochen"));
            }
            return Err(format!("«{name}»: {}", say(&e)));
        }
        // The copy stands (synced, sizes checked): only now the source goes.
        if mode == Mode::Move {
            let gone = if meta.is_dir() { std::fs::remove_dir_all(src) } else { std::fs::remove_file(src) };
            if let Err(e) = gone {
                summary.arrived.push(target);
                return Err(format!("«{name}» kopiert, Original blieb: {}", say(&e)));
            }
        }
        summary.arrived.push(target);
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn shows_names_readably() {
        let a = shown("Ashnikko - Daisy (Official Music Video) [6i01tOMgBDU].mp4");
        assert_eq!((a.title.as_str(), a.ext.as_str(), a.partial), ("Ashnikko - Daisy (Official Music Video)", "MP4", false));
        let b = shown("Meister Eder und sein Pumuckl (1982) ｜ Ganzer Film [LGqgwWt4_uw].f136.mp4.part");
        assert_eq!((b.title.as_str(), b.ext.as_str(), b.partial), ("Meister Eder und sein Pumuckl (1982) | Ganzer Film", "MP4", true));
        let c = shown("Paramore： Hard Times [OFFICIAL VIDEO] [AEB6ibtdPZc].mp4");
        assert_eq!(c.title, "Paramore: Hard Times [OFFICIAL VIDEO]");
        // Brackets that are no id stay.
        assert_eq!(shown("Notizen [Entwurf].txt").title, "Notizen [Entwurf]");
        assert_eq!(shown("README").ext, "");
        assert_eq!(shown(".bashrc").title, ".bashrc");
    }

    #[test]
    fn splits_names() {
        assert_eq!(split_name("Lied.mp3"), ("Lied", ".mp3"));
        assert_eq!(split_name(".bashrc"), (".bashrc", ""));
        assert_eq!(split_name("README"), ("README", ""));
        assert_eq!(split_name("Archiv.tar.gz"), ("Archiv", ".tar.gz"));
        assert_eq!(split_name("a.b.c"), ("a.b", ".c"));
    }

    #[test]
    fn free_names_count_up() {
        let dir = Path::new("/stick");
        let taken: HashSet<PathBuf> = ["/stick/Datei.mp4", "/stick/Datei (2).mp4", "/stick/Ordner", "/stick/Foto (4).png"].iter().map(PathBuf::from).collect();
        let exists = |p: &Path| taken.contains(p);
        assert_eq!(free_name(dir, "Neu.mp4", exists), PathBuf::from("/stick/Neu.mp4"));
        assert_eq!(free_name(dir, "Datei.mp4", exists), PathBuf::from("/stick/Datei (3).mp4"));
        assert_eq!(free_name(dir, "Ordner", exists), PathBuf::from("/stick/Ordner (2)"));
        assert_eq!(free_name(dir, "Foto (4).png", exists), PathBuf::from("/stick/Foto (5).png"));
    }

    fn scratch(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("mk-fileops-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn copies_with_progress_and_clash() {
        let base = scratch("copy");
        let (a, b) = (base.join("a"), base.join("b"));
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        let data = vec![7u8; CHUNK + 1234];
        std::fs::write(a.join("gross.bin"), &data).unwrap();
        std::fs::write(b.join("gross.bin"), b"alt").unwrap();
        let mut seen = Vec::new();
        let s = transfer(&[a.join("gross.bin")], &b, Mode::Copy, &AtomicBool::new(false), |p| seen.push(p.done)).unwrap();
        assert_eq!(s.arrived, vec![b.join("gross (2).bin")]);
        assert_eq!(std::fs::read(b.join("gross (2).bin")).unwrap(), data);
        assert_eq!(std::fs::read(b.join("gross.bin")).unwrap(), b"alt", "never overwritten");
        assert!(a.join("gross.bin").exists(), "a copy keeps the source");
        assert_eq!(*seen.last().unwrap(), data.len() as u64);
        assert!(seen.len() >= 3, "reported per chunk: {seen:?}");
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn moves_folders_and_files() {
        let base = scratch("move");
        let (a, b) = (base.join("a"), base.join("b"));
        std::fs::create_dir_all(a.join("Ordner/innen")).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        std::fs::write(a.join("Ordner/innen/x.txt"), b"x").unwrap();
        std::fs::write(a.join("lied.mp3"), b"musik").unwrap();
        let s = transfer(&[a.join("Ordner"), a.join("lied.mp3")], &b, Mode::Move, &AtomicBool::new(false), |_| {}).unwrap();
        assert_eq!(s.arrived.len(), 2);
        assert!(!a.join("Ordner").exists() && !a.join("lied.mp3").exists());
        assert_eq!(std::fs::read(b.join("Ordner/innen/x.txt")).unwrap(), b"x");
        assert_eq!(std::fs::read(b.join("lied.mp3")).unwrap(), b"musik");
        // A folder into itself is refused, nothing moves.
        let err = transfer(&[b.join("Ordner")], &b.join("Ordner/innen"), Mode::Copy, &AtomicBool::new(false), |_| {}).unwrap_err();
        assert!(err.contains("in sich selbst"), "{err}");
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn a_cancelled_copy_leaves_no_half_file() {
        let base = scratch("cancel");
        std::fs::write(base.join("f.bin"), vec![1u8; 1000]).unwrap();
        std::fs::create_dir_all(base.join("z")).unwrap();
        let err = transfer(&[base.join("f.bin")], &base.join("z"), Mode::Move, &AtomicBool::new(true), |_| {}).unwrap_err();
        assert_eq!(err, "Abgebrochen");
        assert!(base.join("f.bin").exists(), "the source stays");
        assert!(!base.join("z/f.bin").exists());
        let _ = std::fs::remove_dir_all(base);
    }
}
