//! Matrix Tube's library (R210) — everything YouTube, nothing visible.
//!
//! * **Search** goes to YouTube's own web API (InnerTube `/search`), which
//!   needs no proof-of-origin token. HTTP through curl, like the other
//!   Matrix apps (no TLS stack in the Kit).
//! * **Streams** come from the replaceable helper `yt-dlp` (decided by
//!   The owner, 01.11.2026): client `mweb`, PO tokens from the local bgutil
//!   service, the JS challenges solved by Node. The answer is turned into a
//!   `matrixkit_video::Media` — picture and sound URLs with their headers,
//!   played by the video family.
//! * **SponsorBlock** marks the parts to skip (sponsor, self promotion,
//!   interaction reminders, intro, outro).
//! * **Thumbnails** are fetched once into the cache.
//!
//! No advertising exists here by construction: a client of our own never
//! asks for it.

use matrixkit_video::{Input, Media};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

/// One search hit.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub id: String,
    pub title: String,
    pub channel: String,
    /// "48:19" — empty for live streams.
    pub length: String,
    /// "170 727 Aufrufe"
    pub views: String,
    /// "vor 1 Jahr"
    pub age: String,
    /// The first lines of the description, if YouTube gives a snippet.
    pub snippet: String,
}

/// A part to skip (SponsorBlock), in seconds.
#[derive(Debug, Clone, PartialEq)]
pub struct Skip {
    pub from: f64,
    pub to: f64,
    pub category: String,
}

/// Everything about one video once resolved: what to play and what to show.
#[derive(Debug, Clone)]
pub struct Watch {
    pub media: Media,
    pub channel: String,
    pub description: String,
    pub views: u64,
    pub height: u32,
    /// Subtitle tracks the video offers (WebVTT), own ones first.
    pub subtitles: Vec<SubTrack>,
}

/// One subtitle track.
#[derive(Debug, Clone, PartialEq)]
pub struct SubTrack {
    /// Language code as YouTube names it ("de", "en", "de-orig" …).
    pub lang: String,
    /// The name to show ("Deutsch", "Englisch (automatisch)" …).
    pub name: String,
    pub url: String,
    /// Made by speech recognition.
    pub auto: bool,
}

/// Subtitle cues come from the video family (WebVTT/SRT, `cue_at`).
pub use matrixkit_video::subtitles::{cue_at, Cue};

/// The subtitle tracks of a yt-dlp answer (pure — testable): the video's
/// own tracks, and of the automatic ones only the spoken language
/// («-orig») plus German and English — YouTube offers a hundred machine
/// translations, a list nobody scrolls on a phone.
/// German name of a language code (YouTube names them in English).
pub fn language_name(code: &str) -> String {
    let base = code.split(['-', '_']).next().unwrap_or(code);
    let name = match base {
        "de" => "Deutsch",
        "en" => "Englisch",
        "fr" => "Französisch",
        "it" => "Italienisch",
        "es" => "Spanisch",
        "pt" => "Portugiesisch",
        "nl" => "Niederländisch",
        "rm" => "Rätoromanisch",
        "tr" => "Türkisch",
        "ru" => "Russisch",
        "uk" => "Ukrainisch",
        "pl" => "Polnisch",
        "ar" => "Arabisch",
        "ja" => "Japanisch",
        "zh" => "Chinesisch",
        "ko" => "Koreanisch",
        _ => return code.to_string(),
    };
    name.to_string()
}

pub fn subtracks_from(answer: &Value) -> Vec<SubTrack> {
    let mut out = Vec::new();
    let autos: Vec<String> = answer.get("automatic_captions").and_then(Value::as_object).map(|m| m.keys().cloned().collect()).unwrap_or_default();
    for (key, auto) in [("subtitles", false), ("automatic_captions", true)] {
        let Some(map) = answer.get(key).and_then(Value::as_object) else { continue };
        for (lang, formats) in map {
            if lang == "live_chat" {
                continue;
            }
            if auto && !(lang.ends_with("-orig") || lang == "de" || lang == "en") {
                continue;
            }
            // «de» next to «de-orig» is the same speech translated onto itself.
            if auto && !lang.ends_with("-orig") && autos.iter().any(|k| *k == format!("{lang}-orig")) {
                continue;
            }
            let Some(list) = formats.as_array() else { continue };
            let Some(vtt) = list.iter().find(|f| f.get("ext").and_then(Value::as_str) == Some("vtt")) else { continue };
            let Some(url) = vtt.get("url").and_then(Value::as_str) else { continue };
            let base = language_name(lang);
            let name = match (auto, lang.ends_with("-orig")) {
                (true, true) => format!("{base} (Originalton, automatisch)"),
                (true, false) => format!("{base} (automatisch übersetzt)"),
                _ => base,
            };
            out.push(SubTrack { lang: lang.clone(), name, url: url.to_string(), auto });
        }
    }
    out
}

/// Fetch a subtitle track (empty on failure).
pub fn cues(track: &SubTrack) -> Vec<Cue> {
    matrixkit_video::subtitles::fetch(&track.url)
}


const UA: &str = "Mozilla/5.0 (X11; Linux aarch64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36";

/// POST JSON through curl, answer parsed (None on any failure).
fn post_json(url: &str, body: &Value) -> Option<Value> {
    let out = Command::new("curl")
        .args(["-s", "--max-time", "15", "-A", UA, "-H", "Content-Type: application/json", "--data"])
        .arg(body.to_string())
        .arg(url)
        .output()
        .ok()?;
    serde_json::from_slice(&out.stdout).ok()
}

/// Text of a YouTube text object: `simpleText` or the joined `runs`.
fn text(v: &Value) -> String {
    if let Some(s) = v.get("simpleText").and_then(Value::as_str) {
        return s.to_string();
    }
    v.get("runs")
        .and_then(Value::as_array)
        .map(|runs| runs.iter().filter_map(|r| r.get("text").and_then(Value::as_str)).collect())
        .unwrap_or_default()
}

