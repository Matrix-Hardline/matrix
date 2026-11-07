//! Matrix Aufnahme — app #29 (R69): screenshot and screen recording
//! in the reference grammar, read empirically from the reference system.
//!
//! The capture panel: ONE small panel bundles all capture kinds
//! (whole screen, region, window, recording). After every photo the
//! THUMBNAIL floats bottom right (~6 s, reference behavior): a click
//! opens, ✕ discards the preview — the file has long been saved.
//! File names speak reference German: "Bildschirmfoto 2026-07-17 um
//! 00.31.12.png". While a recording runs, the BAR shows the red stop
//! dot (reference menu-bar behavior) — a click ends it.
//!
//! Backend: the photo paths go through the canvas facade
//! (screenshot/-screen/-window, inkl. niris eingebauter Bereichswahl);
//! the recording via wf-recorder (screencopy — the fork speaks it; the
//! tool ships with the image, round 69).

use matrixkit_widgets::iced;
use matrixkit_widgets::iced_layershell;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use matrixkit_widgets::iced::widget::{column, container, image, row};
use matrixkit_widgets::iced::{Color, Element, Font, Length, Subscription, Task};
use matrixkit_widgets::iced_layershell::to_layer_message;
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;

/// Float duration of the thumbnail — the reference shows it ~6 s.
const THUMB_VISIBLE_S: u64 = 6;
const THUMB_B: f32 = 236.0;
const THUMB_H: f32 = 156.0;

/// XDG user folders from `user-dirs.dirs` — German on Matrix
/// (PICTURES → ~/Bilder), with fallback to the German name.
fn xdg_folder(key: &str, fallback: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    let conf = PathBuf::from(&home).join(".config/user-dirs.dirs");
    if let Ok(text) = std::fs::read_to_string(conf) {
        let prefix = format!("XDG_{key}_DIR=\"");
        for line in text.lines() {
            if let Some(rest) = line.trim().strip_prefix(&prefix) {
                let value = rest.trim_end_matches('"').replace("$HOME", &home);
                return PathBuf::from(value);
            }
        }
    }
    PathBuf::from(home).join(fallback)
}

/// Photos land in "Bilder/Bildschirmfotos" — the gallery of Matrix
/// Dateien looks there. MUST match the compositor's screenshot-path
/// (the wait loop counts files in this folder).
fn screen_folder() -> PathBuf {
    xdg_folder("PICTURES", "Bilder").join("Bildschirmfotos")
}

/// Recordings land in "Videos/Bildschirmaufnahmen" — the second
/// Gallery-Wurzel von Matrix Dateien.
fn film_folder() -> PathBuf {
    xdg_folder("VIDEOS", "Videos").join("Bildschirmaufnahmen")
}

/// Reference German: "Bildschirmfoto 2026-07-17 um 00.31.12.png".
fn time_stamp_name(prefix: &str, extension: &str) -> String {
    let now = std::process::Command::new("date")
        .arg("+%Y-%m-%d um %H.%M.%S")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    format!("{prefix} {now}.{extension}")
}

/// Is a usable NVIDIA encoder there? (The device node suffices as a
/// fast, reliable anchor — yes on the PC, no on the Surface.)
fn nvenc_present() -> bool {
    std::path::Path::new("/dev/nvidia0").exists()
}

fn recording_status_file() -> PathBuf {
    let run = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(run).join("matrix-capture-recording")
}

/// Stored recording target height ("1080") — missing or "nativ" means:
/// full display resolution, the default.
fn film_height() -> Option<u32> {
    mk::settings::read("capture-resolution")?.parse().ok()
}

/// Physical display resolution ("Current mode" of the canvas) — labels
/// the native entry and filters steps that would not shrink.
fn display_resolution() -> Option<(u32, u32)> {
    let out = mk::canvas::raw(&["msg", "outputs"])?;
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let line = text.lines().find(|z| z.contains("Current mode:"))?;
    let mode = line.split("Current mode:").nth(1)?.trim().split_whitespace().next()?;
    let (b, h) = mode.split_once('x')?;
    Some((b.parse().ok()?, h.parse().ok()?))
}

mod licht;

