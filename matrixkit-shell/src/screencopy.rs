//! Round 99: the screen, frame by frame, without wf-recorder.
//!
//! A wlr-screencopy client of its own: one output, one buffer, `grab()`
//! returns the current picture as the compositor's 32-bit pixels
//! (XRGB/ARGB8888, little-endian = B G R X in memory). wf-recorder did the
//! same and then converted every pixel to NV12 on the CPU (swscale, 73 Mpx/s
//! on the Duo for 1350×1800 at 30 fps); the kit hands the pixels to the GPU
//! instead (`matrixkit_gpu::nv12`).
//!
//! Round 100: the buffer is a DMABUF where the machine has ION (the Duo).
//! With shm the compositor renders the output a second time, reads it back
//! from the GPU and copies it with the CPU (niri: ~75 % of a core while
//! filming); into a dmabuf it renders on the GPU and is done. The CPU
//! reads the cached ION mapping (DMA_BUF_IOCTL_SYNC around it). Should the
//! compositor refuse the buffer, the grabber falls back to shm for good.

use std::os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd};

use wayland_client::protocol::{wl_buffer, wl_output, wl_registry, wl_shm, wl_shm_pool};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, WEnum};
use wayland_protocols::wp::linux_dmabuf::zv1::client::{zwp_linux_buffer_params_v1 as params_v1, zwp_linux_dmabuf_v1 as dmabuf_v1};
use wayland_protocols_wlr::screencopy::v1::client::{zwlr_screencopy_frame_v1 as frame_v1, zwlr_screencopy_manager_v1 as manager_v1};

/// One grabbed picture (borrowed from the grabber's buffer).
pub struct Shot<'a> {
    pub width: u32,
    pub height: u32,
    /// Bytes per row.
    pub stride: u32,
    /// Rows run bottom-up (the compositor's flag).
    pub y_invert: bool,
    /// Red sits in the low byte (XBGR/ABGR) instead of the third.
    pub bgr: bool,
    pub pixels: &'a [u8],
}

#[derive(Default)]
struct State {
    shm: Option<wl_shm::WlShm>,
    dmabuf: Option<dmabuf_v1::ZwpLinuxDmabufV1>,
    manager: Option<manager_v1::ZwlrScreencopyManagerV1>,
    manager_version: u32,
    outputs: Vec<(wl_output::WlOutput, Option<String>)>,
    // the frame in flight
    format: Option<wl_shm::Format>,
    size: (u32, u32, u32),
    dmabuf_offer: Option<(u32, u32, u32)>,
    buffer_done: bool,
    y_invert: bool,
    ready: bool,
    failed: bool,
}

/// The buffer the compositor copies into.
struct Target {
    buffer: wl_buffer::WlBuffer,
    pool: Option<wl_shm_pool::WlShmPool>,
    /// The dmabuf fd (ION) — kept for the cache syncs.
    dmabuf: Option<OwnedFd>,
    map: *mut u8,
    len: usize,
    /// width, height, stride, fourcc/shm format
    key: (u32, u32, u32, u32),
}

impl Drop for Target {
    fn drop(&mut self) {
        self.buffer.destroy();
        if let Some(p) = &self.pool {
            p.destroy();
        }
        // SAFETY: our own mapping of `len` bytes.
        unsafe { libc::munmap(self.map as *mut _, self.len) };
    }
}

/// The grabber of one output.
pub struct ScreenGrab {
    conn: Connection,
    queue: EventQueue<State>,
    state: State,
    output: wl_output::WlOutput,
    target: Option<Target>,
    /// false after the compositor refused a dmabuf (or there is no ION).
    try_dmabuf: bool,
    /// The current picture is under a CPU read sync (ended before the next).
    synced: bool,
    /// false = the GPU reads the dmabuf in place: no CPU cache sync.
    cpu_read: bool,
    /// Counts new buffers (a reader that imported the dmabuf re-imports).
    serial: u64,
}

