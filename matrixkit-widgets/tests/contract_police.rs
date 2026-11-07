//! Contract police — config keys are API (2026-07-21).
//!
//! The English migration proved that the compiler is blind to string
//! contracts: renaming a `~/.config/matrix/` key silently resets user
//! settings (the leinwand-blaettern incident). This test pins every key
//! against the register in `matrixkit/CONTRACTS.txt`: an unregistered
//! key — or a registered key that vanished from the code — fails the
//! suite. Renames therefore always happen consciously: code + register
//! + a migration fallback in the app.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn harvest_keys(root: &Path) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                if name != "target" && name != ".git" {
                    dirs.push(p);
                }
            } else if p.extension().is_some_and(|x| x == "rs") {
                let Ok(src) = fs::read_to_string(&p) else { continue };
                let mut rest = src.as_str();
                // `file: "..."` harvests registry-family stores (R134):
                // the key moved from the call site into Store::file, and
                // it must stay pinned against the register there.
                for marker in ["settings::read(", "settings::write(", "conf_read(",
                               "conf_write(", "switch_read(", "switch_write(",
                               "file: "] {
                    let mut hay = rest;
                    while let Some(pos) = hay.find(marker) {
                        let after = &hay[pos + marker.len()..];
                        // The key must be a literal RIGHT after the paren —
                        // forwarded variables (write(key, "an")) don't count.
                        let ws = after.len() - after.trim_start().len();
                        if let Some(q) = after.find('"').filter(|q| *q == ws) {
                            if let Some(q2) = after[q + 1..].find('"') {
                                let key = &after[q + 1..q + 1 + q2];
                                if key.chars().all(|c| c.is_ascii_lowercase()
                                    || c.is_ascii_digit() || c == '-')
                                    && !key.is_empty()
                                {
                                    keys.insert(key.to_string());
                                }
                            }
                        }
                        hay = &hay[pos + marker.len()..];
                    }
                    let _ = rest;
                }
                // Keys named once as a constant (`const KEY_ON: &str = "hearing-guard"`,
                // the hearing guard) and then passed on by name: the constant is the
                // contract.
                for line in src.lines() {
                    let t = line.trim_start();
                    let t = t.strip_prefix("pub ").unwrap_or(t);
                    if let Some(rest_c) = t.strip_prefix("const KEY_") {
                        if let Some((_, lit)) = rest_c.split_once(": &str = \"") {
                            if let Some((key, _)) = lit.split_once('"') {
                                if !key.is_empty() && key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                                    keys.insert(key.to_string());
                                }
                            }
                        }
                    }
                }
                rest = src.as_str();
                let _ = rest;
            }
        }
    }
    keys
}

#[test]
fn contract_police() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let register_path = root.join("CONTRACTS.txt");
    let register: BTreeSet<String> = fs::read_to_string(&register_path)
        .expect("CONTRACTS.txt missing — the contract register is mandatory")
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect();
    let used = harvest_keys(&root);

    let unregistered: Vec<_> = used.difference(&register).cloned().collect();
    let orphaned: Vec<_> = register.difference(&used).cloned().collect();

    assert!(
        unregistered.is_empty(),
        "\n\nContract police: unregistered config key(s) {:?} — add them to \
         matrixkit/CONTRACTS.txt (and think about live-config migration!)\n",
        unregistered
    );
    assert!(
        orphaned.is_empty(),
        "\n\nContract police: registered key(s) {:?} no longer appear in the \
         code — either dead (remove from register) or silently renamed \
         (that would reset user settings: add a migration fallback!)\n",
        orphaned
    );
}