fn main() -> Result<(), iced_layershell::Error> {
    matrixkit_widgets::present_mailbox();
    let args: Vec<String> = std::env::args().collect();
    // Stage 67 of the iced replacement: --erstlicht was the window-house path.
    // Round 64: the house is the DEFAULT for the two surfaces (the panel
    // and the floating thumbnail) — `--iced` is the way back. The capture
    // modes themselves (photo, region, recording …) have no surface.
    let iced = args.iter().any(|a| a == "--iced");
    let mode = args.iter().skip(1).find(|a| !a.starts_with("--")).map(String::as_str).unwrap_or("panel");
    if !iced && (mode == "panel" || mode == "thumb") {
        let thumb = args.iter().position(|a| a == "thumb").and_then(|i| args.get(i + 1));
        let outcome = match thumb {
            Some(path) => licht::run_thumb(PathBuf::from(path)),
            None => licht::run_panel(
                recording_status_file().exists(),
                film_height(),
                display_resolution(),
            ),
        };
        if let Err(e) = outcome {
            eprintln!("[aufnahme] Erstes Licht: {e}");
        }
        return Ok(());
    }
    let arg = mode.to_string();
    match arg.as_str() {
        "photo" => {
            record("screenshot-screen");
            Ok(())
        }
        "region" => {
            // niri's built-in region picker — the user drags the frame.
            record("screenshot");
            Ok(())
        }
        "fenster" => {
            record("screenshot-window");
            Ok(())
        }
        // Round 131: one panel or both — the choice is remembered, and the
        // control centre's switch («recording») films like the last time.
        "recording" => {
            film_start();
            Ok(())
        }
        "recording-one" => {
            mk::settings::write("film-panels", "one");
            film_start();
            Ok(())
        }
        "recording-both" => {
            mk::settings::write("film-panels", "both");
            film_start();
            Ok(())
        }
        "venus-film" => {
            venus_film(std::env::args().skip(2).collect());
            Ok(())
        }
        "recording-stop" => {
            film_stop();
            Ok(())
        }
        "thumb" => {
            let path = std::env::args().nth(2).unwrap_or_default();
            thumb_ui(PathBuf::from(path))
        }
        _ => panel_ui(),
    }
}

// ------------------------------------------------------------ Aufnahme

/// Take a photo: trigger the action, wait for the NEW file (the
/// storage doctrine: count instead of believe), give the reference
/// name, show the thumbnail. Region picking may take its time (60 s).
fn record(action: &str) {
    let folder = screen_folder();
    let _ = std::fs::create_dir_all(&folder);
    let before: usize = std::fs::read_dir(&folder).map(|d| d.count()).unwrap_or(0);
    // Round 151: a photo of an opened duo shows BOTH panels, as they lie
    // (05.11.2026: «why not from both screens?» — niri's
    // screenshot-screen takes the focused output only).
    if action == "screenshot-screen" && both_panels() {
        let target = folder.join(time_stamp_name("Bildschirmfoto", "png"));
        let ok = photo_stacked(&target);
        if ok {
            // As niri does: the photo is on the clipboard too.
            if let Ok(file) = std::fs::File::open(&target) {
                let _ = std::process::Command::new("wl-copy").args(["--type", "image/png"]).stdin(file).spawn();
            }
            let _ = std::process::Command::new(matrixkit_theme::process::own_exe_result().unwrap_or_default()).arg("thumb").arg(&target).spawn();
            return;
        }
        eprintln!("[ma] grim fehlgeschlagen — nur der Bildschirm mit Fokus");
    }
    if !mk::canvas::action(&["msg", "action", action]) {
        eprintln!("[ma] Leinwand-Aktion {action} fehlgeschlagen");
        return;
    }
    let deadline = Instant::now();
    let newest = loop {
        std::thread::sleep(Duration::from_millis(400));
        let now: usize = std::fs::read_dir(&folder).map(|d| d.count()).unwrap_or(0);
        if now > before {
            // youngest file in the folder
            let mut files: Vec<_> = std::fs::read_dir(&folder)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.path())
                .collect();
            files.sort_by_key(|p| {
                std::fs::metadata(p)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::UNIX_EPOCH)
            });
            break files.pop();
        }
        // Region picking can take a while; Esc in the picker = never a file.
        if deadline.elapsed() > Duration::from_secs(60) {
            break None;
        }
    };
    let Some(raw) = newest else { return };
    let target = folder.join(time_stamp_name("Bildschirmfoto", "png"));
    let path = if std::fs::rename(&raw, &target).is_ok() { target } else { raw };
    // Reference detail (user finding 2026-07-19): window screenshots have
    // round corners on transparent ground in the reference — our window
    // buffer delivers them square (the iced canvas behind the rounded
    // root is opaque). Self-calibrating re-rounding; foreign windows
    // MatrixKit-Kante bleiben ehrlich eckig.
    if action == "screenshot-window" {
        round_corners(&path);
    }
    // let the preview float — own process, we are done.
    let _ = std::process::Command::new(matrixkit_theme::process::own_exe_result().unwrap_or_default())
        .arg("thumb")
        .arg(&path)
        .spawn();
}

