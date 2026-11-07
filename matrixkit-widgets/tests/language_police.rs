//! Language police — German UI stays German (2026-07-21).
//!
//! The migration's sweeps corrupted German UI strings with English words
//! ("Dein Account", "Physische Disk", "auf einen Peek") — invisible to
//! the compiler, visible to every user. This test scans string literals:
//! a German-looking sentence that carries one of the known migration
//! words fails the suite. UI language is a contract too.

use std::fs;
use std::path::Path;

/// English words that the migration introduced and that must never sit
/// inside a German sentence.
const SUSPECTS: &[&str] = &[
    "Account", "Disk", "Step", "Entry", "Tile", "Peek", "Movie", "Gallery",
    "Node", "Column", "Portion", "Viewer", "Courier",
];

/// Cheap German markers: articles/prepositions that only appear in
/// German prose (with surrounding spaces so app ids never match).
const GERMAN_MARKERS: &[&str] = &[
    " der ", " die ", " das ", " ein ", " eine ", " dein ", " diese ",
    " dieses ", " und ", " nicht ", " wird ", " für ", " mit ", "ä", "ö",
    "ü", "ß", "„",
];

fn strings_of(src: &str) -> Vec<String> {
    let b: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            '"' => {
                let mut s = String::new();
                i += 1;
                while i < b.len() && b[i] != '"' {
                    if b[i] == '\\' && i + 1 < b.len() {
                        s.push(b[i]);
                        s.push(b[i + 1]);
                        i += 2;
                    } else {
                        s.push(b[i]);
                        i += 1;
                    }
                }
                i += 1;
                out.push(s);
            }
            '/' if i + 1 < b.len() && b[i + 1] == '/' => {
                while i < b.len() && b[i] != '\n' {
                    i += 1;
                }
            }
            // Char literals ('"', '\\'…) must not open a fake string.
            '\'' if i + 2 < b.len() && (b[i + 2] == '\'' || b[i + 1] == '\\') => {
                i += if b[i + 1] == '\\' { 4 } else { 3 };
            }
            _ => i += 1,
        }
    }
    out
}

#[test]
fn language_police() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut findings = Vec::new();
    let mut dirs = vec![root.join("apps")];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().unwrap_or_default() != "target" {
                    dirs.push(p);
                }
            } else if p.extension().is_some_and(|x| x == "rs") {
                let Ok(src) = fs::read_to_string(&p) else { continue };
                for s in strings_of(&src) {
                    if s.len() < 12 {
                        continue;
                    }
                    let is_german = GERMAN_MARKERS.iter().any(|m| s.contains(m));
                    if !is_german {
                        continue;
                    }
                    for w in SUSPECTS {
                        // Word-ish match: suspect surrounded by non-letters.
                        if let Some(pos) = s.find(w) {
                            let before_ok = pos == 0
                                || !s[..pos].chars().next_back().unwrap().is_ascii_alphanumeric();
                            let after = &s[pos + w.len()..];
                            let after_ok = after.is_empty()
                                || !after.chars().next().unwrap().is_ascii_alphanumeric();
                            if before_ok && after_ok {
                                findings.push(format!(
                                    "{} — \"{}\" in »{}…«",
                                    p.display(),
                                    w,
                                    &s[..s.len().min(60)]
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    findings.sort();
    findings.dedup();
    assert!(
        findings.is_empty(),
        "\n\nLanguage police: {} English word(s) inside German UI text:\n{}\n",
        findings.len(),
        findings.join("\n")
    );
}
