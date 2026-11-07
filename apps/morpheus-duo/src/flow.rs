//! The start's steps and what the tried script says about them.
//!
//! The way into Fedora is `duo-fedora.sh` (on the laptop, ~/duo-a1, proven
//! since 31.10.2026): Android → `adb reboot bootloader` → `fastboot boot`
//! of the start image (RAM only; Android stays the default) → stage 1 starts
//! Fedora from the partition, or the laptop sends the system over USB →
//! SSH answers on 172.16.42.1. The app does not re-invent it: it runs the
//! script and reads its lines («duo-fedora: …») into steps.

/// The five steps, in order.
pub const STEPS: [&str; 5] = ["Duo gefunden", "Bootloader", "Startabbild geladen", "Stufe 1", "Fedora meldet sich"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Waiting,
    Running,
    Done,
    Failed,
}

/// What one line of the script means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Step `n` began (the ones before it are done).
    Begin(usize),
    /// All done: Fedora runs.
    Finished,
    /// The current step failed.
    Failed,
    /// Only for the log.
    Note,
    /// The bootloader is locked: the install cannot begin (Morpheus shows how to unlock).
    Locked,
}

/// Read one script line.
pub fn read(line: &str) -> Event {
    let l = line.trim_start_matches("duo-fedora:").trim();
    if l.starts_with("FERTIG") {
        Event::Finished
    } else if l.starts_with("warte auf das Duo") {
        Event::Begin(0)
    } else if l.starts_with("Android erkannt") {
        Event::Begin(1)
    } else if l.starts_with("RAM-Boot") {
        Event::Begin(2)
    } else if l.starts_with("warte auf Stufe 1") {
        Event::Begin(3)
    } else if l.starts_with("Stufe 1 hat übergeben") || l.starts_with("Stufe 1 im Rettungsmodus") || l.starts_with("warte auf Fedora") {
        Event::Begin(4)
    } else if l.starts_with("kein Bootloader")
        || l.starts_with("Abbild fehlt")
        || l.contains("meldet sich nicht")
        || l.contains("fehlgeschlagen")
    {
        Event::Failed
    } else {
        Event::Note
    }
}

/// The steps after `event` (any number of steps: the start way has five,
/// the install way six).
pub fn apply(steps: &mut [State], event: Event) {
    match event {
        Event::Begin(n) => {
            for (k, s) in steps.iter_mut().enumerate() {
                if k < n {
                    *s = State::Done;
                } else if k == n {
                    *s = State::Running;
                }
            }
        }
        Event::Finished => {
            for s in steps.iter_mut() {
                *s = State::Done;
            }
        }
        Event::Failed | Event::Locked => {
            if let Some(s) = steps.iter_mut().find(|s| **s == State::Running) {
                *s = State::Failed;
            }
        }
        Event::Note => {}
    }
}

/// The install way's end: the backup may have been skipped (it stays
/// waiting), everything else done.
pub fn install_done(steps: &[State]) -> bool {
    steps.iter().enumerate().all(|(k, s)| *s == State::Done || (k == 4 && *s == State::Waiting))
}

/// MORPHEUS, THE INSTALL WAY (06.11.2026): the laptop's part of putting
/// Matrix on a new duo — `morpheus-live.sh` (bootloader check, the live
/// system in RAM) and then `morpheus-lieferant.sh` (backup and image on the
/// duo's request). The choices are made on the duo; the laptop delivers.
pub const INSTALL_STEPS: [&str; 6] = ["Duo gefunden", "Bootloader entsperrt", "Live-System gestartet", "Installer auf dem Duo", "Sicherung", "Matrix übertragen"];

/// What one line of the install scripts means. The backup is optional:
/// a step that never ran stays waiting and is not a failure.
pub fn read_install(line: &str) -> Event {
    let l = line.trim_start_matches("morpheus:").trim();
    if l.starts_with("Duo fertig") {
        Event::Finished
    } else if l.starts_with("warte auf das Duo") {
        Event::Begin(0)
    } else if l.starts_with("Bootloader entsperrt: yes") || l.starts_with("RAM-Boot") {
        Event::Begin(2)
    } else if l.starts_with("Bootloader entsperrt") || l.starts_with("Android erkannt") {
        Event::Begin(1)
    } else if l.starts_with("FERTIG") || l.starts_with("Lieferant bereit") {
        Event::Begin(3)
    } else if l.starts_with("sicherung läuft") || l.starts_with("sicherung fertig") {
        Event::Begin(4)
    } else if l.starts_with("abbild") && !l.contains("fehler") {
        Event::Begin(5)
    } else if l.starts_with("GESPERRT") {
        Event::Locked
    } else if l.contains("fehler") || l.contains("meldet sich nicht") || l.starts_with("kein Bootloader") || l.contains("fehlt") || l.contains("nimmt nichts an") || l.contains("fehlgeschlagen") {
        Event::Failed
    } else {
        Event::Note
    }
}

