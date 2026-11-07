//! MORPHEUS, the installer (round 176, 06.11.2026): in the live system a
//! new Surface Duo boots from the laptop (/etc/morpheus-live), the login
//! screen leads through putting Matrix beside Android — the device, the
//! backup (recommended), the split, an explicit two-step confirmation, the
//! progress, the restart. The work itself is done by `morpheus-helfer`
//! (root, /run/morpheus.sock); the laptop carries backup and image.

use matrixkit_scene::scene as fam;
use matrixkit_scene::symbol;
use matrixkit_shell::SceneBuilder;
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::Rect;
use serde_json::{json, Value};

pub const LIVE: &str = "/etc/morpheus-live";
const SOCK: &str = "/run/morpheus.sock";
/// The words the helper wants with «installieren» — the second tap sends them.
const CONFIRM: &str = "ANDROID LÖSCHEN";

/// Roles of the installer's page (from 200).
pub const R_NEXT: usize = 200;
pub const R_BACK: usize = 201;
pub const R_SKIP: usize = 202;
pub const R_RESTART: usize = 203;
/// Matrix Recovery (06.11.2026): the answer to the laptop's one question.
pub const R_YES: usize = 204;
pub const R_NO: usize = 205;
pub const R_SPLIT: usize = 210;

/// Android sizes on offer (GB); the helper refuses what does not fit.
const SPLITS: [u32; 5] = [16, 24, 32, 48, 64];

#[derive(Debug, Clone, Copy, PartialEq)]
enum Page {
    Welcome,
    Backup,
    Split,
    Confirm,
    Running,
}

pub struct Installer {
    page: Page,
    android_gib: u32,
    /// The first tap on «Installieren» arms it; the second sends (round 177: the kit's two taps, with a time limit).
    armed: mk::confirm::Armed<()>,
    /// The helper's last state (`{"tu":"stand"}`).
    state: Value,
    /// The plan for the chosen split (or why not).
    plan: Result<Value, String>,
    asked_backup: bool,
}

/// One request to the helper (blocking, local socket — milliseconds).
fn ask(req: Value) -> Result<Value, String> {
    use std::io::{BufRead, Write};
    let mut s = std::os::unix::net::UnixStream::connect(SOCK).map_err(|e| format!("Installer-Dienst nicht erreichbar: {e}"))?;
    s.set_read_timeout(Some(std::time::Duration::from_secs(3))).ok();
    s.write_all(format!("{req}\n").as_bytes()).map_err(|e| e.to_string())?;
    let mut line = String::new();
    std::io::BufReader::new(s).read_line(&mut line).map_err(|e| e.to_string())?;
    serde_json::from_str(&line).map_err(|e| e.to_string())
}

/// German or English, by the language chosen (mk::language).
fn l<'a>(de: &'a str, en: &'a str) -> &'a str {
    mk::language::pick(de, en)
}

/// The helper's step names (German) in the chosen language.
fn step_name(de: &str) -> &str {
    match de {
        "Vollsicherung auf den Laptop" => l(de, "Full backup to the laptop"),
        "Speicher aufteilen" => l(de, "Split the storage"),
        "Android zurücksetzen" => l(de, "Reset Android"),
        "Matrix-Partition einrichten" => l(de, "Set up the Matrix partition"),
        "Matrix übertragen" => l(de, "Transfer Matrix"),
        "Prüfen" => l(de, "Check"),
        _ => de,
    }
}

pub fn live() -> bool {
    std::path::Path::new(LIVE).exists()
}

impl Installer {
    pub fn new() -> Installer {
        let mut me = Installer { page: Page::Welcome, android_gib: 32, armed: mk::confirm::Armed::new(), state: Value::Null, plan: Err(String::new()), asked_backup: false };
        me.refresh();
        me
    }

    /// Read the helper's state (every beat).
    pub fn refresh(&mut self) -> bool {
        // An armed «Installieren» that ran out: redraw the question away.
        if self.armed.lapsed() {
            return true;
        }
        match ask(json!({"tu": "stand"})) {
            Ok(v) if v != self.state => {
                self.state = v;
                if self.page == Page::Backup && self.backup_done() {
                    self.page = Page::Split;
                    self.replan();
                }
                true
            }
            Ok(_) => false,
            Err(e) => {
                let changed = self.state.get("meldung").and_then(Value::as_str) != Some(e.as_str());
                self.state = json!({"phase": "fehler", "meldung": e});
                changed
            }
        }
    }

    fn s(&self, key: &str) -> &str {
        self.state.get(key).and_then(Value::as_str).unwrap_or("")
    }

