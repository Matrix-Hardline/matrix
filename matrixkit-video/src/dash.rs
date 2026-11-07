//! DASH INDEX (round 128): where a time lies in a fragmented MP4 — the
//! way a web stream seeks without the throttled single request.
//!
//! YouTube's DASH tracks (H.264 video, AAC sound) begin with `ftyp`, `moov`
//! and a `sidx` box: the byte size and duration of every fragment. A seek
//! then fetches the head (everything up to the end of `sidx`) and the
//! fragments from the one that holds the time — in fast range pieces like
//! the stock does from the start. ffmpeg reads head + fragments as one
//! stream; the first fragment's own time stamp says where it begins.
//!
//! Measured on the Duo (05.11.2026): ffmpeg's own seek on such a URL got
//! 20 s of sound in 11 s (1.8× real time, throttled); one 1 MB range piece
//! came in 0.15 s.

use crate::Input;
use std::process::{Command, Stdio};

/// The head and the fragments of a fragmented MP4.
#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    /// ftyp … sidx: what ffmpeg needs before any fragment.
    pub head: Vec<u8>,
    /// (byte offset, start time in seconds) of every fragment.
    pub segments: Vec<(u64, f64)>,
}

impl Index {
    /// The fragment holding `t`: its byte offset and start time.
    pub fn locate(&self, t: f64) -> Option<(u64, f64)> {
        let i = self.segments.partition_point(|&(_, s)| s <= t + 1e-6);
        self.segments.get(i.checked_sub(1)?).copied()
    }
}

/// Read the head of a fragmented MP4 (the first bytes of the file).
/// None: no `sidx` in `bytes` (not fragmented, or the head is longer).
pub fn parse(bytes: &[u8]) -> Option<Index> {
    let mut at = 0usize;
    while at + 8 <= bytes.len() {
        let size32 = u32::from_be_bytes(bytes[at..at + 4].try_into().ok()?) as u64;
        let kind = &bytes[at + 4..at + 8];
        let (size, body) = match size32 {
            1 => (u64::from_be_bytes(bytes.get(at + 8..at + 16)?.try_into().ok()?), at + 16),
            0 => ((bytes.len() - at) as u64, at + 8),
            s => (s, at + 8),
        };
        if size < 8 {
            return None;
        }
        let end = at.checked_add(size as usize)?;
        if kind == b"sidx" {
            if end > bytes.len() {
                return None;
            }
            return sidx(&bytes[body..end], end as u64).map(|segments| Index { head: bytes[..end].to_vec(), segments });
        }
        if kind == b"moof" || kind == b"mdat" {
            return None; // fragments before any index
        }
        at = end;
    }
    None
}

/// The `sidx` body; `anchor` = the first byte after the box.
fn sidx(b: &[u8], anchor: u64) -> Option<Vec<(u64, f64)>> {
    let u32_at = |i: usize| -> Option<u32> { Some(u32::from_be_bytes(b.get(i..i + 4)?.try_into().ok()?)) };
    let u64_at = |i: usize| -> Option<u64> { Some(u64::from_be_bytes(b.get(i..i + 8)?.try_into().ok()?)) };
    let version = *b.first()?;
    let timescale = u32_at(8)? as f64;
    if timescale <= 0.0 {
        return None;
    }
    let (earliest, first_offset, mut i) = if version == 0 {
        (u32_at(12)? as u64, u32_at(16)? as u64, 20)
    } else {
        (u64_at(12)?, u64_at(20)?, 28)
    };
    i += 2; // reserved
    let count = u16::from_be_bytes(b.get(i..i + 2)?.try_into().ok()?) as usize;
    i += 2;
    let mut offset = anchor + first_offset;
    let mut time = earliest;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let size = (u32_at(i)? & 0x7fff_ffff) as u64;
        let duration = u32_at(i + 4)? as u64;
        out.push((offset, time as f64 / timescale));
        offset += size;
        time += duration;
        i += 12;
    }
    (!out.is_empty()).then_some(out)
}