impl ScreenGrab {
    /// Connect and find the output called `name` (as niri names it, «DSI-1»).
    pub fn open(name: &str) -> Result<ScreenGrab, String> {
        ScreenGrab::open_with(name, true)
    }

    /// `dmabuf: false` = shm only (the way back when a dmabuf broke the
    /// connection: create_immed failures are protocol errors).
    pub fn open_with(name: &str, dmabuf: bool) -> Result<ScreenGrab, String> {
        let conn = Connection::connect_to_env().map_err(|e| format!("Wayland: {e}"))?;
        let mut queue = conn.new_event_queue();
        let qh = queue.handle();
        conn.display().get_registry(&qh, ());
        let mut state = State::default();
        // Two round trips: the globals, then the outputs' names.
        queue.roundtrip(&mut state).map_err(|e| e.to_string())?;
        queue.roundtrip(&mut state).map_err(|e| e.to_string())?;
        if state.manager.is_none() {
            return Err("der Compositor bietet kein screencopy".into());
        }
        if state.shm.is_none() {
            return Err("kein wl_shm".into());
        }
        let output = state
            .outputs
            .iter()
            .find(|(_, n)| n.as_deref() == Some(name))
            .map(|(o, _)| o.clone())
            .ok_or_else(|| format!("Ausgang {name} nicht gefunden"))?;
        let try_dmabuf = dmabuf && state.dmabuf.is_some() && std::path::Path::new("/dev/ion").exists() && std::env::var_os("MATRIX_SCREENCOPY_SHM").is_none();
        Ok(ScreenGrab { conn, queue, state, output, target: None, try_dmabuf, synced: false, cpu_read: true, serial: 0 })
    }

    /// Round 100: the GPU reads the dmabuf itself — skip the CPU cache sync.
    pub fn set_cpu_read(&mut self, on: bool) {
        self.cpu_read = on;
    }

    /// The dmabuf the compositor copies into (fd, size) and its buffer serial.
    pub fn dmabuf(&self) -> Option<(std::os::fd::BorrowedFd<'_>, u64, u64)> {
        let t = self.target.as_ref()?;
        Some((t.dmabuf.as_ref()?.as_fd(), t.len as u64, self.serial))
    }

    /// Does the grabber copy through a dmabuf (true) or shm?
    pub fn uses_dmabuf(&self) -> bool {
        self.target.as_ref().is_some_and(|t| t.dmabuf.is_some())
    }