/// What the duo is right now (the watcher, while no start runs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Link {
    None,
    Android,
    Bootloader,
    /// Stage 1 (telnet on 172.16.42.1:23).
    Stage1,
    /// Fedora answers (SSH on 172.16.42.1:22).
    Fedora,
    /// 06.11.2026 (Fremden-Test am PC): the duo is there, but this user may not
    /// talk to it — `no permissions` from adb/fastboot. Linux without the
    /// Android udev rule (systemd < 258, e.g. Ubuntu 24.04): Morpheus offers
    /// to install it.
    NoAccess,
    /// Android asks «USB-Debugging zulassen?» on its screen (adb `unauthorized`).
    Unauthorized,
}

impl Link {
    pub fn says(self) -> &'static str {
        match self {
            Link::None => "Kein Duo verbunden",
            Link::Android => "Android läuft",
            Link::Bootloader => "Im Bootloader",
            Link::Stage1 => "Stufe 1 startet",
            Link::Fedora => "Fedora läuft",
            Link::NoAccess => "Kein Zugriff auf das Duo",
            Link::Unauthorized => "Auf dem Duo bestätigen",
        }
    }

    /// Can a start begin from here?
    pub fn can_start(self) -> bool {
        matches!(self, Link::Android | Link::Bootloader)
    }
}

/// From `adb devices` and `fastboot devices` output and the two ports.
pub fn link_of(adb: &str, fastboot: &str, ssh: bool, telnet: bool) -> Link {
    if ssh {
        Link::Fedora
    } else if telnet {
        Link::Stage1
    } else if fastboot.lines().any(|l| l.trim_end().ends_with("fastboot")) {
        Link::Bootloader
    } else if adb.lines().skip(1).any(|l| l.trim_end().ends_with("device")) {
        Link::Android
    } else if fastboot.lines().chain(adb.lines().skip(1)).any(|l| l.contains("no permissions")) {
        Link::NoAccess
    } else if adb.lines().skip(1).any(|l| l.trim_end().ends_with("unauthorized")) {
        Link::Unauthorized
    } else {
        Link::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_script_lines_drive_the_steps() {
        let mut s = [State::Waiting; 5];
        for line in [
            "duo-fedora: warte auf das Duo (Android oder Bootloader) ...",
            "duo-fedora: Android erkannt -> Bootloader",
            "duo-fedora: RAM-Boot boot-duo1-fedora-stufe1-v7.img",
            "duo-fedora: warte auf Stufe 1 ...",
        ] {
            apply(&mut s, read(line));
        }
        assert_eq!(s, [State::Done, State::Done, State::Done, State::Running, State::Waiting]);
        apply(&mut s, read("duo-fedora: Stufe 1 hat übergeben (Start von der Partition)"));
        assert_eq!(s[4], State::Running);
        apply(&mut s, read("duo-fedora: FERTIG – Fedora läuft von der Partition, bitte anmelden"));
        assert_eq!(s, [State::Done; 5]);
    }

    #[test]
    fn a_failure_marks_the_running_step() {
        let mut s = [State::Waiting; 5];
        apply(&mut s, read("duo-fedora: warte auf das Duo (Android oder Bootloader) ..."));
        apply(&mut s, read("duo-fedora: kein Bootloader gefunden"));
        assert_eq!(s[0], State::Failed);
        assert_eq!(read("duo-fedora: Fedora meldet sich nicht per USB (WLAN prüfen)"), Event::Failed);
        assert_eq!(read("duo-fedora: Übertragung fehlgeschlagen, siehe /tmp/v6-squash.log"), Event::Failed);
        assert_eq!(read("Sending 'boot.img' (12000 KB)"), Event::Note);
    }

    #[test]
    fn the_install_lines_drive_the_install_steps() {
        let mut s = [State::Waiting; 6];
        for line in [
            "morpheus: warte auf das Duo (Android oder Bootloader) ...",
            "morpheus: Android erkannt -> Bootloader",
            "morpheus: Bootloader entsperrt: yes",
            "morpheus: übertrage das Live-System (1008 MB) ...",
            "morpheus: FERTIG – Live-System läuft, Morpheus auf dem Duo",
            "morpheus: Lieferant bereit",
        ] {
            apply(&mut s, read_install(line));
        }
        assert_eq!(s, [State::Done, State::Done, State::Done, State::Running, State::Waiting, State::Waiting]);
        apply(&mut s, read_install("morpheus: abbild läuft 1008 MB"));
        assert_eq!(s[5], State::Running);
        apply(&mut s, read_install("morpheus: Duo fertig"));
        assert!(install_done(&s));
    }

    #[test]
    fn a_locked_bootloader_and_failures_stop_the_way() {
        let mut s = [State::Waiting; 6];
        apply(&mut s, read_install("morpheus: warte auf das Duo (Android oder Bootloader) ..."));
        apply(&mut s, read_install("morpheus: Bootloader entsperrt: no"));
        assert_eq!(read_install("morpheus: GESPERRT – Bootloader zuerst entsperren (siehe Anleitung)"), Event::Locked);
        apply(&mut s, Event::Locked);
        assert_eq!(s[1], State::Failed);
        assert_eq!(read_install("morpheus: sicherung fehler Prüfsummen"), Event::Failed);
        assert_eq!(read_install("morpheus: abbild fehler Übertragung"), Event::Failed);
        let skipped = [State::Done, State::Done, State::Done, State::Done, State::Waiting, State::Done];
        assert!(install_done(&skipped), "the backup may be skipped");
    }

    #[test]
    fn the_watcher_names_the_duo() {
        let adb_none = "List of devices attached\n\n";
        let adb_one = "List of devices attached\n0F0012345678\tdevice\n";
        let adb_unauth = "List of devices attached\n0F0012345678\tunauthorized\n";
        assert_eq!(link_of(adb_none, "", false, false), Link::None);
        assert_eq!(link_of(adb_one, "", false, false), Link::Android);
        assert_eq!(link_of(adb_unauth, "", false, false), Link::Unauthorized, "unauthorized cannot reboot");
        assert!(!Link::Unauthorized.can_start());
        // A PC without the Android udev rule (Fremden-Test 06.11.2026).
        let fb_noperm = "????????????\tno permissions (missing udev rules? user is in the plugdev group); see [http://developer.android.com/tools/device.html]\n";
        assert_eq!(link_of(adb_none, fb_noperm, false, false), Link::NoAccess);
        let adb_noperm = "List of devices attached\n0F0012345678\tno permissions (missing udev rules? user is in the plugdev group); see [http://developer.android.com/tools/device.html]\n";
        assert_eq!(link_of(adb_noperm, "", false, false), Link::NoAccess);
        assert!(!Link::NoAccess.can_start());
        assert_eq!(link_of(adb_none, "0F0012345678\tfastboot\n", false, false), Link::Bootloader);
        assert_eq!(link_of(adb_none, "", false, true), Link::Stage1);
        assert_eq!(link_of(adb_one, "", true, false), Link::Fedora);
        assert!(Link::Android.can_start() && Link::Bootloader.can_start() && !Link::Fedora.can_start());
    }
}

/// MATRIX RECOVERY (06.11.2026): what `morpheus-recovery zustand` reports,
/// read from its one flat JSON line («morpheus: ZUSTAND {…}»).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Device {
    pub matrix: bool,
    pub model: String,
    pub serial: String,
    pub accounts: Vec<String>,
    pub matrix_gib: String,
    pub data_mb: u64,
    pub system: String,
    pub dry: bool,
}