/// Fetch the head of `input` (one fast range piece) and read its index.
pub fn fetch(input: &Input) -> Option<Index> {
    let mut curl = Command::new("curl");
    curl.args(["-s", "-L", "--fail", "--max-time", "4", "-r", "0-262143"]);
    for (k, v) in &input.headers {
        curl.arg("-H").arg(format!("{k}: {v}"));
    }
    let out = curl.arg(&input.url).stdin(Stdio::null()).stderr(Stdio::null()).output().ok()?;
    parse(&out.stdout)
}

/// The total size of `input` in bytes (a one-byte range: Content-Range).
pub fn total_size(input: &Input) -> Option<u64> {
    let mut curl = Command::new("curl");
    curl.args(["-s", "-L", "--fail", "--max-time", "8", "-r", "0-0", "-D", "-", "-o", "/dev/null"]);
    for (k, v) in &input.headers {
        curl.arg("-H").arg(format!("{k}: {v}"));
    }
    let out = curl.arg(&input.url).stdin(Stdio::null()).stderr(Stdio::null()).output().ok()?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.to_ascii_lowercase().starts_with("content-range:"))
        .last()
        .and_then(|l| l.rsplit('/').next())
        .and_then(|n| n.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boxed(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut v = ((body.len() + 8) as u32).to_be_bytes().to_vec();
        v.extend_from_slice(kind);
        v.extend_from_slice(body);
        v
    }

    /// ftyp, moov, sidx (v0, timescale 1000, three fragments of 1000,
    /// 2000 and 500 bytes lasting 5 s, 5 s, 2.5 s), then a moof.
    fn sample() -> Vec<u8> {
        let mut sidx = vec![0u8; 4]; // version 0, flags
        sidx.extend_from_slice(&1u32.to_be_bytes()); // reference id
        sidx.extend_from_slice(&1000u32.to_be_bytes()); // timescale
        sidx.extend_from_slice(&0u32.to_be_bytes()); // earliest
        sidx.extend_from_slice(&0u32.to_be_bytes()); // first offset
        sidx.extend_from_slice(&[0, 0]);
        sidx.extend_from_slice(&3u16.to_be_bytes());
        for (size, dur) in [(1000u32, 5000u32), (2000, 5000), (500, 2500)] {
            sidx.extend_from_slice(&size.to_be_bytes());
            sidx.extend_from_slice(&dur.to_be_bytes());
            sidx.extend_from_slice(&0x9000_0000u32.to_be_bytes());
        }
        let mut file = boxed(b"ftyp", b"dash0000");
        file.extend(boxed(b"moov", &[0u8; 40]));
        file.extend(boxed(b"sidx", &sidx));
        file.extend(boxed(b"moof", &[0u8; 16]));
        file
    }

    #[test]
    fn the_index_names_every_fragment() {
        let file = sample();
        let idx = parse(&file).expect("an index");
        let head = idx.head.len() as u64;
        assert_eq!(idx.segments, vec![(head, 0.0), (head + 1000, 5.0), (head + 3000, 10.0)]);
        assert!(file[..head as usize].ends_with(&0x9000_0000u32.to_be_bytes()));
    }

    #[test]
    fn a_time_finds_its_fragment() {
        let idx = parse(&sample()).unwrap();
        assert_eq!(idx.locate(0.0).map(|s| s.1), Some(0.0));
        assert_eq!(idx.locate(7.3).map(|s| s.1), Some(5.0));
        assert_eq!(idx.locate(11.0).map(|s| s.1), Some(10.0));
        assert_eq!(idx.locate(-1.0), None);
    }

    #[test]
    fn no_index_no_seek() {
        let mut file = boxed(b"ftyp", b"isom0000");
        file.extend(boxed(b"mdat", &[0u8; 32]));
        assert_eq!(parse(&file), None);
    }
}
