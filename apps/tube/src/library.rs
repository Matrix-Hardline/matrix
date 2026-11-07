//! The library (R210, the owner 02.11.2026: «download as MP3 and MP4,
//! and make playlists from them for on the go»): the list gets two tabs — «Suche» and
//! «Bibliothek». The library shows the playlists (tap = play all) and the
//! downloads (tap = play, «+» = into a playlist or delete). The player view
//! downloads the running video as MP3 or MP4. The stage plays local files
//! in order; a sound-only file shows its cover.

use crate::{Mode, Tube, Work, BAR_BASE};
use matrixkit_image::Fit;
use matrixkit_scene::scene as fam;
use matrixkit_scene::symbol;
use matrixkit_shell::{Pict, Refresh, SceneBuilder};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::Rect;
use matrixkit_tube::library::{Item, Library};
use matrixkit_tube::{Hit, Keep, Watch};
use matrixkit_video::remote::Command;
use std::path::PathBuf;

pub const TAB_SEARCH: usize = 13;
/// The compact tab keys' size (round 172).
pub const TAB_KEY: f32 = 44.0;
pub const TAB_LIBRARY: usize = 14;
pub const NEW_PLAYLIST: usize = 15;
pub const DELETE: usize = 17;
pub const DL_MP3: usize = 36;
pub const DL_MP4: usize = 37;
pub const PLAYLIST_BASE: usize = 2000;
pub const TO_PLAYLIST_BASE: usize = 2100;
pub const ROW_BASE: usize = 2200;
pub const ADD_BASE: usize = 2600;
/// A playlist's «⋯» (open: its titles, rename, delete).
pub const PL_MORE_BASE: usize = 2700;
/// A title in the open playlist: remove it from there.
pub const PL_TITLE_BASE: usize = 2800;
/// A recent search (tap = search again).
pub const HISTORY_BASE: usize = 2900;
pub const PL_RENAME: usize = 18;
pub const PL_DELETE: usize = 19;

/// Library taps that act on RELEASE (a finger that moves scrolls
/// instead — 02.11.2026: «I can't scroll», the big playlist rows
/// played on touch-down). Rows already defer through `press`.
pub fn deferred(role: usize) -> bool {
    matches!(role, NEW_PLAYLIST | DELETE | PL_RENAME | PL_DELETE)
        || (PLAYLIST_BASE..ROW_BASE).contains(&role)
        || (ADD_BASE..HISTORY_BASE + 50).contains(&role)
}

/// A playable time as text.
fn length(s: f64) -> String {
    fam::clock_text(s as f32)
}

impl Tube {
    /// Round 172: the tabs as two round keys beside the field (little room:
    /// a keyboard up, one panel lying wide) — `x` their left edge.
    pub fn build_tab_keys(&self, engine: &mut TextEngine, b: &mut SceneBuilder, x: f32, y: f32) {
        let d = TAB_KEY;
        fam::lay_round_icon(b, engine, symbol::SEARCH, x, y, d, !self.tab_library, Some(TAB_SEARCH), None, self.palette);
        fam::lay_round_icon(b, engine, symbol::VIDEO_LIBRARY, x + d + mk::spacing::S, y, d, self.tab_library, Some(TAB_LIBRARY), None, self.palette);
    }

    /// «Suche | Bibliothek» under the field.
    pub fn build_tabs(&self, engine: &mut TextEngine, b: &mut SceneBuilder, wf: f32, y: f32) -> f32 {
        let m = mk::spacing::M;
        let options = vec![String::from("Suche"), format!("Bibliothek · {}", self.library.items.len())];
        fam::lay_choices(b, engine, &options, Some(self.tab_library as usize), m, y, wf - 2.0 * m, TAB_SEARCH, self.palette) + mk::spacing::S
    }