/// Bring a window screenshot onto MatrixKit corners: the radius is
/// measured in the image ITSELF (run length of the corner color in the
/// top row) — no scale guessing, and without a recognizable kit edge
/// (4–80 px, four corners agreeing) nothing happens. Then: corner
/// quarters to alpha 0 by circle distance, 1 px feathering against stairs.
fn round_corners(path: &std::path::Path) {
    let Ok(image) = ::image::open(path) else { return };
    let mut rgba = image.to_rgba8();
    let (b, h) = rgba.dimensions();
    if b < 200 || h < 120 {
        return;
    }
    let eck = *rgba.get_pixel(0, 0);
    if eck.0[3] == 0 {
        return; // corners already transparent — nothing to do
    }
    // Geometry instead of color (the empty color can reappear in the UI,
    // e.g. as sidebar ground): the inset of the empty color per row must
    // form a circularly decreasing arc at BOTH top corners.
    let indent = |y: u32, right: bool| -> u32 {
        let mut n = 0;
        for i in 0..b {
            let x = if right { b - 1 - i } else { i };
            if *rgba.get_pixel(x, y) == eck { n += 1 } else { break }
        }
        n
    };
    let r = indent(0, false);
    if !(4..=80).contains(&r) || indent(0, true).abs_diff(r) > 2 {
        return; // no MatrixKit edge — leave foreign windows untouched
    }
    let mut before = r;
    for y in 1..r {
        let e = indent(y, false).min(r + 2);
        if e > before {
            return; // no arc — hands off
        }
        before = e;
    }
    if before > 2 {
        return; // the arc does not run out to zero — no rounding
    }
    let rf = r as f32;
    for (cx, cy, x0, y0) in [
        (rf, rf, 0, 0),
        (b as f32 - rf, rf, b - r, 0),
        (rf, h as f32 - rf, 0, h - r),
        (b as f32 - rf, h as f32 - rf, b - r, h - r),
    ] {
        for y in y0..y0 + r {
            for x in x0..x0 + r {
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                let p = rgba.get_pixel_mut(x, y);
                if d > rf {
                    p.0[3] = 0;
                } else if d > rf - 1.0 {
                    p.0[3] = (p.0[3] as f32 * (rf - d)) as u8;
                }
            }
        }
    }
    let _ = rgba.save(path);
}

