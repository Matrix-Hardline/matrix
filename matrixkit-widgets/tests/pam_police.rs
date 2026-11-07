//! PAM police (R151): every PAM service an app asks for must have its
//! rule file in the image.
//!
//! Born from a real lockout (27.07.2026): the lock screen asks PAM for
//! the service "matrix-lock", but only the pre-R105 German file
//! "matrix-sperre" was ever shipped. PAM answers a missing service
//! with /etc/pam.d/other — which denies EVERY password. The user was
//! locked out of his own machine and had to power it off.
//!
//! The rename of a contract is only half done until its file follows.

use std::path::{Path, PathBuf};

/// Walk up from this crate until a directory carries `probe`.
/// The kit is also built from a partial tree (the station's work bench
/// holds matrixkit/ alone) — there the repo root simply is not there.
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

/// Service names an app hands to pam_start, collected from the source.
fn services_asked_for() -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Some(kit) = upwards("apps") else { return out };
    let apps = kit.join("apps");
    let Ok(dir) = std::fs::read_dir(&apps) else {
        return out;
    };
    for app in dir.flatten() {
        let name = app.file_name().to_string_lossy().to_string();
        let src = app.path().join("src");
        let Ok(files) = std::fs::read_dir(&src) else { continue };
        for f in files.flatten() {
            let Ok(text) = std::fs::read_to_string(f.path()) else { continue };
            if !text.contains("pam_start") {
                continue;
            }
            for service in string_literals_after(&text, "CString::new(\"") {
                // Only PAM-looking names — the file also builds C strings
                // for the user name and the prompt.
                if service.starts_with("matrix-") {
                    out.push((name.clone(), service));
                }
            }
        }
    }
    out
}

fn string_literals_after(text: &str, marker: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find(marker) {
        rest = &rest[i + marker.len()..];
        if let Some(end) = rest.find('"') {
            out.push(rest[..end].to_string());
        }
    }
    out
}

#[test]
fn pam_police() {
    let Some(root) = upwards("etc/pam.d") else {
        // Partial tree (kit only) — the full checkout enforces it.
        eprintln!("PAM police: no repo root in sight, skipping");
        return;
    };
    let asked = services_asked_for();
    assert!(
        !asked.is_empty(),
        "PAM police found no service at all — did the detection break?"
    );

    let mut violations = Vec::new();
    for (app, service) in &asked {
        let rule = root.join("etc/pam.d").join(service);
        if !rule.exists() {
            violations.push(format!(
                "{app}: fragt PAM nach „{service}“, aber etc/pam.d/{service} fehlt \
                 — PAM lehnt dann JEDES Passwort ab (Aussperrung!)"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "\n\nPAM police found {} violation(s):\n\n{}\n",
        violations.len(),
        violations.join("\n")
    );
}