    /// The current picture of the output (blocks until the compositor copied it).
    pub fn grab(&mut self) -> Result<Shot<'_>, String> {
        self.end_sync();
        loop {
            match self.grab_once() {
                Ok(true) => break,
                Ok(false) => {
                    // the compositor refused the dmabuf: shm from now on
                    eprintln!("[screencopy] dmabuf abgelehnt — weiter über shm");
                    self.try_dmabuf = false;
                    self.target = None;
                }
                Err(e) => return Err(e),
            }
        }
        let t = self.target.as_ref().ok_or("kein Puffer")?;
        if let Some(fd) = t.dmabuf.as_ref().filter(|_| self.cpu_read) {
            dma_sync(fd.as_raw_fd(), DMA_BUF_SYNC_START | DMA_BUF_SYNC_READ);
            self.synced = true;
        }
        let (w, h, stride, fmt) = t.key;
        let bgr = if t.dmabuf.is_some() { fmt == DRM_XBGR || fmt == DRM_ABGR } else { fmt == wl_shm::Format::Xbgr8888 as u32 || fmt == wl_shm::Format::Abgr8888 as u32 };
        // SAFETY: the mapping lives as long as the target, which the
        // returned borrow of `self` keeps alive.
        let pixels = unsafe { std::slice::from_raw_parts(t.map, stride as usize * h as usize) };
        Ok(Shot { width: w, height: h, stride, y_invert: self.state.y_invert, bgr, pixels })
    }

    /// The picture of the last `grab()` again (no new copy).
    pub fn grab_last(&self) -> Result<Shot<'_>, String> {
        let t = self.target.as_ref().ok_or("kein Puffer")?;
        let (w, h, stride, fmt) = t.key;
        let bgr = if t.dmabuf.is_some() { fmt == DRM_XBGR || fmt == DRM_ABGR } else { fmt == wl_shm::Format::Xbgr8888 as u32 || fmt == wl_shm::Format::Abgr8888 as u32 };
        // SAFETY: as in grab(): the mapping lives with the target.
        let pixels = unsafe { std::slice::from_raw_parts(t.map, stride as usize * h as usize) };
        Ok(Shot { width: w, height: h, stride, y_invert: self.state.y_invert, bgr, pixels })
    }

    fn end_sync(&mut self) {
        if self.synced {
            if let Some(fd) = self.target.as_ref().and_then(|t| t.dmabuf.as_ref()) {
                dma_sync(fd.as_raw_fd(), DMA_BUF_SYNC_END | DMA_BUF_SYNC_READ);
            }
            self.synced = false;
        }
    }

    /// One copy. Ok(false) = the dmabuf was refused (retry with shm).
    fn grab_once(&mut self) -> Result<bool, String> {
        let qh = self.queue.handle();
        let manager = self.state.manager.clone().ok_or("kein screencopy")?;
        let s = &mut self.state;
        s.format = None;
        s.dmabuf_offer = None;
        s.buffer_done = false;
        s.ready = false;
        s.failed = false;
        s.y_invert = false;
        // overlay_cursor 0: the pointer is not part of the film.
        let frame = manager.capture_output(0, &self.output, &qh, ());
        // v3 announces every buffer type and ends with buffer_done; older
        // versions send one shm «buffer» event.
        let v3 = self.state.manager_version >= 3;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while !(self.state.failed || (self.state.format.is_some() && (!v3 || self.state.buffer_done))) {
            self.dispatch_until(deadline)?;
        }
        if self.state.failed {
            frame.destroy();
            return Err("screencopy verweigert".into());
        }
        let use_dmabuf = self.try_dmabuf && self.state.dmabuf_offer.is_some();
        let key = if use_dmabuf {
            let (fourcc, w, h) = self.state.dmabuf_offer.unwrap_or_default();
            // Adreno render targets want aligned rows: 256 bytes is safe.
            (w, h, (w * 4).next_multiple_of(256), fourcc)
        } else {
            let (w, h, stride) = self.state.size;
            (w, h, stride, self.state.format.map_or(0, |f| f as u32))
        };
        if self.target.as_ref().is_none_or(|t| t.key != key || t.dmabuf.is_some() != use_dmabuf) {
            self.target = None;
            let made = if use_dmabuf { self.make_dmabuf(key, &qh) } else { self.make_shm(key, &qh) };
            match made {
                Ok(t) => {
                    self.target = Some(t);
                    self.serial += 1;
                }
                Err(e) if use_dmabuf => {
                    eprintln!("[screencopy] dmabuf nicht möglich ({e})");
                    frame.destroy();
                    return Ok(false);
                }
                Err(e) => {
                    frame.destroy();
                    return Err(e);
                }
            }
        }
        let t = self.target.as_ref().ok_or("kein Puffer")?;
        frame.copy(&t.buffer);
        while !(self.state.ready || self.state.failed) {
            self.dispatch_until(deadline)?;
        }
        frame.destroy();
        let _ = self.conn.flush();
        if self.state.failed {
            if use_dmabuf {
                return Ok(false);
            }
            return Err("screencopy fehlgeschlagen".into());
        }
        Ok(true)
    }

    /// Wait for events until `deadline` (round 103): the compositor might
    /// never answer (an output that stopped drawing) — then the film ends
    /// and is saved instead of hanging forever.
    fn dispatch_until(&mut self, deadline: std::time::Instant) -> Result<(), String> {
        let _ = self.conn.flush();
        if let Some(guard) = self.queue.prepare_read() {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            let mut fds = [libc::pollfd { fd: guard.connection_fd().as_raw_fd(), events: libc::POLLIN, revents: 0 }];
            // SAFETY: one valid pollfd for the duration of the call.
            let ready = unsafe { libc::poll(fds.as_mut_ptr(), 1, left.as_millis().min(1000) as i32) };
            if ready > 0 {
                guard.read().map_err(|e| e.to_string())?;
            } else {
                drop(guard);
                if std::time::Instant::now() >= deadline {
                    return Err("der Compositor antwortet nicht (2 s)".into());
                }
            }
        }
        self.queue.dispatch_pending(&mut self.state).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn make_shm(&self, (w, h, stride, fmt): (u32, u32, u32, u32), qh: &QueueHandle<State>) -> Result<Target, String> {
        let len = stride as usize * h as usize;
        // SAFETY: plain memfd syscall; the fd is owned right after.
        let fd = unsafe { libc::memfd_create(c"matrix-screencopy".as_ptr(), libc::MFD_CLOEXEC) };
        if fd < 0 {
            return Err("memfd fehlgeschlagen".into());
        }
        // SAFETY: fd is a fresh, valid descriptor we own.
        let file = std::fs::File::from(unsafe { OwnedFd::from_raw_fd(fd) });
        file.set_len(len as u64).map_err(|e| e.to_string())?;
        let map = map_fd(file.as_raw_fd(), len)?;
        let shm = self.state.shm.as_ref().ok_or("kein wl_shm")?;
        let format = wl_shm::Format::try_from(fmt).map_err(|_| "unbekanntes shm-Format")?;
        let pool = shm.create_pool(file.as_fd(), len as i32, qh, ());
        let buffer = pool.create_buffer(0, w as i32, h as i32, stride as i32, format, qh, ());
        Ok(Target { buffer, pool: Some(pool), dmabuf: None, map, len, key: (w, h, stride, fmt) })
    }

    fn make_dmabuf(&self, (w, h, stride, fourcc): (u32, u32, u32, u32), qh: &QueueHandle<State>) -> Result<Target, String> {
        let len = (stride as usize * h as usize).next_multiple_of(4096);
        let fd = ion_alloc(len)?;
        let map = map_fd(fd.as_raw_fd(), len)?;
        let dmabuf = self.state.dmabuf.as_ref().ok_or("kein linux-dmabuf")?;
        let params = dmabuf.create_params(qh, ());
        // modifier 0 = DRM_FORMAT_MOD_LINEAR
        params.add(fd.as_fd(), 0, 0, stride, 0, 0);
        let buffer = params.create_immed(w as i32, h as i32, fourcc, params_v1::Flags::empty(), qh, ());
        params.destroy();
        Ok(Target { buffer, pool: None, dmabuf: Some(fd), map, len, key: (w, h, stride, fourcc) })
    }
}

