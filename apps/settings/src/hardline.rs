//! "Hardline" (R182) — the keyring gets a face.
//!
//! the owner'ss question on 04.08.2026: "How does the user see where and which
//! key is active? How does he delete it, how does he get a
//! new one? Where is all of this?" The honest answer was: nowhere —
//! the identity lay in `~/.local/share/hardline` and was only visible through
//! the terminal. This area is the answer.
//!
//! Two limits this area keeps:
//! - **Secrets are never shown.** The node delivers
//!   fingerprints; they are there for COMPARING (screen against
//!   screen), not for typing off. A private key appears
//!   on no pixel.
//! - **Destructive actions ask for the password.** A new identity
//!   and deleting are irreversible: the address is gone, and
//!   every petname someone else gave you points
//!   into the void afterwards. That is stated as a sentence before you type.

use matrixkit_widgets::iced;
use matrixkit_widgets::iced::widget::{column, row, Space};
use matrixkit_widgets::iced::{Element, Length};
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;

#[derive(Debug, Clone)]
pub enum Msg {
    Reload,
    /// R184: the consent — the one act that creates an address.
    Init,
    Policy(&'static str),
    Reach(&'static str),
    /// R187: the folder's visibility in Matrix Dateien — a SEPARATE
    /// decision from switching the service off.
    Visible(bool),
    Drop(String),
    /// Which dangerous act is being confirmed — None = none.
    Ask(Option<Act>),
    Password(String),
    Confirm,
    /// The anchor goes to a removable medium; the path is typed.
    AnchorPath(String),
    MoveAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    Fresh,
    Delete,
}

impl Act {
    fn title(self) -> &'static str {
        match self {
            Act::Fresh => "Neue Identität erzeugen",
            Act::Delete => "Identität löschen",
        }
    }
    /// The consequence, spelled out. This text IS the safeguard.
    fn warning(self) -> &'static str {
        match self {
            Act::Fresh => "Dieses Gerät bekommt eine neue Adresse. Die alte ist danach unwiderruflich weg: Wer dir einen Namen gegeben hat, erreicht dich damit nicht mehr, und Inhalte, die auf deine alte Adresse verweisen, zeigen ins Leere. Dein Regal bleibt erhalten.",
            Act::Delete => "Alle Schlüssel dieses Geräts werden gelöscht. Es ist danach nicht mehr im Hardline-Netz erreichbar, bis eine neue Identität erzeugt wird. Das Regal bleibt erhalten.",
        }
    }
}

#[derive(Default)]
pub struct Hardline {
    facts: Vec<(String, String)>,
    /// The AUSLAGE lines, tab-split. Empty when the node is away.
    showcase: Vec<Vec<String>>,
    /// True once KEYS answered — false means: no identity yet.
    identity: bool,
    /// R187: does ~/Hardline show up in Matrix Dateien?
    folder_visible: bool,
    /// None when the node is not running — then we say so instead of
    /// showing an empty page.
    reachable: bool,
    asking: Option<Act>,
    password: String,
    anchor_path: String,
    notice: Option<(bool, String)>,
}