/// Every object stored under `key` anywhere in `v` (depth first, in order).
fn collect<'a>(v: &'a Value, key: &str, out: &mut Vec<&'a Value>) {
    match v {
        Value::Object(map) => {
            for (k, child) in map {
                if k == key {
                    out.push(child);
                } else {
                    collect(child, key, out);
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|i| collect(i, key, out)),
        _ => {}
    }
}

/// The hits of a search answer (pure — testable against saved answers).
pub fn hits_from(answer: &Value) -> Vec<Hit> {
    let mut found = Vec::new();
    collect(answer, "videoRenderer", &mut found);
    found
        .into_iter()
        .filter_map(|v| {
            let id = v.get("videoId")?.as_str()?.to_string();
            let snippet = v
                .get("detailedMetadataSnippets")
                .and_then(|s| s.get(0))
                .and_then(|s| s.get("snippetText"))
                .map(text)
                .unwrap_or_default();
            Some(Hit {
                id,
                title: v.get("title").map(text).unwrap_or_default(),
                channel: v.get("ownerText").map(text).unwrap_or_default(),
                length: v.get("lengthText").map(text).unwrap_or_default(),
                views: v.get("shortViewCountText").or_else(|| v.get("viewCountText")).map(text).unwrap_or_default(),
                age: v.get("publishedTimeText").map(text).unwrap_or_default(),
                snippet,
            })
        })
        .collect()
}

/// Search YouTube (Swiss German interface language, Swiss region).
pub fn search(query: &str) -> Vec<Hit> {
    let body = serde_json::json!({
        "context": {"client": {"clientName": "WEB", "clientVersion": "2.20251101.00.00", "hl": "de", "gl": "CH"}},
        "query": query,
    });
    post_json("https://www.youtube.com/youtubei/v1/search?prettyPrint=false", &body)
        .map(|a| hits_from(&a))
        .unwrap_or_default()
}

/// YouTube's «Als Nächstes» for a video (pure — testable): the
/// recommendation lockups of a `next` answer — id, title, channel, views,
/// age, length. Playlists and shorts shelves are left out.
pub fn related_from(answer: &Value) -> Vec<Hit> {
    let mut lockups = Vec::new();
    collect(answer, "lockupViewModel", &mut lockups);
    let content = |v: &Value| v.get("content").and_then(Value::as_str).unwrap_or_default().to_string();
    lockups
        .into_iter()
        .filter(|l| l.get("contentType").and_then(Value::as_str) == Some("LOCKUP_CONTENT_TYPE_VIDEO"))
        .filter_map(|l| {
            let id = l.get("contentId")?.as_str()?.to_string();
            let meta = l.pointer("/metadata/lockupMetadataViewModel")?;
            let title = meta.get("title").map(content).unwrap_or_default();
            let rows: Vec<&Value> = meta
                .pointer("/metadata/contentMetadataViewModel/metadataRows")
                .and_then(Value::as_array)
                .map(|r| r.iter().collect())
                .unwrap_or_default();
            let part = |row: usize, k: usize| -> String {
                rows.get(row)
                    .and_then(|r| r.get("metadataParts"))
                    .and_then(Value::as_array)
                    .and_then(|p| p.get(k))
                    .and_then(|p| p.get("text"))
                    .map(content)
                    .unwrap_or_default()
            };
            let mut badges = Vec::new();
            collect(l, "thumbnailBadgeViewModel", &mut badges);
            let length = badges.first().and_then(|b| b.get("text")).and_then(Value::as_str).unwrap_or_default().to_string();
            let views = rows
                .get(1)
                .and_then(|r| r.pointer("/metadataParts/0/accessibilityLabel"))
                .and_then(Value::as_str)
                .map(String::from)
                .unwrap_or_else(|| part(1, 0));
            Some(Hit { id, title, channel: part(0, 0), length, views, age: part(1, 1), snippet: String::new() })
        })
        .collect()
}

/// Ask YouTube what comes next after `id` (empty on failure).
pub fn related(id: &str) -> Vec<Hit> {
    let body = serde_json::json!({
        "context": {"client": {"clientName": "WEB", "clientVersion": "2.20251101.00.00", "hl": "de", "gl": "CH"}},
        "videoId": id,
    });
    post_json("https://www.youtube.com/youtubei/v1/next?prettyPrint=false", &body)
        .map(|a| related_from(&a))
        .unwrap_or_default()
}

/// The video quality to ask for — HD, measured on the duo (01.11.2026):
/// 1. up to `max_height` (1080p) when it exists at ≤30 fps;
/// 2. else 720p at any rate — every 60 fps picture must be DECODED even
///    when only 30 reach the screen; on a 1350-px panel 720p looks the same;
/// 3. else the best up to `max_height`, and the combined 360p stream 18
///    (no token needed) as the last resort.
///
/// Codec order (round 53, 03.11.2026, measured on the duo with the same
/// 1080p24 video): H.264 (avc1, OpenH264) ≈ 0.38 of a core, AV1 (dav1d)
/// 1.85–2.07 cores — five times the work for half the bytes. The duo's
/// video hardware has no AV1 at all (SM8150); H.264 first, VP9 next, AV1
/// only when nothing else exists. Costs about twice the download.
pub fn format_choice(max_height: u32) -> String {
    // Within one height/rate rule: avc1, then vp09, then whatever is left.
    let tiers = |filter: &str| {
        // Round 128: AAC sound (m4a) first — fragmented MP4 with an index, so
        // a seek fetches from its fragment instead of one throttled request.
        format!("bv*{filter}[vcodec^=avc1]+ba[ext=m4a]/bv*{filter}[vcodec^=avc1]+ba/bv*{filter}[vcodec^=vp09]+ba/bv*{filter}+ba/")
    };
    // A lower choice (quality menu: 480, 360 …) asks for exactly that or less.
    if max_height < 720 {
        return format!("{}b[height<={h}]/18", tiers(&format!("[height<={h}]", h = max_height)), h = max_height);
    }
    format!(
        "{}{}bv*[height<={h}]+ba/18",
        tiers(&format!("[height<={h}][height>=720][fps<=30]", h = max_height)),
        tiers("[height<=720]"),
        h = max_height
    )
}

/// The stream helper's answer (pure — testable): picture and sound inputs
/// with headers, measures, texts.
pub fn watch_from(answer: &Value) -> Option<Watch> {
    let formats: Vec<&Value> = match answer.get("requested_formats").and_then(Value::as_array) {
        Some(list) if !list.is_empty() => list.iter().collect(),
        _ => vec![answer],
    };
    let inputs: Vec<Input> = formats
        .iter()
        .filter_map(|f| {
            let url = f.get("url")?.as_str()?.to_string();
            let headers = f
                .get("http_headers")
                .and_then(Value::as_object)
                .map(|h| {
                    h.iter()
                        .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                        .collect()
                })
                .unwrap_or_default();
            // Total bit rate in kbit/s -> bytes per second (the stock's size).
            let byterate = f.get("tbr").and_then(Value::as_f64).unwrap_or(0.0) * 1000.0 / 8.0;
            Some(Input { url, headers, byterate })
        })
        .collect();
    if inputs.is_empty() {
        return None;
    }
    let picture = formats.iter().find(|f| f.get("vcodec").and_then(Value::as_str).is_some_and(|c| c != "none"))?;
    let width = picture.get("width").and_then(Value::as_u64).unwrap_or(0) as u32;
    let height = picture.get("height").and_then(Value::as_u64).unwrap_or(0) as u32;
    let fps = picture.get("fps").and_then(Value::as_f64).unwrap_or(30.0);
    Some(Watch {
        media: Media {
            inputs,
            duration_s: answer.get("duration").and_then(Value::as_f64).unwrap_or(0.0),
            fps: if fps > 0.0 { fps } else { 30.0 },
            codec: picture.get("vcodec").and_then(Value::as_str).unwrap_or_default().to_string(),
            width,
            height,
            title: answer.get("title").and_then(Value::as_str).unwrap_or_default().to_string(),
        },
        channel: answer.get("channel").or_else(|| answer.get("uploader")).and_then(Value::as_str).unwrap_or_default().to_string(),
        description: answer.get("description").and_then(Value::as_str).unwrap_or_default().to_string(),
        views: answer.get("view_count").and_then(Value::as_u64).unwrap_or(0),
        height,
        subtitles: subtracks_from(answer),
    })
}

// ---------------------------------------------------------------- the helper, kept current
//
// 07.11.2026 (freshly installed Duo: no sound, stuttering): YouTube changes its protection every few weeks, and a
// yt-dlp frozen into a system image stops working (Fedora 42 ships 2026.03.17: «HTTP Error 403», «SABR-only
// streaming»). Tube therefore keeps its OWN copy of the official release in ~/.local/share/matrix-tube/yt-dlp:
// fetched from the yt-dlp project's GitHub releases, installed only when its SHA-256 matches the published
// SHA2-256SUMS, renewed in the background when older than three days and at once when a video fails.
// Fixing a YouTube client (mweb) broke too: the helper's own default choice is the one its authors maintain.

const HELPER_BASE: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download";
const HELPER_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(3 * 24 * 3600);

fn own_helper() -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    home.join(".local/share/matrix-tube/yt-dlp")
}

