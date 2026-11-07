//! List and stage talk (R210 duo form, the owner 01.11.2026): the stage on
//! one panel shows ONLY the video; the list on the other panel turns into
//! the player view — back to the search, title and channel, the media bar,
//! quality and subtitles. The grammar is the video family's remote
//! (`matrixkit_video::remote`): the list sends commands, the stage answers
//! four times a second with its `PlayerState`.

use crate::{Mode, Tube, BAR_BASE};
use matrixkit_scene::scene as fam;
use matrixkit_scene::symbol;
use matrixkit_shell::{Refresh, SceneBuilder};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_video::remote::{Command, PlayerState};

/// The list's companion role (the stage is `crate::STAGE_ROLE`).
pub const LIST_ROLE: &str = "liste";

// Roles of the player view (below the chrome 500 and root 700 bases).
pub const BACK: usize = 4;
pub const NOW_ROW: usize = 5;
/// «Als Nächstes» now, and calling it off.
pub const NEXT_NOW: usize = 6;
pub const CANCEL: usize = 7;
pub const AUTOPLAY_BASE: usize = 40;
/// The chip row: quality, subtitles (unfold), autoplay (switch).
pub const CHIP_QUALITY: usize = 8;
pub const CHIP_SUBS: usize = 9;
pub const CHIP_AUTOPLAY: usize = 10;
pub const CHIP_SLEEP: usize = 11;
/// «Ja, weiter» on the «Bist du noch da?» card.
pub const STILL_HERE: usize = 12;
pub const SLEEP_BASE: usize = 30;
/// The sleep timer's menu: label and minutes (0 = at the video's end).
pub const SLEEPS: [(&str, Option<u32>); 6] = [("Aus", None), ("30 min", Some(30)), ("1 h", Some(60)), ("1½ h", Some(90)), ("2 h", Some(120)), ("Videoende", Some(0))];
/// How long «Bist du noch da?» waits before the screens go off.
pub const ASK_S: f32 = 20.0 * 60.0;
/// The recommendation rows.
pub const REL_BASE: usize = 200;
/// Playlist steps and rows (R210, 02.11.2026).
pub const PREV: usize = 18;
pub const NEXT_TRACK: usize = 19;
pub const QUEUE_BASE: usize = 3000;
/// Seconds of countdown at the end before the next video starts.
pub const COUNTDOWN_S: f32 = 5.0;
pub const SUBSIZE_BASE: usize = 45;
pub const QUALITY_BASE: usize = 50;
pub const SUBS_BASE: usize = 60;

/// The quality menu: label and the highest picture asked for (0 = Auto:
/// 1080p, down to 720p by itself when the decoder falls behind).
pub const QUALITIES: [(&str, u32); 5] = [("Auto", 0), ("1080p", 1080), ("720p", 720), ("480p", 480), ("360p", 360)];

/// A command to the stage.
pub fn to_stage(c: Command) {
    mk::companion::send(crate::APP_ID, crate::STAGE_ROLE, &c.word());
}

impl Tube {
    /// The stage tells the list where it stands (every 250 ms).
    pub fn report(&mut self) {
        // Cinema mode follows the playing picture (stage, or the one window).
        if self.mode != Mode::List {
            // Sound only (an MP3) is no cinema.
            let picture = self.watch.as_ref().is_some_and(|w| w.media.width > 0);
            let playing = picture && self.video.as_ref().is_some_and(|v| !v.paused() && !v.done() && v.running());
            self.kino.follow(playing);
        }
        if self.mode != Mode::Stage || self.last_report.elapsed().as_millis() < 250 {
            return;
        }
        self.last_report = std::time::Instant::now();
        let Some(hit) = &self.current else { return };
        let (position, playing, ahead) = match &self.video {
            Some(v) => (v.position_s() as f32, !v.paused() && !v.done(), v.stocked_s()),
            None => (self.resume_at as f32, false, None),
        };
        let state = PlayerState {
            id: hit.id.clone(),
            title: hit.title.clone(),
            channel: hit.channel.clone(),
            position,
            duration: self.watch.as_ref().map(|w| w.media.duration_s as f32).unwrap_or(0.0),
            playing,
            ahead,
            marks: self.skips.iter().map(|s| (s.from as f32, s.to as f32)).collect(),
            quality: self.quality,
            subs: self.watch.as_ref().map(|w| w.subtitles.iter().map(|t| (t.lang.clone(), t.name.clone())).collect()).unwrap_or_default(),
            sub: self.sub_lang.clone(),
            sub_size: self.sub_size,
            status: self.status.clone(),
            autoplay: self.autoplay,
            sleep: self.sleep_choice,
            sleep_left: self.sleep_at.map(|t| t.saturating_duration_since(std::time::Instant::now()).as_secs_f32()),
            asking: self.asking_since.map(|t| (ASK_S - t.elapsed().as_secs_f32()).max(0.0)),
            playlist: self.queue_name.clone(),
            queue: self.queue_info.clone(),
            queue_at: self.queue.at(),
            countdown: self.next_at.map(|at| (COUNTDOWN_S - at.elapsed().as_secs_f32()).max(0.0)),
            up_next: self.up_next.iter().take(12).map(|h| (h.id.clone(), h.title.clone(), h.channel.clone(), h.length.clone())).collect(),
        };
        mk::companion::send(crate::APP_ID, LIST_ROLE, &state.word());
    }