    fn inv(&self, key: &str) -> Value {
        self.state.get("inventar").and_then(|i| i.get(key)).cloned().unwrap_or(Value::Null)
    }

    fn dry(&self) -> bool {
        self.state.get("trocken").and_then(Value::as_bool).unwrap_or(false)
    }

    fn backup_done(&self) -> bool {
        let b = self.state.get("sicherung").and_then(Value::as_str).unwrap_or("");
        b.starts_with("sicherung fertig") || b == "übersprungen"
    }

    fn replan(&mut self) {
        self.plan = match ask(json!({"tu": "plan", "android_gib": self.android_gib})) {
            Ok(v) if v.get("ok").and_then(Value::as_bool) == Some(true) => Ok(v["plan"].clone()),
            Ok(v) => Err(v.get("grund").and_then(Value::as_str).unwrap_or("Plan nicht möglich").to_string()),
            Err(e) => Err(e),
        };
        self.armed.clear();
    }

    /// A tap on one of the installer's roles; true = redraw.
    pub fn strike(&mut self, role: usize) -> bool {
        mk::feel::tap();
        // Recovery: the only taps are yes and no to the laptop's question.
        if self.s("phase") == "recovery" {
            if role == R_YES || role == R_NO {
                let _ = ask(json!({"tu": "antwort", "ja": role == R_YES}));
                self.refresh();
                return true;
            }
            return false;
        }
        match (self.page, role) {
            (Page::Welcome, R_NEXT) => {
                if self.inv("matrix_da").as_bool() != Some(true) {
                    self.page = Page::Backup;
                }
            }
            (Page::Backup, R_NEXT) if !self.asked_backup => {
                self.asked_backup = true;
                let _ = ask(json!({"tu": "sicherung", "ja": true}));
            }
            (Page::Backup, R_SKIP) if !self.asked_backup => {
                self.asked_backup = true;
                let _ = ask(json!({"tu": "sicherung", "ja": false}));
                self.page = Page::Split;
                self.replan();
            }
            (Page::Split, r) if (R_SPLIT..R_SPLIT + SPLITS.len()).contains(&r) => {
                self.android_gib = SPLITS[r - R_SPLIT];
                self.replan();
            }
            (Page::Split, R_BACK) => self.page = Page::Backup,
            (Page::Split, R_NEXT) if self.plan.is_ok() => self.page = Page::Confirm,
            (Page::Confirm, R_BACK) => {
                self.page = Page::Split;
                self.armed.clear();
            }
            (Page::Confirm, R_NEXT) => {
                if !self.armed.tap(()) {
                    mk::feel::warn();
                } else if let Ok(plan) = &self.plan {
                    let id = plan.get("id").cloned().unwrap_or(Value::Null);
                    match ask(json!({"tu": "installieren", "plan": id, "bestaetigt": CONFIRM})) {
                        Ok(v) if v.get("ok").and_then(Value::as_bool) == Some(true) => self.page = Page::Running,
                        Ok(v) => self.plan = Err(v.get("grund").and_then(Value::as_str).unwrap_or("abgelehnt").to_string()),
                        Err(e) => self.plan = Err(e),
                    }
                    self.armed.clear();
                }
            }
            (_, R_RESTART) => {
                let _ = ask(json!({"tu": "neustart"}));
            }
            _ => return false,
        }
        true
    }

