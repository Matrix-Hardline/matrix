//! THE COMPANION (06.11.2026, the owner: «Morpheus is the iTunes of
//! Matrix»): the paired duo running Matrix, reached over the USB cable only.
//!
//! * `hello` / `pair` — the duo's `matrix-kopplung` (172.16.42.1:7471, USB
//!   only): who is there, and «Trust this laptop?» on the duo's screen. A
//!   trusted laptop's key may log in as the duo's person — from 172.16.42.2
//!   (the cable) only.
//! * everything else — `matrix-begleiter` over ssh with that key: device
//!   info, folders, photo previews, files both ways, backup, bootloader.
//!
//! All calls block (they run on worker threads); errors come back as text.

use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

pub const DUO: &str = "172.16.42.1";
const PAIR_PORT: u16 = 7471;

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"))
}

/// The laptop's key for the duo (also the live system's one-time key).
pub fn key_path() -> PathBuf {
    home().join(".ssh/morpheus_ed25519")
}

/// The public key — made once if it does not exist yet.
pub fn public_key() -> Option<String> {
    let key = key_path();
    if !key.is_file() {
        let _ = std::fs::create_dir_all(key.parent()?);
        let _ = Command::new("ssh-keygen").args(["-q", "-t", "ed25519", "-N", "", "-C", "morpheus", "-f"]).arg(&key).stdin(Stdio::null()).status();
    }
    std::fs::read_to_string(key.with_extension("pub")).ok().map(|s| s.trim().to_string())
}