fn film_start() {
    if recording_status_file().exists() {
        return; // already running
    }
    let folder = film_folder();
    let _ = std::fs::create_dir_all(&folder);
    // R153: the container must match the codec. NVENC writes H.264 —
    // that is an .mp4 and plays everywhere, Apple included. Without
    // NVENC the only encoder Fedora's patent-free ffmpeg really offers
    // is VP9 (openh264 is a placeholder that refuses to start, and the
    // Intel VA-API driver carries no encode profile — both measured on
    // the Surface, 28.07.2026). VP9 belongs in .webm; the earlier
    // VP9-in-.mp4 was a lie about the file that made QuickTime refuse
    // the recording outright.
    // Round 65: the Snapdragon's own encoder (Surface Duo) — full size
    // H.264 in an .mp4, through the kit's Venus family.
    if let Some(device) = matrixkit_video::venus::encoder_device() {
        if venus_start(&folder, &device) {
            return;
        }
    }
    let hardware = nvenc_present();
    let target = folder.join(time_stamp_name(
        "Bildschirmaufnahme",
        if hardware { "mp4" } else { "webm" },
    ));
    let mut kdo = std::process::Command::new("wf-recorder");
    kdo.arg("-f").arg(&target);
    // R84b: steady 60 frames/s — otherwise wf-recorder only records on
    // image change (variable rate), which makes players stutter.
    // Round 64: software encoding keeps up only at 30 (Duo, measured).
    kdo.args(["-r", if hardware { "60" } else { "30" }]);
    // Round 64 (Duo): -D = a frame every tick, not only on damage. With
    // damage-only capture a calm screen delivers no frame, wf-recorder
    // waits and then hangs on SIGINT without writing a file.
    kdo.arg("-D");
    // Several outputs (the Duo's two panels): without -o wf-recorder asks
    // on stdin and exits — film the output that has the focus.
    if let Some(out) = matrixkit_theme::canvas::focused_output() {
        kdo.arg("-o").arg(out);
    }
    kdo.stdin(std::process::Stdio::null());
    if hardware {
        // Hardware encoding on the NVIDIA GPU (NVENC): does 4K/60 in real
        // time where software VP9 dropped frames (the owner's stutter,
        // measured ~16 fps). H.264 is Apple-compatible on the side — the
        // recording plays directly on the Mac, no transcoding.
        kdo.args([
            "-c", "h264_nvenc",
            "-p", "preset=p5",
            "-p", "tune=hq",
            "-p", "rc=vbr",
            "-p", "cq=20",
        ]);
    } else {
        // Without NVIDIA (Surface): VP9 in constant-quality mode. Fedora's
        // patent-free ffmpeg brings no x264, hence no H.264 software.
        kdo.args([
            "-c", "libvpx-vp9",
            "-p", "b=0",
            "-p", "crf=32",
            "-p", "deadline=realtime",
            "-p", "cpu-used=8",
            "-p", "row-mt=1",
        ]);
    }
    let mut filter = String::from("format=yuv420p");
    // Round 64: without a hardware encoder and without a chosen size, film
    // at most 900 lines — the Duo's 1350×1800 at full size reached 1.5 of
    // 6 s (frames lost), at 900 lines 7.6 of 8 s.
    let height = film_height().or(if hardware { None } else { Some(900) });
    if let Some(h) = height {
        // Downscale while recording: -2 keeps the width even
        // (aspect ratio stays).
        filter = format!("scale=-2:{h}:flags=lanczos,format=yuv420p");
    }
    kdo.args(["-F", &filter]);
    let Ok(child) = kdo.stderr(std::process::Stdio::null()).spawn() else {
        eprintln!("[ma] wf-recorder fehlt — Film kommt mit dem nächsten Image");
        // Round 64: say it where it is seen (the control centre's switch).
        matrixkit_theme::notify::send(false, "Bildschirmaufnahme nicht möglich", "Das Werkzeug «wf-recorder» fehlt noch — installieren mit: sudo dnf install wf-recorder");
        return;
    };
    // Status: PID + path + start time — the bar reads it for the stop dot.
    let start = std::process::Command::new("date")
        .arg("+%s")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let _ = std::fs::write(
        recording_status_file(),
        format!("{}\n{}\n{start}\n", child.id(), target.display()),
    );
}