    /// The list's player view: back, titles, media bar, quality, subtitles.
    pub fn build_player(&mut self, engine: &mut TextEngine, b: &mut SceneBuilder, wf: f32, hf: f32, top: f32) {
        let p = self.palette;
        let m = mk::spacing::M;
        let r = self.remote.clone().unwrap_or_default();
        let hit = self.current.clone();
        // The view scrolls: everything moves up by the roll.
        let mut y = top + mk::spacing::S - self.player_roll.value().max(0.0);
        let start = y;
        // Round 173: wide (≥ 820 px) = controls left (55 %), lists right.
        let wide = (wf >= 820.0).then(|| (wf * 0.55).floor());
        let full_w = wf;
        let wf = wide.unwrap_or(wf);
        // The top row (owner 02.11.2026): back to the search on the left,
        // quality · subtitles · autoplay on the right — one line.
        let w = wf - 2.0 * m;
        let quality = QUALITIES.iter().find(|(_, h)| *h == r.quality).map(|(l, _)| *l).unwrap_or("Auto");
        let sub_name = match &r.sub {
            None => String::from("Aus"),
            Some(lang) => r.subs.iter().find(|(l, _)| l == lang).map(|(_, n)| n.split(" (").next().unwrap_or(n).to_string()).unwrap_or_else(|| lang.clone()),
        };
        let gap = mk::spacing::S;
        let sleep_value = match (r.sleep, r.sleep_left) {
            (None, _) => String::from("Aus"),
            (Some(0), _) => String::from("Videoende"),
            (Some(_), Some(left)) => {
                let min = (left / 60.0).ceil() as u32;
                format!("{}:{:02}", min / 60, min % 60)
            }
            (Some(m), None) => format!("{m} min"),
        };
        // Round 173 (owner: «optimise for all views»): the row
        // loses its words step by step — «Zurück zur Suche», «Suche», the
        // arrow alone, then the chips as symbols (quality keeps its value);
        // if even that does not fit, the chips move to a second line.
        let chip_widths = |engine: &mut TextEngine, words: bool| -> [f32; 4] {
            let v = |s: &str| if words { s.to_string() } else { String::new() };
            [
                fam::value_chip_width(engine, symbol::HIGH_QUALITY, quality, false),
                fam::value_chip_width(engine, symbol::CLOSED_CAPTION, &v(&sub_name), false).min(150.0),
                fam::value_chip_width(engine, symbol::REPEAT, &v("Autoplay"), true),
                fam::value_chip_width(engine, symbol::TIMER, &v(&sleep_value), false),
            ]
        };
        let full = chip_widths(engine, true);
        let bare = chip_widths(engine, false);
        let sum = |c: &[f32; 4]| c.iter().sum::<f32>() + 3.0 * gap;
        let back = |engine: &mut TextEngine, t: &str| fam::pill_width(engine, Some(symbol::ARROW_BACK), t) + gap;
        let levels = [
            ("Zurück zur Suche", true),
            ("Suche", true),
            ("", true),
            ("", false),
        ];
        let widths: Vec<f32> = levels.iter().map(|(t, words)| back(engine, t) + sum(if *words { &full } else { &bare })).collect();
        let level = fam::row_fit(&widths, w);
        let (back_text, words) = levels[level];
        let fits = widths[level] <= w;
        let chips = if words { full } else { bare };
        let v = |s: &str| if words { s.to_string() } else { String::new() };
        let (_, bh) = fam::lay_pill(b, engine, Some(symbol::ARROW_BACK), back_text, m, y, None, Some(BACK), fam::PillTone::Quiet, p);
        // Not even symbols fit beside the arrow: the chips take a line below.
        let (mut x, chip_y) = if fits { (m + w - sum(&chips), y + (bh - 36.0) / 2.0) } else { (m, y + bh + mk::spacing::S) };
        let (qw, _) = fam::lay_value_chip(b, engine, symbol::HIGH_QUALITY, quality, self.unfolded == 1, None, x, chip_y, chips[0], CHIP_QUALITY, p);
        x += qw + gap;
        let (sw, _) = fam::lay_value_chip(b, engine, symbol::CLOSED_CAPTION, &v(&sub_name), self.unfolded == 2, None, x, chip_y, chips[1], CHIP_SUBS, p);
        x += sw + gap;
        let (aw, _) = fam::lay_value_chip(b, engine, symbol::REPEAT, &v("Autoplay"), false, Some(r.autoplay), x, chip_y, chips[2], CHIP_AUTOPLAY, p);
        x += aw + gap;
        let _ = fam::lay_value_chip(b, engine, symbol::TIMER, &v(&sleep_value), self.unfolded == 3, None, x, chip_y, chips[3], CHIP_SLEEP, p);
        y += bh + mk::spacing::S + if fits { 0.0 } else { 36.0 + mk::spacing::S };
        // The unfolded choices, right under the row.
        y += self.lay_unfolded(engine, b, &r, m, y, w);
        y += mk::spacing::S;

        // Title (two lines) and channel · views · age.
        let title = if !r.title.is_empty() { r.title.clone() } else { hit.as_ref().map(|h| h.title.clone()).unwrap_or_default() };
        let sub = mk::typo::SUBTITLE;
        let chars = (((wf - 2.0 * m) / (sub.px() * 0.5)) as usize).max(16);
        for line in mk::wrap_words(&title, chars, 2) {
            let (_, lh) = b.styled(engine, line, sub, p.on_surface, m, y, None, None);
            y += lh;
        }
        let channel = if !r.channel.is_empty() { r.channel.clone() } else { hit.as_ref().map(|h| h.channel.clone()).unwrap_or_default() };
        let meta = match &hit {
            Some(h) => fam::dot_join(&[&channel, &h.views, &h.age]),
            None => channel.clone(),
        };
        let cap = mk::typo::CAPTION;
        let (_, ch) = b.styled(engine, mk::elide(&meta, chars + 10), cap, p.on_surface_variant, m, y + 2.0, None, None);
        y += ch + m;

        // The media bar steers the stage.
        // Round 172 (the owner): a playlist steers IN the bar — previous
        // before play, next at its end — no row of its own below it.
        let queued = r.queue.len() > 1;
        let view = fam::MediaView {
            position: r.position,
            duration: r.duration,
            playing: r.playing,
            ahead: r.ahead,
            marks: &r.marks,
            prev: queued.then_some(true),
            next: queued.then_some(r.queue_at + 1 < r.queue.len()),
        };
        y += self.bar.build(engine, b, y, wf, &view, p);
        if !r.status.is_empty() {
            let (_, sh) = b.styled(engine, mk::elide(&r.status, 70), cap, p.on_surface_variant, m, y + mk::spacing::XS, None, None);
            y += sh + mk::spacing::XS;
        }
        y += m;
        // Keep it: MP3 or MP4 into the library (`library`).
        y += self.build_download(engine, b, m, y);

        // Round 173: a wide window (a TV, a desk) sets the lists BESIDE the
        // controls — the same roll scrolls both columns.
        if let Some(lw) = wide {
            let mut col = SceneBuilder::new();
            // `wf` is the controls' width here; the column takes the rest.
            let end = self.build_player_lists(engine, &mut col, &r, full_w - lw - m, hf, start);
            b.append_shifted(&col.into_part(), lw - m);
            y = y.max(end);
        } else {
            y = self.build_player_lists(engine, b, &r, w, hf, y);
        }
        // How far the view can scroll (its content below the panel).
        self.player_max = (y + self.player_roll.value().max(0.0) + m - hf).max(0.0);
        let wf = full_w;

        // «Bist du noch da?» over everything.
        if let Some(left) = r.asking {
            b.quad(0.0, 0.0, wf, hf, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.6 }, 0.0, Some(STILL_HERE), None);
            let cw = (wf - 2.0 * m).min(420.0);
            let ch = 170.0;
            let (cx, cy) = ((wf - cw) / 2.0, (hf - ch) / 2.0);
            b.quad(cx, cy, cw, ch, p.surface_container_high, mk::radius::LARGE, Some(STILL_HERE), None);
            let t = mk::typo::TITLE;
            let (tw, th) = engine.measure_styled("Bist du noch da?", t.px(), t.weight);
            b.styled(engine, "Bist du noch da?", t, p.on_surface, cx + (cw - tw) / 2.0, cy + m, Some(STILL_HERE), None);
            let note = format!("Der Schlaftimer hat pausiert. Bildschirme aus in {}:{:02}.", (left as u32) / 60, (left as u32) % 60);
            let (nw, nh) = engine.measure_styled(&note, cap.px(), cap.weight);
            b.styled(engine, note, cap, p.on_surface_variant, cx + (cw - nw) / 2.0, cy + m + th + mk::spacing::XS, Some(STILL_HERE), None);
            let bw = fam::pill_width(engine, Some(symbol::PLAY_ARROW), "Ja, weiter");
            fam::lay_pill(b, engine, Some(symbol::PLAY_ARROW), "Ja, weiter", cx + (cw - bw) / 2.0, cy + m + th + nh + m + mk::spacing::S, None, Some(STILL_HERE), fam::PillTone::Accent, p);
        }
    }

    /// The player's lists — the playlist and «Als Nächstes» — from `y`
    /// across `w` at margin `m`; returns where they end (round 173: beside
    /// the controls in a wide window, below them in a narrow one).
    fn build_player_lists(&mut self, engine: &mut TextEngine, b: &mut SceneBuilder, r: &PlayerState, w: f32, hf: f32, mut y: f32) -> f32 {
        let p = self.palette;
        let m = mk::spacing::M;
        let cap = mk::typo::CAPTION;
        // THE PLAYLIST: every entry, the playing one marked, the next one
        // named — a tap jumps there.
        if r.queue.len() > 1 {
            let head = match &r.playlist {
                Some(name) => format!("Playlist «{name}» · {} von {}", r.queue_at + 1, r.queue.len()),
                None => format!("Wiedergabeliste · {} von {}", r.queue_at + 1, r.queue.len()),
            };
            let (_, ch) = b.styled(engine, head, cap, p.on_surface_variant, m, y, None, None);
            y += ch + mk::spacing::XS;
            for (i, (id, title, channel, length)) in r.queue.iter().enumerate() {
                let now = i == r.queue_at;
                let tag = if now { "Läuft" } else if i == r.queue_at + 1 { "Als Nächstes" } else { "" };
                let meta = fam::dot_join(&[tag, channel, length]);
                let thumb = self.library.item(std::path::Path::new(id)).and_then(|it| it.thumb.clone());
                let row = fam::MediaRow { thumb: thumb.as_deref(), now: now.then_some(r.playing), ..fam::MediaRow::new(title, &meta) };
                if y <= hf && y + row.height() >= 0.0 {
                    fam::lay_media_row(b, engine, &row, m, y, w, Some(QUEUE_BASE + i), None, p);
                }
                y += row.height();
            }
            y += m;
        }

        // «Als Nächstes»: near the end (or counting down) a banner with the
        // way out, then the recommendations.
        if let Some((_, title, ..)) = r.up_next.first() {
            let near_end = r.duration > 0.0 && r.duration - r.position < 10.0;
            if r.autoplay && (r.countdown.is_some() || near_end) {
                let text = match r.countdown {
                    Some(c) => format!("Als Nächstes in {} s: {title}", c.ceil() as u32),
                    None => format!("Als Nächstes: {title}"),
                };
                let cw = fam::pill_width(engine, None, "Abbrechen");
                let (_, bh) = fam::lay_pill(b, engine, Some(symbol::PLAY_ARROW), &text, m, y, Some(w - cw - mk::spacing::S), Some(NEXT_NOW), fam::PillTone::Accent, p);
                fam::lay_pill(b, engine, None, "Abbrechen", m + w - cw, y, Some(cw), Some(CANCEL), fam::PillTone::Quiet, p);
                y += bh + m;
            }
            let (_, ch) = b.styled(engine, "Als Nächstes", cap, p.on_surface_variant, m, y, None, None);
            y += ch + mk::spacing::XS;
            for (i, (id, title, channel, length)) in r.up_next.iter().enumerate() {
                let meta = fam::dot_join(&[channel, length]);
                let row = fam::MediaRow { thumb: self.thumbs.get(id).map(std::path::PathBuf::as_path), ..fam::MediaRow::new(title, &meta) };
                if y <= hf && y + row.height() >= 0.0 {
                    fam::lay_media_row(b, engine, &row, m, y, w, Some(REL_BASE + i), None, p);
                }
                y += row.height();
            }
        }
        y
    }

    /// The unfolded chip's choices (quality, or subtitles with their size).
    fn lay_unfolded(&self, engine: &mut TextEngine, b: &mut SceneBuilder, r: &PlayerState, x: f32, y: f32, w: f32) -> f32 {
        let p = self.palette;
        let m = mk::spacing::M;
        match self.unfolded {
            1 => {
                let options: Vec<String> = QUALITIES.iter().map(|(l, _)| l.to_string()).collect();
                let active = QUALITIES.iter().position(|(_, h)| *h == r.quality);
                fam::lay_choices(b, engine, &options, active, x, y, w, QUALITY_BASE, p) + m
            }
            2 => {
                let (options, active) = if r.subs.is_empty() {
                    (vec![String::from(if r.duration > 0.0 { "Keine angeboten" } else { "Lädt …" })], None)
                } else {
                    let mut o = vec![String::from("Aus")];
                    o.extend(r.subs.iter().map(|(_, name)| name.clone()));
                    let a = match &r.sub {
                        None => Some(0),
                        Some(lang) => r.subs.iter().position(|(l, _)| l == lang).map(|i| i + 1),
                    };
                    (o, a)
                };
                let mut h = fam::lay_choices(b, engine, &options, active, x, y, w, SUBS_BASE, p) + m;
                // The size, with the subtitles (in cinema mode they stand
                // in the dock's place).
                if r.sub.is_some() {
                    let sizes: Vec<String> = mk::cinema::subtitle::SIZES.iter().map(|(l, _)| l.to_string()).collect();
                    h += fam::lay_choice_section(b, engine, "Grösse", &sizes, Some(r.sub_size), x, y + h, w, SUBSIZE_BASE, p);
                }
                h
            }
            3 => {
                let options: Vec<String> = SLEEPS.iter().map(|(l, _)| l.to_string()).collect();
                let active = SLEEPS.iter().position(|(_, m)| *m == r.sleep);
                fam::lay_choice_section(b, engine, "Schlaftimer — pausiert und fragt «Bist du noch da?»", &options, active, x, y, w, SLEEP_BASE, p)
            }
            _ => 0.0,
        }
    }

    /// The «now playing» row above the search results (back to the player).
    pub fn build_now_row(&self, engine: &mut TextEngine, b: &mut SceneBuilder, wf: f32, y: f32) -> f32 {
        let Some(hit) = &self.current else { return 0.0 };
        let m = mk::spacing::M;
        let title = self.remote.as_ref().map(|r| r.title.clone()).filter(|t| !t.is_empty()).unwrap_or_else(|| hit.title.clone());
        let (_, h) = fam::lay_pill(b, engine, Some(symbol::PLAY_ARROW), &format!("Läuft: {title}"), m, y, Some(wf - 2.0 * m), Some(NOW_ROW), fam::PillTone::Accent, self.palette);
        h + mk::spacing::S
    }

    /// Presses in the list's player view (and the now row). None = not ours.
    pub fn player_strike(&mut self, role: Option<usize>, x: f32) -> Option<Refresh> {
        match role {
            Some(BACK) => {
                self.player_view = false;
                self.list_cache = None;
                Some(Refresh::Dirty)
            }
            Some(NOW_ROW) => {
                self.player_view = true;
                Some(Refresh::Dirty)
            }
            Some(STILL_HERE) if self.player_view => {
                to_stage(Command::StillHere);
                if let Some(state) = &mut self.remote {
                    state.asking = None;
                    state.playing = true;
                }
                mk::feel::tap();
                Some(Refresh::Dirty)
            }
            Some(CHIP_SLEEP) if self.player_view => {
                self.unfolded = if self.unfolded == 3 { 0 } else { 3 };
                Some(Refresh::Dirty)
            }
            Some(r) if self.player_view && (SLEEP_BASE..SLEEP_BASE + SLEEPS.len()).contains(&r) => {
                let m = SLEEPS[r - SLEEP_BASE].1;
                to_stage(Command::Sleep(m));
                if let Some(state) = &mut self.remote {
                    state.sleep = m;
                    state.sleep_left = m.filter(|m| *m > 0).map(|m| m as f32 * 60.0);
                }
                self.unfolded = 0;
                mk::feel::tap();
                Some(Refresh::Dirty)
            }
            Some(CHIP_QUALITY) if self.player_view => {
                self.unfolded = if self.unfolded == 1 { 0 } else { 1 };
                Some(Refresh::Dirty)
            }
            Some(CHIP_SUBS) if self.player_view => {
                self.unfolded = if self.unfolded == 2 { 0 } else { 2 };
                Some(Refresh::Dirty)
            }
            Some(r) if self.player_view && (r == CHIP_AUTOPLAY || (AUTOPLAY_BASE..AUTOPLAY_BASE + 2).contains(&r)) => {
                let on = if r == CHIP_AUTOPLAY { !self.remote.as_ref().is_some_and(|s| s.autoplay) } else { r == AUTOPLAY_BASE };
                to_stage(Command::Autoplay(on));
                if let Some(state) = &mut self.remote {
                    state.autoplay = on;
                }
                mk::feel::tap();
                Some(Refresh::Dirty)
            }
            Some(CANCEL) if self.player_view => {
                to_stage(Command::Cancel);
                if let Some(state) = &mut self.remote {
                    state.countdown = None;
                    state.autoplay = false;
                }
                mk::feel::tap();
                Some(Refresh::Dirty)
            }
            Some(PREV) if self.player_view => {
                // Classic players: into the song, back to its start; at its
                // start, the one before.
                let (at, pos) = self.remote.as_ref().map(|s| (s.queue_at, s.position)).unwrap_or((0, 0.0));
                match matrixkit_video::queue::back_step(pos, at) {
                    matrixkit_video::queue::Back::Restart => to_stage(Command::Seek(0.0)),
                    matrixkit_video::queue::Back::To(k) => {
                        to_stage(Command::QueueAt(k));
                        if let Some(state) = &mut self.remote {
                            state.queue_at = k;
                            state.position = 0.0;
                        }
                    }
                }
                mk::feel::tap();
                Some(Refresh::Dirty)
            }
            Some(NEXT_TRACK) if self.player_view => {
                if let Some(state) = &mut self.remote {
                    if state.queue_at + 1 < state.queue.len() {
                        state.queue_at += 1;
                        state.position = 0.0;
                        to_stage(Command::QueueAt(state.queue_at));
                    }
                }
                mk::feel::tap();
                Some(Refresh::Dirty)
            }
            Some(r) if self.player_view && (QUEUE_BASE..QUEUE_BASE + 400).contains(&r) => {
                // Like the recommendations: plays on release (scrolling).
                self.press = Some(mk::touch::Press::new(0.0, 0.0, self.player_roll.target, Some(r)));
                Some(Refresh::Calm)
            }
            Some(r) if self.player_view && (r == NEXT_NOW || (REL_BASE..REL_BASE + 40).contains(&r)) => {
                // A row only plays on RELEASE of a tap: a finger that moves
                // scrolls the view (owner 02.11.2026: «can't scroll»).
                let k = if r == NEXT_NOW { 0 } else { r - REL_BASE };
                self.press = Some(mk::touch::Press::new(0.0, 0.0, self.player_roll.target, Some(k)));
                Some(Refresh::Calm)
            }
            Some(r) if self.player_view && (SUBSIZE_BASE..SUBSIZE_BASE + mk::cinema::subtitle::SIZES.len()).contains(&r) => {
                let k = r - SUBSIZE_BASE;
                to_stage(Command::SubSize(k));
                if let Some(state) = &mut self.remote {
                    state.sub_size = k;
                }
                mk::feel::tap();
                Some(Refresh::Dirty)
            }
            Some(r) if self.player_view && (QUALITY_BASE..QUALITY_BASE + QUALITIES.len()).contains(&r) => {
                let h = QUALITIES[r - QUALITY_BASE].1;
                to_stage(Command::Quality(h));
                self.unfolded = 0;
                if let Some(state) = &mut self.remote {
                    state.quality = h;
                }
                mk::feel::tap();
                Some(Refresh::Dirty)
            }
            Some(r) if self.player_view && (SUBS_BASE..SUBS_BASE + 40).contains(&r) => {
                let k = r - SUBS_BASE;
                let lang = if k == 0 { None } else { self.remote.as_ref().and_then(|s| s.subs.get(k - 1)).map(|(l, _)| l.clone()) };
                to_stage(Command::Subs(lang.clone()));
                if let Some(state) = &mut self.remote {
                    state.sub = lang;
                }
                mk::feel::tap();
                Some(Refresh::Dirty)
            }
            _ if self.player_view => match self.bar.strike(role, x) {
                // Round 172: the bar's skip keys are the playlist's steps.
                Some(fam::MediaDeed::Prev) => self.player_strike(Some(PREV), x),
                Some(fam::MediaDeed::Next) => self.player_strike(Some(NEXT_TRACK), x),
                Some(fam::MediaDeed::Toggle) => {
                    to_stage(Command::Toggle);
                    if let Some(state) = &mut self.remote {
                        state.playing = !state.playing;
                    }
                    Some(Refresh::Dirty)
                }
                Some(fam::MediaDeed::Scrub) => Some(Refresh::Dirty),
                None => Some(Refresh::Calm),
            },
            _ => None,
        }
    }

    /// A recommendation was tapped (released without moving): play it.
    pub fn play_next(&mut self, k: usize) -> Refresh {
        // A playlist row (its role was kept as the press's row).
        if k >= QUEUE_BASE {
            let k = k - QUEUE_BASE;
            to_stage(Command::QueueAt(k));
            if let Some(state) = &mut self.remote {
                state.queue_at = k;
                state.position = 0.0;
            }
            mk::feel::tap();
            return Refresh::Dirty;
        }
        let next = self.remote.as_ref().and_then(|s| s.up_next.get(k).cloned());
        if let Some((id, title, channel, length)) = next {
            to_stage(Command::Play(id.clone()));
            self.current = Some(matrixkit_tube::Hit { id, title, channel, length, views: String::new(), age: String::new(), snippet: String::new() });
            self.remote = None;
            self.player_roll = mk::motion::Spring::smooth(0.0);
            mk::feel::tap();
        }
        Refresh::Dirty
    }

    /// The stage obeys a command of the list.
    pub fn obey(&mut self, word: &str) -> Refresh {
        match Command::parse(word) {
            Some(Command::Play(id)) => {
                let hit = matrixkit_tube::Hit {
                    id,
                    title: String::from("Lädt …"),
                    channel: String::new(),
                    length: String::new(),
                    views: String::new(),
                    age: String::new(),
                    snippet: String::new(),
                };
                self.play_hit(hit)
            }
            Some(Command::Toggle) => self.toggle_pause(),
            // Round 122: the list takes over (one panel) — stop without the
            // background sound and close.
            Some(Command::Quit) => {
                eprintln!("[tube] Bühne: die Liste übernimmt — Ende");
                if let Some(mut v) = self.video.take() {
                    v.stop();
                }
                self.current = None;
                Refresh::Close
            }
            Some(Command::Seek(t)) => {
                if let Some(video) = &mut self.video {
                    eprintln!("[tube] Sprung zu {t:.1} s (Liste)");
                    video.play(t);
                }
                Refresh::Dirty
            }
            Some(Command::Quality(h)) => {
                if h != self.quality {
                    eprintln!("[tube] Qualität {h}");
                    self.quality = h;
                    self.reload_here();
                }
                Refresh::Dirty
            }
            Some(Command::Subs(lang)) => {
                self.cues.clear();
                self.sub_lang = lang.clone();
                if let Some(lang) = lang {
                    let track = self.watch.as_ref().and_then(|w| w.subtitles.iter().find(|t| t.lang == lang).cloned());
                    if let (Some(track), Some(loom)) = (track, &self.loom) {
                        loom.spin(move || crate::Work::Cues(lang, matrixkit_tube::cues(&track)));
                    }
                }
                Refresh::Dirty
            }
            Some(Command::Autoplay(on)) => {
                self.autoplay = on;
                mk::settings::write("tube-autoplay", if on { "on" } else { "off" });
                if !on {
                    self.next_at = None;
                }
                Refresh::Dirty
            }
            Some(Command::Cancel) => {
                self.next_cancelled = true;
                self.next_at = None;
                Refresh::Dirty
            }
            Some(Command::Sleep(m)) => {
                eprintln!("[tube] Schlaftimer: {m:?}");
                self.sleep_choice = m;
                self.sleep_at = m.filter(|m| *m > 0).map(|m| std::time::Instant::now() + std::time::Duration::from_secs(m as u64 * 60));
                Refresh::Dirty
            }
            Some(Command::StillHere) => self.still_here(),
            Some(Command::QueueAt(k)) => self.queue_jump(k),
            Some(Command::PlayFiles(files)) => self.play_files(files.into_iter().map(std::path::PathBuf::from).collect()),
            Some(Command::SubSize(k)) => {
                self.sub_size = k.min(mk::cinema::subtitle::SIZES.len() - 1);
                mk::cinema::subtitle::save_size(self.sub_size);
                self.strip_text = None;
                Refresh::Dirty
            }
            None => Refresh::Calm,
        }
    }

    /// SLEEP TIMER (stage): when it runs out (or the video ends, choice 0)
    /// the video pauses and asks «Bist du noch da?»; unanswered for 20 min,
    /// the screens go off. True while something counts (redraw).
    pub fn sleep_tick(&mut self) -> bool {
        if self.mode == Mode::List {
            return false;
        }
        if let Some(since) = self.asking_since {
            if since.elapsed().as_secs_f32() >= ASK_S {
                eprintln!("[tube] Schlaftimer: keine Antwort — Bildschirme aus");
                self.asking_since = None;
                std::thread::spawn(mk::canvas::screens_off);
            }
            return true;
        }
        let ended = self.video.as_ref().is_some_and(|v| v.done());
        let due = self.sleep_at.is_some_and(|t| std::time::Instant::now() >= t) || (self.sleep_choice == Some(0) && ended);
        if due {
            eprintln!("[tube] Schlaftimer abgelaufen — «Bist du noch da?»");
            if let Some(v) = &mut self.video {
                if !v.done() {
                    v.set_paused(true);
                }
            }
            self.asking_since = Some(std::time::Instant::now());
            self.sleep_at = None;
            self.sleep_choice = None;
            return true;
        }
        self.sleep_at.is_some()
    }

    /// «Bist du noch da?» — yes: play on (the timer is off now).
    pub fn still_here(&mut self) -> Refresh {
        if self.asking_since.take().is_some() {
            eprintln!("[tube] noch da — weiter");
            if let Some(v) = &mut self.video {
                if v.paused() && !v.done() {
                    v.set_paused(false);
                }
            }
        }
        Refresh::Dirty
    }

    /// Switch the list's panel light off (true) or on; returns whether it
    /// was dark before. Off only on the second duo panel (`panel_light_off`).
    pub fn panel_light(&mut self, off: bool) -> bool {
        let was = self.panel_dark;
        if off && !was {
            let me = std::process::id();
            let output = mk::canvas::app_windows(crate::APP_ID).into_iter().find(|w| w.pid == me).map(|w| w.output);
            if output.is_some_and(|o| mk::device::panel_light_off(&o)) {
                eprintln!("[tube] unteres Panel: Licht aus (2 min nur geschaut)");
                self.panel_dark = true;
            }
        } else if !off && was {
            eprintln!("[tube] unteres Panel: Licht an");
            mk::device::panel_light_on();
            self.panel_dark = false;
        }
        was
    }

    /// The list may dim: player view, the stage plays, cinema mode, no question.
    pub fn idle_wanted(&self) -> bool {
        self.mode == Mode::List
            && self.player_view
            && self.remote.as_ref().is_some_and(|r| r.playing && r.asking.is_none())
            && mk::cinema::active()
    }

    /// AUTOPLAY (stage): at the end, count down and start what comes next.
    /// True while it counts (the views redraw).
    pub fn autoplay_tick(&mut self) -> bool {
        if self.mode == Mode::List || !self.autoplay || self.next_cancelled || self.up_next.is_empty() || self.asking_since.is_some() {
            return false;
        }
        if !self.video.as_ref().is_some_and(|v| v.done()) {
            self.next_at = None;
            return false;
        }
        let at = *self.next_at.get_or_insert_with(std::time::Instant::now);
        if at.elapsed().as_secs_f32() >= COUNTDOWN_S {
            let next = self.up_next[0].clone();
            eprintln!("[tube] Autoplay: {} «{}»", next.id, next.title);
            let _ = self.play_hit(next);
        }
        true
    }

    /// Subtitles go to the dock's strip: cinema mode (playing) with a track.
    pub fn strip_on(&self) -> bool {
        self.mode != Mode::List && self.kino.is_on() && self.sub_lang.is_some() && mk::device::panels().is_some()
    }

    /// Hand the current line to the strip (only when it changes).
    pub fn feed_strip(&mut self) {
        if self.strip_on() {
            let text = self
                .video
                .as_ref()
                .and_then(|v| matrixkit_tube::cue_at(&self.cues, v.position_s()))
                .map(|c| c.text.clone())
                .unwrap_or_default();
            if self.strip_text.as_deref() != Some(text.as_str()) {
                mk::cinema::subtitle::write(self.sub_size, &text);
                self.strip_text = Some(text);
            }
        } else if self.strip_text.take().is_some() {
            mk::cinema::subtitle::clear();
        }
    }

    /// The subtitle under the video (stage): the subtitle family — not
    /// while the dock's strip shows it.
    pub fn build_subtitle(&self, engine: &mut TextEngine, b: &mut SceneBuilder, stage: matrixkit_theme::scene::Rect) {
        if self.strip_on() {
            return;
        }
        let Some(video) = &self.video else { return };
        if let Some(cue) = matrixkit_tube::cue_at(&self.cues, video.position_s()) {
            fam::lay_subtitle(b, engine, stage, &cue.text, Some(BAR_BASE));
        }
    }
}