    /// The library in the list's area (scrolls with the list's roll).
    /// Round 19 (owner: «polish … I can't delete playlists»):
    /// playlists as media rows (cover, titles, length) with a «⋯» that opens
    /// their titles (each removable), «Umbenennen» and «Löschen» (asks once
    /// more); a download's «+» toggles it in and out of playlists (✓).
    pub fn build_library(&mut self, engine: &mut TextEngine, b: &mut SceneBuilder, wf: f32, hf: f32, top: f32) {
        let p = self.palette;
        let m = mk::spacing::M;
        let w = wf - 2.0 * m;
        let clip = Some(self.list);
        let cap = mk::typo::CAPTION;
        let body = mk::typo::CALLOUT;
        let mut y = top - self.roll.value().max(0.0);
        let visible = |y: f32, h: f32| y + h >= top && y <= hf;
        let tw = fam::MediaRow::new("", "").thumb_w;

        // ── Playlists: head with «+ Neu» at the right.
        let head = format!("Playlists · {}", self.library.playlists.len());
        let (_, ch) = b.styled(engine, head, cap, p.on_surface_variant, m, y + 8.0, None, clip);
        let new_text = if self.naming { "Name oben eingeben, dann ↵" } else { "Neu" };
        let (nw, nh) = engine.measure_styled(new_text, body.px(), body.weight);
        let pill_w = nw + 2.0 * m + 22.0;
        let pill_h = 34.0;
        let px = m + w - pill_w;
        b.quad(px, y, pill_w, pill_h, p.primary_container.with_alpha(0.55), pill_h / 2.0, Some(NEW_PLAYLIST), clip);
        let symfam = matrixkit_scene::symbol_family();
        let gpx = mk::typo::symbol_px(18.0);
        let (_, gh) = engine.measure_family(symfam, &symbol::ADD.to_string(), gpx, mk::typo::Weight::Normal);
        b.text(engine, symbol::ADD.to_string(), gpx, mk::typo::Weight::Normal, symfam, p.on_primary_container, px + m * 0.7, y + (pill_h - gh) / 2.0, Some(NEW_PLAYLIST), clip);
        b.styled(engine, new_text, body, p.on_primary_container, px + m * 0.7 + 22.0, y + (pill_h - nh) / 2.0, Some(NEW_PLAYLIST), clip);
        y += pill_h.max(ch) + mk::spacing::S;
        if self.library.playlists.is_empty() {
            let (_, eh) = b.styled(engine, "Noch keine — «Neu» legt eine an, «+» bei einem Download füllt sie.", cap, p.on_surface_variant, m, y, None, clip);
            y += eh + mk::spacing::S;
        }
        for (i, pl) in self.library.playlists.iter().enumerate() {
            let open = self.pl_open == Some(i);
            let total = self.library.duration(i);
            let titles = format!("{} Titel", pl.items.len());
            let len = if total > 0.0 { length(total) } else { String::new() };
            let meta = fam::dot_join(&["Playlist", &titles, &len]);
            let row = fam::MediaRow {
                thumb: self.library.cover(i),
                trailing: Some((if open { symbol::EXPAND_LESS } else { symbol::EXPAND_MORE }, open, PL_MORE_BASE + i)),
                picked: open,
                ..fam::MediaRow::new(&pl.name, &meta)
            };
            let row_h = row.height();
            if visible(y, row_h) {
                fam::lay_media_row(b, engine, &row, m, y, w, Some(PLAYLIST_BASE + i), clip, p);
            }
            y += row_h;
            if !open {
                continue;
            }
            // Its titles, each with «✕» (out of this playlist; the file stays).
            let x0 = m + tw + m;
            let line_h = 34.0;
            if pl.items.is_empty() {
                let (_, eh) = b.styled(engine, "Leer — bei einem Download «+» tippen.", cap, p.on_surface_variant, x0, y, None, clip);
                y += eh + mk::spacing::S;
            }
            for (k, f) in pl.items.iter().enumerate() {
                let title = self.library.item(f).map(|it| it.title.clone()).unwrap_or_else(|| f.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default());
                if visible(y, line_h) {
                    let (_, th) = engine.measure_styled(&title, cap.px(), cap.weight);
                    b.styled(engine, format!("{}. {}", k + 1, mk::elide(&title, 46)), cap, p.on_surface, x0, y + (line_h - th) / 2.0, None, clip);
                    fam::lay_round_icon(b, engine, symbol::CLOSE, m + w - 30.0, y + 2.0, 30.0, false, Some(PL_TITLE_BASE + k), clip, p);
                }
                y += line_h;
            }
            y += mk::spacing::XS;
            // «Umbenennen» (rename) and «Löschen» (delete) (the second tap deletes).
            let rename = "Umbenennen";
            let del = if self.confirm { "Wirklich löschen?" } else { "Playlist löschen" };
            let (rw, rh) = engine.measure_styled(rename, body.px(), body.weight);
            b.quad(x0, y, rw + 2.0 * m, 36.0, p.surface_container_high, 18.0, Some(PL_RENAME), clip);
            b.styled(engine, rename, body, p.on_surface, x0 + m, y + (36.0 - rh) / 2.0, Some(PL_RENAME), clip);
            let (dw, dh) = engine.measure_styled(del, body.px(), body.weight);
            let dx = x0 + rw + 2.0 * m + mk::spacing::S;
            let (fill, ink) = if self.confirm { (p.error, if p.is_light { mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } } else { p.surface }) } else { (p.error.with_alpha(0.18), p.error) };
            b.quad(dx, y, dw + 2.0 * m, 36.0, fill, 18.0, Some(PL_DELETE), clip);
            b.styled(engine, del, body, ink, dx + m, y + (36.0 - dh) / 2.0, Some(PL_DELETE), clip);
            y += 36.0 + m;
        }
        y += mk::spacing::S;

