//! Intents police (R156): every manifest in the registry parses and
//! points at a runnable target.
//!
//! Apple extracts the App Intents metadata at BUILD time — a broken
//! manifest never ships. Ours ships as plain files, so the build-time
//! guarantee lives here instead.

use std::path::{Path, PathBuf};

fn upwards(probe: &str) -> Option<PathBuf> {
    let mut dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    for _ in 0..5 {
        if dir.join(probe).exists() {
            return Some(dir);
        }
        dir = dir.parent()?.to_path_buf();
    }
    None
}

/// Non-Matrix runners a manifest may call (each with a reason).
const RUNNERS: &[&str] = &["xdg-open"];

#[test]
fn intents_police() {
    let Some(root) = upwards("usr/share/matrix/intents") else {
        eprintln!("Intents police: no repo root in sight, skipping");
        return;
    };
    let dir = root.join("usr/share/matrix/intents");
    let mut violations = Vec::new();
    let mut seen = 0;
    for e in std::fs::read_dir(&dir).expect("intents dir").flatten() {
        let raw = std::fs::read_to_string(e.path()).expect("readable manifest");
        let name = e.file_name().to_string_lossy().to_string();
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
            violations.push(format!("{name}: kein gueltiges JSON"));
            continue;
        };
        if v["app_id"].as_str().unwrap_or_default().is_empty() {
            violations.push(format!("{name}: app_id fehlt"));
        }
        for i in v["intents"].as_array().cloned().unwrap_or_default() {
            seen += 1;
            let id = i["id"].as_str().unwrap_or("?");
            if i["title"].as_str().unwrap_or_default().is_empty() {
                violations.push(format!("{name}/{id}: title fehlt"));
            }
            let exec: Vec<&str> = i["exec"]
                .as_array()
                .map(|a| a.iter().filter_map(|x| x.as_str()).collect())
                .unwrap_or_default();
            match exec.first() {
                None => violations.push(format!("{name}/{id}: exec fehlt")),
                Some(cmd) if RUNNERS.contains(cmd) => {}
                Some(cmd) => {
                    let app = cmd.strip_prefix("matrix-").unwrap_or("");
                    if app.is_empty() || !root.join("matrixkit/apps").join(app).exists() {
                        violations.push(format!(
                            "{name}/{id}: exec \u{201e}{cmd}\u{201c} ist weder eine Matrix-App noch ein erlaubter Runner"
                        ));
                    }
                }
            }
        }
    }
    assert!(seen > 0, "Intents police found no intent at all");
    assert!(
        violations.is_empty(),
        "\n\nIntents police found {} violation(s):\n\n{}\n",
        violations.len(),
        violations.join("\n")
    );
}