impl Drop for ScreenGrab {
    fn drop(&mut self) {
        self.end_sync();
        self.target = None;
        let _ = self.conn.flush();
    }
}

const DRM_XBGR: u32 = u32::from_le_bytes(*b"XB24");
const DRM_ABGR: u32 = u32::from_le_bytes(*b"AB24");

fn map_fd(fd: i32, len: usize) -> Result<*mut u8, String> {
    // SAFETY: mapping a descriptor we own, `len` bytes.
    let map = unsafe { libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0) };
    if map == libc::MAP_FAILED {
        return Err(format!("mmap: {}", std::io::Error::last_os_error()));
    }
    Ok(map as *mut u8)
}

// ION (the Android allocator of the Duo's 4.14 kernel): a dmabuf from the
// system heap, CACHED — the CPU reads it (same call as matrixkit_video::venus).
const ION_SYSTEM_HEAP_ID: u32 = 25;
const ION_FLAG_CACHED: u32 = 1;

#[repr(C)]
struct IonAlloc {
    len: u64,
    heap_id_mask: u32,
    flags: u32,
    fd: u32,
    unused: u32,
}

const fn ioc(dir: u64, ty: u8, nr: u64, size: u64) -> u64 {
    (dir << 30) | (size << 16) | ((ty as u64) << 8) | nr
}
const ION_IOC_ALLOC: u64 = ioc(3, b'I', 0, 24);
const DMA_BUF_IOCTL_SYNC: u64 = ioc(1, b'b', 0, 8);
const DMA_BUF_SYNC_READ: u64 = 1;
const DMA_BUF_SYNC_START: u64 = 0;
const DMA_BUF_SYNC_END: u64 = 4;