/// What the duo shows in «Trust this laptop?» and in its list of paired
/// laptops: the pretty host name if set, else the device model from the
/// firmware («Surface Laptop, 7th Edition»), else the host name.
pub fn laptop_name() -> String {
    let pretty = Command::new("hostnamectl").arg("--pretty").stdin(Stdio::null()).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let model = std::fs::read_to_string("/sys/class/dmi/id/product_name").ok().map(|s| s.trim().trim_start_matches("Microsoft ").to_string());
    let host = std::fs::read_to_string("/etc/hostname").or_else(|_| std::fs::read_to_string("/proc/sys/kernel/hostname")).ok().map(|s| s.trim().to_string());
    [pretty, model, host]
        .into_iter()
        .flatten()
        .find(|n| !n.is_empty() && n != "localhost" && !n.eq_ignore_ascii_case("To Be Filled By O.E.M."))
        .unwrap_or_else(|| String::from("Laptop"))
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Hello {
    pub model: String,
    pub serial: String,
    pub account: String,
    pub paired: bool,
}

fn s(v: &Value, k: &str) -> String {
    match v.get(k) {
        Some(Value::String(t)) => t.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

fn hello_of(v: &Value) -> Hello {
    Hello { model: s(v, "modell"), serial: s(v, "seriennummer"), account: s(v, "konto"), paired: v.get("gekoppelt").and_then(Value::as_bool).unwrap_or(false) }
}

/// One request to `matrix-kopplung`.
fn ask(req: Value, wait: Duration) -> Result<Value, String> {
    let addr: std::net::SocketAddr = format!("{DUO}:{PAIR_PORT}").parse().map_err(|_| String::from("Adresse"))?;
    let mut c = std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(800)).map_err(|e| e.to_string())?;
    c.set_read_timeout(Some(wait)).ok();
    c.write_all(format!("{req}\n").as_bytes()).map_err(|e| e.to_string())?;
    let mut line = String::new();
    BufReader::new(c).read_line(&mut line).map_err(|e| e.to_string())?;
    serde_json::from_str(&line).map_err(|e| e.to_string())
}

/// Is a duo with Matrix (and its pairing service) at the cable? Fast; None = no.
pub fn hello() -> Option<Hello> {
    let key = public_key().unwrap_or_default();
    ask(serde_json::json!({"tu": "hallo", "schluessel": key}), Duration::from_secs(2)).ok().filter(|v| v.get("ok").and_then(Value::as_bool) == Some(true)).map(|v| hello_of(&v))
}

/// «Trust this laptop?» on the duo — waits for the answer (up to ~2.5 min).
pub fn pair() -> Result<Hello, String> {
    let key = public_key().ok_or("Kein Schlüssel auf diesem Laptop")?;
    let v = ask(serde_json::json!({"tu": "koppeln", "laptop": laptop_name(), "schluessel": key}), Duration::from_secs(170))?;
    if v.get("ok").and_then(Value::as_bool) == Some(true) {
        Ok(hello_of(&v))
    } else {
        Err(s(&v, "grund"))
    }
}

/// `matrix-begleiter <args>` on the duo, as `account`, with the paired key.
fn begleiter(account: &str, args: &[&str]) -> Command {
    let mut c = Command::new("ssh");
    c.arg("-i").arg(key_path()).args([
        "-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=no", "-o", "UserKnownHostsFile=/dev/null", "-o", "LogLevel=ERROR", "-o", "ConnectTimeout=6",
        &format!("{account}@{DUO}"), "matrix-begleiter",
    ]);
    // The remote shell splits words: every argument in single quotes.
    for a in args {
        c.arg(format!("'{}'", a.replace('\'', "'\\''")));
    }
    c
}

fn json_of(account: &str, args: &[&str]) -> Result<Value, String> {
    let out = begleiter(account, args).stdin(Stdio::null()).output().map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_slice(&out.stdout).map_err(|_| {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        if err.is_empty() { String::from("Keine Antwort vom Duo") } else { err }
    })?;
    if v.get("ok").and_then(Value::as_bool) == Some(false) {
        return Err(s(&v, "grund"));
    }
    Ok(v)
}

/// The duo, as Morpheus' overview shows it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Info {
    pub model: String,
    pub serial: String,
    pub account: String,
    pub name: String,
    pub storage_gib: f32,
    pub free_gib: f32,
    pub battery: i32,
    pub charging: bool,
    pub system: String,
    pub fedora: String,
    pub kernel: String,
    pub up_h: f32,
}

pub fn info_of(v: &Value) -> Info {
    let f = |k: &str| v.get(k).and_then(Value::as_f64).unwrap_or(0.0) as f32;
    Info {
        model: s(v, "modell"),
        serial: s(v, "seriennummer"),
        account: s(v, "konto"),
        name: s(v, "name"),
        storage_gib: f("speicher_gib"),
        free_gib: f("frei_gib"),
        battery: v.get("akku").and_then(Value::as_i64).unwrap_or(-1) as i32,
        charging: v.get("laedt").and_then(Value::as_bool).unwrap_or(false),
        system: s(v, "system"),
        fedora: s(v, "fedora"),
        kernel: s(v, "kernel"),
        up_h: f("seit_h"),
    }
}

pub fn info(account: &str) -> Result<Info, String> {
    json_of(account, &["info"]).map(|v| info_of(&v))
}

/// One entry of a folder (on the duo or on the laptop).
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub name: String,
    pub dir: bool,
    pub size: u64,
}

/// A folder on the duo, relative to its person's home ("" = home).
pub fn list_duo(account: &str, rel: &str) -> Result<(String, Vec<Entry>), String> {
    let v = json_of(account, &["liste", rel])?;
    let entries = v
        .get("eintraege")
        .and_then(Value::as_array)
        .map(|a| a.iter().map(|e| Entry { name: s(e, "name"), dir: e.get("ordner").and_then(Value::as_bool).unwrap_or(false), size: e.get("groesse").and_then(Value::as_u64).unwrap_or(0) }).collect())
        .unwrap_or_default();
    Ok((s(&v, "pfad"), entries))
}

