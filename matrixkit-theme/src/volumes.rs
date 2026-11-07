//! REMOVABLE DRIVES (R217 family, 02.11.2026 — out of Matrix Dateien,
//! The owner: «move Downloads to the USB stick»): the USB sticks and
//! cards plugged in, each partition that carries a filesystem; mount it
//! (udisks, no root — polkit allows the active session), unmount and power
//! the stick off for safe removal, and how much room is left.
//!
//! The parsing is pure (tested); the commands run with LC_ALL=C.UTF-8 so
//! their words stay the ones we parse.

use std::path::{Path, PathBuf};

/// One mountable partition (or a stick without a partition table).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    /// The block device to mount: `/dev/sdb1`.
    pub device: String,
    /// The whole disk it lives on (power-off goes there): `/dev/sdb`.
    pub disk: String,
    /// The filesystem's label (may be empty).
    pub label: String,
    /// vfat, exfat, ntfs, ext4 …
    pub fstype: String,
    /// Size of the partition in bytes.
    pub size: u64,
    /// Where it is mounted now (None: not mounted).
    pub mount: Option<PathBuf>,
    /// The disk's model («Cruzer Blade»), may be empty.
    pub model: String,
    /// The transport: usb, mmc … (empty when unknown).
    pub transport: String,
}

impl Volume {
    /// The name a person reads: the label, else the model, else «USB-Stick».
    pub fn name(&self) -> String {
        let label = self.label.trim();
        if !label.is_empty() {
            return label.to_string();
        }
        let model = self.model.trim();
        if !model.is_empty() {
            return model.to_string();
        }
        if self.transport == "mmc" {
            String::from("Speicherkarte")
        } else {
            String::from("USB-Stick")
        }
    }

    pub fn mounted(&self) -> bool {
        self.mount.is_some()
    }
}

/// lsblk writes booleans as true/false (new) or "1"/"0" (old util-linux).
fn flag(v: Option<&serde_json::Value>) -> bool {
    match v {
        Some(serde_json::Value::Bool(b)) => *b,
        Some(serde_json::Value::String(s)) => s == "1" || s == "true",
        Some(serde_json::Value::Number(n)) => n.as_u64() == Some(1),
        _ => false,
    }
}

fn text(v: Option<&serde_json::Value>) -> String {
    match v {
        Some(serde_json::Value::String(s)) => s.trim().to_string(),
        _ => String::new(),
    }
}

fn number(v: Option<&serde_json::Value>) -> u64 {
    match v {
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0),
        Some(serde_json::Value::String(s)) => s.trim().parse().unwrap_or(0),
        _ => 0,
    }
}

/// Mount points that belong to the running system, never offered.
fn system_mount(m: &str) -> bool {
    m == "/" || m == "[SWAP]" || m.starts_with("/boot") || m.starts_with("/sysroot") || m == "/usr" || m == "/var" || m == "/home" || m == "/etc"
}

/// Filesystems one cannot mount and browse as they are.
fn browsable(fstype: &str) -> bool {
    !fstype.is_empty() && fstype != "swap" && fstype != "crypto_LUKS" && fstype != "LVM2_member" && !fstype.ends_with("_raid_member")
}

/// THE PARSE: `lsblk -J -b -o NAME,PATH,SIZE,TRAN,RM,HOTPLUG,FSTYPE,LABEL,MOUNTPOINT,MODEL`
/// → the removable volumes. A disk counts when it hangs on USB or is
/// hot-pluggable (an internal UFS/NVMe never does); of it, every partition
/// with a filesystem — or the disk itself when it carries one directly.
pub fn parse_lsblk(json: &str) -> Vec<Volume> {
    let Ok(root) = serde_json::from_str::<serde_json::Value>(json) else { return Vec::new() };
    let Some(disks) = root.get("blockdevices").and_then(|d| d.as_array()) else { return Vec::new() };
    let mut out = Vec::new();
    for disk in disks {
        let name = text(disk.get("name"));
        if name.starts_with("loop") || name.starts_with("zram") || name.starts_with("ram") {
            continue;
        }
        let transport = text(disk.get("tran"));
        // An SD card in a reader is removable too; an internal disk never.
        let removable = transport == "usb" || flag(disk.get("hotplug")) || (transport == "mmc" && flag(disk.get("rm")));
        if !removable {
            continue;
        }
        let disk_path = {
            let p = text(disk.get("path"));
            if p.is_empty() { format!("/dev/{name}") } else { p }
        };
        let model = text(disk.get("model"));
        let take = |node: &serde_json::Value, out: &mut Vec<Volume>| {
            let fstype = text(node.get("fstype"));
            if !browsable(&fstype) {
                return;
            }
            let mount = text(node.get("mountpoint"));
            if system_mount(&mount) {
                return;
            }
            let device = {
                let p = text(node.get("path"));
                if p.is_empty() { format!("/dev/{}", text(node.get("name"))) } else { p }
            };
            out.push(Volume {
                device,
                disk: disk_path.clone(),
                label: text(node.get("label")),
                fstype,
                size: number(node.get("size")),
                mount: (!mount.is_empty()).then(|| PathBuf::from(mount)),
                model: model.clone(),
                transport: transport.clone(),
            });
        };
        match disk.get("children").and_then(|c| c.as_array()) {
            Some(children) if !children.is_empty() => {
                for part in children {
                    take(part, &mut out);
                }
            }
            _ => take(disk, &mut out),
        }
    }
    out
}