/// The yt-dlp Tube runs: its own current copy, else the system's.
pub fn helper() -> PathBuf {
    let own = own_helper();
    if own.is_file() { own } else { PathBuf::from("yt-dlp") }
}

fn helper_command() -> Command {
    let mut c = Command::new(helper());
    c.args(["--js-runtimes", "node", "--remote-components", "ejs:github", "--no-update"]);
    c
}

/// Fetch the newest official yt-dlp and install it only if its SHA-256 matches the release's list. Blocking (a few
/// seconds); `Err` says why nothing changed. Never replaces a working copy with a broken download.
pub fn update_helper() -> Result<String, String> {
    let target = own_helper();
    let dir = target.parent().ok_or("kein Ordner")?.to_path_buf();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Ordner: {e}"))?;
    let part = dir.join("yt-dlp.neu");
    let sums = dir.join("SHA2-256SUMS.neu");
    let fetch = |url: &str, to: &std::path::Path| {
        Command::new("curl")
            .args(["-fsSL", "--max-time", "120", "-o"])
            .arg(to)
            .arg(url)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };
    let cleanup = || {
        let _ = std::fs::remove_file(&part);
        let _ = std::fs::remove_file(&sums);
    };
    if !fetch(&format!("{HELPER_BASE}/yt-dlp"), &part) || !fetch(&format!("{HELPER_BASE}/SHA2-256SUMS"), &sums) {
        cleanup();
        return Err(String::from("yt-dlp: Download fehlgeschlagen"));
    }
    let want = std::fs::read_to_string(&sums)
        .unwrap_or_default()
        .lines()
        .find_map(|l| {
            let mut it = l.split_whitespace();
            let (sum, name) = (it.next()?, it.next()?);
            (name == "yt-dlp").then(|| sum.to_lowercase())
        })
        .unwrap_or_default();
    let have = Command::new("sha256sum")
        .arg(&part)
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).split_whitespace().next().map(str::to_lowercase))
        .unwrap_or_default();
    if want.len() != 64 || want != have {
        cleanup();
        return Err(String::from("yt-dlp: Prüfsumme passt nicht — nichts installiert"));
    }
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&part, std::fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
    std::fs::rename(&part, &target).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&sums);
    let version = Command::new(&target).arg("--version").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    eprintln!("[tube] yt-dlp erneuert: {version}");
    Ok(version)
}

/// At start: renew the own copy in the background when it is missing or older than three days.
pub fn keep_helper_current() {
    let own = own_helper();
    let stale = std::fs::metadata(&own)
        .and_then(|m| m.modified())
        .map(|t| t.elapsed().map(|e| e > HELPER_MAX_AGE).unwrap_or(true))
        .unwrap_or(true);
    if stale {
        std::thread::spawn(|| {
            if let Err(e) = update_helper() {
                eprintln!("[tube] {e}");
            }
        });
    }
}

/// Resolve a video through the helper (takes a few seconds: the JS
/// challenge). `Err` carries yt-dlp's last line for the user. A failure renews the helper once and tries again
/// (YouTube's changes show up exactly like this).
pub fn resolve(id: &str, max_height: u32) -> Result<Watch, String> {
    match resolve_once(id, max_height) {
        Ok(w) => Ok(w),
        Err(first) => match update_helper() {
            Ok(_) => resolve_once(id, max_height),
            Err(_) => Err(first),
        },
    }
}

fn resolve_once(id: &str, max_height: u32) -> Result<Watch, String> {
    let out = helper_command()
        .args(["--no-warnings", "-j", "-f"])
        .arg(format_choice(max_height))
        .arg(format!("https://www.youtube.com/watch?v={id}"))
        .output()
        .map_err(|_| String::from("yt-dlp fehlt"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(err.lines().last().unwrap_or("yt-dlp scheiterte").to_string());
    }
    let answer: Value = serde_json::from_slice(&out.stdout).map_err(|_| String::from("yt-dlp: keine Antwort"))?;
    watch_from(&answer).ok_or_else(|| String::from("yt-dlp: kein abspielbares Format"))
}

/// What a download keeps: the sound (MP3) or the video (MP4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keep {
    Mp3,
    Mp4,
}

impl Keep {
    pub fn word(self) -> &'static str {
        match self {
            Keep::Mp3 => "mp3",
            Keep::Mp4 => "mp4",
        }
    }
}

/// An XDG user folder (MUSIC, VIDEOS) with a Matrix Tube folder inside.
fn user_folder(kind: &str, fallback: &str) -> PathBuf {
    let base = Command::new("xdg-user-dir")
        .arg(kind)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(fallback));
    base.join("Matrix Tube")
}