/// A folder on the laptop: folders first, then files, by name; no hidden ones.
pub fn list_laptop(dir: &Path) -> Vec<Entry> {
    let mut out: Vec<Entry> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                return None;
            }
            let md = e.metadata().ok()?;
            Some(Entry { name, dir: md.is_dir(), size: md.len() })
        })
        .collect();
    out.sort_by(|a, b| b.dir.cmp(&a.dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    out
}

/// A free name in `dir` (the kit's: «name (2).ext» …) — never overwrite.
pub fn unique(dir: &Path, name: &str) -> PathBuf {
    matrixkit_theme::fileops::free_name(dir, name, |p| p.exists())
}

fn join_rel(dir: &str, name: &str) -> String {
    if dir.is_empty() { name.to_string() } else { format!("{dir}/{name}") }
}

/// Duo -> laptop: a file (`lesen`) or a whole folder (`paket` | tar -x).
pub fn copy_to_laptop(account: &str, duo_dir: &str, entry: &Entry, into: &Path) -> Result<PathBuf, String> {
    let rel = join_rel(duo_dir, &entry.name);
    if entry.dir {
        let target = unique(into, &entry.name);
        let staging = into.join(format!(".morpheus-{}", std::process::id()));
        std::fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
        let mut src = begleiter(account, &["paket", &rel]).stdin(Stdio::null()).stdout(Stdio::piped()).spawn().map_err(|e| e.to_string())?;
        let ok = Command::new("tar").args(["-xf", "-", "--no-same-owner", "-C"]).arg(&staging).stdin(Stdio::from(src.stdout.take().ok_or("kein Strom")?)).status().map(|s| s.success()).unwrap_or(false);
        let src_ok = src.wait().map(|s| s.success()).unwrap_or(false);
        let moved = ok && src_ok && std::fs::rename(staging.join(&entry.name), &target).is_ok();
        let _ = std::fs::remove_dir_all(&staging);
        return if moved { Ok(target) } else { Err(String::from("Ordner nicht kopiert")) };
    }
    let target = unique(into, &entry.name);
    let part = target.with_extension("morpheus-teil");
    let file = std::fs::File::create(&part).map_err(|e| e.to_string())?;
    let ok = begleiter(account, &["lesen", &rel]).stdin(Stdio::null()).stdout(file).status().map(|s| s.success()).unwrap_or(false);
    if !ok {
        let _ = std::fs::remove_file(&part);
        return Err(String::from("Datei nicht kopiert"));
    }
    std::fs::rename(&part, &target).map_err(|e| e.to_string())?;
    Ok(target)
}

/// Laptop -> duo: a file (`schreiben`) or a folder (tar | `auspacken`). Never overwrites on the duo.
pub fn copy_to_duo(account: &str, local: &Path, duo_dir: &str) -> Result<String, String> {
    let name = local.file_name().map(|n| n.to_string_lossy().to_string()).ok_or("kein Name")?;
    if local.is_dir() {
        let parent = local.parent().ok_or("kein Ordner")?;
        let mut tar = Command::new("tar").arg("-cf").arg("-").arg("-C").arg(parent).arg(&name).stdout(Stdio::piped()).spawn().map_err(|e| e.to_string())?;
        let ok = begleiter(account, &["auspacken", duo_dir]).stdin(Stdio::from(tar.stdout.take().ok_or("kein Strom")?)).stdout(Stdio::null()).status().map(|s| s.success()).unwrap_or(false);
        let tar_ok = tar.wait().map(|s| s.success()).unwrap_or(false);
        return if ok && tar_ok { Ok(name) } else { Err(String::from("Ordner nicht kopiert")) };
    }
    let file = std::fs::File::open(local).map_err(|e| e.to_string())?;
    let out = begleiter(account, &["schreiben", &join_rel(duo_dir, &name)]).stdin(file).output().map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_slice(&out.stdout).map_err(|_| String::from("Datei nicht kopiert"))?;
    if v.get("ok").and_then(Value::as_bool) == Some(true) { Ok(s(&v, "pfad")) } else { Err(String::from("Datei nicht kopiert")) }
}

/// One photo on the duo, with its preview on the laptop.
#[derive(Debug, Clone, PartialEq)]
pub struct Photo {
    pub path: String,
    pub preview: PathBuf,
    pub size: u64,
}

/// Where this duo's previews and opened photos live on the laptop.
pub fn cache(serial: &str) -> PathBuf {
    home().join(".cache/matrix-morpheus").join(if serial.is_empty() { "duo" } else { serial })
}

/// The duo's photos (Bilder/…), newest first, with previews fetched to the laptop.
pub fn photos(account: &str, serial: &str) -> Result<Vec<Photo>, String> {
    let v = json_of(account, &["vorschau"])?;
    let dir = cache(serial).join("vorschau");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut src = begleiter(account, &["vorschau-tar"]).stdin(Stdio::null()).stdout(Stdio::piped()).spawn().map_err(|e| e.to_string())?;
    let ok = Command::new("tar").args(["-xf", "-", "-C"]).arg(&dir).stdin(Stdio::from(src.stdout.take().ok_or("kein Strom")?)).status().map(|s| s.success()).unwrap_or(false);
    let _ = src.wait();
    if !ok {
        return Err(String::from("Vorschaubilder nicht geladen"));
    }
    Ok(v.get("fotos")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .map(|p| Photo { path: s(p, "pfad"), preview: dir.join(s(p, "vorschau")), size: p.get("groesse").and_then(Value::as_u64).unwrap_or(0) })
                .filter(|p| p.preview.is_file())
                .collect()
        })
        .unwrap_or_default())
}