    /// The page, centred in a column, in white on the wallpaper.
    pub fn lay(&self, b: &mut SceneBuilder, engine: &mut TextEngine, w: f32, h: f32, p: mk::Palette) {
        let white = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let soft = white.with_alpha(0.78);
        let col = 460f32.min(w - 2.0 * mk::spacing::L);
        let x0 = (w - col) / 2.0;
        // A dark veil: white words stay readable on any wallpaper (the falcon, 06.11.2026).
        b.quad(0.0, 0.0, w, h, mk::Rgba { r: 0.02, g: 0.03, b: 0.08, a: 0.62 }, 0.0, None, None);
        let mut y = (h * 0.07).max(mk::spacing::L);
        // Round 177: the kit's centred paragraph.
        let title = |b: &mut SceneBuilder, engine: &mut TextEngine, t: &str, y: f32| -> f32 { fam::lay_centred(b, engine, t, mk::typo::LARGE_TITLE, white, w / 2.0, col, y) };
        let para = |b: &mut SceneBuilder, engine: &mut TextEngine, t: &str, y: f32, ink: mk::Rgba| -> f32 { fam::lay_centred(b, engine, t, mk::typo::BODY, ink, w / 2.0, col, y) };
        // Morpheus on every page; the dry run said loudly.
        let (bw, bh) = engine.measure_styled("MORPHEUS", mk::typo::LABEL.px(), mk::typo::LABEL.weight);
        b.styled(engine, "MORPHEUS", mk::typo::LABEL, soft, (w - bw) / 2.0, y, None, None);
        y += bh + mk::spacing::S;
        if self.dry() {
            let t = l("Trockenlauf — es wird nichts geschrieben", "Dry run — nothing is written");
            let (tw, th) = engine.measure_styled(t, mk::typo::CAPTION.px(), mk::typo::CAPTION.weight);
            b.quad((w - tw) / 2.0 - 10.0, y - 3.0, tw + 20.0, th + 6.0, p.primary_container, (th + 6.0) / 2.0, None, None);
            b.styled(engine, t, mk::typo::CAPTION, p.on_primary_container, (w - tw) / 2.0, y, None, None);
            y += th + mk::spacing::M;
        }
        let phase = self.s("phase");
        if phase == "recovery" {
            self.lay_recovery(b, engine, w, y, white, soft, p);
            return;
        }
        let page = if matches!(phase, "installiert" | "fertig" | "fehler") && self.page != Page::Welcome { Page::Running } else { self.page };
        let mut buttons: Vec<(char, &str, usize, fam::PillTone)> = Vec::new();
        match page {
            Page::Welcome => {
                y += title(b, engine, l("Matrix installieren", "Install Matrix"), y) + mk::spacing::S;
                y += para(b, engine, l("Morpheus richtet Matrix neben Android ein. Android bleibt erhalten, wird aber zurückgesetzt.", "Morpheus sets up Matrix beside Android. Android stays, but is reset."), y, soft) + mk::spacing::L;
                let model = self.inv("modell").as_str().unwrap_or("?").to_string();
                let lines = [
                    model,
                    format!("{} {} GB · {} {} GB", l("Speicher", "Storage"), self.inv("speicher_gib"), l("Android jetzt", "Android now"), self.inv("android_gib")),
                    format!("Slot {} · {} {}", self.inv("slot").as_str().unwrap_or("?"), l("Seriennummer", "Serial number"), self.inv("seriennummer").as_str().unwrap_or("?")),
                ];
                for l in lines {
                    y += para(b, engine, &l, y, white) + 2.0;
                }
                y += mk::spacing::L;
                if self.inv("matrix_da").as_bool() == Some(true) {
                    y += para(b, engine, l("Auf diesem Duo ist Matrix schon installiert. Morpheus installiert nur auf neue Geräte.", "Matrix is already installed on this Duo. Morpheus only installs on new devices."), y, mk::Rgba { r: 1.0, g: 0.62, b: 0.60, a: 1.0 });
                } else if phase == "fehler" {
                    y += para(b, engine, self.s("meldung"), y, mk::Rgba { r: 1.0, g: 0.62, b: 0.60, a: 1.0 });
                } else {
                    buttons.push((symbol::ARROW_FORWARD, l("Los geht's", "Let's go"), R_NEXT, fam::PillTone::Accent));
                }
            }
            Page::Backup => {
                y += title(b, engine, l("Zuerst sichern?", "Back up first?"), y) + mk::spacing::S;
                y += para(b, engine, l("Eine Vollsicherung aller Partitionen auf den Laptop — empfohlen. Damit lässt sich das Duo jederzeit in den heutigen Zustand zurückbringen. Dauer etwa 25 Minuten, das Kabel bleibt gesteckt.", "A full backup of all partitions to the laptop — recommended. With it the Duo can always be brought back to how it is today. About 25 minutes; keep the cable plugged in."), y, soft) + mk::spacing::L;
                let b_state = self.state.get("sicherung").and_then(Value::as_str).unwrap_or("");
                if self.asked_backup {
                    let laptop = self.s("laptop");
                    let line = if b_state.starts_with("fehler") { b_state.to_string() } else if laptop.is_empty() { String::from(l("Warte auf den Laptop …", "Waiting for the laptop …")) } else { laptop.to_string() };
                    y += para(b, engine, &line, y, white);
                    if b_state.starts_with("fehler") {
                        buttons.push((symbol::ARROW_FORWARD, l("Ohne Sicherung weiter", "Continue without backup"), R_SKIP, fam::PillTone::Quiet));
                    }
                } else {
                    buttons.push((symbol::ARROW_FORWARD, l("Überspringen", "Skip"), R_SKIP, fam::PillTone::Quiet));
                    buttons.push((symbol::SAVE, l("Sichern", "Back up"), R_NEXT, fam::PillTone::Accent));
                }
            }
            Page::Split => {
                y += title(b, engine, l("Speicher aufteilen", "Split the storage"), y) + mk::spacing::S;
                y += para(b, engine, l("Wie viel Platz behält Android? Der Rest wird Matrix.", "How much room does Android keep? The rest becomes Matrix."), y, soft) + mk::spacing::L;
                let options: Vec<String> = SPLITS.iter().map(|g| format!("{g} GB")).collect();
                let active = SPLITS.iter().position(|g| *g == self.android_gib);
                y += fam::lay_choices(b, engine, &options, active, x0, y, col, R_SPLIT, p) + mk::spacing::L;
                match &self.plan {
                    Ok(plan) => {
                        let t = format!("Android {} GB · Matrix {} GB", plan["android_gib"], plan["matrix_gib"]);
                        y += para(b, engine, &t, y, white);
                        buttons.push((symbol::ARROW_BACK, l("Zurück", "Back"), R_BACK, fam::PillTone::Quiet));
                        buttons.push((symbol::ARROW_FORWARD, l("Weiter", "Continue"), R_NEXT, fam::PillTone::Accent));
                    }
                    Err(why) => {
                        y += para(b, engine, why, y, mk::Rgba { r: 1.0, g: 0.62, b: 0.60, a: 1.0 });
                        buttons.push((symbol::ARROW_BACK, l("Zurück", "Back"), R_BACK, fam::PillTone::Quiet));
                    }
                }
            }
            Page::Confirm => {
                y += title(b, engine, l("Bitte bestätigen", "Please confirm"), y) + mk::spacing::M;
                let plan = self.plan.as_ref().ok();
                let backup = self.state.get("sicherung").and_then(Value::as_str).unwrap_or("");
                let lines = [
                    String::from(l("• Android wird zurückgesetzt: alle Daten in Android werden gelöscht.", "• Android is reset: all data in Android is erased.")),
                    format!("{} {} GB, {} {} GB.", l("• Android behält", "• Android keeps"), plan.map(|p| p["android_gib"].to_string()).unwrap_or_default(), l("Matrix bekommt", "Matrix gets"), plan.map(|p| p["matrix_gib"].to_string()).unwrap_or_default()),
                    String::from(l("• Bootloader, Firmware und Kalibrierung bleiben unberührt.", "• Bootloader, firmware and calibration stay untouched.")),
                    if backup.starts_with("sicherung fertig") { String::from(l("• Die Vollsicherung liegt auf dem Laptop.", "• The full backup is on the laptop.")) } else { String::from(l("• Ohne Vollsicherung.", "• Without a full backup.")) },
                ];
                for l in lines {
                    y += para(b, engine, &l, y, white) + 4.0;
                }
                y += mk::spacing::M;
                if self.armed.is(&()) {
                    y += para(b, engine, l("Nochmals tippen: jetzt installieren.", "Tap again: install now."), y, mk::Rgba { r: 1.0, g: 0.75, b: 0.45, a: 1.0 });
                }
                buttons.push((symbol::ARROW_BACK, l("Zurück", "Back"), R_BACK, fam::PillTone::Quiet));
                buttons.push((symbol::ARROW_FORWARD, if self.armed.is(&()) { l("Wirklich installieren", "Really install") } else { l("Installieren", "Install") }, R_NEXT, fam::PillTone::Accent));
            }
            Page::Running => {
                let (t, line) = match phase {
                    "fertig" => (l("Matrix ist installiert", "Matrix is installed"), l("Starte das Duo neu. Danach in Morpheus auf dem Laptop «Matrix starten» — beim ersten Start richtest du dein Konto ein.", "Restart the Duo. Then, in Morpheus on the laptop, «Start Matrix» — on the first start you set up your account.")),
                    "fehler" => (l("Installation angehalten", "Installation stopped"), l("Es ist ein Fehler aufgetreten. Android und die Sicherung bleiben erhalten; der Laptop kann das Duo wieder starten.", "Something went wrong. Android and the backup stay; the laptop can start the Duo again.")),
                    _ => (l("Matrix wird installiert", "Installing Matrix"), l("Bitte das Kabel nicht ziehen und das Duo nicht ausschalten.", "Please do not unplug the cable or switch the Duo off.")),
                };
                y += title(b, engine, t, y) + mk::spacing::S;
                y += para(b, engine, line, y, soft) + mk::spacing::L;
                if let Some(steps) = self.state.get("schritte").and_then(Value::as_array) {
                    for s in steps {
                        let name = s.get("name").and_then(Value::as_str).unwrap_or("");
                        let st = s.get("stand").and_then(Value::as_str).unwrap_or("");
                        let mark = match st {
                            "fertig" => "✓",
                            "fehler" => "✗",
                            _ => "…",
                        };
                        let (_, lh) = b.styled(engine, format!("{mark}  {}", step_name(name)), mk::typo::BODY, white, x0, y, None, None);
                        y += lh + 4.0;
                    }
                }
                y += mk::spacing::S;
                let msg = self.s("meldung");
                if !msg.is_empty() {
                    let ink = if phase == "fehler" { mk::Rgba { r: 1.0, g: 0.62, b: 0.60, a: 1.0 } } else { soft };
                    y += para(b, engine, msg, y, ink);
                }
                let laptop = self.s("laptop");
                if phase == "installiert" && !laptop.is_empty() {
                    y += para(b, engine, laptop, y, soft);
                }
                if phase == "fertig" || phase == "fehler" {
                    buttons.push((symbol::RESTART, l("Neu starten", "Restart"), R_RESTART, fam::PillTone::Accent));
                }
            }
        }
        // The buttons in one row under the page.
        if !buttons.is_empty() {
            y += mk::spacing::L;
            let row: Vec<fam::PillSpec> = buttons.iter().map(|(g, t, role, tone)| (Some(*g), *t, Some(*role), *tone)).collect();
            fam::lay_pill_row(b, engine, &row, w / 2.0, y, mk::spacing::M, p);
        }
        let _ = Rect { x: x0, y, w: col, h: 0.0 };
    }

