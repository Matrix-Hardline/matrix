//! The iPhone bridge (R149): an attached iOS device appears in the
//! sidebar like a volume and mounts itself when clicked.
//!
//! The system layer already works without us: udev starts usbmuxd the
//! moment an Apple device shows up (39-usbmuxd.rules), and the pairing
//! lives in /var/lib/lockdown. What was missing was the last mile —
//! the file manager knowing that the device exists.
//!
//! Reference grammar (Finder): the device is a place, not a folder in
//! $HOME. So we mount inside the runtime directory and let the sidebar
//! carry the name; nothing is left behind in the home directory, and a
//! reboot cannot leave a stale mount point around.
//!
//! Cost discipline (R135, no polling): asking `idevice_id` costs a
//! process, so we only ask when sysfs says an Apple device is on the
//! bus — a few file reads. With nothing plugged in, the bridge is free.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Apple's USB vendor id — the cheap sysfs gate in front of every query.
const APPLE_VENDOR: &str = "05ac";

/// One attached iOS device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub udid: String,
    /// The device's own name ("Anna's iPhone") — never a guess we made
    /// up: an unpaired device answers nothing, and then the UDID stands.
    pub name: String,
    /// A valid pairing exists; without it iOS refuses to open the disk
    /// and the user has to tap "Trust" on the device itself.
    pub paired: bool,
}

impl Device {
    /// Where this device gets mounted — one directory per UDID, so two
    /// iPhones never collide.
    pub fn mount_point(&self) -> PathBuf {
        mount_root().join(&self.udid)
    }
}

/// Runtime directory for the mounts: it dies with the session, so no
/// stale mount points survive a reboot.
fn mount_root() -> PathBuf {
    let base = std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("matrix/iphone")
}

/// Is an Apple device on the USB bus? Pure file reads — this is the
/// gate that keeps the bridge free while nothing is plugged in.
pub fn attached() -> bool {
    let Ok(dir) = std::fs::read_dir("/sys/bus/usb/devices") else {
        return false;
    };
    for e in dir.flatten() {
        if let Ok(v) = std::fs::read_to_string(e.path().join("idVendor")) {
            if v.trim().eq_ignore_ascii_case(APPLE_VENDOR) {
                return true;
            }
        }
    }
    false
}

/// UDIDs from `idevice_id -l` output. Errors go to stderr, so an empty
/// or noisy stdout simply yields nothing.
pub fn parse_udids(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && l.len() >= 20)
        .filter(|l| l.chars().all(|c| c.is_ascii_hexdigit() || c == '-'))
        .map(str::to_string)
        .collect()
}

/// Every attached iOS device, with name and pairing state.
pub fn devices() -> Vec<Device> {
    if !attached() {
        return Vec::new();
    }
    let out = match Command::new("idevice_id").arg("-l").output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => return Vec::new(), // tool missing — bridge stays silent
    };
    parse_udids(&out)
        .into_iter()
        .map(|udid| {
            let paired = Command::new("idevicepair")
                .args(["-u", &udid, "validate"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            let name = Command::new("idevicename")
                .args(["-u", &udid])
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| String::from("iPhone"));
            Device { udid, name, paired }
        })
        .collect()
}

/// Is that path a mount point right now? Read from the kernel's own
/// table — asking the mounter would only tell us what we intended.
pub fn is_mounted(path: &Path) -> bool {
    let Ok(info) = std::fs::read_to_string("/proc/self/mountinfo") else {
        return false;
    };
    let want = path.to_string_lossy();
    info.lines()
        .filter_map(|l| l.split_whitespace().nth(4))
        .any(|m| m == want)
}

/// Mount the device, or hand back a sentence the user can act on.
/// Already mounted is success — clicking a mounted device just opens it.
pub fn mount(dev: &Device) -> Result<PathBuf, String> {
    let mp = dev.mount_point();
    if is_mounted(&mp) {
        return Ok(mp);
    }
    if !dev.paired {
        return Err(String::from(
            "Bitte am iPhone entsperren und „Vertrauen“ antippen.",
        ));
    }
    std::fs::create_dir_all(&mp).map_err(|e| e.to_string())?;
    let out = Command::new("ifuse")
        .args(["-u", &dev.udid])
        .arg(&mp)
        .output()
        .map_err(|_| String::from("ifuse fehlt im System."))?;
    if !out.status.success() || !is_mounted(&mp) {
        let _ = std::fs::remove_dir(&mp);
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if err.is_empty() {
            String::from("Das iPhone ließ sich nicht öffnen.")
        } else {
            err
        });
    }
    Ok(mp)
}

/// Unmount and take the empty directory with us — a leftover mount
/// point would show up as a phantom device on the next look.
pub fn eject(mount_point: &Path) -> Result<(), String> {
    let out = Command::new("fusermount")
        .arg("-u")
        .arg(mount_point)
        .output()
        .map_err(|_| String::from("fusermount fehlt im System."))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if err.is_empty() {
            String::from("Das iPhone ist noch in Benutzung.")
        } else {
            err
        });
    }
    let _ = std::fs::remove_dir(mount_point);
    Ok(())
}

/// Does that path lie on a mounted device? Used for the eject button.
pub fn inside_device(path: &Path) -> Option<PathBuf> {
    let root = mount_root();
    let rest = path.strip_prefix(&root).ok()?;
    let udid = rest.components().next()?;
    let mp = root.join(udid);
    is_mounted(&mp).then_some(mp)
}

/// udev lines worth waking up for. `udevadm monitor --property` prints
/// the vendor id and the flag the usbmuxd rule sets, so we react to
/// Apple devices only — and to nothing else on the bus.
pub fn is_ios_event(line: &str) -> bool {
    line.contains("ID_VENDOR_ID=05ac") || line.contains("USBMUX_SUPPORTED=1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn udids_are_read_and_noise_is_dropped() {
        let out = "00008130-000E612E2288001C\n\n";
        assert_eq!(parse_udids(out), vec!["00008130-000E612E2288001C"]);
        // The error case prints to stderr; stdout stays empty.
        assert!(parse_udids("").is_empty());
        // Prose is never a UDID (a stray warning on stdout).
        assert!(parse_udids("No device found.").is_empty());
    }

    #[test]
    fn one_directory_per_device() {
        let a = Device { udid: String::from("aaaa1111bbbb2222cccc"), name: String::new(), paired: true };
        let b = Device { udid: String::from("dddd3333eeee4444ffff"), name: String::new(), paired: true };
        assert_ne!(a.mount_point(), b.mount_point());
        assert!(a.mount_point().starts_with(mount_root()));
    }

    #[test]
    fn nothing_is_mounted_at_a_made_up_path() {
        assert!(!is_mounted(Path::new("/matrix/no/such/mount")));
    }

    #[test]
    fn only_apple_events_wake_the_bridge() {
        assert!(is_ios_event("ID_VENDOR_ID=05ac"));
        assert!(is_ios_event("USBMUX_SUPPORTED=1"));
        assert!(!is_ios_event("ID_VENDOR_ID=1d6b")); // root hub
        assert!(!is_ios_event("ACTION=add"));
    }

    #[test]
    fn unpaired_device_asks_for_trust() {
        let dev = Device { udid: String::from("0000f00dbaadf00d0000"), name: String::new(), paired: false };
        let err = mount(&dev).unwrap_err();
        assert!(err.contains("Vertrauen"), "sentence must name the tap: {err}");
    }
}