        // ── Downloads: head with count and space taken.
        let gb = self.library.size_bytes() as f64 / 1e9;
        let size = if gb >= 1.0 { format!("{gb:.1} GB") } else { format!("{:.0} MB", gb * 1000.0) }.replace('.', ",");
        let head = fam::dot_join(&["Heruntergeladen", &self.library.items.len().to_string(), &size]);
        let (_, ch) = b.styled(engine, head, cap, p.on_surface_variant, m, y, None, clip);
        y += ch + mk::spacing::XS;
        if self.library.items.is_empty() {
            let (_, eh) = b.styled(engine, "Noch nichts — in der Wiedergabe-Ansicht «MP3» oder «MP4» wählen.", cap, p.on_surface_variant, m, y, None, clip);
            y += eh;
        }
        for (i, item) in self.library.items.iter().enumerate() {
            let kind = match item.keep {
                Keep::Mp3 => "MP3",
                Keep::Mp4 => "MP4",
            };
            let lists: Vec<&str> = self.library.playlists.iter().filter(|pl| pl.items.contains(&item.path)).map(|pl| pl.name.as_str()).collect();
            let inside = lists.join(", ");
            let meta = fam::dot_join(&[kind, &item.channel, &length(item.duration), &inside]);
            let open = self.lib_open.as_ref() == Some(&item.path);
            let row = fam::MediaRow {
                thumb: item.thumb.as_deref(),
                trailing: Some((if open { symbol::EXPAND_LESS } else { symbol::ADD }, open, ADD_BASE + i)),
                picked: open,
                ..fam::MediaRow::new(&item.title, &meta)
            };
            let row_h = row.height();
            if visible(y, row_h) {
                fam::lay_media_row(b, engine, &row, m, y, w, Some(ROW_BASE + i), clip, p);
            }
            y += row_h;
            if open {
                // In which playlists: ✓ = in (tap: out), + = not yet (tap: in).
                let mut options: Vec<String> = self
                    .library
                    .playlists
                    .iter()
                    .enumerate()
                    .map(|(k, pl)| format!("{} {}", if self.library.contains(k, &item.path) { "✓" } else { "+" }, pl.name))
                    .collect();
                if options.is_empty() {
                    options.push(String::from("Erst oben eine Playlist anlegen"));
                }
                y += fam::lay_choices(b, engine, &options, None, m + tw + m, y, w - tw - m, TO_PLAYLIST_BASE, p) + mk::spacing::S;
                let del = if self.confirm { "Wirklich löschen?" } else { "Datei löschen" };
                let (dw, dh) = engine.measure_styled(del, body.px(), body.weight);
                let (fill, ink) = if self.confirm { (p.error, if p.is_light { mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } } else { p.surface }) } else { (p.error.with_alpha(0.18), p.error) };
                b.quad(m + tw + m, y, dw + 2.0 * m, 36.0, fill, 18.0, Some(DELETE), clip);
                b.styled(engine, del, body, ink, m + tw + m + m, y + (36.0 - dh) / 2.0, Some(DELETE), clip);
                y += 36.0 + m;
            }
        }
        self.roll_max = (y + self.roll.value().max(0.0) + m - hf).max(0.0);
    }

    /// Presses in the tabs and the library (and the download pills). The
    /// rows play on release (a moving finger scrolls). None = not ours.
    pub fn library_strike(&mut self, role: Option<usize>, x: f32, y: f32) -> Option<Refresh> {
        let r = role?;
        // Any other tap disarms a pending «Wirklich löschen?».
        let was_confirm = self.confirm;
        if r != DELETE && r != PL_DELETE {
            self.confirm = false;
        }
        match r {
            TAB_SEARCH | TAB_LIBRARY => {
                self.tab_library = r == TAB_LIBRARY;
                if self.tab_library {
                    self.library = Library::load();
                }
                self.roll = mk::motion::Spring::smooth(0.0);
                self.list_cache = None;
                self.lib_open = None;
                self.pl_open = None;
                Some(Refresh::Dirty)
            }
            NEW_PLAYLIST => {
                // The field names it: type, then Enter.
                self.naming = true;
                self.renaming = None;
                self.search.clear();
                self.cursor = 0;
                self.field_focus = true;
                Some(Refresh::Dirty)
            }
            DL_MP3 | DL_MP4 => {
                self.start_download(if r == DL_MP3 { Keep::Mp3 } else { Keep::Mp4 });
                Some(Refresh::Dirty)
            }
            DELETE => {
                if !was_confirm {
                    self.confirm = true;
                    return Some(Refresh::Dirty);
                }
                self.confirm = false;
                // By path: a download finishing meanwhile shifts the rows.
                if let Some(path) = self.lib_open.take() {
                    eprintln!("[tube] gelöscht: {}", path.display());
                    self.library.delete(&path);
                    self.library.save();
                    mk::feel::confirm();
                }
                Some(Refresh::Dirty)
            }
            PL_DELETE => {
                if !was_confirm {
                    self.confirm = true;
                    return Some(Refresh::Dirty);
                }
                self.confirm = false;
                if let Some(gone) = self.pl_open.take().and_then(|i| self.library.remove_playlist(i)) {
                    eprintln!("[tube] Playlist gelöscht: «{}»", gone.name);
                    self.library.save();
                    self.note.set(format!("Playlist «{}» gelöscht — die Dateien bleiben", gone.name));
                    mk::feel::confirm();
                }
                Some(Refresh::Dirty)
            }
            PL_RENAME => {
                if let Some(i) = self.pl_open {
                    self.renaming = Some(i);
                    self.naming = false;
                    self.search = self.library.playlists.get(i).map(|p| p.name.clone()).unwrap_or_default();
                    self.cursor = self.search.chars().count();
                    self.field_focus = true;
                }
                Some(Refresh::Dirty)
            }
            _ if (PLAYLIST_BASE..TO_PLAYLIST_BASE).contains(&r) => {
                let files = self.library.queue(r - PLAYLIST_BASE);
                if files.is_empty() {
                    self.note.set(String::from("Diese Playlist ist leer"));
                } else {
                    self.play_files_remote(files);
                }
                Some(Refresh::Dirty)
            }
            _ if (TO_PLAYLIST_BASE..ROW_BASE).contains(&r) => {
                // Toggle: in → out, out → in (the row stays open).
                if let Some(path) = self.lib_open.clone() {
                    let k = r - TO_PLAYLIST_BASE;
                    if k < self.library.playlists.len() {
                        self.library.toggle_in(k, &path);
                        self.library.save();
                        mk::feel::tap();
                    }
                }
                Some(Refresh::Dirty)
            }
            _ if (ROW_BASE..ADD_BASE).contains(&r) => {
                // Played on release (see `release`), so a finger can scroll.
                self.press = Some(mk::touch::Press::new(x, y, self.roll.target, Some(r - ROW_BASE)));
                Some(Refresh::Calm)
            }
            _ if (ADD_BASE..PL_MORE_BASE).contains(&r) => {
                let path = self.library.items.get(r - ADD_BASE).map(|it| it.path.clone());
                self.lib_open = if self.lib_open == path { None } else { path };
                self.pl_open = None;
                Some(Refresh::Dirty)
            }
            _ if (PL_MORE_BASE..PL_TITLE_BASE).contains(&r) => {
                let i = r - PL_MORE_BASE;
                self.pl_open = if self.pl_open == Some(i) { None } else { Some(i) };
                self.lib_open = None;
                Some(Refresh::Dirty)
            }
            _ if (PL_TITLE_BASE..HISTORY_BASE).contains(&r) => {
                if let Some(i) = self.pl_open {
                    let k = r - PL_TITLE_BASE;
                    if let Some(path) = self.library.playlists.get(i).and_then(|pl| pl.items.get(k)).cloned() {
                        self.library.remove_from(i, &path);
                        self.library.save();
                        mk::feel::tap();
                    }
                }
                Some(Refresh::Dirty)
            }
            _ if (HISTORY_BASE..HISTORY_BASE + 50).contains(&r) => {
                if let Some(q) = self.history.get(r - HISTORY_BASE).cloned() {
                    self.search = q;
                    self.cursor = self.search.chars().count();
                    self.start_search();
                    self.list_cache = None;
                }
                Some(Refresh::Dirty)
            }
            _ => None,
        }
    }

    /// A library row was tapped: play that file.
    pub fn play_library_item(&mut self, i: usize) -> Refresh {
        let Some(path) = self.library.items.get(i).map(|it| it.path.clone()) else { return Refresh::Calm };
        self.play_files_remote(vec![path]);
        Refresh::Dirty
    }

    /// The list asks the stage to play files (started if needed).
    pub fn play_files_remote(&mut self, files: Vec<PathBuf>) {
        let Some(first) = files.first().cloned() else { return };
        let words: Vec<String> = files.iter().map(|f| f.to_string_lossy().into_owned()).collect();
        let item = self.library.item(&first).cloned();
        eprintln!("[tube] an die Bühne: {} Datei(en)", words.len());
        mk::feel::tap();
        if self.mode == Mode::List {
            if let Ok(me) = matrixkit_theme::process::own_exe_result() {
                let mut start = std::process::Command::new(me);
                start.arg("--buehne").arg("--dateien").arg(words.join("\t"));
                let log = matrixkit_tube::cache_dir().join("buehne.log");
                if let Ok(file) = std::fs::File::create(&log) {
                    start.stderr(file);
                }
                mk::companion::send_or_start(crate::APP_ID, crate::STAGE_ROLE, &Command::PlayFiles(words).word(), &mut start);
            }
            self.current = Some(local_hit(&first, item.as_ref()));
            self.remote = None;
            self.player_view = true;
            self.player_roll = mk::motion::Spring::smooth(0.0);
        } else {
            self.play_files(files);
        }
    }

    /// The stage plays local files in order — the whole list stays, so
    /// the list can show it and step back (owner 02.11.2026: «I can't see
    /// what's up next» / «can't go back»).
    pub fn play_files(&mut self, files: Vec<PathBuf>) -> Refresh {
        let Some(first) = files.first().cloned() else { return Refresh::Calm };
        let lib = Library::load();
        self.queue_name = lib.playlists.iter().find(|p| p.items == files).map(|p| p.name.clone());
        self.queue_info = files
            .iter()
            .map(|f| {
                let hit = local_hit(f, lib.item(f));
                (hit.id, hit.title, hit.channel, hit.length)
            })
            .collect();
        self.queue = matrixkit_video::queue::Queue::new(files);
        self.play_file(first)
    }

    /// Jump to an entry of the running list (the list's rows, ⏮ ⏭).
    pub fn queue_jump(&mut self, k: usize) -> Refresh {
        let Some(path) = self.queue.jump(k).cloned() else { return Refresh::Calm };
        self.asking_since = None;
        self.play_file(path)
    }

    /// The stage plays one local file (video, or sound with its cover).
    pub fn play_file(&mut self, path: PathBuf) -> Refresh {
        eprintln!("[tube] Datei {}", path.display());
        if let Some(mut v) = self.video.take() {
            v.stop();
        }
        self.skips.clear();
        self.shown = None;
        self.fresh = None;
        self.resume_at = 0.0;
        self.sub_lang = None;
        self.cues.clear();
        self.up_next.clear();
        self.next_at = None;
        self.next_cancelled = false;
        let lib = Library::load();
        let item = lib.item(&path).cloned();
        self.cover = item.as_ref().and_then(|i| i.thumb.clone());
        self.current = Some(local_hit(&path, item.as_ref()));
        let Some(media) = matrixkit_video::probe_any(&path) else {
            self.status = String::from("Datei nicht lesbar");
            return Refresh::Dirty;
        };
        let sound_only = media.width == 0;
        self.watch = Some(Watch {
            media: media.clone(),
            channel: item.as_ref().map(|i| i.channel.clone()).unwrap_or_default(),
            description: String::new(),
            views: 0,
            height: media.height,
            subtitles: Vec::new(),
        });
        self.status.clear();
        if sound_only {
            // Sound only: no picture to wait for — play at once.
            let mut video = matrixkit_video::Video::new(
                media,
                matrixkit_video::Spec { width: 2, exact: None, sound_name: String::from("Matrix Tube"), sound_only: true, max_fps: None, yuv: false },
            );
            video.play(0.0);
            self.video = Some(video);
        } else {
            // A video file starts at once too (folded shut, the next one
            // of a playlist must not wait for a view).
            self.start_now();
        }
        Refresh::Dirty
    }

    /// The next file of the queue at the end (true when it started one).
    pub fn queue_tick(&mut self) -> bool {
        if self.mode == Mode::List || !self.queue.has_next() || self.asking_since.is_some() {
            return false;
        }
        if !self.video.as_ref().is_some_and(|v| v.done()) {
            return false;
        }
        let _ = self.queue_jump(self.queue.at() + 1);
        true
    }

    /// A sound-only file on the stage: its cover, title and channel.
    pub fn build_cover(&self, engine: &mut TextEngine, b: &mut SceneBuilder, stage: Rect) {
        let p = self.palette;
        let white = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.95 };
        let box_h = (stage.h * 0.6).min(stage.w * 9.0 / 16.0);
        let box_w = box_h * 16.0 / 9.0;
        let (bx, by) = (stage.x + (stage.w - box_w) / 2.0, stage.y + stage.h * 0.12);
        let anchor = b.quad(bx, by, box_w, box_h, p.surface_container_high, mk::radius::LARGE, Some(BAR_BASE), None);
        if let Some(cover) = &self.cover {
            b.pict(Pict {
                path: cover.clone(),
                box_w: box_w as u32,
                box_h: box_h as u32,
                fit: Fit::Cover,
                x: bx,
                y: by,
                radius: mk::radius::LARGE,
                home: Rect::of(&b.scene.placed[anchor]),
                anchor,
                clip: None,
            });
        }
        if let Some(hit) = &self.current {
            let st = mk::typo::SUBTITLE;
            let line = mk::elide(&hit.title, 60);
            let (tw, th) = engine.measure_styled(&line, st.px(), st.weight);
            let ty = by + box_h + mk::spacing::M;
            b.styled(engine, line, st, white, stage.x + (stage.w - tw) / 2.0, ty, Some(BAR_BASE), None);
            let cap = mk::typo::CALLOUT;
            let (cw, _) = engine.measure_styled(&hit.channel, cap.px(), cap.weight);
            b.styled(engine, hit.channel.clone(), cap, mk::Rgba { a: 0.7, ..white }, stage.x + (stage.w - cw) / 2.0, ty + th + 2.0, Some(BAR_BASE), None);
        }
    }

    /// Download the running video (list): MP3 or MP4, progress in the view.
    /// While one runs, the next waits in the queue (it starts on its own).
    pub fn start_download(&mut self, keep: Keep) {
        use matrixkit_tube::downloads::{Job, Pushed};
        let Some(hit) = &self.current else { return };
        if hit.id.starts_with('/') {
            return;
        }
        let job = Job::new(&hit.id, &hit.title, keep);
        mk::feel::tap();
        match self.downloads.push(job.clone()) {
            Pushed::Start => self.run_download(job),
            Pushed::Waits(n) => {
                eprintln!("[tube] Download {} als {} wartet ({n} vor ihm)", job.id, keep.word());
                self.dl_note.set(format!("{} wartet — {n} vor ihm", keep.word().to_uppercase()));
            }
            Pushed::Twice => {}
        }
    }

    fn run_download(&mut self, job: matrixkit_tube::downloads::Job) {
        eprintln!("[tube] Download {} als {}", job.id, job.keep.word());
        if let Some(loom) = &self.loom {
            let feeder = loom.clone();
            let (id, keep) = (job.id, job.keep);
            loom.spin(move || {
                let result = matrixkit_tube::download(&id, keep, |pct| feeder.feed(Work::DlProgress(pct)));
                Work::DlDone(result)
            });
        }
    }

    /// The download pills in the player view (or the progress).
    pub fn build_download(&self, engine: &mut TextEngine, b: &mut SceneBuilder, x: f32, y: f32) -> f32 {
        let p = self.palette;
        let Some(current) = self.current.as_ref() else { return 0.0 };
        if current.id.starts_with('/') {
            return 0.0;
        }
        let gap = mk::spacing::S;
        // This video is downloading: its progress — at 100 % the file is
        // there and ffmpeg converts it (an album: minutes, measured 02.11.2026).
        if let Some(job) = self.downloads.running() {
            if job.id == current.id {
                let text = job.phase();
                let (_, h) = fam::lay_pill(b, engine, Some(symbol::DOWNLOAD), &text, x, y, None, None, fam::PillTone::Accent, p);
                return h + gap;
            }
        }
        if let Some(note) = self.dl_note.text() {
            let (_, h) = fam::lay_pill(b, engine, Some(symbol::CHECK), note, x, y, None, None, fam::PillTone::Quiet, p);
            return h + gap;
        }
        let waits = |k: Keep| self.downloads.waits(&current.id, k);
        let label = |k: Keep| if waits(k) { format!("{} wartet", k.word().to_uppercase()) } else { k.word().to_uppercase() };
        let (w1, h) = fam::lay_pill(b, engine, Some(symbol::DOWNLOAD), &label(Keep::Mp3), x, y, None, Some(DL_MP3), fam::PillTone::Quiet, p);
        let (w2, _) = fam::lay_pill(b, engine, Some(symbol::DOWNLOAD), &label(Keep::Mp4), x + w1 + gap, y, None, Some(DL_MP4), fam::PillTone::Quiet, p);
        // Another video downloads meanwhile: say so, quietly.
        if let Some(job) = self.downloads.running() {
            let state = if job.percent >= 100.0 { String::from("wandelt um") } else { format!("{:.0} %", job.percent) };
            let text = format!("Im Hintergrund: {} {} · {}", job.keep.word().to_uppercase(), mk::elide(&job.title, 24), state);
            fam::lay_pill(b, engine, None, &text, x + w1 + w2 + 2.0 * gap, y, None, None, fam::PillTone::Quiet, p);
        }
        h + gap
    }

    /// A finished download comes home.
    pub fn download_done(&mut self, result: Result<Item, String>) -> Refresh {
        let (done, next) = self.downloads.finish();
        let keep = done.map(|d| d.keep);
        match result {
            Ok(item) => {
                eprintln!("[tube] gespeichert: {}", item.path.display());
                let place = match item.keep {
                    Keep::Mp3 => "Musik",
                    Keep::Mp4 => "Videos",
                };
                self.dl_note.set(format!("Gespeichert in {place}/Matrix Tube"));
                self.library = Library::load();
            }
            Err(e) => {
                eprintln!("[tube] Download scheiterte ({:?}): {e}", keep);
                self.dl_note.set(format!("Download scheiterte: {}", mk::elide(&e, 40)));
            }
        }
        // The next one in the queue starts on its own.
        if let Some(job) = next {
            self.run_download(job);
        }
        Refresh::Dirty
    }
}

/// A local file as a hit: its path is the id.
pub fn local_hit(path: &std::path::Path, item: Option<&Item>) -> Hit {
    Hit {
        id: path.to_string_lossy().into_owned(),
        title: item.map(|i| i.title.clone()).unwrap_or_else(|| path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()),
        channel: item.map(|i| i.channel.clone()).unwrap_or_default(),
        length: item.map(|i| length(i.duration)).unwrap_or_default(),
        views: String::new(),
        age: String::new(),
        snippet: String::new(),
    }
}
