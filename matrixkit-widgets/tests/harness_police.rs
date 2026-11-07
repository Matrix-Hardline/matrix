//! Harness police (R148): every GUI app follows the app canon.
//!
//! Born from the consistency audit that found the ENTIRE layershell
//! fleet running without the mailbox present mode the newer apps
//! treat as canon. Rules:
//!  * every iced GUI app sets the present mode (present_mailbox or
//!    the explicit env pattern)
//!  * every windowed (non-layershell) GUI app is a singleton unless
//!    it is on the multi-instance list with a reason
//! Exceptions live HERE, named and justified — not scattered.

use std::path::Path;

/// Windowed apps that WANT multiple instances (a reason each).
const MULTI_INSTANCE: &[(&str, &str)] = &[
    ("terminal", "several shells side by side are the point"),
    ("player", "several videos side by side are legitimate"),
    ("web-content", "carrier: one per matrix-web, managed by it"),
];

#[test]
fn harness_police() {
    let apps = Path::new(env!("CARGO_MANIFEST_DIR")).join("../apps");
    let mut violations = Vec::new();

    for entry in std::fs::read_dir(&apps).expect("apps dir") {
        let entry = entry.expect("entry");
        let name = entry.file_name().to_string_lossy().to_string();
        let main = entry.path().join("src/main.rs");
        let Ok(src) = std::fs::read_to_string(&main) else { continue };

        let is_iced = src.contains("iced::application") || src.contains("iced_layershell");
        let is_gtk = src.contains("gtk::Application");
        if !is_iced && !is_gtk {
            continue; // daemons and helpers are not harness citizens
        }
        if is_gtk {
            continue; // the carrier follows GTK rules, not iced's
        }

        if !src.contains("present_mailbox") && !src.contains("ICED_PRESENT_MODE") {
            violations.push(format!("{name}: kein Present-Mode-Kanon (mkw::present_mailbox())"));
        }

        let windowed = src.contains("iced::application") && !src.contains("iced_layershell");
        let excused = MULTI_INSTANCE.iter().any(|(a, _)| *a == name);
        if windowed && !excused && !src.contains("single_instance") {
            violations.push(format!(
                "{name}: Fenster-App ohne single_instance (oder ohne Eintrag in MULTI_INSTANCE)"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "\n\nHarness police found {} violation(s):\n\n{}\n",
        violations.len(),
        violations.join("\n")
    );
}