/// One value of a flat JSON object, raw (string without quotes, number,
/// bool, or an array's inner text). Enough for the helper's line.
pub fn json_field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let at = json.find(&format!("\"{key}\""))? + key.len() + 2;
    let rest = json[at..].trim_start().strip_prefix(':')?.trim_start();
    if let Some(s) = rest.strip_prefix('"') {
        return s.find('"').map(|e| &s[..e]);
    }
    if let Some(s) = rest.strip_prefix('[') {
        return s.find(']').map(|e| &s[..e]);
    }
    let end = rest.find([',', '}']).unwrap_or(rest.len());
    Some(rest[..end].trim())
}

/// The device from a «ZUSTAND» line; None for any other line.
pub fn read_device(line: &str) -> Option<Device> {
    let json = line.trim_start_matches("morpheus:").trim().strip_prefix("ZUSTAND")?.trim();
    let text = |k: &str| json_field(json, k).unwrap_or("").to_string();
    Some(Device {
        matrix: json_field(json, "matrix_da") == Some("true"),
        model: text("modell"),
        serial: text("seriennummer"),
        accounts: json_field(json, "konten")
            .unwrap_or("")
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        matrix_gib: text("matrix_gib"),
        data_mb: json_field(json, "daten_mb").and_then(|v| v.parse().ok()).unwrap_or(0),
        system: text("system"),
        dry: json_field(json, "trocken") == Some("true"),
    })
}

#[cfg(test)]
mod recovery_tests {
    use super::*;

    #[test]
    fn the_device_is_read_from_its_line() {
        let line = r#"morpheus: ZUSTAND {"trocken": false, "matrix_da": true, "modell": "Surface Duo DV_A", "seriennummer": "0000000000", "matrix_gib": 72.0, "konten": ["mobile", "anna"], "system": "2026-10-31", "frei_gib": 47.4, "daten_mb": 10793}"#;
        let d = read_device(line).unwrap();
        assert!(d.matrix && !d.dry);
        assert_eq!(d.model, "Surface Duo DV_A");
        assert_eq!(d.serial, "0000000000");
        assert_eq!(d.accounts, vec!["mobile", "anna"]);
        assert_eq!(d.matrix_gib, "72.0");
        assert_eq!(d.data_mb, 10793);
        assert_eq!(d.system, "2026-10-31");
        let fresh = read_device(r#"morpheus: ZUSTAND {"trocken": true, "matrix_da": false, "modell": "?"}"#).unwrap();
        assert!(!fresh.matrix && fresh.dry && fresh.accounts.is_empty());
        assert_eq!(read_device("morpheus: RAM-Boot boot.img"), None);
    }
}