fn ion_alloc(len: usize) -> Result<OwnedFd, String> {
    let ion = std::fs::File::open("/dev/ion").map_err(|e| format!("/dev/ion: {e}"))?;
    let mut a = IonAlloc { len: len as u64, heap_id_mask: 1 << ION_SYSTEM_HEAP_ID, flags: ION_FLAG_CACHED, fd: 0, unused: 0 };
    // SAFETY: the ioctl fills `a` (24 bytes, the kernel's ion_allocation_data).
    let r = unsafe { libc::ioctl(ion.as_raw_fd(), ION_IOC_ALLOC as _, &mut a as *mut IonAlloc) };
    if r < 0 {
        return Err(format!("ION: {}", std::io::Error::last_os_error()));
    }
    // SAFETY: the kernel handed us this fd.
    Ok(unsafe { OwnedFd::from_raw_fd(a.fd as i32) })
}

fn dma_sync(fd: i32, flags: u64) {
    let mut f = flags;
    // SAFETY: DMA_BUF_IOCTL_SYNC takes one u64.
    unsafe { libc::ioctl(fd, DMA_BUF_IOCTL_SYNC as _, &mut f as *mut u64) };
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(s: &mut Self, reg: &wl_registry::WlRegistry, event: wl_registry::Event, _: &(), _: &Connection, qh: &QueueHandle<Self>) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            match interface.as_str() {
                "wl_shm" => s.shm = Some(reg.bind(name, 1, qh, ())),
                "zwp_linux_dmabuf_v1" if version >= 2 => s.dmabuf = Some(reg.bind(name, version.min(3), qh, ())),
                "zwlr_screencopy_manager_v1" => {
                    s.manager_version = version.min(3);
                    s.manager = Some(reg.bind(name, s.manager_version, qh, ()));
                }
                // v4 brings the output's name.
                "wl_output" if version >= 4 => s.outputs.push((reg.bind(name, 4, qh, ()), None)),
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for State {
    fn event(s: &mut Self, out: &wl_output::WlOutput, event: wl_output::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        if let wl_output::Event::Name { name } = event {
            if let Some(slot) = s.outputs.iter_mut().find(|(o, _)| o == out) {
                slot.1 = Some(name);
            }
        }
    }
}

impl Dispatch<frame_v1::ZwlrScreencopyFrameV1, ()> for State {
    fn event(s: &mut Self, _: &frame_v1::ZwlrScreencopyFrameV1, event: frame_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        match event {
            frame_v1::Event::Buffer { format: WEnum::Value(format), width, height, stride } => {
                s.format = Some(format);
                s.size = (width, height, stride);
            }
            frame_v1::Event::LinuxDmabuf { format, width, height } => s.dmabuf_offer = Some((format, width, height)),
            frame_v1::Event::Flags { flags: WEnum::Value(f) } => s.y_invert = f.contains(frame_v1::Flags::YInvert),
            frame_v1::Event::BufferDone => s.buffer_done = true,
            frame_v1::Event::Ready { .. } => s.ready = true,
            frame_v1::Event::Failed => s.failed = true,
            _ => {}
        }
    }
}

wayland_client::delegate_noop!(State: ignore wl_shm::WlShm);
wayland_client::delegate_noop!(State: ignore wl_shm_pool::WlShmPool);
wayland_client::delegate_noop!(State: ignore wl_buffer::WlBuffer);
wayland_client::delegate_noop!(State: ignore manager_v1::ZwlrScreencopyManagerV1);
wayland_client::delegate_noop!(State: ignore dmabuf_v1::ZwpLinuxDmabufV1);
wayland_client::delegate_noop!(State: ignore params_v1::ZwpLinuxBufferParamsV1);