    /// MATRIX RECOVERY: Matrix is already on this Duo. Morpheus on the laptop
    /// does the work («like iTunes»); here: what happens, and the one question
    /// before anything is erased or replaced.
    #[allow(clippy::too_many_arguments)]
    fn lay_recovery(&self, b: &mut SceneBuilder, engine: &mut TextEngine, w: f32, top: f32, white: mk::Rgba, soft: mk::Rgba, p: mk::Palette) {
        let col = 460f32.min(w - 2.0 * mk::spacing::L);
        let centred = |b: &mut SceneBuilder, engine: &mut TextEngine, t: &str, st: mk::typo::TextStyle, ink: mk::Rgba, y: f32| -> f32 { fam::lay_centred(b, engine, t, st, ink, w / 2.0, col, y) };
        let mut y = top;
        y += centred(b, engine, "Matrix Recovery", mk::typo::LARGE_TITLE, white, y) + mk::spacing::S;
        y += centred(b, engine, l("Matrix ist auf diesem Duo installiert. Bediene Morpheus am Laptop: Matrix starten, sichern, wiederherstellen oder neu installieren.", "Matrix is installed on this Duo. Use Morpheus on the laptop: start, back up, restore or reinstall Matrix."), mk::typo::BODY, soft, y) + mk::spacing::L;
        let model = self.inv("modell").as_str().unwrap_or("?").to_string();
        let serial = self.inv("seriennummer").as_str().unwrap_or("?").to_string();
        y += centred(b, engine, &format!("{model} · {} {serial}", l("Seriennummer", "Serial number")), mk::typo::BODY, white, y) + mk::spacing::L;
        // What the laptop is doing (the last lines of morpheus-recovery).
        if let Some(lines) = self.state.get("recovery").and_then(Value::as_array) {
            for line in lines.iter().filter_map(Value::as_str).rev().take(4).collect::<Vec<_>>().into_iter().rev() {
                let ink = if line.starts_with("FEHLER") { mk::Rgba { r: 1.0, g: 0.62, b: 0.60, a: 1.0 } } else { soft };
                y += centred(b, engine, line, mk::typo::CAPTION, ink, y) + 2.0;
            }
            y += mk::spacing::M;
        }
        // The one question.
        let question = self.s("frage");
        if !question.is_empty() {
            let pad = mk::spacing::M;
            // The card's padding lies above the question: room for it, so it never covers the lines above.
            y += pad;
            let text_h = mk::wrap_words(question, ((col / (mk::typo::BODY.px() * 0.5)) as usize).max(20), 4).len() as f32 * mk::typo::BODY.px() * 1.3;
            b.quad((w - col) / 2.0 - pad, y - pad, col + 2.0 * pad, text_h + 2.0 * pad + 56.0, mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.10 }, 18.0, None, None);
            y += centred(b, engine, question, mk::typo::BODY, white, y) + mk::spacing::M;
            fam::lay_pill_row(
                b,
                engine,
                &[(Some(symbol::CLOSE), l("Nein", "No"), Some(R_NO), fam::PillTone::Quiet), (Some(symbol::CHECK), l("Ja, fortfahren", "Yes, continue"), Some(R_YES), fam::PillTone::Accent)],
                w / 2.0,
                y,
                mk::spacing::M,
                p,
            );
        }
    }
}