/// What `udisksctl mount` said → the mount point.
/// «Mounted /dev/sdb1 at /run/media/anna/STICK» (udisks ≥ 2.10) or the
/// same with a closing «.» (older). The path may hold spaces.
pub fn parse_mounted(out: &str) -> Option<PathBuf> {
    for line in out.lines() {
        let line = line.trim();
        if !line.starts_with("Mounted ") {
            continue;
        }
        let (_, at) = line.split_once(" at ")?;
        let mut path = at.trim();
        if path.len() > 1 && path.ends_with('.') && !Path::new(path).exists() {
            path = &path[..path.len() - 1];
        }
        if path.starts_with('/') {
            return Some(PathBuf::from(path));
        }
    }
    None
}

/// A command with words we can parse.
fn c_cmd(program: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    cmd.env("LC_ALL", "C.UTF-8");
    cmd
}

/// The error text of a failed command, short.
fn failure(out: &std::process::Output) -> String {
    let err = String::from_utf8_lossy(&out.stderr);
    let line = err.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim().to_string();
    // udisks wraps its reason in a D-Bus error name; the reason is last.
    match line.rsplit_once(": ") {
        Some((_, reason)) if !reason.is_empty() => reason.to_string(),
        _ => line,
    }
}

/// The removable volumes now (an empty list when lsblk is missing).
pub fn list() -> Vec<Volume> {
    let out = c_cmd("lsblk").args(["-J", "-b", "-o", "NAME,PATH,SIZE,TRAN,RM,HOTPLUG,FSTYPE,LABEL,MOUNTPOINT,MODEL"]).output();
    match out {
        Ok(o) if o.status.success() => parse_lsblk(&String::from_utf8_lossy(&o.stdout)),
        _ => Vec::new(),
    }
}

/// Mount a volume (by its device) and say where. Already mounted: where it is.
pub fn mount(device: &str) -> Result<PathBuf, String> {
    let out = c_cmd("udisksctl").args(["mount", "-b", device, "--no-user-interaction"]).output().map_err(|e| format!("udisksctl fehlt ({e})"))?;
    if out.status.success() {
        if let Some(p) = parse_mounted(&String::from_utf8_lossy(&out.stdout)) {
            return Ok(p);
        }
    }
    // Already mounted (or an unexpected wording): ask lsblk.
    if let Some(p) = list().into_iter().find(|v| v.device == device).and_then(|v| v.mount) {
        return Ok(p);
    }
    Err(if out.status.success() { String::from("Einhängepunkt unbekannt") } else { failure(&out) })
}

