//! PAIRED LAPTOPS (06.11.2026, the owner: «Gekoppelte Laptops … zum Entziehen
//! per Fingertipp»): the laptops trusted in Morpheus («Diesem Laptop
//! vertrauen?»). Each is one line in ~/.ssh/authorized_keys, written by
//! `matrix-kopplung`, marked `morpheus:<name>[:<date>]` and limited to the
//! USB cable (`from="172.16.42.2"`). Withdrawing removes exactly that line.

use std::path::PathBuf;

const MARK: &str = "morpheus:";

#[derive(Debug, Clone, PartialEq)]
pub struct Paired {
    pub name: String,
    /// The day of pairing («2026-11-06»), empty for lines from before it was noted.
    pub since: String,
    /// The whole line — what is removed.
    line: String,
}

fn keys() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default().join(".ssh/authorized_keys")
}

/// The Morpheus lines of an authorized_keys text (other keys are not ours).
pub fn parse(text: &str) -> Vec<Paired> {
    text.lines()
        .filter_map(|line| {
            let mark = line.split_whitespace().find(|w| w.starts_with(MARK))?;
            let mut parts = mark[MARK.len()..].splitn(2, ':');
            // Spaces travel as «_» (the mark is one word).
            let name = parts.next().filter(|n| !n.is_empty()).unwrap_or("Laptop").replace('_', " ");
            let since = parts.next().unwrap_or("").to_string();
            Some(Paired { name, since, line: line.to_string() })
        })
        .collect()
}

/// The text without exactly this line (every other key stays as it was).
pub fn without(text: &str, line: &str) -> String {
    let mut out: String = text.lines().filter(|l| *l != line).map(|l| format!("{l}\n")).collect();
    if out.is_empty() {
        out.clear();
    }
    out
}

pub fn list() -> Vec<Paired> {
    std::fs::read_to_string(keys()).map(|t| parse(&t)).unwrap_or_default()
}

/// Withdraw one laptop's trust: rewrite the file without its line (atomic, 0600).
pub fn withdraw(p: &Paired) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let path = keys();
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    if !text.lines().any(|l| l == p.line) {
        return Err(String::from("schon entfernt"));
    }
    let tmp = path.with_extension("morpheus-neu");
    std::fs::write(&tmp, without(&text, &p.line)).map_err(|e| e.to_string())?;
    std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEYS: &str = "ssh-ed25519 AAAAmine anna@laptop\nfrom=\"172.16.42.2\",no-port-forwarding,no-X11-forwarding,no-agent-forwarding ssh-ed25519 AAAAlaptop morpheus:Surface_Laptop_7th_Edition:2026-11-06\nfrom=\"172.16.42.2\",no-port-forwarding ssh-ed25519 AAAAold morpheus:altgeraet\n";

    #[test]
    fn only_morpheus_lines_are_paired_laptops() {
        let l = parse(KEYS);
        assert_eq!(l.len(), 2);
        assert_eq!((l[0].name.as_str(), l[0].since.as_str()), ("Surface Laptop 7th Edition", "2026-11-06"));
        assert_eq!((l[1].name.as_str(), l[1].since.as_str()), ("altgeraet", ""));
    }

    #[test]
    fn withdrawing_removes_exactly_one_line() {
        let l = parse(KEYS);
        let rest = without(KEYS, &l[0].line);
        assert!(rest.contains("AAAAmine") && rest.contains("AAAAold") && !rest.contains("AAAAlaptop"));
        assert_eq!(rest.lines().count(), 2);
        assert_eq!(parse(&rest).len(), 1);
    }
}
