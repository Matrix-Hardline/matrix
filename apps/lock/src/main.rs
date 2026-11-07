//! Matrix Sperre — app #20, the lock screen (round 123: in the house,
//! without iced).
//!
//! A REAL lock: the house mounts it as the session lock (ext-session-lock,
//! `shell::Mount::Lock`) — the compositor shows nothing else, and even a
//! crash of this process keeps the session locked (starting the lock again
//! takes over). The session opens only after PAM accepted the code of the
//! logged-in user (src/pam.rs, service /etc/pam.d/matrix-lock → system-auth).
//!
//! One piece, like a phone: the wallpaper, the date and a large glass clock
//! (`matrixkit_scene::glass_clock`). Swiping up lifts the whole face with
//! the finger and brings the glass keypad up from below; letting go springs
//! to the nearer side. The code length is learnt at the first success (only
//! the length, never the code) — from then on the last digit unlocks by
//! itself. A wrong code shakes the dots. Physical keys type as well.
//!
//! Modes:
//!   (none)          the lock
//!   --probe         the same lock, but it opens by itself after 45 s and
//!                   «demo» is accepted too — a harmless full test
//!   --check <pw>    PAM check only, exit 0/1 — no screen, no lock
//!
//! Look (settings): `lock-clock-stretch` (1.5–3.2), `lock-clock-glass`
//! (0 solid … 1 glass).

mod face;
mod pam;

fn user() -> String {
    std::env::var("USER").unwrap_or_else(|_| String::from("user"))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--check") {
        let pw = args.get(2).cloned().unwrap_or_default();
        let ok = pam::verify(&user(), &pw);
        println!("{}", if ok { "ok" } else { "abgelehnt" });
        std::process::exit(if ok { 0 } else { 1 });
    }
    // The old name of the probe stays understood.
    let probe = args.iter().any(|a| a == "--probe" || a == "--demo");
    if let Err(e) = face::run(probe) {
        eprintln!("[sperre] {e}");
        std::process::exit(1);
    }
}