/// Unmount one volume.
pub fn unmount(device: &str) -> Result<(), String> {
    let out = c_cmd("udisksctl").args(["unmount", "-b", device, "--no-user-interaction"]).output().map_err(|e| format!("udisksctl fehlt ({e})"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(failure(&out))
    }
}

/// SAFE REMOVAL: write everything out, unmount every mounted volume of the
/// disk, then power the disk off. After Ok the stick may be pulled.
pub fn eject(disk: &str) -> Result<(), String> {
    let _ = std::process::Command::new("sync").status();
    for v in list().into_iter().filter(|v| v.disk == disk && v.mounted()) {
        unmount(&v.device).map_err(|e| format!("{} lässt sich nicht aushängen: {e}", v.name()))?;
    }
    let out = c_cmd("udisksctl").args(["power-off", "-b", disk, "--no-user-interaction"]).output().map_err(|e| format!("udisksctl fehlt ({e})"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(failure(&out))
    }
}

/// Room on the filesystem that holds `path`: (free for us, total) in bytes.
pub fn space(path: &Path) -> Option<(u64, u64)> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: statvfs fills the zeroed struct; the path is a valid C string.
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut st) } != 0 {
        return None;
    }
    let unit = if st.f_frsize > 0 { st.f_frsize } else { st.f_bsize } as u64;
    Some((st.f_bavail as u64 * unit, st.f_blocks as u64 * unit))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LSBLK: &str = r#"{
   "blockdevices": [
      {"name":"sda", "path":"/dev/sda", "size":256060514304, "tran":"ufs", "rm":false, "hotplug":false, "fstype":null, "label":null, "mountpoint":null, "model":"KLUDG4UHDB-B2D1",
         "children": [
            {"name":"sda7", "path":"/dev/sda7", "size":100000000000, "tran":null, "rm":false, "hotplug":false, "fstype":"ext4", "label":"fedora", "mountpoint":"/", "model":null}
         ]
      },
      {"name":"sdb", "path":"/dev/sdb", "size":31004295168, "tran":"usb", "rm":true, "hotplug":true, "fstype":null, "label":null, "mountpoint":null, "model":"Cruzer Blade",
         "children": [
            {"name":"sdb1", "path":"/dev/sdb1", "size":31003246592, "tran":null, "rm":true, "hotplug":true, "fstype":"vfat", "label":"STICK", "mountpoint":"/run/media/anna/STICK", "model":null},
            {"name":"sdb2", "path":"/dev/sdb2", "size":1048576, "tran":null, "rm":true, "hotplug":true, "fstype":null, "label":null, "mountpoint":null, "model":null}
         ]
      },
      {"name":"sdc", "path":"/dev/sdc", "size":"64023257088", "tran":"usb", "rm":"1", "hotplug":"1", "fstype":"exfat", "label":null, "mountpoint":null, "model":"  Ultra  "},
      {"name":"zram0", "path":"/dev/zram0", "size":8000000000, "tran":null, "rm":false, "hotplug":false, "fstype":"swap", "label":null, "mountpoint":"[SWAP]", "model":null}
   ]
}"#;

    #[test]
    fn finds_the_sticks_not_the_system() {
        let v = parse_lsblk(LSBLK);
        assert_eq!(v.len(), 2, "{v:?}");
        assert_eq!(v[0].device, "/dev/sdb1");
        assert_eq!(v[0].disk, "/dev/sdb");
        assert_eq!(v[0].name(), "STICK");
        assert_eq!(v[0].fstype, "vfat");
        assert_eq!(v[0].size, 31003246592);
        assert_eq!(v[0].mount.as_deref(), Some(Path::new("/run/media/anna/STICK")));
        // A stick without a partition table, old lsblk wording (strings).
        assert_eq!(v[1].device, "/dev/sdc");
        assert_eq!(v[1].disk, "/dev/sdc");
        assert_eq!(v[1].name(), "Ultra");
        assert_eq!(v[1].size, 64023257088);
        assert!(!v[1].mounted());
    }

    #[test]
    fn bad_json_is_no_drive() {
        assert!(parse_lsblk("").is_empty());
        assert!(parse_lsblk("{\"blockdevices\": []}").is_empty());
    }

    #[test]
    fn names_without_label() {
        let mut v = parse_lsblk(LSBLK).remove(0);
        v.label.clear();
        assert_eq!(v.name(), "Cruzer Blade");
        v.model.clear();
        assert_eq!(v.name(), "USB-Stick");
        v.transport = String::from("mmc");
        assert_eq!(v.name(), "Speicherkarte");
    }

    #[test]
    fn reads_the_mount_point() {
        assert_eq!(parse_mounted("Mounted /dev/sdb1 at /run/media/anna/STICK\n"), Some(PathBuf::from("/run/media/anna/STICK")));
        assert_eq!(parse_mounted("Mounted /dev/sdb1 at /run/media/anna/MEIN STICK.\n"), Some(PathBuf::from("/run/media/anna/MEIN STICK")));
        assert_eq!(parse_mounted("Error mounting /dev/sdb1: busy"), None);
    }

    #[test]
    fn space_of_the_root() {
        let (free, total) = space(Path::new("/")).expect("statvfs on /");
        assert!(total > 0 && free <= total);
    }
}
