//! Round 102: screen films as a kit family.
//!
//! What `matrix-capture` grew over rounds 99–101 — the compositor's picture
//! (`matrixkit_shell::screencopy`, a dmabuf the GPU reads in place),
//! RGB → NV12 on the GPU (`matrixkit_gpu::nv12`), H.264 on the Venus block
//! (`venus::Encoder`), packed into an .mp4 — now any app can film:
//!
//! ```ignore
//! let stop = AtomicBool::new(false);
//! let opts = film::Options { fps: 30, sound: mk::film::sound() };
//! film::record(&target, "DSI-1", 1350, 1800, "normal", &device, opts, &stop)?;
//! ```
//!
//! NEW: the sound track. What the device plays (the default sink's monitor,
//! `pw-record`) runs alongside and goes into the film as AAC; the start lag
//! of the recorder (~0.3 s, measured on the Duo) is evened out when packing.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// How a film is made.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub fps: u32,
    /// Record what the device plays.
    pub sound: bool,
}

impl Default for Options {
    fn default() -> Options {
        Options { fps: 30, sound: true }
    }
}

/// What a finished film holds.
#[derive(Debug, Clone)]
pub struct Summary {
    pub frames: u64,
    pub seconds: f64,
    pub sound: bool,
    /// Mean GPU conversion time per frame in ms.
    pub gpu_ms: f64,
}

/// The sound track while it records.
pub struct SoundTrack {
    child: std::process::Child,
    path: PathBuf,
}

