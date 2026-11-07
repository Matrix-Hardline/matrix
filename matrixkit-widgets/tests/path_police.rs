//! Path police (R107) — guards the raw-path blind spot.
//!
//! The naming police scans identifiers, the contract police scans
//! `settings::read(...)` markers. Neither sees a German file name baked
//! into a raw `~/.config/matrix/...` or `/etc/matrix/...` string literal
//! — that is exactly how `hintergrund-hell/-dunkel`, `wachter.conf` and
//! `/etc/matrix/schluessel` survived every earlier audit (user finding
//! 2026-07-21). This test reads the STRING LITERALS other tests skip.
//!
//! A German path fragment is allowed only when it is a documented legacy
//! read — marked by `legacy` or `pre-R107`/`R105` on the same or the
//! previous line (the dual-accept migration keeps reading the old name).

use std::fs;
use std::path::Path;

/// German file-name fragments that must not appear as the CANONICAL name
/// in a config/runtime path. Deliberately specific — not broad syllables.
const GERMAN_PATHS: &[&str] = &[
    "hintergrund", "wachter", "schluessel", "wache-frist", "avatar-lebendig",
    "zwischenablage", "matrix/fenster", "berechtigungen", "klaenge", "getoent",
    "einstellungen-bereich", "settings-bereich", "morpheus-konto", "gong-aus",
];

fn scan(dir: &Path, findings: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
        if p.is_dir() {
            if name != "target" && name != ".git" {
                scan(&p, findings);
            }
            continue;
        }
        if p.extension().is_none_or(|x| x != "rs") {
            continue;
        }
        // Don't scan the police's own fragment list.
        if name == "path_police.rs" {
            continue;
        }
        let Ok(src) = fs::read_to_string(&p) else { continue };
        let lines: Vec<&str> = src.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            // Only care about lines mentioning a matrix config/runtime path.
            if !(line.contains(".config/matrix")
                || line.contains("/etc/matrix")
                || line.contains("matrix-zwischenablage"))
            {
                continue;
            }
            // A legacy marker within the surrounding 3 lines clears it.
            let lo = i.saturating_sub(3);
            let context = lines[lo..=i].join("\n");
            let is_legacy = context.contains("legacy")
                || context.contains("pre-R107")
                || context.contains("R105")
                || context.contains("R107");
            if is_legacy {
                continue;
            }
            for frag in GERMAN_PATHS {
                if line.contains(frag) {
                    findings.push(format!("{}:{} — `{frag}`", p.display(), i + 1));
                }
            }
        }
    }
}

#[test]
fn path_police() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut findings = Vec::new();
    scan(&root, &mut findings);
    findings.sort();
    findings.dedup();
    assert!(
        findings.is_empty(),
        "\n\nPath police: {} raw German path(s) — migrate with dual-accept \
         (English canonical, German read as legacy + a `legacy` comment):\n{}\n",
        findings.len(),
        findings.join("\n")
    );
}