/// One photo in full, to the laptop's cache (for the big view).
pub fn open_photo(account: &str, serial: &str, photo: &Photo) -> Result<PathBuf, String> {
    let dir = cache(serial).join("gross");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let name = photo.preview.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let ext = Path::new(&photo.path).extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_else(|| String::from("jpg"));
    let target = dir.join(format!("{}.{ext}", name.trim_end_matches(".jpg")));
    if target.is_file() {
        return Ok(target);
    }
    let file = std::fs::File::create(&target).map_err(|e| e.to_string())?;
    let ok = begleiter(account, &["lesen", &photo.path]).stdin(Stdio::null()).stdout(file).status().map(|s| s.success()).unwrap_or(false);
    if ok { Ok(target) } else { let _ = std::fs::remove_file(&target); Err(String::from("Foto nicht geladen")) }
}

/// The duo restarts into its bootloader (for Matrix Recovery). The
/// connection drops when the duo goes down — that is the expected end.
pub fn to_bootloader(account: &str) {
    let _ = begleiter(account, &["bootloader"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_never_overwritten() {
        let d = std::env::temp_dir().join(format!("morpheus-unique-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        assert_eq!(unique(&d, "foto.jpg"), d.join("foto.jpg"));
        std::fs::write(d.join("foto.jpg"), b"x").unwrap();
        assert_eq!(unique(&d, "foto.jpg"), d.join("foto (2).jpg"));
        std::fs::write(d.join("foto (2).jpg"), b"x").unwrap();
        assert_eq!(unique(&d, "foto.jpg"), d.join("foto (3).jpg"));
        std::fs::create_dir_all(d.join("Ordner")).unwrap();
        assert_eq!(unique(&d, "Ordner"), d.join("Ordner (2)"));
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn the_info_is_read() {
        let v: Value = serde_json::from_str(r#"{"ok": true, "modell": "Surface Duo DV_A", "seriennummer": "0000000000", "konto": "mobile", "name": "duo", "speicher_gib": 70.9, "frei_gib": 47.4, "akku": 88, "laedt": true, "system": "2026-10-31", "fedora": "Fedora Linux 42 (Adams)", "kernel": "4.14", "seit_h": 1.5}"#).unwrap();
        let i = info_of(&v);
        assert_eq!((i.account.as_str(), i.battery, i.charging), ("mobile", 88, true));
        assert!((i.storage_gib - 70.9).abs() < 0.01 && (i.free_gib - 47.4).abs() < 0.01);
    }

    #[test]
    fn laptop_folders_come_first_and_hidden_stays_hidden() {
        let d = std::env::temp_dir().join(format!("morpheus-list-{}", std::process::id()));
        std::fs::create_dir_all(d.join("b-ordner")).unwrap();
        std::fs::write(d.join("a.txt"), b"x").unwrap();
        std::fs::write(d.join(".versteckt"), b"x").unwrap();
        let l = list_laptop(&d);
        assert_eq!(l.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), vec!["b-ordner", "a.txt"]);
        std::fs::remove_dir_all(&d).unwrap();
    }
}