impl Hardline {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let visible = std::fs::read_to_string(format!("{home}/.config/matrix/hardline-folder"))
            .map(|raw| raw.trim() != "aus")
            .unwrap_or(true);
        let mut h = Self {
            anchor_path: String::from("/run/media/"),
            folder_visible: visible,
            ..Default::default()
        };
        h.reload();
        h
    }

    fn reload(&mut self) {
        match node_ask("AUSLAGE") {
            Ok(text) => {
                self.reachable = true;
                self.showcase = text
                    .lines()
                    .map(|l| l.split('\t').map(str::to_string).collect())
                    .collect();
            }
            Err(_) => {
                self.reachable = false;
                self.showcase.clear();
                self.facts.clear();
                self.identity = false;
                return;
            }
        }
        match node_ask("KEYS") {
            Ok(text) => {
                self.identity = true;
                self.facts = text
                    .lines()
                    .filter_map(|l| l.split_once('\t'))
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
            }
            Err(_) => {
                // The node runs but no identity exists — the consent
                // view is the whole page then.
                self.identity = false;
                self.facts.clear();
            }
        }
    }

    fn showcase_get(&self, key: &str) -> Option<&Vec<String>> {
        self.showcase.iter().find(|row| row.first().map(String::as_str) == Some(key))
    }

    fn fact(&self, key: &str) -> String {
        self.facts
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    }

    fn store_dir(&self) -> String {
        let f = self.fact("ablage");
        if f.is_empty() {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{home}/Hardline")
        } else {
            f
        }
    }

    /// R185: the keys live in their 0700 drawer inside ~/Hardline.
    fn keys_dir(&self) -> String {
        format!("{}/Schlüssel", self.store_dir())
    }

    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Reload => self.reload(),
            Msg::Init => {
                self.notice = Some(match node_ask("INIT") {
                    Ok(reply) => {
                        let addr = reply.split('\t').nth(1).unwrap_or("").trim().to_string();
                        (true, format!("Identität erzeugt. Deine Adresse: {addr}"))
                    }
                    Err(reason) => (false, reason),
                });
                self.reload();
            }
            Msg::Visible(show) => {
                // The contract lives where all Matrix contracts live.
                let home = std::env::var("HOME").unwrap_or_default();
                let path = format!("{home}/.config/matrix/hardline-folder");
                let _ = std::fs::create_dir_all(format!("{home}/.config/matrix"));
                let _ = std::fs::write(&path, if show { "an" } else { "aus" });
                self.folder_visible = show;
                self.notice = Some((
                    true,
                    if show {
                        String::from("Der Ordner erscheint wieder in Matrix Dateien.")
                    } else {
                        String::from("Der Ordner ist in Matrix Dateien ausgeblendet — die Dateien bleiben, wo sie sind.")
                    },
                ));
            }
            Msg::Reach(word) => {
                self.notice = match node_ask(&format!("REACH {word}")) {
                    Ok(_) => None,
                    Err(reason) => Some((false, reason)),
                };
                self.reload();
            }
            Msg::Policy(word) => {
                self.notice = match node_ask(&format!("POLICY {word}")) {
                    Ok(_) => None,
                    Err(reason) => Some((false, reason)),
                };
                self.reload();
            }
            Msg::Drop(uri) => {
                self.notice = Some(match node_ask(&format!("DROP {uri}")) {
                    Ok(_) => (true, String::from("Aus dem Regal genommen.")),
                    Err(reason) => (false, reason),
                });
                self.reload();
            }
            Msg::Ask(act) => {
                self.asking = act;
                self.password.clear();
                self.notice = None;
            }
            Msg::Password(p) => self.password = p,
            Msg::AnchorPath(p) => self.anchor_path = p,
            Msg::Confirm => {
                let Some(act) = self.asking else { return };
                if !mk::permissions::password_verify(&self.password) {
                    self.notice = Some((false, String::from("Passwort stimmt nicht.")));
                    self.password.clear();
                    return;
                }
                self.password.clear();
                self.asking = None;
                let dir = self.keys_dir();
                // The node holds the OLD identity in memory until it
                // restarts — say so rather than pretend otherwise.
                let keys = ["main.key", "device.key", "device.cert", "kem.seed", "revocation.cert"];
                for k in keys {
                    let _ = std::fs::remove_file(format!("{dir}/{k}"));
                }
                self.notice = Some(match act {
                    Act::Fresh => (
                        true,
                        String::from(
                            "Alte Schlüssel entfernt. Die neue Identität entsteht beim nächsten Start des Knotens — bis dahin gilt noch die alte.",
                        ),
                    ),
                    Act::Delete => (
                        true,
                        String::from(
                            "Schlüssel gelöscht. Der Knoten spricht noch mit der alten Identität, bis er neu startet.",
                        ),
                    ),
                });
                self.reload();
            }
            Msg::MoveAnchor => {
                let dir = self.keys_dir();
                let target = self.anchor_path.trim().to_string();
                if target.is_empty() {
                    return;
                }
                let src = format!("{dir}/main.key");
                if !std::path::Path::new(&src).exists() {
                    self.notice = Some((false, String::from("Der Anker liegt nicht (mehr) hier.")));
                    return;
                }
                // Copy first, verify, and only THEN remove: a move
                // that half-succeeds would destroy the anchor.
                let dest = format!("{}/main.key", target.trim_end_matches('/'));
                self.notice = Some(match std::fs::copy(&src, &dest) {
                    Ok(_) => match (std::fs::read(&src), std::fs::read(&dest)) {
                        (Ok(a), Ok(b)) if a == b => {
                            let _ = std::fs::remove_file(&src);
                            (true, format!("Anker liegt jetzt auf {dest} — und nur dort."))
                        }
                        _ => (false, String::from("Kopie stimmt nicht überein — hier nichts gelöscht.")),
                    },
                    Err(e) => (false, format!("Auslagern fehlgeschlagen: {e}")),
                });
                self.reload();
            }
        }
    }

    pub fn view(&self, p: mk::Palette) -> Element<'_, Msg> {
        let mut body = column![].spacing(mk::spacing::M).width(Length::Fill);

        if !self.reachable {
            body = body.push(mkw::notice(
                false,
                String::from(
                    "Der Hardline-Knoten läuft nicht. Ohne ihn ist dieses Gerät nicht erreichbar — und dieser Bereich weiß nichts.",
                ),
                p,
            ));
            body = body.push(mkw::text_button(mkw::txt("Erneut nachsehen", mk::typo::BODY, p.primary), Msg::Reload));
            return body.into();
        }

        // --- R184: no identity = the consent view IS the page -------
        if !self.identity {
            let mut consent = column![].spacing(mk::spacing::XS);
            consent = consent.push(mkw::txt(
                "Dieses Gerät ist im Hardline-Netz nicht vorhanden",
                mk::typo::SUBTITLE,
                p.on_surface,
            ));
            consent = consent.push(mkw::txt(
                "Es gibt keine Adresse und keine Schlüssel, und der Netz-Anschluss ist geschlossen. Nichts wird geteilt.",
                mk::typo::BODY,
                p.on_surface_variant,
            ));
            consent = consent.push(Space::new().height(mk::spacing::S));
            consent = consent.push(mkw::txt(
                "Mit deiner Zustimmung entsteht ein Schlüsselpaar und daraus eine Adresse. Sie ist ein Pseudonym: für Fremde eine Nummer ohne Namen, die niemand finden oder erraten kann. Einen Namen hat sie nur für Menschen, denen du sie selbst gibst. Alles, was dein Gerät spricht, ist Ende-zu-Ende verschlüsselt.",
                mk::typo::BODY,
                p.on_surface,
            ));
            consent = consent.push(Space::new().height(mk::spacing::S));
            consent = consent.push(mkw::text_button(mkw::txt("Ich stimme zu — Identität erzeugen", mk::typo::BODY, p.primary), Msg::Init));
            body = body.push(mkw::tile_chrome(consent.into(), p));
            if let Some((ok, text)) = &self.notice {
                body = body.push(mkw::notice(*ok, text.clone(), p));
            }
            return body.into();
        }

        // --- R186: the connection — reachable at all? --------------
        {
            let on = self
                .showcase_get("erreichbar")
                .and_then(|r| r.get(1))
                .map(|v| v == "an")
                .unwrap_or(false);
            let mut conn = column![].spacing(mk::spacing::XS);
            conn = conn.push(mkw::txt("Erreichbarkeit", mk::typo::SUBTITLE, p.on_surface));
            conn = conn.push(mkw::txt(
                if on {
                    "An — dein Gerät lauscht im Netz und kann angesprochen werden. Wer etwas bekommt, entscheidet die Sichtbarkeit unten."
                } else {
                    "Aus — dein Gerät lauscht nicht. Niemand kann es ansprechen oder sich einen Weg hinein bahnen. Du kannst trotzdem Seiten anderer abrufen."
                },
                mk::typo::BODY,
                p.on_surface,
            ));
            let mut sw = row![].spacing(mk::spacing::S).align_y(iced::Alignment::Center);
            for (word, label) in [("aus", "Aus"), ("an", "An")] {
                let active = on == (word == "an");
                sw = sw.push(mkw::text_button(mkw::txt(
                        label,
                        mk::typo::BODY,
                        if active { p.primary } else { p.on_surface_variant },
                    ), Msg::Reach(word)));
            }
            conn = conn.push(sw);
            conn = conn.push(mkw::txt(
                "Hardline zu benutzen heisst nicht, erreichbar zu sein — das ist eine eigene Entscheidung.",
                mk::typo::CAPTION,
                p.on_surface_variant,
            ));
            body = body.push(mkw::tile_chrome(conn.into(), p));
        }

        // --- R187: does the folder show in Matrix Dateien? ---------
        {
            let mut fold = column![].spacing(mk::spacing::XS);
            fold = fold.push(mkw::txt("Ordner in Matrix Dateien", mk::typo::SUBTITLE, p.on_surface));
            fold = fold.push(mkw::txt(
                if self.folder_visible {
                    "Sichtbar — „Hardline“ steht in der Seitenleiste, hinter deinem Passwort."
                } else {
                    "Ausgeblendet — der Ordner existiert weiter, Matrix Dateien zeigt ihn nur nicht."
                },
                mk::typo::BODY,
                p.on_surface,
            ));
            let mut sw = row![].spacing(mk::spacing::S).align_y(iced::Alignment::Center);
            for (val, label) in [(true, "Sichtbar"), (false, "Ausgeblendet")] {
                let active = self.folder_visible == val;
                sw = sw.push(mkw::text_button(mkw::txt(
                        label,
                        mk::typo::BODY,
                        if active { p.primary } else { p.on_surface_variant },
                    ), Msg::Visible(val)));
            }
            fold = fold.push(sw);
            fold = fold.push(mkw::txt(
                "Hardline im Dock auszuschalten blendet hier nichts aus — Dienst und Ordner sind zwei Entscheidungen.",
                mk::typo::CAPTION,
                p.on_surface_variant,
            ));
            body = body.push(mkw::tile_chrome(fold.into(), p));
        }

        // --- R184: the Auslage — what this device shows ------------
        {
            let policy = self
                .showcase_get("politik")
                .and_then(|r| r.get(1))
                .cloned()
                .unwrap_or_else(|| String::from("bekannte"));
            let peers = self
                .showcase_get("gegenstellen")
                .and_then(|r| r.get(1))
                .cloned()
                .unwrap_or_default();
            let mut show = column![].spacing(mk::spacing::XS);
            show = show.push(mkw::txt("Was dieses Gerät zeigt", mk::typo::SUBTITLE, p.on_surface));
            show = show.push(mkw::txt(
                match policy.as_str() {
                    "niemand" => String::from("Sichtbar für: niemanden — dein Gerät beantwortet keine Anfragen."),
                    "alle" => String::from("Sichtbar für: jeden, der deine Adresse hat."),
                    _ => format!("Sichtbar für: nur bekannte Geräte ({peers}). Niemand kann dich finden — deine Adresse hat nur, wem du sie gegeben hast."),
                },
                mk::typo::BODY,
                p.on_surface,
            ));
            let mut pol = row![].spacing(mk::spacing::S).align_y(iced::Alignment::Center);
            for (word, label) in [
                ("niemand", "Niemand"),
                ("bekannte", "Bekannte"),
                ("alle", "Alle"),
            ] {
                let active = policy == word;
                pol = pol.push(mkw::text_button(mkw::txt(
                        label,
                        mk::typo::BODY,
                        if active { p.primary } else { p.on_surface_variant },
                    ), Msg::Policy(word)));
            }
            show = show.push(pol);
            show = show.push(Space::new().height(mk::spacing::XS));
            if let Some(row) = self.showcase_get("seite") {
                show = show.push(mkw::txt(
                    if row.get(1).map(String::as_str) == Some("ja") {
                        format!(
                            "Deine Seite: „{}“ ({} Zeichen)",
                            row.get(3).cloned().unwrap_or_default(),
                            row.get(2).cloned().unwrap_or_default()
                        )
                    } else {
                        String::from("Deine Seite: keine")
                    },
                    mk::typo::BODY,
                    p.on_surface,
                ));
            }
            let objects: Vec<&Vec<String>> = self
                .showcase
                .iter()
                .filter(|r| r.first().map(String::as_str) == Some("objekt"))
                .collect();
            show = show.push(mkw::txt(
                format!("Im Regal: {} Objekt(e) — holen kann sie nur, wer den Verweis hat.", objects.len()),
                mk::typo::BODY,
                p.on_surface,
            ));
            for obj in objects {
                let uri = obj.get(1).cloned().unwrap_or_default();
                let len = obj.get(2).cloned().unwrap_or_default();
                let name = obj.get(3).cloned().unwrap_or_else(|| {
                    if uri.len() > 24 { format!("{}…", &uri[..24]) } else { uri.clone() }
                });
                show = show.push(
                    row![
                        mkw::txt(format!("{name}  ({len} B)"), mk::typo::CAPTION, p.on_surface_variant),
                        Space::new().width(mk::spacing::S),
                        mkw::text_button(mkw::txt("Entfernen", mk::typo::CAPTION, p.error), Msg::Drop(uri.clone())),
                    ]
                    .align_y(iced::Alignment::Center),
                );
            }
            show = show.push(Space::new().height(mk::spacing::XS));
            show = show.push(mkw::txt(
                "Nie geteilt: Petnamen, Bekanntenliste, Verlauf, Schlüssel.",
                mk::typo::CAPTION,
                p.on_surface_variant,
            ));
            body = body.push(mkw::tile_chrome(show.into(), p));
        }

        // --- Who am I ------------------------------------------------
        let mut who = column![].spacing(mk::spacing::XS);
        who = who.push(mkw::txt("Deine Adresse", mk::typo::CAPTION, p.on_surface_variant));
        who = who.push(mkw::txt(self.fact("adresse"), mk::typo::BODY, p.on_surface));
        who = who.push(Space::new().height(mk::spacing::XS));
        who = who.push(mkw::txt(
            "Sie ist der Fingerabdruck deines Ankers — niemand kann sie sich ausdenken, und sie verrät nichts über dich.",
            mk::typo::CAPTION,
            p.on_surface_variant,
        ));
        body = body.push(mkw::tile_chrome(who.into(), p));

        // --- The two keys, as fingerprints --------------------------
        let short = |s: String| -> String {
            if s.len() > 16 {
                format!("{}…{}", &s[..8], &s[s.len() - 8..])
            } else {
                s
            }
        };
        let mut keys = column![].spacing(mk::spacing::XS);
        keys = keys.push(mkw::txt("Schlüssel", mk::typo::SUBTITLE, p.on_surface));
        keys = keys.push(mkw::txt(
            format!("Anker    {}", short(self.fact("anker"))),
            mk::typo::BODY,
            p.on_surface,
        ));
        keys = keys.push(mkw::txt(
            format!("Gerät    {}", short(self.fact("geraet"))),
            mk::typo::BODY,
            p.on_surface,
        ));
        keys = keys.push(mkw::txt(
            "Der Anker bürgt für den Gerätschlüssel. Gesprochen wird immer mit dem Gerätschlüssel — der Anker existiert nur, um zu bürgen und zu widerrufen.",
            mk::typo::CAPTION,
            p.on_surface_variant,
        ));
        body = body.push(mkw::tile_chrome(keys.into(), p));

        // --- Where the anchor lives ---------------------------------
        let anchor_here = self.fact("anker-hier") == "ja";
        let mut anchor = column![].spacing(mk::spacing::XS);
        anchor = anchor.push(mkw::txt("Wo der Anker liegt", mk::typo::SUBTITLE, p.on_surface));
        anchor = anchor.push(mkw::txt(
            if anchor_here {
                "Auf diesem Gerät — neben dem Gerätschlüssel. Wer die Platte hat, hat beides."
            } else {
                "Nicht auf diesem Gerät. Genau so ist es gedacht: Der Anker wird nur gebraucht, um zu bürgen oder zu widerrufen."
            },
            mk::typo::BODY,
            if anchor_here { p.on_surface } else { p.on_surface_variant },
        ));
        if anchor_here {
            anchor = anchor.push(Space::new().height(mk::spacing::XS));
            anchor = anchor.push(row![
                mkw::input_field(
                    "/run/media/stick",
                    &self.anchor_path,
                    Msg::AnchorPath,
                    Some(Msg::MoveAnchor),
                    false,
                    p
                ),
                Space::new().width(mk::spacing::S),
                mkw::text_button(mkw::txt("Anker auslagern", mk::typo::BODY, p.primary), Msg::MoveAnchor),
            ]
            .align_y(iced::Alignment::Center));
        }
        body = body.push(mkw::tile_chrome(anchor.into(), p));

        // --- The emergency brake ------------------------------------
        let mut revoke = column![].spacing(mk::spacing::XS);
        revoke = revoke.push(mkw::txt("Notbremse", mk::typo::SUBTITLE, p.on_surface));
        revoke = revoke.push(mkw::txt(
            if self.fact("widerruf") == "ja" {
                "Ein Widerrufs-Zertifikat liegt bereit. Es wurde bei der Geburt dieser Identität erzeugt und erklärt sie für ungültig, falls dir jemand die Schlüssel stiehlt."
            } else {
                "Kein Widerrufs-Zertifikat vorhanden."
            },
            mk::typo::BODY,
            p.on_surface_variant,
        ));
        body = body.push(mkw::tile_chrome(revoke.into(), p));

        // --- Dangerous acts -----------------------------------------
        if let Some(act) = self.asking {
            let mut ask = column![].spacing(mk::spacing::XS);
            ask = ask.push(mkw::txt(act.title(), mk::typo::SUBTITLE, p.error));
            ask = ask.push(mkw::txt(act.warning(), mk::typo::BODY, p.on_surface));
            ask = ask.push(Space::new().height(mk::spacing::XS));
            ask = ask.push(row![
                mkw::input_field(
                    "Passwort zum Bestätigen",
                    &self.password,
                    Msg::Password,
                    Some(Msg::Confirm),
                    true,
                    p
                ),
                Space::new().width(mk::spacing::S),
                mkw::text_button(mkw::txt("Bestätigen", mk::typo::BODY, p.error), Msg::Confirm),
                Space::new().width(mk::spacing::S),
                mkw::text_button(mkw::txt("Abbrechen", mk::typo::BODY, p.on_surface_variant), Msg::Ask(None)),
            ]
            .align_y(iced::Alignment::Center));
            body = body.push(mkw::tile_chrome(ask.into(), p));
        } else {
            body = body.push(
                row![
                    mkw::text_button(mkw::txt("Neue Identität …", mk::typo::BODY, p.on_surface), Msg::Ask(Some(Act::Fresh))),
                    Space::new().width(mk::spacing::L),
                    mkw::text_button(mkw::txt("Identität löschen …", mk::typo::BODY, p.error), Msg::Ask(Some(Act::Delete))),
                ]
                .align_y(iced::Alignment::Center),
            );
        }

        if let Some((ok, text)) = &self.notice {
            body = body.push(mkw::notice(*ok, text.clone(), p));
        }
        body.into()
    }

    pub fn footer_text(&self) -> String {
        if self.reachable {
            format!("Schlüssel liegen in {} — Geheimnisse verlassen diese Ablage nie", self.store_dir())
        } else {
            String::from("Knoten nicht erreichbar — hardline-node serve startet ihn")
        }
    }
}

/// One line-request to the node, answer as text.
fn node_ask(request: &str) -> Result<String, String> {
    let path = match std::env::var("XDG_RUNTIME_DIR") {
        Ok(dir) => format!("{dir}/hardline-node.sock"),
        Err(_) => {
            let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
            format!("/tmp/hardline-node-{user}.sock")
        }
    };
    let mut stream = UnixStream::connect(path).map_err(|_| String::from("Knoten läuft nicht"))?;
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
    stream
        .write_all(format!("{request}\n").as_bytes())
        .map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    let line = line.trim_end();
    if let Some(reason) = line.strip_prefix("MISS ") {
        return Err(reason.to_string());
    }
    let Some(len) = line.strip_prefix("OK ").and_then(|n| n.parse::<usize>().ok()) else {
        return Err(String::from("unerwartete Antwort"));
    };
    let mut data = vec![0u8; len.min(64 * 1024)];
    reader.read_exact(&mut data).map_err(|e| e.to_string())?;
    String::from_utf8(data).map_err(|_| String::from("unlesbar"))
}