impl SoundTrack {
    /// Start recording what the default output plays into `path` (.wav).
    pub fn start(path: &Path) -> Option<SoundTrack> {
        let child = std::process::Command::new("pw-record")
            .args(["-P", "{ stream.capture.sink = true }"])
            .arg(path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok()?;
        Some(SoundTrack { child, path: path.to_path_buf() })
    }

    /// Stop (SIGINT lets pw-record finish the file); the seconds recorded.
    pub fn stop(mut self) -> Option<(PathBuf, f64)> {
        // SAFETY: signalling our own child.
        unsafe { libc::kill(self.child.id() as i32, libc::SIGINT) };
        let _ = self.child.wait();
        let bytes = std::fs::metadata(&self.path).ok()?.len();
        // 48 kHz, two channels, 16 bit (pw-record's default WAV)
        let seconds = bytes.saturating_sub(44) as f64 / (48_000.0 * 4.0);
        (seconds > 0.1).then(|| (self.path.clone(), seconds))
    }
}

/// The lag of the sound track behind the picture: both stop together, the
/// sound started late by what it lacks (clamped to a sane range).
pub fn sound_lag(video_seconds: f64, sound_seconds: f64) -> f64 {
    (video_seconds - sound_seconds).clamp(0.0, 2.0)
}

/// Record `output` (native mode `w`×`h`) until `stop` is set. Err before the
/// first frame = the GPU path cannot run here (the caller may fall back);
/// the encoder session is closed again in that case.
#[allow(clippy::too_many_arguments)]
pub fn record(target: &Path, output: &str, w: u32, h: u32, transform: &str, device: &Path, opts: Options, stop: &AtomicBool) -> Result<Summary, String> {
    let settings = crate::venus::Settings { fps: opts.fps, ..crate::venus::Settings::new(w, h) };
    let mut encoder = crate::venus::Encoder::open(device, settings).map_err(|e| format!("Video-Encoder: {e}"))?;
    let raw_path = target.with_extension("h264");
    let file = std::fs::File::create(&raw_path).map_err(|e| e.to_string())?;
    let mut sink = std::io::BufWriter::with_capacity(1 << 20, file);
    let sound_path = target.with_extension("wav");
    let mut sound = None;
    let result = gpu_frames(&mut encoder, &mut sink, output, w, h, opts.fps, stop, || {
        // the sound starts with the first picture
        if opts.sound {
            sound = SoundTrack::start(&sound_path);
        }
    });
    let track = sound.and_then(SoundTrack::stop);
    if encoder.frames_in == 0 {
        drop(sink);
        let _ = std::fs::remove_file(&raw_path);
        if let Some((p, _)) = &track {
            let _ = std::fs::remove_file(p);
        }
        return Err(result.err().unwrap_or_else(|| "keine Bilder".into()));
    }
    if let Err(e) = &result {
        eprintln!("[film] abgebrochen: {e}");
    }
    let _ = encoder.finish(&mut sink);
    let _ = sink.flush();
    drop(sink);
    let frames = encoder.frames_in;
    drop(encoder);
    let seconds = frames as f64 / opts.fps as f64;
    let gpu_ms = result.unwrap_or(0.0);
    let audio = track.map(|(p, s)| (p, sound_lag(seconds, s)));
    let with_sound = pack(&raw_path, target, opts.fps, transform, true, audio.as_ref().map(|(p, l)| (p.as_path(), *l)));
    Ok(Summary { frames, seconds, sound: with_sound, gpu_ms })
}

/// Round 131: BOTH panels of a duo in one film — each panel grabbed and
/// turned into NV12 on the GPU, the two laid side by side into one frame
/// twice as wide (native panel orientation). Both panels share the
/// transform, so the film's rotation turns the pair into the device's
/// own shape: one panel above the other.
#[allow(clippy::too_many_arguments)]
pub fn record_pair(target: &Path, outputs: [&str; 2], w: u32, h: u32, transform: &str, device: &Path, opts: Options, stop: &AtomicBool) -> Result<Summary, String> {
    let settings = crate::venus::Settings { fps: opts.fps, kbps: 12_000, ..crate::venus::Settings::new(2 * w, h) };
    let mut encoder = crate::venus::Encoder::open(device, settings).map_err(|e| format!("Video-Encoder: {e}"))?;
    let raw_path = target.with_extension("h264");
    let file = std::fs::File::create(&raw_path).map_err(|e| e.to_string())?;
    let mut sink = std::io::BufWriter::with_capacity(1 << 20, file);
    let sound_path = target.with_extension("wav");
    let mut sound = None;
    // Each panel in a thread of its own: a grab waits for its panel's next
    // picture, and two grabs in a row waited twice per frame (26 ms per
    // pair instead of ~7 per panel — the film stuttered, 05.11.2026). The
    // filming loop takes the newest of each at its own steady rate.
    let (cw, ch) = (2 * w as usize, h as usize);
    let latest: [std::sync::Arc<std::sync::Mutex<Option<Vec<u8>>>>; 2] = Default::default();
    let failed: std::sync::Arc<std::sync::Mutex<Option<String>>> = Default::default();
    let gpu_us = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let grabbed = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let result = std::thread::scope(|scope| -> Result<f64, String> {
        for k in 0..2 {
            let (slot, failed, gpu_us, grabbed) = (latest[k].clone(), failed.clone(), gpu_us.clone(), grabbed.clone());
            let output = outputs[k].to_string();
            scope.spawn(move || {
                let mut src = match Source::open(&output, w, h) {
                    Ok(s) => s,
                    Err(e) => {
                        *failed.lock().unwrap_or_else(|e| e.into_inner()) = Some(e);
                        return;
                    }
                };
                // Two buffers taking turns (no allocation per picture).
                let mut spare = vec![0u8; w as usize * ch * 3 / 2];
                while !stop.load(Ordering::Relaxed) {
                    let t = Instant::now();
                    match src.frame() {
                        Ok((rows, row)) => {
                            for y in 0..ch * 3 / 2 {
                                spare[y * w as usize..][..w as usize].copy_from_slice(&rows[y * row..][..w as usize]);
                            }
                            gpu_us.fetch_add(t.elapsed().as_micros() as u64, Ordering::Relaxed);
                            grabbed.fetch_add(1, Ordering::Relaxed);
                            let mut g = slot.lock().unwrap_or_else(|e| e.into_inner());
                            match g.as_mut() {
                                Some(shown) => std::mem::swap(shown, &mut spare),
                                None => *g = Some(std::mem::replace(&mut spare, vec![0u8; w as usize * ch * 3 / 2])),
                            }
                        }
                        Err(e) => {
                            *failed.lock().unwrap_or_else(|e| e.into_inner()) = Some(e);
                            return;
                        }
                    }
                }
            });
        }
        let mut frame = vec![0u8; cw * ch * 3 / 2];
        let period = Duration::from_secs_f64(1.0 / opts.fps as f64);
        // Wait for the first picture of both panels (at most 3 s).
        let began = Instant::now();
        while latest.iter().any(|s| s.lock().map(|g| g.is_none()).unwrap_or(true)) {
            if let Some(e) = failed.lock().unwrap_or_else(|e| e.into_inner()).take() {
                stop.store(true, Ordering::Relaxed);
                return Err(e);
            }
            if began.elapsed() > Duration::from_secs(3) || stop.load(Ordering::Relaxed) {
                stop.store(true, Ordering::Relaxed);
                return Err("kein Bild von beiden Panels".into());
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        let mut due = Instant::now();
        let mut first = true;
        while !stop.load(Ordering::Relaxed) {
            if let Some(e) = failed.lock().unwrap_or_else(|e| e.into_inner()).take() {
                eprintln!("[film] Panel-Abgriff endete: {e}");
                break;
            }
            for (k, slot) in latest.iter().enumerate() {
                if let Some(rows) = slot.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
                    for y in 0..ch * 3 / 2 {
                        frame[y * cw + k * w as usize..][..w as usize].copy_from_slice(&rows[y * w as usize..][..w as usize]);
                    }
                }
            }
            encoder.push_rows(&frame, cw, &mut sink).map_err(|e| e.to_string())?;
            if first {
                first = false;
                due = Instant::now();
                if opts.sound {
                    sound = SoundTrack::start(&sound_path);
                }
            }
            due += period;
            let mut repeats = 0;
            while Instant::now() >= due + period && repeats < opts.fps * 5 {
                encoder.push_rows(&frame, cw, &mut sink).map_err(|e| e.to_string())?;
                due += period;
                repeats += 1;
            }
            if let Some(wait) = due.checked_duration_since(Instant::now()) {
                std::thread::sleep(wait);
            }
        }
        // The panel threads end with the stop.
        stop.store(true, Ordering::Relaxed);
        let n = grabbed.load(Ordering::Relaxed).max(1);
        Ok(gpu_us.load(Ordering::Relaxed) as f64 / 1000.0 / n as f64)
    });
    let track = sound.and_then(SoundTrack::stop);
    if encoder.frames_in == 0 {
        drop(sink);
        let _ = std::fs::remove_file(&raw_path);
        if let Some((p, _)) = &track {
            let _ = std::fs::remove_file(p);
        }
        return Err(result.err().unwrap_or_else(|| "keine Bilder".into()));
    }
    if let Err(e) = &result {
        eprintln!("[film] abgebrochen: {e}");
    }
    let _ = encoder.finish(&mut sink);
    let _ = sink.flush();
    drop(sink);
    let frames = encoder.frames_in;
    drop(encoder);
    let seconds = frames as f64 / opts.fps as f64;
    let gpu_ms = result.unwrap_or(0.0);
    let audio = track.map(|(p, s)| (p, sound_lag(seconds, s)));
    let with_sound = pack(&raw_path, target, opts.fps, transform, true, audio.as_ref().map(|(p, l)| (p.as_path(), *l)));
    Ok(Summary { frames, seconds, sound: with_sound, gpu_ms })
}

/// One grabbed output turned into NV12 on the GPU (round 131 — the
/// single-output loop's steps as a part, so two can run side by side).
struct Source {
    grab: matrixkit_shell::screencopy::ScreenGrab,
    nv12: matrixkit_gpu::nv12::Nv12,
    stride: u32,
    w: u32,
    h: u32,
    imported: Option<u64>,
    import_failed: bool,
}

impl Source {
    fn open(output: &str, w: u32, h: u32) -> Result<Source, String> {
        use matrixkit_shell::screencopy::ScreenGrab;
        let mut grab = ScreenGrab::open(output)?;
        let stride = match grab.grab() {
            Ok(s) => s.stride,
            Err(e) => {
                eprintln!("[film] {output}: dmabuf-Abgriff misslang ({e}) — shm");
                grab = ScreenGrab::open_with(output, false)?;
                grab.grab()?.stride
            }
        };
        let nv12 = matrixkit_gpu::nv12::Nv12::new(w, h, stride)?;
        let import_failed = !nv12.can_import();
        Ok(Source { grab, nv12, stride, w, h, imported: None, import_failed })
    }

    /// The next picture: (NV12 rows, row length).
    fn frame(&mut self) -> Result<(&[u8], usize), String> {
        let (sw, sh, sstride, bgr, invert) = {
            let shot = self.grab.grab()?;
            (shot.width, shot.height, shot.stride, shot.bgr, shot.y_invert)
        };
        if sw < self.w || sh < self.h {
            return Err(format!("Bild {sw}×{sh} kleiner als {}×{}", self.w, self.h));
        }
        if sstride != self.stride {
            self.stride = sstride;
            self.nv12 = matrixkit_gpu::nv12::Nv12::new(self.w, self.h, sstride)?;
            self.imported = None;
            self.import_failed = !self.nv12.can_import();
        }
        if !self.import_failed {
            if let Some((fd, len, serial)) = self.grab.dmabuf() {
                if self.imported != Some(serial) {
                    match self.nv12.import(fd, len) {
                        Ok(()) => self.imported = Some(serial),
                        Err(e) => {
                            eprintln!("[film] dmabuf-Import nicht möglich ({e}) — Hochladen");
                            self.import_failed = true;
                        }
                    }
                }
            }
        }
        let in_place = self.imported.is_some() && !self.import_failed && self.grab.uses_dmabuf();
        self.grab.set_cpu_read(!in_place);
        let row = self.nv12.row_bytes();
        let rows = if in_place {
            self.nv12.convert_imported(bgr, invert)?
        } else {
            let shot = self.grab.grab_last()?;
            self.nv12.convert(shot.pixels, bgr, invert)?
        };
        Ok((rows, row))
    }
}

/// The GPU picture loop (round 99–100). Returns the mean conversion time.
#[allow(clippy::too_many_arguments)]
fn gpu_frames(
    encoder: &mut crate::venus::Encoder,
    sink: &mut impl Write,
    output: &str,
    w: u32,
    h: u32,
    fps: u32,
    stop: &AtomicBool,
    mut on_first: impl FnMut(),
) -> Result<f64, String> {
    use matrixkit_shell::screencopy::ScreenGrab;
    // A dmabuf first (the compositor blits its frame on the GPU); should it
    // break the first grab, shm.
    let mut grab = ScreenGrab::open(output)?;
    let mut stride = match grab.grab() {
        Ok(s) => s.stride,
        Err(e) => {
            eprintln!("[film] dmabuf-Abgriff misslang ({e}) — shm");
            grab = ScreenGrab::open_with(output, false)?;
            grab.grab()?.stride
        }
    };
    eprintln!("[film] Abgriff über {}", if grab.uses_dmabuf() { "dmabuf (ION)" } else { "shm" });
    let mut nv12 = matrixkit_gpu::nv12::Nv12::new(w, h, stride)?;
    let row = nv12.row_bytes();
    let mut imported: Option<u64> = None;
    let mut import_failed = !nv12.can_import();
    let period = Duration::from_secs_f64(1.0 / fps as f64);
    let mut due = Instant::now();
    let mut pushed: u64 = 0;
    let mut gpu_time = Duration::ZERO;
    let mut first = true;
    while !stop.load(Ordering::Relaxed) {
        let (sw, sh, sstride, bgr, invert) = {
            let shot = grab.grab()?;
            (shot.width, shot.height, shot.stride, shot.bgr, shot.y_invert)
        };
        if sw < w || sh < h {
            return Err(format!("Bild {sw}×{sh} kleiner als {w}×{h}"));
        }
        if sstride != stride {
            // the grabber fell back (dmabuf → shm): rows of another length
            stride = sstride;
            nv12 = matrixkit_gpu::nv12::Nv12::new(w, h, stride)?;
            imported = None;
            import_failed = !nv12.can_import();
        }
        if !import_failed {
            if let Some((fd, len, serial)) = grab.dmabuf() {
                if imported != Some(serial) {
                    match nv12.import(fd, len) {
                        Ok(()) => imported = Some(serial),
                        Err(e) => {
                            eprintln!("[film] dmabuf-Import nicht möglich ({e}) — Hochladen");
                            import_failed = true;
                        }
                    }
                }
            }
        }
        let in_place = imported.is_some() && !import_failed && grab.uses_dmabuf();
        grab.set_cpu_read(!in_place);
        let t = Instant::now();
        let frame = if in_place {
            nv12.convert_imported(bgr, invert)?
        } else {
            let shot = grab.grab_last()?;
            nv12.convert(shot.pixels, bgr, invert)?
        };
        gpu_time += t.elapsed();
        encoder.push_rows(frame, row, sink).map_err(|e| e.to_string())?;
        if first {
            first = false;
            due = Instant::now();
            on_first();
        }
        pushed += 1;
        due += period;
        // A steady rate: a late frame is repeated so the film's clock stays
        // the wall clock (players stutter on gaps; the sound needs it too).
        let mut repeats = 0;
        while Instant::now() >= due + period && repeats < fps * 5 {
            encoder.push_rows(frame, row, sink).map_err(|e| e.to_string())?;
            pushed += 1;
            due += period;
            repeats += 1;
        }
        if let Some(wait) = due.checked_duration_since(Instant::now()) {
            std::thread::sleep(wait);
        }
    }
    Ok(gpu_time.as_secs_f64() * 1000.0 / pushed.max(1) as f64)
}

/// Pack the raw H.264 (and the sound track, `lag` seconds late) into the
/// .mp4 without re-encoding the picture: rate and rotation are container
/// facts. `bt709` tags the colours (the GPU path converts in BT.709; in the
/// container — the stream filter left players at «unknown»). Returns whether
/// the sound made it in. The raw files go once the film is there.
pub fn pack(raw: &Path, target: &Path, fps: u32, transform: &str, bt709: bool, sound: Option<(&Path, f64)>) -> bool {
    let rotation = crate::venus::display_rotation(transform);
    let run = |audio: Option<(&Path, f64)>, tag: bool| {
        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.args(["-hide_banner", "-loglevel", "error", "-y", "-r", &fps.to_string(), "-display_rotation", &rotation.to_string(), "-i"])
            .arg(raw);
        if let Some((wav, lag)) = audio {
            cmd.args(["-itsoffset", &format!("{lag:.3}"), "-i"]).arg(wav);
            cmd.args(["-map", "0:v", "-map", "1:a", "-c:v", "copy", "-c:a", "aac", "-b:a", "160k"]);
        } else {
            cmd.args(["-c", "copy"]);
        }
        if tag {
            cmd.args(["-color_primaries", "bt709", "-color_trc", "bt709", "-colorspace", "bt709", "-color_range", "tv"]);
        }
        cmd.args(["-movflags", "+faststart"]).arg(target).status().is_ok_and(|s| s.success())
    };
    let with_sound = sound.is_some() && run(sound, bt709);
    let packed = with_sound || (bt709 && run(None, true)) || run(None, false);
    if packed {
        let _ = std::fs::remove_file(raw);
        if let Some((wav, _)) = sound {
            let _ = std::fs::remove_file(wav);
        }
    } else {
        eprintln!("[film] Verpacken misslang — Rohfilm bleibt: {}", raw.display());
    }
    with_sound
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_sound_lag_is_what_it_lacks() {
        assert!((super::sound_lag(10.0, 9.7) - 0.3).abs() < 1e-9);
        assert_eq!(super::sound_lag(10.0, 10.4), 0.0);
        assert_eq!(super::sound_lag(10.0, 2.0), 2.0);
    }
}