fn film_stop() {
    let Ok(content) = std::fs::read_to_string(recording_status_file()) else { return };
    let mut lines = content.lines();
    let pid = lines.next().unwrap_or_default().to_string();
    let path = PathBuf::from(lines.next().unwrap_or_default());
    // SIGINT lets wf-recorder finish the file cleanly.
    let _ = std::process::Command::new("kill")
        .args(["-INT", &pid])
        .status();
    let _ = std::fs::remove_file(recording_status_file());
    std::thread::sleep(Duration::from_millis(600));
    // Round 65: the Venus film still drains and packs its .mp4 — wait
    // for the recorder process to end (at most 20 s).
    let proc_dir = PathBuf::from(format!("/proc/{pid}"));
    for _ in 0..80 {
        if !proc_dir.exists() {
            break;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    if path.exists() {
        let _ = std::process::Command::new(matrixkit_theme::process::own_exe_result().unwrap_or_default())
            .arg("thumb")
            .arg(&path)
            .spawn();
    }
}

// ------------------------------------------------------------ Venus

/// The recording process of the Venus path (spawned by `venus_start`).
static VENUS_CHILD: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// Round 99: the stop of the GPU path (no child to forward to).
static VENUS_STOP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

extern "C" fn venus_forward_int(_sig: libc::c_int) {
    VENUS_STOP.store(true, std::sync::atomic::Ordering::Relaxed);
    let pid = VENUS_CHILD.load(std::sync::atomic::Ordering::Relaxed);
    if pid > 0 {
        // SAFETY: kill is async-signal-safe.
        unsafe { libc::kill(pid, libc::SIGINT) };
    }
}

/// Start the Venus film: spawn `matrix-capture venus-film …` as the
/// recorder process (its PID goes into the status file — the stop dot
/// and `film_stop` work unchanged).
fn venus_start(folder: &std::path::Path, device: &std::path::Path) -> bool {
    let Some(output) = pick_outputs() else { return false };
    let first = output.split('+').next().unwrap_or_default().to_string();
    let Some((w, h, transform)) = output_geometry(&first) else { return false };
    let target = folder.join(time_stamp_name("Bildschirmaufnahme", "mp4"));
    let Ok(child) = std::process::Command::new(matrixkit_theme::process::own_exe_result().unwrap_or_default())
        .arg("venus-film")
        .arg(&target)
        .arg(&output)
        .arg(w.to_string())
        .arg(h.to_string())
        .arg(transform)
        .arg(device)
        .stdin(std::process::Stdio::null())
        .spawn()
    else {
        return false;
    };
    let start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let _ = std::fs::write(recording_status_file(), format!("{}\n{}\n{start}\n", child.id(), target.display()));
    eprintln!("[ma] Venus-Film {w}x{h} → {}", target.display());
    true
}

/// Round 152: the two panels ONE ABOVE THE OTHER, like the film of both
/// (upper panel DSI-1 on top) — kit: `mk::canvas::photo_stacked`.
fn photo_stacked(target: &std::path::Path) -> bool {
    mk::canvas::photo_stacked(&["DSI-1", "DSI-2"], target)
}

/// Round 151: both panels of an opened duo (same size and turn) — for the
/// photo and the film alike; «one» (`film-panels`) keeps one panel.
fn both_panels() -> bool {
    !mk::settings::read("film-panels").is_some_and(|v| v.trim() == "one") && mk::canvas::panels_alike()
}

/// Round 131: what to film — both panels of an opened duo («DSI-1+DSI-2»,
/// upper first; unless «one» was chosen) when they have the same size and
/// turn, else the focused panel.
fn pick_outputs() -> Option<String> {
    let focused = mk::canvas::focused_output()?;
    // An opened duo films both panels unless one was chosen (05.11.2026:
    // the control centre's switch filmed one panel without a choice).
    if both_panels() {
        return Some(String::from("DSI-1+DSI-2"));
    }
    Some(focused)
}

/// Native mode size and transform of an output (kit: `mk::canvas::screens`).
fn output_geometry(name: &str) -> Option<(u32, u32, String)> {
    mk::canvas::screens().into_iter().find(|s| s.name == name && s.width > 0).map(|s| (s.width, s.height, s.transform))
}

/// The recorder process: the GPU path (round 99) or, where it cannot run,
/// wf-recorder handing raw NV12 frames; the Venus encoder turns them into
/// H.264, ffmpeg packs the .mp4 WITHOUT re-encoding (display rotation as
/// metadata — free). SIGINT (the stop) ends either.
fn venus_film(args: Vec<String>) {
    use std::io::Read;
    let [target, output, w, h, transform, device] = args.as_slice() else {
        eprintln!("[ma] venus-film: Argumente fehlen");
        return;
    };
    let (Ok(w), Ok(h)) = (w.parse::<u32>(), h.parse::<u32>()) else { return };
    let target = PathBuf::from(target);
    let raw_path = target.with_extension("h264");
    let fps = 30;
    let device = std::path::Path::new(device);
    // SAFETY: installing a handler that only stores a flag and calls kill().
    unsafe {
        libc::signal(libc::SIGINT, venus_forward_int as *const () as libc::sighandler_t);
        libc::signal(libc::SIGTERM, venus_forward_int as *const () as libc::sighandler_t);
    }
    // Round 102: the kit's film family (GPU picture + the sound track).
    let opts = matrixkit_video::film::Options { fps, sound: mk::film::sound() };
    // Round 131: two panels in one film («DSI-1+DSI-2»).
    let filmed = match output.split_once('+') {
        Some((a, b)) => matrixkit_video::film::record_pair(&target, [a, b], w, h, transform, device, opts, &VENUS_STOP),
        None => matrixkit_video::film::record(&target, output, w, h, transform, device, opts, &VENUS_STOP),
    };
    match filmed {
        Ok(f) => {
            eprintln!("[ma] Film: {} Bilder, {:.1} s, Ton {}, GPU {:.1} ms je Bild", f.frames, f.seconds, if f.sound { "ja" } else { "nein" }, f.gpu_ms);
            film_saved(&target, f.seconds, Some(f.sound));
            return;
        }
        Err(e) => eprintln!("[ma] GPU-Film nicht möglich ({e}) — wf-recorder übernimmt"),
    }
    if VENUS_STOP.load(std::sync::atomic::Ordering::Relaxed) {
        let _ = std::fs::remove_file(recording_status_file());
        return;
    }
    let settings = matrixkit_video::venus::Settings { fps, ..matrixkit_video::venus::Settings::new(w, h) };
    let mut encoder = match matrixkit_video::venus::Encoder::open(device, settings) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[ma] Venus: {e}");
            matrixkit_theme::notify::send(false, "Bildschirmaufnahme nicht möglich", &format!("Der Video-Encoder antwortet nicht ({e})."));
            let _ = std::fs::remove_file(recording_status_file());
            return;
        }
    };
    let Ok(mut recorder) = std::process::Command::new("wf-recorder")
        .args(["-y", "-D", "-o", output, "-r", &fps.to_string(), "-c", "rawvideo", "-x", "nv12", "-m", "rawvideo", "-f", "/dev/stdout"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
    else {
        matrixkit_theme::notify::send(false, "Bildschirmaufnahme nicht möglich", "Das Werkzeug «wf-recorder» fehlt noch — installieren mit: sudo dnf install wf-recorder");
        let _ = std::fs::remove_file(recording_status_file());
        return;
    };
    VENUS_CHILD.store(recorder.id() as i32, std::sync::atomic::Ordering::Relaxed);
    let sound = if mk::film::sound() { matrixkit_video::film::SoundTrack::start(&target.with_extension("wav")) } else { None };
    let Some(mut frames) = recorder.stdout.take() else { return };
    let Ok(file) = std::fs::File::create(&raw_path) else { return };
    let mut sink = std::io::BufWriter::with_capacity(1 << 20, file);
    let mut frame = vec![0u8; encoder.frame_len()];
    let outcome = loop {
        match frames.read_exact(&mut frame) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break Ok(()),
            Err(e) => break Err(e),
        }
        if let Err(e) = encoder.push(&frame, &mut sink) {
            break Err(e);
        }
    };
    if let Err(e) = &outcome {
        eprintln!("[ma] Venus-Film abgebrochen: {e}");
        venus_forward_int(0);
    }
    let _ = encoder.finish(&mut sink);
    let _ = recorder.wait();
    let track = sound.and_then(matrixkit_video::film::SoundTrack::stop);
    drop(sink);
    eprintln!("[ma] Venus: {} Bilder, {} kB", encoder.frames_in, encoder.bytes_out / 1024);
    let seconds = encoder.frames_in as f64 / fps as f64;
    drop(encoder);
    let audio = track.map(|(p, s)| (p, matrixkit_video::film::sound_lag(seconds, s)));
    let sound = matrixkit_video::film::pack(&raw_path, &target, fps, transform, false, audio.as_ref().map(|(p, l)| (p.as_path(), *l)));
    film_saved(&target, seconds, Some(sound));
}

/// Round 109: «saved» — in the bar's island with the Player's icon; a tap
/// plays it (mk::notify::Note + mk::open).
fn film_saved(target: &std::path::Path, seconds: f64, sound: Option<bool>) {
    if !target.exists() {
        return;
    }
    let s = seconds.round() as u64;
    let mut text = format!("{}:{:02}", s / 60, s % 60);
    match sound {
        Some(true) => text.push_str(" · mit Ton"),
        Some(false) => text.push_str(" · ohne Ton"),
        None => {}
    }
    text.push_str(" — tippen zum Ansehen");
    mk::notify::Note::new("Bildschirmaufnahme gespeichert").app("Matrix Player").text(&text).opens(target).send();
}

// ------------------------------------------------------------ the panel

/// The capture panel: floats above the dock, one action per item.
fn panel_ui() -> Result<(), iced_layershell::Error> {
    use matrixkit_widgets::iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
    use matrixkit_widgets::iced_layershell::settings::{LayerShellSettings, Settings};
    // R122: the popup grammar of every other panel — a second call
    // toggles the open one away instead of stacking a twin.
    if !mkw::panel_toggle() {
        return Ok(());
    }
    iced_layershell::application(
        Panel::new,
        || String::from("matrix-capture"),
        Panel::update,
        Panel::view,
    )
    .style(|_s, _t| iced::theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
    })
    .settings(Settings {
        layer_settings: LayerShellSettings {
            // R103 popup grammar: the panel belongs to its BAR widget —
            // fullscreen dismiss overlay, card top-right below the bar.
            size: Some((0, 0)),
            anchor: Anchor::Top | Anchor::Right | Anchor::Bottom | Anchor::Left,
            margin: (mkw::panel::BAR_STRIP, 0, 0, 0),
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: -1,
            ..Default::default()
        },
        default_font: Font::with_name("Inter Variable"),
        fonts: mkw::load_symbol_fonts(),
        ..Default::default()
    })
    .run()
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum PanelMsg {
    Action(&'static str),
    /// Choose the recording target height; 0 = native (full resolution).
    Resolution(u32),
    Close,
}

struct Panel {
    palette: mk::Palette,
    film_running: bool,
    film_height: Option<u32>,
    native: Option<(u32, u32)>,
}

impl Panel {
    fn new() -> (Self, Task<PanelMsg>) {
        (
            Panel {
                palette: mk::Palette::load().unwrap_or_default(),
                film_running: recording_status_file().exists(),
                film_height: film_height(),
                native: display_resolution(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, msg: PanelMsg) -> Task<PanelMsg> {
        match msg {
            PanelMsg::Action(mode) => {
                let _ = std::process::Command::new(
                    matrixkit_theme::process::own_exe_result().unwrap_or_default(),
                )
                .arg(mode)
                .spawn();
                std::process::exit(0);
            }
            PanelMsg::Resolution(h) => {
                // The panel stays open — the check mark moves visibly.
                self.film_height = (h != 0).then_some(h);
                mk::settings::write(
                    "capture-resolution",
                    &self.film_height.map_or_else(|| String::from("nativ"), |h| h.to_string()),
                );
                Task::none()
            }
            PanelMsg::Close => std::process::exit(0),
            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, PanelMsg> {
        use mkw::ui::MenuItem as E;
        let p = self.palette;
        let mut entries = vec![
            E::Item {
                glyph: Some(mkw::symbol::IMAGE),
                title: String::from("Ganzen Bildschirm fotografieren"),
                color: None,
                msg: PanelMsg::Action("photo"),
            },
            E::Item {
                glyph: Some(mkw::symbol::SEARCH),
                title: String::from("Bereich auswählen …"),
                color: None,
                msg: PanelMsg::Action("region"),
            },
            E::Item {
                glyph: Some(mkw::symbol::APPS),
                title: String::from("Fenster fotografieren"),
                color: None,
                msg: PanelMsg::Action("fenster"),
            },
            E::Separator,
        ];
        if self.film_running {
            entries.push(E::Item {
                glyph: Some(mkw::symbol::CLOSE),
                title: String::from("Film beenden"),
                color: Some(p.error),
                msg: PanelMsg::Action("recording-stop"),
            });
        } else {
            entries.push(E::Item {
                glyph: Some(mkw::symbol::PLAY_ARROW),
                title: String::from("Bildschirm filmen"),
                color: None,
                msg: PanelMsg::Action("recording"),
            });
        }
        // Recording resolution (reference options thought): native = default,
        // below it only steps that really shrink.
        entries.push(E::Separator);
        entries.push(E::Disabled {
            glyph: None,
            title: String::from("Filmauflösung"),
        });
        let native_title = match self.native {
            Some((b, h)) => format!("Nativ ({b} × {h})"),
            None => String::from("Nativ (Displayauflösung)"),
        };
        let mut levels = vec![(0u32, native_title)];
        for h in [1440u32, 1080, 720] {
            if self.native.is_none_or(|(_, nh)| h < nh) {
                levels.push((h, format!("{h}p")));
            }
        }
        for (h, title) in levels {
            let selected =
                if h == 0 { self.film_height.is_none() } else { self.film_height == Some(h) };
            entries.push(E::Item {
                glyph: selected.then_some(mkw::symbol::CHECK),
                title,
                color: None,
                msg: PanelMsg::Resolution(h),
            });
        }
        let card = mkw::ui::menu_family(
            Some(mkw::txt("Aufnahme", mk::typo::HEADLINE, p.on_surface).into()),
            entries,
            p,
        );
        // R103 popup grammar: dismiss backdrop below, card top-right.
        iced::widget::stack![
            mkw::panel::backdrop(PanelMsg::Close),
            container(card)
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(iced::alignment::Vertical::Top)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(iced::Padding {
                    top: mkw::panel::POPUP_TOP,
                    right: mkw::panel::POPUP_RIGHT,
                    left: 0.0,
                    bottom: 0.0,
                }),
        ]
        .into()
    }
}

// -------------------------------------------------------- the thumbnail

/// The floating preview (reference): bottom right, ~6 s, a click
/// opens the file, ✕ discards only the preview.
fn thumb_ui(path: PathBuf) -> Result<(), iced_layershell::Error> {
    use matrixkit_widgets::iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
    use matrixkit_widgets::iced_layershell::settings::{LayerShellSettings, Settings};
    let edge = mkw::panel::SHADOW_MARGIN as u32;
    THUMB_PATH.set(path).ok();
    iced_layershell::application(
        Thumb::new,
        || String::from("matrix-capture"),
        Thumb::update,
        Thumb::view,
    )
    .subscription(Thumb::subscription)
    .style(|_s, _t| iced::theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
    })
    .settings(Settings {
        layer_settings: LayerShellSettings {
            size: Some((THUMB_B as u32 + 2 * edge, THUMB_H as u32 + 2 * edge)),
            anchor: Anchor::Bottom | Anchor::Right,
            margin: (0, 16 - mkw::panel::SHADOW_MARGIN as i32, 140 - mkw::panel::SHADOW_MARGIN as i32, 0),
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: 0,
            ..Default::default()
        },
        default_font: Font::with_name("Inter Variable"),
        fonts: mkw::load_symbol_fonts(),
        ..Default::default()
    })
    .run()
}

static THUMB_PATH: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

#[to_layer_message]
#[derive(Debug, Clone)]
enum ThumbMsg {
    Tick,
    Open,
    Gone,
}

struct Thumb {
    palette: mk::Palette,
    since: Instant,
    image: Option<image::Handle>,
}

impl Thumb {
    fn new() -> (Self, Task<ThumbMsg>) {
        let path = THUMB_PATH.get().cloned().unwrap_or_default();
        let is_photo = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("png"));
        (
            Thumb {
                palette: mk::Palette::load().unwrap_or_default(),
                since: Instant::now(),
                image: is_photo.then(|| image::Handle::from_path(&path)),
            },
            Task::none(),
        )
    }

    fn update(&mut self, msg: ThumbMsg) -> Task<ThumbMsg> {
        match msg {
            ThumbMsg::Tick => {
                if self.since.elapsed().as_secs() >= THUMB_VISIBLE_S {
                    std::process::exit(0);
                }
                Task::none()
            }
            ThumbMsg::Open => {
                if let Some(p) = THUMB_PATH.get() {
                    // Round 109: the kit opens it (the Duo has no xdg-open).
                    let _ = mk::open::open(&p.to_string_lossy());
                }
                std::process::exit(0);
            }
            ThumbMsg::Gone => std::process::exit(0),
            _ => Task::none(),
        }
    }

    fn subscription(&self) -> Subscription<ThumbMsg> {
        mkw::tick("capture-thumb", Duration::from_millis(500)).map(|_| ThumbMsg::Tick)
    }

    fn view(&self) -> Element<'_, ThumbMsg> {
        let p = self.palette;
        let preview: Element<'_, ThumbMsg> = match &self.image {
            Some(h) => image(h.clone())
                .width(Length::Fixed(THUMB_B - 24.0))
                .height(Length::Fixed(THUMB_H - 48.0))
                .into(),
            None => container(mkw::symbol::<ThumbMsg>(
                mkw::symbol::PLAY_ARROW,
                mk::icon_size::HERO,
                p.primary,
            ))
            .center_x(Length::Fixed(THUMB_B - 24.0))
            .center_y(Length::Fixed(THUMB_H - 48.0))
            .into(),
        };
        let header = row![
            mkw::txt("Aufnahme gesichert", mk::typo::CAPTION, p.text_level(2)),
            iced::widget::Space::new().width(Length::Fill),
            mkw::ui::header_text_button("✕", ThumbMsg::Gone, p),
        ]
        .align_y(iced::Alignment::Center);
        let card = container(column![header, preview].spacing(mk::spacing::XS))
            .padding(mk::spacing::M)
            .width(Length::Fixed(THUMB_B))
            .style(move |_| mkw::panel::pill(p, mk::radius::SMALL, mk::spacing::M));
        let clickable = iced::widget::mouse_area(mkw::panel::shadow_layers(
            card.into(),
            mk::radius::SMALL + mk::spacing::M,
        ))
        .on_press(ThumbMsg::Open);
        container(clickable)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .align_y(iced::alignment::Vertical::Bottom)
            .padding(mkw::panel::SHADOW_MARGIN)
            .into()
    }
}