/// DOWNLOAD (R210, the owner 02.11.2026: «für unterwegs»): the sound as MP3
/// into ~/Musik/Matrix Tube, or the video (720p, H.264 + AAC when offered —
/// plays everywhere) as MP4 into ~/Videos/Matrix Tube. `progress` hears the
/// percent as yt-dlp reports it. The finished file joins the library.
pub fn download(id: &str, keep: Keep, progress: impl Fn(f32)) -> Result<library::Item, String> {
    use std::io::BufRead;
    let dir = match keep {
        Keep::Mp3 => user_folder("MUSIC", "Musik"),
        Keep::Mp4 => user_folder("VIDEOS", "Videos"),
    };
    std::fs::create_dir_all(&dir).map_err(|e| format!("Ordner: {e}"))?;
    let mut cmd = helper_command();
    cmd.args([
        "--no-warnings", "--newline", "--progress",
        "--progress-template", "download:%(progress._percent_str)s",
        "--print", "after_move:%(filepath)s\t%(title)s\t%(channel)s\t%(duration)s",
        "-o",
    ])
    .arg(dir.join("%(title).120B [%(id)s].%(ext)s"));
    match keep {
        Keep::Mp3 => cmd.args(["-f", "ba", "-x", "--audio-format", "mp3", "--audio-quality", "0", "--embed-metadata"]),
        Keep::Mp4 => cmd.args([
            "-f", "bv*[height<=720][vcodec^=avc1]+ba[ext=m4a]/bv*[height<=720]+ba/b[height<=720]/18",
            "--merge-output-format", "mp4", "--embed-metadata",
        ]),
    };
    cmd.arg(format!("https://www.youtube.com/watch?v={id}"))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = cmd.spawn().map_err(|_| String::from("yt-dlp fehlt"))?;
    // Round 178: stderr is drained on its own thread WHILE stdout is read — more than a pipe's worth of warnings
    // (64 KiB) used to block yt-dlp and this thread forever. Only the tail is kept.
    let errs = child.stderr.take().map(|e| {
        std::thread::spawn(move || {
            let mut tail: Vec<String> = Vec::new();
            for l in std::io::BufReader::new(e).lines().map_while(Result::ok) {
                tail.push(l);
                if tail.len() > 40 {
                    tail.remove(0);
                }
            }
            tail.join("\n")
        })
    });
    // The percent lines and, at the end, the finished file's line come on
    // stdout (measured on the duo 02.11.2026); errors on stderr.
    let mut line = None;
    if let Some(out) = child.stdout.take() {
        for l in std::io::BufReader::new(out).lines().map_while(Result::ok) {
            let t = l.trim();
            if t.contains('\t') {
                line = Some(t.to_string());
            } else if let Some(pct) = t.strip_suffix('%').and_then(|n| n.trim().parse::<f32>().ok()) {
                progress(pct);
            }
        }
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    let err = errs.and_then(|h| h.join().ok()).unwrap_or_default();
    let line = line.ok_or_else(|| err.lines().last().map(String::from).unwrap_or_else(|| format!("yt-dlp scheiterte ({status})")))?;
    let parts: Vec<&str> = line.split('\t').collect();
    let path = PathBuf::from(parts.first().copied().unwrap_or_default());
    // The cover: the search's thumbnail, kept beside the library.
    let thumb = thumbnail(id).and_then(|t| {
        let to = library::dir().join("bilder").join(format!("{id}.jpg"));
        std::fs::create_dir_all(to.parent()?).ok()?;
        std::fs::copy(&t, &to).ok()?;
        Some(to)
    });
    let item = library::Item {
        id: id.to_string(),
        keep,
        path,
        title: parts.get(1).map(|s| s.to_string()).unwrap_or_default(),
        channel: parts.get(2).map(|s| s.to_string()).unwrap_or_default(),
        duration: parts.get(3).and_then(|d| d.parse().ok()).unwrap_or(0.0),
        thumb,
    };
    let mut lib = library::Library::load();
    lib.add(item.clone());
    lib.save();
    Ok(item)
}

/// THE LIBRARY (R210): what was downloaded, and playlists of it — one JSON
/// file in ~/.local/share/matrix-tube. Items vanish from it when their file
/// is gone (deleted in the file manager).
pub mod library {
    use super::Keep;
    use serde_json::{json, Value};
    use std::path::PathBuf;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Item {
        pub id: String,
        pub keep: Keep,
        pub path: PathBuf,
        pub title: String,
        pub channel: String,
        pub duration: f64,
        pub thumb: Option<PathBuf>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Playlist {
        pub name: String,
        /// Files in order.
        pub items: Vec<PathBuf>,
    }

    #[derive(Debug, Clone, Default, PartialEq)]
    pub struct Library {
        pub items: Vec<Item>,
        pub playlists: Vec<Playlist>,
    }

    pub fn dir() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share"));
        base.join("matrix-tube")
    }

    fn file() -> PathBuf {
        dir().join("bibliothek.json")
    }

    /// A finished download file as an item: «Title [id].mp3» (pure — tested).
    pub fn orphan(path: &std::path::Path, keep: Keep) -> Option<Item> {
        let name = path.file_name()?.to_str()?;
        let stem = name.strip_suffix(&format!(".{}", keep.word()))?;
        if stem.contains(".temp") || stem.rsplit('.').next().is_some_and(|t| t.starts_with('f') && t[1..].chars().all(|c| c.is_ascii_digit()) && t.len() > 1) {
            return None;
        }
        let (title, id) = match stem.rsplit_once(" [") {
            Some((t, rest)) => (t.to_string(), rest.trim_end_matches(']').to_string()),
            None => (stem.to_string(), String::new()),
        };
        let thumb = Some(dir().join("bilder").join(format!("{id}.jpg"))).filter(|t| !id.is_empty() && t.exists());
        Some(Item { id, keep, path: path.to_path_buf(), title, channel: String::new(), duration: 0.0, thumb })
    }

    impl Library {
        pub fn load() -> Library {
            let mut lib = std::fs::read(file())
                .ok()
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                .map(|v| Library::from_json(&v))
                .unwrap_or_default();
            lib.items.retain(|i| i.path.exists());
            lib.adopt();
            lib
        }

        /// ADOPT (02.11.2026): finished files in the two Matrix Tube folders
        /// the library does not know — a download whose window closed
        /// before it ended (yt-dlp finishes alone, nobody wrote it down).
        /// yt-dlp's half-made files (`.part`, `.temp.`, `.f137.`) stay out.
        pub fn adopt(&mut self) {
            for (keep, dir) in [(Keep::Mp3, super::user_folder("MUSIC", "Musik")), (Keep::Mp4, super::user_folder("VIDEOS", "Videos"))] {
                let Ok(entries) = std::fs::read_dir(&dir) else { continue };
                for path in entries.flatten().map(|e| e.path()) {
                    if self.items.iter().any(|i| i.path == path) {
                        continue;
                    }
                    if let Some(item) = orphan(&path, keep) {
                        self.items.push(item);
                    }
                }
            }
        }

        pub fn from_json(v: &Value) -> Library {
            let s = |x: &Value, k: &str| x.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
            let items = v
                .get("items")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .map(|x| Item {
                            id: s(x, "id"),
                            keep: if s(x, "keep") == "mp4" { Keep::Mp4 } else { Keep::Mp3 },
                            path: PathBuf::from(s(x, "path")),
                            title: s(x, "title"),
                            channel: s(x, "channel"),
                            duration: x.get("duration").and_then(Value::as_f64).unwrap_or(0.0),
                            thumb: x.get("thumb").and_then(Value::as_str).map(PathBuf::from),
                        })
                        .collect()
                })
                .unwrap_or_default();
            let playlists = v
                .get("playlists")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .map(|x| Playlist {
                            name: s(x, "name"),
                            items: x
                                .get("items")
                                .and_then(Value::as_array)
                                .map(|i| i.iter().filter_map(|p| p.as_str().map(PathBuf::from)).collect())
                                .unwrap_or_default(),
                        })
                        .collect()
                })
                .unwrap_or_default();
            Library { items, playlists }
        }

        pub fn to_json(&self) -> Value {
            json!({
                "items": self.items.iter().map(|i| json!({
                    "id": i.id, "keep": i.keep.word(), "path": i.path, "title": i.title,
                    "channel": i.channel, "duration": i.duration, "thumb": i.thumb,
                })).collect::<Vec<_>>(),
                "playlists": self.playlists.iter().map(|p| json!({"name": p.name, "items": p.items})).collect::<Vec<_>>(),
            })
        }

        pub fn save(&self) {
            let _ = std::fs::create_dir_all(dir());
            let tmp = file().with_extension("json.neu");
            if std::fs::write(&tmp, self.to_json().to_string()).is_ok() {
                let _ = std::fs::rename(&tmp, file());
            }
        }

        /// A finished download (a second download of the same file replaces it).
        pub fn add(&mut self, item: Item) {
            self.items.retain(|i| i.path != item.path);
            self.items.insert(0, item);
        }

        pub fn item(&self, path: &std::path::Path) -> Option<&Item> {
            self.items.iter().find(|i| i.path == path)
        }

        /// A new playlist (a free name: «Playlist 2» when «Playlist» exists).
        pub fn create(&mut self, name: &str) -> usize {
            let base = if name.trim().is_empty() { "Playlist" } else { name.trim() };
            let mut chosen = base.to_string();
            let mut n = 2;
            while self.playlists.iter().any(|p| p.name == chosen) {
                chosen = format!("{base} {n}");
                n += 1;
            }
            self.playlists.push(Playlist { name: chosen, items: Vec::new() });
            self.playlists.len() - 1
        }

        /// Add a file to a playlist (once).
        pub fn add_to(&mut self, playlist: usize, path: &std::path::Path) {
            if let Some(p) = self.playlists.get_mut(playlist) {
                if !p.items.iter().any(|i| i == path) {
                    p.items.push(path.to_path_buf());
                }
            }
        }

        /// Delete a download: the file, its cover and every mention.
        pub fn delete(&mut self, path: &std::path::Path) {
            if let Some(i) = self.items.iter().position(|i| i.path == path) {
                let item = self.items.remove(i);
                let _ = std::fs::remove_file(&item.path);
                if let Some(t) = item.thumb {
                    let _ = std::fs::remove_file(t);
                }
            }
            for p in &mut self.playlists {
                p.items.retain(|i| i != path);
            }
        }

        /// The files of a playlist that still exist, in order.
        pub fn queue(&self, playlist: usize) -> Vec<PathBuf> {
            self.playlists.get(playlist).map(|p| p.items.iter().filter(|i| i.exists()).cloned().collect()).unwrap_or_default()
        }

        /// Delete a playlist (its files stay — R210 round 19: «ich kann
        /// keine Playlists löschen»).
        pub fn remove_playlist(&mut self, playlist: usize) -> Option<Playlist> {
            (playlist < self.playlists.len()).then(|| self.playlists.remove(playlist))
        }

        /// Rename a playlist; a taken or empty name is refused.
        pub fn rename(&mut self, playlist: usize, name: &str) -> bool {
            let name = name.trim();
            if name.is_empty() || self.playlists.iter().enumerate().any(|(i, p)| i != playlist && p.name == name) {
                return false;
            }
            match self.playlists.get_mut(playlist) {
                Some(p) => {
                    p.name = name.to_string();
                    true
                }
                None => false,
            }
        }

        pub fn contains(&self, playlist: usize, path: &std::path::Path) -> bool {
            self.playlists.get(playlist).is_some_and(|p| p.items.iter().any(|i| i == path))
        }

        /// In → out, out → in; true when it is in afterwards.
        pub fn toggle_in(&mut self, playlist: usize, path: &std::path::Path) -> bool {
            if self.contains(playlist, path) {
                self.remove_from(playlist, path);
                false
            } else {
                self.add_to(playlist, path);
                self.contains(playlist, path)
            }
        }

        pub fn remove_from(&mut self, playlist: usize, path: &std::path::Path) {
            if let Some(p) = self.playlists.get_mut(playlist) {
                p.items.retain(|i| i != path);
            }
        }

        /// Total playing time of a playlist in seconds (known items).
        pub fn duration(&self, playlist: usize) -> f64 {
            self.playlists.get(playlist).map(|p| p.items.iter().filter_map(|f| self.item(f)).map(|i| i.duration).sum()).unwrap_or(0.0)
        }

        /// The playlist's cover: its first item's that has one.
        pub fn cover(&self, playlist: usize) -> Option<&std::path::Path> {
            self.playlists.get(playlist)?.items.iter().filter_map(|f| self.item(f)).find_map(|i| i.thumb.as_deref())
        }

        /// Bytes the downloads take on disk.
        pub fn size_bytes(&self) -> u64 {
            self.items.iter().filter_map(|i| std::fs::metadata(&i.path).ok()).map(|m| m.len()).sum()
        }

        /// Is this video id downloaded (as what)?
        pub fn kept(&self, id: &str) -> Vec<Keep> {
            self.items.iter().filter(|i| i.id == id).map(|i| i.keep).collect()
        }
    }

    /// The recent searches (newest first, at most `max`, no doubles —
    /// case-insensitive): pure, tested.
    pub fn push_recent(list: &mut Vec<String>, query: &str, max: usize) {
        let q = query.trim();
        if q.is_empty() {
            return;
        }
        list.retain(|s| !s.eq_ignore_ascii_case(q));
        list.insert(0, q.to_string());
        list.truncate(max);
    }

    fn history_file() -> PathBuf {
        dir().join("suchen.txt")
    }

    pub fn history_load() -> Vec<String> {
        std::fs::read_to_string(history_file()).map(|t| t.lines().filter(|l| !l.trim().is_empty()).map(String::from).collect()).unwrap_or_default()
    }

    pub fn history_save(list: &[String]) {
        let _ = std::fs::create_dir_all(dir());
        let _ = std::fs::write(history_file(), list.join("\n"));
    }

    #[cfg(test)]
    mod playlist_tests {
        use super::*;

        fn item(id: &str, path: &str, duration: f64, thumb: Option<&str>) -> Item {
            Item { id: id.into(), keep: Keep::Mp3, path: PathBuf::from(path), title: id.into(), channel: String::new(), duration, thumb: thumb.map(PathBuf::from) }
        }

        #[test]
        fn playlists_rename_toggle_remove() {
            let mut lib = Library::default();
            lib.add(item("a", "/m/a.mp3", 60.0, None));
            lib.add(item("b", "/m/b.mp3", 90.0, Some("/c/b.jpg")));
            let u = lib.create("Unterwegs");
            let c = lib.create("Compilations");
            assert!(lib.toggle_in(u, std::path::Path::new("/m/a.mp3")));
            assert!(lib.toggle_in(u, std::path::Path::new("/m/b.mp3")));
            assert_eq!(lib.duration(u), 150.0);
            assert_eq!(lib.cover(u), Some(std::path::Path::new("/c/b.jpg")));
            assert!(!lib.toggle_in(u, std::path::Path::new("/m/a.mp3")));
            assert_eq!(lib.playlists[u].items.len(), 1);
            assert!(!lib.rename(c, "Unterwegs"), "taken name");
            assert!(!lib.rename(c, "  "), "empty name");
            assert!(lib.rename(c, "Sport"));
            assert_eq!(lib.remove_playlist(u).map(|p| p.name), Some(String::from("Unterwegs")));
            assert_eq!(lib.playlists.len(), 1);
            assert!(lib.remove_playlist(5).is_none());
            assert_eq!(lib.kept("b"), vec![Keep::Mp3]);
            let mut recent = vec![String::from("Paramore"), String::from("Ashnikko")];
            push_recent(&mut recent, " ashnikko ", 3);
            push_recent(&mut recent, "Oasis", 3);
            push_recent(&mut recent, "Keane", 3);
            assert_eq!(recent, vec!["Keane", "Oasis", "ashnikko"]);
            assert!(lib.kept("z").is_empty());
        }
    }
}

