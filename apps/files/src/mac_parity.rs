//! The macOS-parity trio (R141), extracted from the live limit dig:
//!  * reflink copies (APFS clonefile → btrfs cp --reflink)
//!  * provenance ("Woher": the kMDItemWhereFroms extract via xattr)
//!  * Unicode-normalization-aware search (APFS folds NFC/NFD; btrfs
//!    is byte-exact, and Mac rsync delivers NFD names).

use std::path::Path;
use unicode_normalization::UnicodeNormalization;

/// Copy a file, preferring a reflink (copy-on-write) — instant and
/// free on the same btrfs filesystem, graceful normal copy across
/// filesystems. Mirrors macOS `cp -c` / clonefile.
pub fn copy_reflink(from: &Path, to: &Path) -> std::io::Result<()> {
    let ok = std::process::Command::new("cp")
        .arg("--reflink=auto")
        .arg("--preserve=timestamps")
        .arg("--")
        .arg(from)
        .arg(to)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        return Ok(());
    }
    // Last resort: the plain byte copy (different fs, no cp, …).
    std::fs::copy(from, to).map(|_| ())
}

/// The "Woher" line: download origin from xattr. Reads our own
/// `user.matrix.wherefroms` first, then the freedesktop standard
/// `user.xdg.origin.url` that other browsers write. None if neither.
pub fn wherefroms(path: &Path) -> Option<String> {
    for key in ["user.matrix.wherefroms", "user.xdg.origin.url"] {
        let out = std::process::Command::new("getfattr")
            .args(["--only-values", "-n", key, "--"])
            .arg(path)
            .output()
            .ok()?;
        if out.status.success() && !out.stdout.is_empty() {
            let raw = String::from_utf8_lossy(&out.stdout);
            // wherefroms carries "url\treferrer"; show the URL.
            let url = raw.split(['\t', '\n']).next().unwrap_or("").trim();
            if !url.is_empty() {
                return Some(url.to_string());
            }
        }
    }
    None
}

/// Fold a string for search/compare: lowercase + NFC. So a Mac-rsync'd
/// NFD "a\u{308}" matches a typed NFC "ä", and case is ignored — the
/// APFS behavior, made explicit on byte-exact btrfs.
pub fn fold(s: &str) -> String {
    s.nfc().flat_map(|c| c.to_lowercase()).collect()
}

/// Does `name` contain `needle`, normalization- and case-insensitively?
pub fn matches(name: &str, needle_folded: &str) -> bool {
    needle_folded.is_empty() || fold(name).contains(needle_folded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_bridges_nfd_and_case() {
        // Mac rsync delivers NFD ("a" + combining diaeresis); typed
        // needles are NFC. Both worlds must meet.
        let nfd = "Bildschirmfoto a\u{308}hnlich";
        assert!(matches(nfd, &fold("ähnlich")));
        assert!(matches("Test-CASE", &fold("test-case")));
        assert!(matches("anything", &fold("")));
        assert!(!matches("Plan", &fold("ähnlich")));
    }
}
