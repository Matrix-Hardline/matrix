//! The family police (R47): consistency is a property of the build,
//! not a virtue. This test scans the app sources for patterns that
//! build past the MatrixKit families — and turns red when someone
//! (including the AI) does it again.
//!
//! Exceptions are allowed, but only DECLARED: a line
//! `// family-exception: <Grund>` directly above the finding lets it
//! pass. Deviation thus remains a conscious decision with a reason
//! in the code — never silent sprawl again.

use std::path::{Path, PathBuf};

struct Rule {
    name: &'static str,
    /// Substring OR (with `regex_zahl`) hand-rolled pattern matching.
    muster: &'static str,
    family: &'static str,
}

const RULES: &[Rule] = &[
    Rule {
        name: "rohes text_input",
        muster: "text_input(",
        family: "mkw::eingabefeld / mkw::suchfeld",
    },
    Rule {
        name: "handgerollter Knopf-Stil",
        muster: "button::Style {",
        family: "mkw::knopf (Stil x Rolle x Groesse) / mkw::ui::werkzeug_knopf",
    },
    Rule {
        name: "Mono-Schrift-Literal",
        muster: "Font::with_name(\"Adwaita Mono",
        family: "mkw::mono() / mkw::load_mono_font()",
    },
    Rule {
        name: "rohes pick_list",
        muster: "pick_list(",
        family: "mkw::select_menu / mkw::select_menu_prompt",
    },
    Rule {
        name: "handgerollter Fortschritt",
        muster: "progress_bar(",
        family: "mkw::progress",
    },
    Rule {
        name: "roher Editor-Stil",
        muster: "text_editor::Style {",
        family: "mkw::editor_style",
    },
    Rule {
        name: "rohes radio",
        muster: " radio(",
        family: "mkw::radio_item / mkw::row_radio",
    },
    Rule {
        name: "rohes checkbox",
        muster: "checkbox(",
        family: "mkw::check_item",
    },
    Rule {
        name: "999er-Pseudo-Kapsel",
        muster: "radius: 999",
        family: "mk::radius::capsule(height)",
    },
    // R88: rigid column counts in views — grids breathe with the
    // window. Non-UI chunks (crypto, bytes) declare the exception.
    Rule {
        name: "starres chunks-Gitter",
        muster: ".chunks(",
        family: "mkw::ui::raster (R88) / raster_spalten (R87)",
    },
    // R88: color literals bypass the living palette.
    Rule {
        name: "Farb-Literal",
        muster: "Color::from_rgb",
        family: "mk::Palette-Rollen (p.primary, p.on_surface, ...)",
    },
    // R134 (trapdoor police): a silently swallowed error froze the
    // Zentrale for two days — the layershell runtime dropped Outdated
    // presents into an uninitialized tracing sink. Every dropped Err
    // needs a declared reason.
    Rule {
        name: "stumme Falltuer",
        muster: "Err(_) => {}",
        family: "handle it, log it, or declare a family-exception",
    },
];

fn apps_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../apps")
}

fn rs_files(dir: &Path, acc: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            rs_files(&p, acc);
        } else if p.extension().is_some_and(|e| e == "rs") {
            acc.push(p);
        }
    }
}

/// Line `i` is pardoned if a declared exception stands directly
/// above it (blank lines in between do not count).
fn pardoned(rows: &[&str], i: usize) -> bool {
    i > 0 && rows[i - 1].contains("family-exception:")
}

/// Raw `text(...).size(...)` instead of mkw::txt with a typo role? (R88)
/// text_input has legitimate .size() calls and stays exempt.
fn text_size_literal(row: &str) -> bool {
    row.contains("text(")
        && row.contains(").size(")
        && !row.contains("mkw::txt")
        && !row.contains("text_input")
}

/// Naked number literal as symbol size? (mkw::symbol(X, 18.0, ...))
fn symbol_literal(row: &str) -> bool {
    let Some(pos) = row.find("mkw::symbol") else {
        return false;
    };
    let rest = &row[pos..];
    // first comma after the opening paren, then digit + dot?
    let Some(bracket) = rest.find('(') else {
        return false;
    };
    let Some(comma) = rest[bracket..].find(',') else {
        return false;
    };
    let after = rest[bracket + comma + 1..].trim_start();
    after.chars().next().is_some_and(|c| c.is_ascii_digit())
        && after.contains('.')
}

/// Stage 0 of the iced replacement (facade police): mkw is the ONE
/// gate to iced. No app declares iced, iced_layershell or
/// iced_sessionlock itself — whoever needs the library speaks
/// `mkw::iced::…`. When iced falls (stage 4), exactly ONE edge falls.
/// Exception as everywhere: `# family-exception: <reason>` directly above
/// the line.
#[test]
fn dependency_police() {
    let mut findings: Vec<String> = Vec::new();
    let Ok(entries) = std::fs::read_dir(apps_dir()) else {
        panic!("Apps-Verzeichnis nicht gefunden");
    };
    let mut scanned = 0;
    for e in entries.flatten() {
        let manifest = e.path().join("Cargo.toml");
        let Ok(content) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        scanned += 1;
        let rows: Vec<&str> = content.lines().collect();
        for (i, row) in rows.iter().enumerate() {
            let bare = row.trim_start();
            let declares = bare.starts_with("iced")
                && bare[4..].trim_start().starts_with(['=', '_'])
                || bare.starts_with("[dependencies.iced");
            if declares && !(i > 0 && rows[i - 1].contains("family-exception:")) {
                findings.push(format!(
                    "{}:{} — App deklariert iced selbst (nimm mkw::iced / mkw::iced_layershell — mkw ist das Tor)",
                    manifest.display(),
                    i + 1
                ));
            }
        }
    }
    assert!(scanned > 10, "keine App-Manifeste gefunden — Polizei blind");
    assert!(
        findings.is_empty(),
        "\n\nThe dependency police found {} violation(s):\n\n{}\n",
        findings.len(),
        findings.join("\n")
    );
}

#[test]
fn family_police() {
    let mut files = Vec::new();
    rs_files(&apps_dir(), &mut files);
    assert!(
        files.len() > 10,
        "Apps-Verzeichnis nicht gefunden — Polizei kann nicht patrouillieren"
    );

    let mut findings: Vec<String> = Vec::new();
    for path in &files {
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let rows: Vec<&str> = content.lines().collect();
        for (i, row) in rows.iter().enumerate() {
            if row.trim_start().starts_with("//") {
                continue;
            }
            for rule in RULES {
                if row.contains(rule.muster) && !pardoned(&rows, i) {
                    findings.push(format!(
                        "{}:{} — {} (nimm {})",
                        path.display(),
                        i + 1,
                        rule.name,
                        rule.family
                    ));
                }
            }
            if text_size_literal(row) && !pardoned(&rows, i) {
                findings.push(format!(
                    "{}:{} — rohe Textgröße (nimm mkw::txt mit mk::typo-Rolle)",
                    path.display(),
                    i + 1
                ));
            }
            if symbol_literal(row) && !pardoned(&rows, i) {
                findings.push(format!(
                    "{}:{} — nacktes Symbolgrößen-Literal (nimm mk::icon_size oder eine benannte Konstante)",
                    path.display(),
                    i + 1
                ));
            }
        }
    }

    assert!(
        findings.is_empty(),
        "\n\nThe family police found {} violation(s):\n\n{}\n\n\
         Either move onto the family or declare directly above the line:\n\
         // family-exception: <reason>\n",
        findings.len(),
        findings.join("\n")
    );
}