/// The SponsorBlock categories Matrix Tube skips.
pub const SKIP_CATEGORIES: [&str; 5] = ["sponsor", "selfpromo", "interaction", "intro", "outro"];

/// SponsorBlock's answer (pure — testable).
pub fn skips_from(answer: &Value) -> Vec<Skip> {
    let mut skips: Vec<Skip> = answer
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|s| {
                    let seg = s.get("segment")?.as_array()?;
                    Some(Skip {
                        from: seg.first()?.as_f64()?,
                        to: seg.get(1)?.as_f64()?,
                        category: s.get("category")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    skips.sort_by(|a, b| a.from.total_cmp(&b.from));
    skips
}

/// Ask SponsorBlock for the parts to skip (none known = empty).
pub fn skips(id: &str) -> Vec<Skip> {
    let categories = serde_json::to_string(&SKIP_CATEGORIES).unwrap_or_default();
    let out = Command::new("curl")
        .args(["-s", "--max-time", "8", "-G", "https://sponsor.ajay.app/api/skipSegments"])
        .arg("--data-urlencode")
        .arg(format!("videoID={id}"))
        .arg("--data-urlencode")
        .arg(format!("categories={categories}"))
        .output();
    out.ok()
        .and_then(|o| serde_json::from_slice::<Value>(&o.stdout).ok())
        .map(|a| skips_from(&a))
        .unwrap_or_default()
}

/// Where a skip ends if `pos_s` lies inside one (with a small lead so the
/// jump lands after the boundary).
pub fn skip_target(skips: &[Skip], pos_s: f64) -> Option<f64> {
    skips.iter().find(|s| pos_s >= s.from && pos_s < s.to - 0.5).map(|s| s.to)
}

/// The video id in a YouTube link (watch?v=, youtu.be/, shorts/, embed/)
/// or a bare 11-character id — None for anything else (search words).
pub fn video_id(arg: &str) -> Option<String> {
    let valid = |s: &str| s.len() == 11 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    let arg = arg.trim();
    if valid(arg) && !arg.chars().all(|c| c.is_ascii_alphabetic()) {
        return Some(arg.to_string());
    }
    let cut = |s: &str| s.split(['&', '?', '#', '/']).next().unwrap_or("").to_string();
    for marker in ["watch?v=", "youtu.be/", "/shorts/", "/embed/", "/live/"] {
        if let Some(i) = arg.find(marker) {
            let id = cut(&arg[i + marker.len()..]);
            if valid(&id) {
                return Some(id);
            }
        }
    }
    if let Some(i) = arg.find("&v=") {
        let id = cut(&arg[i + 3..]);
        if valid(&id) {
            return Some(id);
        }
    }
    None
}

/// A medium as JSON — the hand-over to the background sound process.
pub fn media_json(media: &Media) -> Value {
    serde_json::json!({
        "title": media.title,
        "duration": media.duration_s,
        "fps": media.fps,
        "codec": media.codec,
        "width": media.width,
        "height": media.height,
        "inputs": media.inputs.iter().map(|i| serde_json::json!({
            "url": i.url,
            "headers": i.headers.iter().map(|(k, v)| serde_json::json!([k, v])).collect::<Vec<_>>(),
            "byterate": i.byterate,
        })).collect::<Vec<_>>(),
    })
}

/// The medium back from `media_json`.
pub fn media_from_json(v: &Value) -> Option<Media> {
    let inputs = v
        .get("inputs")?
        .as_array()?
        .iter()
        .filter_map(|i| {
            Some(Input {
                url: i.get("url")?.as_str()?.to_string(),
                headers: i
                    .get("headers")
                    .and_then(Value::as_array)
                    .map(|h| {
                        h.iter()
                            .filter_map(|p| Some((p.get(0)?.as_str()?.to_string(), p.get(1)?.as_str()?.to_string())))
                            .collect()
                    })
                    .unwrap_or_default(),
                byterate: i.get("byterate").and_then(Value::as_f64).unwrap_or(0.0),
            })
        })
        .collect::<Vec<_>>();
    (!inputs.is_empty()).then(|| Media {
        inputs,
        duration_s: v.get("duration").and_then(Value::as_f64).unwrap_or(0.0),
        fps: v.get("fps").and_then(Value::as_f64).unwrap_or(30.0),
        codec: v.get("codec").and_then(Value::as_str).unwrap_or_default().to_string(),
        width: v.get("width").and_then(Value::as_u64).unwrap_or(0) as u32,
        height: v.get("height").and_then(Value::as_u64).unwrap_or(0) as u32,
        title: v.get("title").and_then(Value::as_str).unwrap_or_default().to_string(),
    })
}

/// The cache folder of Matrix Tube.
pub fn cache_dir() -> PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache"));
    base.join("matrix-tube")
}

/// The thumbnail (320 × 180) of a video, fetched once into the cache.
pub fn thumbnail(id: &str) -> Option<PathBuf> {
    let dir = cache_dir().join("thumbs");
    let path = dir.join(format!("{id}.jpg"));
    if path.exists() {
        return Some(path);
    }
    std::fs::create_dir_all(&dir).ok()?;
    let ok = Command::new("curl")
        .args(["-s", "--max-time", "10", "-f", "-o"])
        .arg(&path)
        .arg(format!("https://i.ytimg.com/vi/{id}/mqdefault.jpg"))
        .status()
        .ok()?
        .success();
    if ok {
        Some(path)
    } else {
        let _ = std::fs::remove_file(&path);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hits_are_read_from_a_search_answer() {
        let answer = serde_json::json!({"contents": {"list": [
            {"videoRenderer": {
                "videoId": "abc",
                "title": {"runs": [{"text": "Züri "}, {"text": "Tram"}]},
                "ownerText": {"runs": [{"text": "Kanal"}]},
                "lengthText": {"simpleText": "4:20"},
                "shortViewCountText": {"simpleText": "1,2 Mio. Aufrufe"},
                "publishedTimeText": {"simpleText": "vor 2 Jahren"}
            }},
            {"reelShelfRenderer": {}}
        ]}});
        let hits = hits_from(&answer);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Züri Tram");
        assert_eq!(hits[0].length, "4:20");
    }

    #[test]
    fn watch_takes_picture_and_sound_with_headers() {
        let answer = serde_json::json!({
            "title": "T", "duration": 252.0, "channel": "K",
            "requested_formats": [
                {"url": "v", "vcodec": "av01", "width": 1920, "height": 1080, "fps": 24.0, "http_headers": {"User-Agent": "UA"}},
                {"url": "a", "vcodec": "none", "acodec": "opus"}
            ]
        });
        let w = watch_from(&answer).unwrap();
        assert_eq!(w.media.inputs.len(), 2);
        assert_eq!(w.media.inputs[0].headers, vec![("User-Agent".into(), "UA".into())]);
        assert_eq!((w.media.width, w.media.height, w.media.fps), (1920, 1080, 24.0));
    }

    #[test]
    fn ids_come_out_of_links() {
        assert_eq!(video_id("https://www.youtube.com/watch?v=9bZkp7q19f0&t=10").as_deref(), Some("9bZkp7q19f0"));
        assert_eq!(video_id("https://youtu.be/9bZkp7q19f0?si=x").as_deref(), Some("9bZkp7q19f0"));
        assert_eq!(video_id("https://youtube.com/shorts/9bZkp7q19f0").as_deref(), Some("9bZkp7q19f0"));
        assert_eq!(video_id("9bZkp7q19f0").as_deref(), Some("9bZkp7q19f0"));
        assert_eq!(video_id("Strassenbahn"), None);
        assert_eq!(video_id("Zuerichtram"), None);
    }

    #[test]
    fn media_survives_the_hand_over() {
        let m = Media {
            inputs: vec![Input { url: "u".into(), headers: vec![("A".into(), "b".into())], byterate: 9000.0 }],
            duration_s: 10.0,
            fps: 24.0,
            codec: "avc1.64001f".into(),
            width: 2,
            height: 2,
            title: "T".into(),
        };
        assert_eq!(media_from_json(&media_json(&m)), Some(m));
    }

    #[test]
    fn skips_sort_and_target() {
        let answer = serde_json::json!([
            {"segment": [60.0, 75.0], "category": "outro"},
            {"segment": [10.0, 30.0], "category": "sponsor"}
        ]);
        let s = skips_from(&answer);
        assert_eq!(s[0].category, "sponsor");
        assert_eq!(skip_target(&s, 12.0), Some(30.0));
        assert_eq!(skip_target(&s, 40.0), None);
        assert_eq!(skips_from(&serde_json::json!("Not Found")), Vec::new());
    }

    #[test]
    fn subtitle_tracks_keep_own_and_few_automatic() {
        let a = serde_json::json!({
            "subtitles": {"de": [{"ext": "json3", "url": "x"}, {"ext": "vtt", "url": "u-de", "name": "Deutsch"}]},
            "automatic_captions": {
                "de-orig": [{"ext": "vtt", "url": "u-auto", "name": "Deutsch"}],
                "fr": [{"ext": "vtt", "url": "u-fr", "name": "Französisch"}]
            }
        });
        let t = subtracks_from(&a);
        assert_eq!(t.len(), 2);
        assert_eq!((t[0].name.as_str(), t[0].auto), ("Deutsch", false));
        assert_eq!(t[1].name, "Deutsch (Originalton, automatisch)");
    }

    #[test]
    fn h264_first_av1_last() {
        let c = format_choice(1080);
        let (avc, vp9) = (c.find("vcodec^=avc1").unwrap(), c.find("vcodec^=vp09").unwrap());
        assert!(avc < vp9, "H.264 before VP9");
        assert!(!c.contains("av01"), "AV1 never asked for by name");
        assert!(c.ends_with("/18"));
        // AAC sound first (a seek fetches its fragments), any sound after.
        assert!(c.starts_with("bv*[height<=1080][height>=720][fps<=30][vcodec^=avc1]+ba[ext=m4a]/bv*[height<=1080][height>=720][fps<=30][vcodec^=avc1]+ba/"));
    }

    #[test]
    fn low_quality_asks_for_low() {
        assert!(format_choice(480).contains("height<=480"));
        assert!(!format_choice(480).contains("height<=720"));
    }

    #[test]
    fn related_reads_video_lockups_only() {
        let a = serde_json::json!({"contents": {"results": [
            {"lockupViewModel": {
                "contentId": "abcdefghijk", "contentType": "LOCKUP_CONTENT_TYPE_VIDEO",
                "contentImage": {"thumbnailViewModel": {"overlays": [{"thumbnailBottomOverlayViewModel": {"badges": [{"thumbnailBadgeViewModel": {"text": "22:33"}}]}}]}},
                "metadata": {"lockupMetadataViewModel": {
                    "title": {"content": "Kassensturz"},
                    "metadata": {"contentMetadataViewModel": {"metadataRows": [
                        {"metadataParts": [{"text": {"content": "SRF News"}}]},
                        {"metadataParts": [{"text": {"content": "155.584"}, "accessibilityLabel": "155.584 Aufrufe"}, {"text": {"content": "vor 10 Tagen"}}]}
                    ]}}
                }}
            }},
            {"lockupViewModel": {"contentId": "PLxyz", "contentType": "LOCKUP_CONTENT_TYPE_PLAYLIST"}}
        ]}});
        let r = related_from(&a);
        assert_eq!(r.len(), 1);
        assert_eq!((r[0].id.as_str(), r[0].title.as_str(), r[0].channel.as_str()), ("abcdefghijk", "Kassensturz", "SRF News"));
        assert_eq!((r[0].views.as_str(), r[0].age.as_str(), r[0].length.as_str()), ("155.584 Aufrufe", "vor 10 Tagen", "22:33"));
    }

    #[test]
    fn library_playlists_round_trip() {
        let mut lib = library::Library::default();
        lib.add(library::Item { id: "a".into(), keep: Keep::Mp3, path: "/x/a.mp3".into(), title: "A".into(), channel: "K".into(), duration: 61.0, thumb: None });
        let p = lib.create("Unterwegs");
        assert_eq!(lib.create("Unterwegs"), 1);
        assert_eq!(lib.playlists[1].name, "Unterwegs 2");
        lib.add_to(p, std::path::Path::new("/x/a.mp3"));
        lib.add_to(p, std::path::Path::new("/x/a.mp3"));
        assert_eq!(lib.playlists[0].items.len(), 1);
        let back = library::Library::from_json(&lib.to_json());
        assert_eq!(back, lib);
    }

    #[test]
    fn orphans_only_finished_files() {
        let a = library::orphan(std::path::Path::new("/m/Me at the zoo [jNQXAC9IVRw].mp3"), Keep::Mp3).unwrap();
        assert_eq!((a.title.as_str(), a.id.as_str()), ("Me at the zoo", "jNQXAC9IVRw"));
        assert!(library::orphan(std::path::Path::new("/m/x [id].temp.mp4"), Keep::Mp4).is_none());
        assert!(library::orphan(std::path::Path::new("/m/x [id].f137.mp4"), Keep::Mp4).is_none());
        assert!(library::orphan(std::path::Path::new("/m/x [id].mp4.part"), Keep::Mp4).is_none());
        assert!(library::orphan(std::path::Path::new("/m/x [id].webm"), Keep::Mp4).is_none());
    }
}

/// THE DOWNLOAD QUEUE (R210 family, round 18, 02.11.2026): one download
/// runs, the others wait their turn — a second tap while an album
/// converted was dropped silently before. Pure bookkeeping, tested; the
/// app runs the job it is handed.
pub mod downloads {
    use super::Keep;

    /// One download: the video id, its title (for «im Hintergrund»), kind.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Job {
        pub id: String,
        pub title: String,
        pub keep: Keep,
        /// Percent as yt-dlp reports it; ≥ 100 = the file is there and is
        /// being converted (MP3) or merged (MP4).
        pub percent: f32,
    }

    impl Job {
        pub fn new(id: &str, title: &str, keep: Keep) -> Job {
            Job { id: id.to_string(), title: title.to_string(), keep, percent: 0.0 }
        }

        /// What it does right now, in words.
        pub fn phase(&self) -> String {
            match (self.percent >= 100.0, self.keep) {
                (true, Keep::Mp3) => String::from("Wandelt in MP3 um …"),
                (true, Keep::Mp4) => String::from("Fügt MP4 zusammen …"),
                _ => format!("Lädt {} herunter … {:.0} %", self.keep.word().to_uppercase(), self.percent),
            }
        }
    }

    /// What a push did.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Pushed {
        /// Nothing ran — start this one now.
        Start,
        /// Waits; this many are ahead of it (the running one included).
        Waits(usize),
        /// Already running or waiting.
        Twice,
    }

    #[derive(Debug, Default)]
    pub struct Queue {
        running: Option<Job>,
        waiting: Vec<Job>,
    }

    impl Queue {
        pub fn push(&mut self, job: Job) -> Pushed {
            let same = |j: &Job| j.id == job.id && j.keep == job.keep;
            if self.running.as_ref().is_some_and(same) || self.waiting.iter().any(same) {
                return Pushed::Twice;
            }
            if self.running.is_none() {
                self.running = Some(job);
                return Pushed::Start;
            }
            self.waiting.push(job);
            Pushed::Waits(self.waiting.len())
        }

        pub fn running(&self) -> Option<&Job> {
            self.running.as_ref()
        }

        pub fn progress(&mut self, percent: f32) {
            if let Some(j) = &mut self.running {
                j.percent = percent;
            }
        }

        /// The running one ended: it leaves, the next (if any) is running
        /// now and returned so the app starts it.
        pub fn finish(&mut self) -> (Option<Job>, Option<Job>) {
            let done = self.running.take();
            if !self.waiting.is_empty() {
                self.running = Some(self.waiting.remove(0));
            }
            (done, self.running.clone())
        }

        pub fn waits(&self, id: &str, keep: Keep) -> bool {
            self.waiting.iter().any(|j| j.id == id && j.keep == keep)
        }

        pub fn busy(&self) -> bool {
            self.running.is_some() || !self.waiting.is_empty()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn one_runs_the_others_wait_in_order() {
            let mut q = Queue::default();
            assert_eq!(q.push(Job::new("a", "Album", Keep::Mp3)), Pushed::Start);
            assert_eq!(q.push(Job::new("b", "Hard Times", Keep::Mp4)), Pushed::Waits(1));
            assert_eq!(q.push(Job::new("a", "Album", Keep::Mp3)), Pushed::Twice);
            assert_eq!(q.push(Job::new("a", "Album", Keep::Mp4)), Pushed::Waits(2));
            assert!(q.waits("b", Keep::Mp4) && !q.waits("b", Keep::Mp3));
            q.progress(100.0);
            assert_eq!(q.running().unwrap().phase(), "Wandelt in MP3 um …");
            let (done, next) = q.finish();
            assert_eq!(done.unwrap().id, "a");
            assert_eq!(next.as_ref().unwrap().id, "b");
            assert_eq!(next.unwrap().phase(), "Lädt MP4 herunter … 0 %");
            let _ = q.finish();
            let (_, last) = q.finish();
            assert!(last.is_none() && !q.busy());
        }
    }
}
