//! Matrix Anmeldung — app #16, the login screen, in the house (no iced).
//!
//! Runs in its own niri kiosk session under greetd as the greetd account:
//! the person's wallpaper and lock clock (shared by their session, see
//! mk::login_share), a password field, the login pill, glass power keys and
//! on a duo the keyboard on the lower panel. Round 166 (05.11.2026): the old
//! iced fallback (`--iced`) is gone — the house is the only login screen.
//!
//! `--probe` runs it inside a session to look at it: the login is only
//! pretended, Escape ends it.

mod greetd;
mod haus;
mod installer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let probe = args.iter().any(|a| a == "--probe" || a == "--demo");
    if let Err(e) = haus::run(probe) {
        eprintln!("[anmeldung] Fensterhaus-Fehler: {e}");
        std::process::exit(1);
    }
}

/// The person to log in: the first ordinary account (uid 1000…), its full
/// name for people. Morpheus (round 175): None = a NEW device — the login
/// screen runs the first-start setup instead.
fn user_find() -> Option<(String, String)> {
    if let Ok(passwd) = std::fs::read_to_string("/etc/passwd") {
        for l in passwd.lines() {
            let f: Vec<&str> = l.split(':').collect();
            if f.len() >= 7 {
                let uid: u32 = f[2].parse().unwrap_or(0);
                if (1000..60000).contains(&uid)
                    && !f[6].contains("nologin")
                    && f[0] != "wache"
                {
                    let gecos = f[4].split(',').next().unwrap_or("").trim();
                    let display = if gecos.is_empty() {
                        large(f[0])
                    } else {
                        large(gecos)
                    };
                    return Some((f[0].to_string(), display));
                }
            }
        }
    }
    None
}

fn large(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_else(|| s.to_string())
}

/// The sessions on offer (name, command), the Matrix canvas first by the caller.
fn sessions_find() -> Vec<(String, String)> {
    let mut list = Vec::new();
    for dir in [
        "/usr/local/share/wayland-sessions",
        "/usr/share/wayland-sessions",
    ] {
        let Ok(rd) = std::fs::read_dir(dir) else { continue };
        for e in rd.flatten() {
            let Ok(content) = std::fs::read_to_string(e.path()) else {
                continue;
            };
            let (mut name, mut exec) = (None, None);
            for z in content.lines() {
                if let Some(v) = z.strip_prefix("Name=") {
                    name.get_or_insert(v.trim().to_string());
                }
                if let Some(v) = z.strip_prefix("Exec=") {
                    exec.get_or_insert(v.trim().to_string());
                }
            }
            if let (Some(n), Some(x)) = (name, exec) {
                if !list.iter().any(|(ln, _): &(String, String)| ln == &n) {
                    list.push((n, x));
                }
            }
        }
    }
    if list.is_empty() {
        list.push((String::from("Niri"), String::from("niri-session")));
    }
    list
}
