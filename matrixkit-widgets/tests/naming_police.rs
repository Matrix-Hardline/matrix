//! Naming police — guards the English-code doctrine (2026-07-21).
//!
//! After the great migration every identifier is English. This test keeps
//! it that way: any German-looking identifier outside strings/comments
//! fails the suite immediately — errors surface at `cargo test` time,
//! never at runtime. Declared exceptions: `// family-exception:` above
//! the line (same grammar as the family police).

use std::fs;
use std::path::Path;

/// Strip string literals, char literals and line comments so only real
/// code identifiers are scanned.
fn code_only(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let b: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            // Raw strings (r"…", br#"…"#): first count the hashes,
            // then jump to the matching end. Without that the
            // scanner loses its bearings at an SVG literal and
            // arrests fragments from comments (found on
            // 16.08.2026, camp 62).
            'r' | 'b'
                if {
                    let mut j = i;
                    if b[j] == 'b' && b.get(j + 1) == Some(&'r') {
                        j += 1;
                    }
                    b.get(j) == Some(&'r')
                        && {
                            let mut k = j + 1;
                            while b.get(k) == Some(&'#') {
                                k += 1;
                            }
                            b.get(k) == Some(&'"')
                        }
                } =>
            {
                let mut j = i;
                if b[j] == 'b' {
                    j += 1;
                }
                let mut hashes = 0;
                let mut k = j + 1;
                while b.get(k) == Some(&'#') {
                    hashes += 1;
                    k += 1;
                }
                // k points at the opening quotation mark.
                i = k + 1;
                loop {
                    if i >= b.len() {
                        break;
                    }
                    if b[i] == '"' {
                        let mut m = 0;
                        while m < hashes && b.get(i + 1 + m) == Some(&'#') {
                            m += 1;
                        }
                        if m == hashes {
                            i += 1 + hashes;
                            break;
                        }
                    }
                    i += 1;
                }
            }
            '"' => {
                i += 1;
                while i < b.len() && b[i] != '"' {
                    i += if b[i] == '\\' { 2 } else { 1 };
                }
                i += 1;
            }
            '/' if i + 1 < b.len() && b[i + 1] == '/' => {
                while i < b.len() && b[i] != '\n' {
                    i += 1;
                }
            }
            '\'' if i + 2 < b.len() && (b[i + 2] == '\'' || (b[i + 1] == '\\')) => {
                out.push(' ');
                i += if b[i + 1] == '\\' { 4 } else { 3 };
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// German syllable fingerprints in identifiers. Kept deliberately close
/// to the audit detector that drove the migration.
const GERMAN: &[&str] = &[
    "eintr", "zeil", "pfad", "knopf", "fenster", "geist", "klang", "farb",
    "kachel", "knoten", "konto", "geraet", "stufe", "leiste", "meldung",
    "kreis", "reihe", "glanz", "schatt", "erlaub", "verweiger", "fuss",
    "lupe", "zieh", "blatt", "gruss", "hilfe", "sperre", "wache", "platte",
    "datei", "lauf", "schluessel", "setze", "loeschen", "pruef",
    "waehl", "hoeh", "groess", "zaehl", "uebersicht", "taste",
    "rechner", "ereignis", "quittung", "netz", "abbrech", "bestaetig",
    "helligkeit", "hintergrund", "auswahl", "sofort", "wechsel", "anteil",
    "rollo", "zustand", "einstell", "speicher", "verbind", "eingab",
    "ausgab", "grenze", "einheit", "glyphen", "zeichn", "ziel", "warte",
    "quell", "ordner", "fertig", "bereit", "lage", "misch", "zeichen",
    "befehl", "kanal", "tipp", "sekunde", "sprung", "dauer", "zahl",
    "teil", "glatt", "zackig", "packt", "mitteilung", "frist", "prozent",
    "faktor", "innen", "aussen", "abstand", "fuell", "duenn", "mittel",
    // R104 harvest — words that survived R101 disguised as English or
    // simply missing here. Deliberately NOT listed: aus (pause), ein
    // (reinstall), vor (vortex), kern (kernel), puls (impulse), rad
    // (radius), tag/alt/links (English), bis (bisect), rund (round).
    "fokus", "maus", "uhr", "zug", "seit", "blick", "grund", "legende",
    "linie", "rendern", "senden", "takt", "nach", "auf", "weg", "datum",
    "ruf", "ueb", "stil", "sitzung", "englisch",
];

/// R103 harvest: short German particles that hid below the 4-letter
/// floor (_da/_an/_ab suffix family). Matched as EXACT word parts —
/// substring matching would arrest half of English. Deliberately not
/// listed: was/gen/alt/hut (English), zu/ein/aus (checked above).
const GERMAN_SHORT: &[&str] = &[
    "da", "an", "ab", "um", "vom", "wie", "wer", "ist", "hat", "und",
];

/// English words the ae/oe/ue transcription detector must not flag.
const ALLOWED: &[&str] = &[
    "value", "values", "queue", "issuer", "true", "continue", "guard",
    "layershell", "daemon", "shell", "listen", "samples", "bluetooth",
    // "still" carries the German "stil" inside it (stillness_costs_nothing).
    "still",
];

#[test]
fn naming_police() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut findings = Vec::new();
    let mut dirs = vec![root.join("apps"), root.clone()];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                // vendor/ and vendor-crates/ are third-party code (R133,
                // UNABHAENGIGKEIT) — not ours to rename. Missing the
                // second one made this suite scan 33k foreign files: it
                // ran for ten minutes and arrested `ab_glyph_rasterizer`
                // and aho-corasick's `an` — a guard everybody believed
                // was green (found 2026-08-16).
                if name != "target" && name != ".git" && name != "vendor"
                    && name != "vendor-crates"
                {
                    dirs.push(p);
                }
            } else if p.extension().is_some_and(|x| x == "rs") {
                let Ok(src) = fs::read_to_string(&p) else { continue };
                let code = code_only(&src);
                for word in code.split(|c: char| !c.is_ascii_alphanumeric() && c != '_') {
                    let w = word.to_ascii_lowercase();
                    // Short particles: exact match on the snake_case parts.
                    // R103 meant SUFFIXES (_da/_an/_ab) — an English
                    // sentence-name may START with one (`an_old_file…`,
                    // `an_oversized_word…`), so the first part is exempt.
                    if w.split('_').skip(1).any(|t| GERMAN_SHORT.contains(&t)) {
                        findings.push(format!("{} — `{}`", p.display(), word));
                        continue;
                    }
                    if w.len() < 4 || ALLOWED.iter().any(|a| w.contains(a)) {
                        continue;
                    }
                    if GERMAN.iter().any(|g| w.contains(g)) {
                        findings.push(format!("{} — `{}`", p.display(), word));
                    }
                }
            }
        }
    }
    findings.sort();
    findings.dedup();
    assert!(
        findings.is_empty(),
        "\n\nNaming police: {} German identifier(s) found — code must be English:\n{}\n",
        findings.len(),
        findings.join("\n")
    );
}
