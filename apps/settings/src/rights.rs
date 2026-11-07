//! "Benutzer und Rechte" (users and rights) + "Apps und Rechte" (apps and rights) (R139).
//!
//! v1 is the honest overview: WHO exists on this system (accounts,
//! admin badge from the wheel group) and WHAT each MatrixKit app has
//! been granted (the binding permission register under
//! ~/.config/matrix/permissions). Changing grants stays where it is
//! decided today — in the app's root panel — and the footer says so.

use matrixkit_widgets::iced;
use matrixkit_widgets::iced::Element;
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;

fn wheel_members() -> Vec<String> {
    std::fs::read_to_string("/etc/group")
        .ok()
        .and_then(|g| {
            g.lines().find(|l| l.starts_with("wheel:")).map(|l| {
                l.rsplit(':')
                    .next()
                    .unwrap_or("")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
        })
        .unwrap_or_default()
}

/// Real accounts: uid 1000…59999 out of /etc/passwd.
fn accounts() -> Vec<(String, u32, String)> {
    let Ok(passwd) = std::fs::read_to_string("/etc/passwd") else {
        return Vec::new();
    };
    passwd
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split(':').collect();
            let uid: u32 = f.get(2)?.parse().ok()?;
            (1000..60000).contains(&uid).then(|| {
                (
                    f.first().unwrap_or(&"?").to_string(),
                    uid,
                    f.get(6).unwrap_or(&"").to_string(),
                )
            })
        })
        .collect()
}

pub fn users_view<M: Clone + 'static>(p: mk::Palette) -> Element<'static, M> {
    let admins = wheel_members();
    let rows: Vec<Element<'static, M>> = accounts()
        .into_iter()
        .map(|(name, uid, shell)| {
            let badge = if admins.contains(&name) {
                "Administrator (wheel)"
            } else if shell.contains("nologin") || shell.contains("false") {
                "Dienstkonto"
            } else {
                "Standard"
            };
            mkw::row_item(
                format!("{name}  ·  UID {uid}").leak_to_row(),
                Some(badge),
                None,
                None,
                p,
            )
        })
        .collect();
    iced::widget::column![
        mkw::section("KONTEN AUF DIESEM SYSTEM", rows, p),
        iced::widget::Space::new().height(mk::spacing::L),
        mkw::section(
            "VERWALTUNG",
            vec![mkw::row_item(
                "Konten anlegen und Passwörter ändern",
                Some("Morpheus (Umzug) oder Terminal — Passwörter tippst immer du"),
                None,
                None,
                p,
            )],
            p,
        ),
    ]
    .spacing(0)
    .into()
}

/// Every app that ever asked for something, with its grants.
pub fn apps_view<M: Clone + 'static>(p: mk::Palette) -> Element<'static, M> {
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/root"));
    let dir = format!("{home}/.config/matrix/permissions");
    let mut rows: Vec<Element<'static, M>> = Vec::new();
    let mut apps: Vec<(String, String)> = std::fs::read_dir(&dir)
        .map(|rd| {
            rd.flatten()
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let grants = std::fs::read_to_string(e.path()).unwrap_or_default();
                    let list: Vec<&str> =
                        grants.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
                    Some((name, if list.is_empty() {
                        String::from("keine Rechte erteilt")
                    } else {
                        list.join(", ")
                    }))
                })
                .collect()
        })
        .unwrap_or_default();
    apps.sort();
    for (app, grants) in apps {
        rows.push(mkw::row_item(app.leak_to_row(), Some(grants.leak_to_row()), None, None, p));
    }
    if rows.is_empty() {
        rows.push(mkw::row_item(
            "Noch keine Einträge",
            Some("Apps erscheinen hier, sobald sie erstmals um ein Recht bitten"),
            None,
            None,
            p,
        ));
    }
    iced::widget::column![
        mkw::section("ERTEILTE RECHTE JE APP", rows, p),
        iced::widget::Space::new().height(mk::spacing::L),
        mkw::section(
            "GRUNDSATZ",
            vec![mkw::row_item(
                "Rechte vergibt die App selbst beim ersten Bedarf",
                Some("Entzug: Eintrag hier löschen — die App fragt beim nächsten Mal neu"),
                None,
                None,
                p,
            )],
            p,
        ),
    ]
    .spacing(0)
    .into()
}

/// row_item wants &'static str for the title — these overview rows are
/// few and rebuilt rarely; a deliberate, documented leak per render
/// would be a trapdoor, so we go through a small interner instead.
trait LeakToRow {
    fn leak_to_row(self) -> &'static str;
}
impl LeakToRow for String {
    fn leak_to_row(self) -> &'static str {
        use std::collections::BTreeSet;
        use std::sync::Mutex;
        static INTERN: Mutex<BTreeSet<&'static str>> = Mutex::new(BTreeSet::new());
        let mut set = INTERN.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(existing) = set.iter().find(|s| **s == self) {
            existing
        } else {
            let leaked: &'static str = Box::leak(self.into_boxed_str());
            set.insert(leaked);
            leaked
        }
    }
}
