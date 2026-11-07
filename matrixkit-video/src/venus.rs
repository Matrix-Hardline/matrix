//! THE VENUS FAMILY (round 65, 03.11.2026): the Snapdragon's hardware
//! video ENCODER through the downstream Android driver (msm_vidc, kernel
//! 4.14 — the Surface Duo). ffmpeg's and GStreamer's v4l2m2m cannot use it
//! (dig GRABUNG-VENUS-ENCODER.md): the driver has no TRY_FMT, no MMAP and
//! no DMABUF; buffers are ION dma-bufs whose fd travels in
//! `plane.reserved[0]` with USERPTR memory, and the CODED format must be
//! set before the raw one. This module speaks exactly that.
//!
//! Measured on the Duo: 1350×1800 H.264 at ~150 frames/s, ~19 % CPU for
//! the client; the firmware loads on the first open.

use std::io::{self, Write};
use std::os::fd::RawFd;

const BUF_TYPE_CAPTURE_MPLANE: u32 = 9;
const BUF_TYPE_OUTPUT_MPLANE: u32 = 10;
const MEMORY_USERPTR: u32 = 2;
const CID_MPEG_BASE: u32 = 0x0099_0000 | 0x900; // V4L2_CTRL_CLASS_MPEG | 0x900
const CID_BITRATE_MODE: u32 = CID_MPEG_BASE + 200;
const CID_BITRATE: u32 = CID_MPEG_BASE + 207;
const CID_MSM_BASE: u32 = 0x0099_0000 | 0x2000;
const CID_IDR_PERIOD: u32 = CID_MSM_BASE + 5;
const CID_NUM_P_FRAMES: u32 = CID_MSM_BASE + 6;
const CID_STREAM_OUTPUT_MODE: u32 = CID_MSM_BASE + 22;
const BITRATE_MODE_VBR: i32 = 0;
const ION_SYSTEM_HEAP_ID: u32 = 25;

const fn ioc(dir: u64, ty: u8, nr: u64, size: usize) -> u64 {
    (dir << 30) | ((size as u64) << 16) | ((ty as u64) << 8) | nr
}
const RW: u64 = 3;
const R: u64 = 2;
const W: u64 = 1;
const VIDIOC_QUERYCAP: u64 = ioc(R, b'V', 0, 104);
const VIDIOC_G_FMT: u64 = ioc(RW, b'V', 4, 208);
const VIDIOC_S_FMT: u64 = ioc(RW, b'V', 5, 208);
const VIDIOC_REQBUFS: u64 = ioc(RW, b'V', 8, 20);
const VIDIOC_QBUF: u64 = ioc(RW, b'V', 15, std::mem::size_of::<V4l2Buffer>());
const VIDIOC_DQBUF: u64 = ioc(RW, b'V', 17, std::mem::size_of::<V4l2Buffer>());
const VIDIOC_STREAMON: u64 = ioc(W, b'V', 18, 4);
const VIDIOC_STREAMOFF: u64 = ioc(W, b'V', 19, 4);
const VIDIOC_S_PARM: u64 = ioc(RW, b'V', 22, 204);
const VIDIOC_S_CTRL: u64 = ioc(RW, b'V', 28, 8);
const VIDIOC_G_CTRL: u64 = ioc(RW, b'V', 27, 8);
const ION_IOC_ALLOC: u64 = ioc(RW, b'I', 0, 24);
const VIDIOC_DQEVENT: u64 = ioc(R, b'V', 89, 136);
const VIDIOC_SUBSCRIBE_EVENT: u64 = ioc(W, b'V', 90, 32);
const VIDIOC_DECODER_CMD: u64 = ioc(RW, b'V', 96, 72);
const QCOM_CMD_SESSION_CONTINUE: u32 = 5;
const EVENT_MSM_VIDC_START: u32 = 0x0800_0000 + 0x1000;
const EVENT_PORT_SETTINGS_SUFFICIENT: u32 = EVENT_MSM_VIDC_START + 2;
const EVENT_PORT_SETTINGS_INSUFFICIENT: u32 = EVENT_MSM_VIDC_START + 3;
const EVENT_SYS_ERROR: u32 = EVENT_MSM_VIDC_START + 5;
const QCOM_BUF_FLAG_EOS: u32 = 0x0200_0000;
/// The decoder's poll says "ready" even with nothing to dequeue; a loop
/// that found nothing naps this long (2 ms still cost 35 % CPU in driver
/// calls; the picture queue holds ~0.4 s, so 10 ms are invisible).
const IDLE_NAP: std::time::Duration = std::time::Duration::from_millis(10);
const QCOM_BUF_FLAG_CODECCONFIG: u32 = 0x0002_0000;

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct V4l2Plane {
    bytesused: u32,
    length: u32,
    m: u64,
    data_offset: u32,
    reserved: [u32; 11],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct V4l2Buffer {
    index: u32,
    kind: u32,
    bytesused: u32,
    flags: u32,
    field: u32,
    timestamp: [i64; 2],
    timecode: [u32; 4],
    sequence: u32,
    memory: u32,
    planes: *mut V4l2Plane,
    length: u32,
    reserved2: u32,
    request_fd: i32,
}

#[repr(C)]
struct IonAlloc {
    len: u64,
    heap_id_mask: u32,
    flags: u32,
    fd: u32,
    unused: u32,
}

fn ioctl<T>(fd: RawFd, req: u64, arg: &mut T) -> io::Result<()> {
    loop {
        // SAFETY: `arg` is a properly sized, initialized struct for `req`.
        let r = unsafe { libc::ioctl(fd, req as _, arg as *mut T) };
        if r >= 0 {
            return Ok(());
        }
        let e = io::Error::last_os_error();
        if e.kind() != io::ErrorKind::Interrupted {
            return Err(e);
        }
    }
}

/// One ION buffer, mapped for the CPU.
struct IonBuf {
    fd: RawFd,
    map: *mut u8,
    len: usize,
}

impl IonBuf {
    fn alloc(ion: RawFd, len: usize) -> io::Result<IonBuf> {
        IonBuf::alloc_as(ion, len, true)
    }

    /// `cached` = the CPU reads the buffer (pictures out). Round 67: a
    /// buffer the CPU only WRITES (packets in) goes uncached — the driver
    /// cleans the cache of every cached buffer on each queue, and for the
    /// decoder's 16 MB input buffers that was 35 % kernel time.
    fn alloc_as(ion: RawFd, len: usize, cached: bool) -> io::Result<IonBuf> {
        let mut a = IonAlloc { len: len as u64, heap_id_mask: 1 << ION_SYSTEM_HEAP_ID, flags: u32::from(cached), fd: 0, unused: 0 };
        ioctl(ion, ION_IOC_ALLOC, &mut a)?;
        let fd = a.fd as RawFd;
        // SAFETY: mapping a dma-buf fd we own, length as allocated.
        let map = unsafe { libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0) };
        if map == libc::MAP_FAILED {
            let e = io::Error::last_os_error();
            unsafe { libc::close(fd) };
            return Err(e);
        }
        Ok(IonBuf { fd, map: map as *mut u8, len })
    }
}

// SAFETY: the mapping is owned by this buffer alone; moving it to another
// thread (the decoding thread) moves the ownership with it.
unsafe impl Send for IonBuf {}

impl Drop for IonBuf {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.map as *mut _, self.len);
            libc::close(self.fd);
        }
    }
}

fn raw_fmt(kind: u32, w: u32, h: u32, fourcc: &[u8; 4]) -> [u8; 208] {
    let mut b = [0u8; 208];
    b[0..4].copy_from_slice(&kind.to_le_bytes());
    b[8..12].copy_from_slice(&w.to_le_bytes());
    b[12..16].copy_from_slice(&h.to_le_bytes());
    b[16..20].copy_from_slice(fourcc);
    b[8 + 20 + 160] = 1; // num_planes
    b
}

fn u32_at(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

/// The encoder device, if this machine has one the family speaks to:
/// a node whose driver card is `msm_vidc_venc`.
pub fn encoder_device() -> Option<std::path::PathBuf> {
    device_with_card("msm_vidc_venc")
}

/// The decoder device (round 66): card `msm_vidc_vdec`.
pub fn decoder_device() -> Option<std::path::PathBuf> {
    device_with_card("msm_vidc_vdec")
}

fn device_with_card(wanted: &str) -> Option<std::path::PathBuf> {
    let dir = std::fs::read_dir("/dev").ok()?;
    let mut nodes: Vec<_> = dir
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("video")))
        .collect();
    nodes.sort();
    nodes.into_iter().find(|p| card_of(p).as_deref() == Some(wanted))
}

fn card_of(path: &std::path::Path) -> Option<String> {
    use std::os::fd::AsRawFd;
    let f = std::fs::OpenOptions::new().read(true).write(true).custom_flags_nonblock().open(path).ok()?;
    let mut cap = [0u8; 104];
    ioctl(f.as_raw_fd(), VIDIOC_QUERYCAP, &mut cap).ok()?;
    let card = &cap[16..48];
    let end = card.iter().position(|&c| c == 0).unwrap_or(card.len());
    Some(String::from_utf8_lossy(&card[..end]).into_owned())
}

trait NonBlock {
    fn custom_flags_nonblock(&mut self) -> &mut Self;
}
impl NonBlock for std::fs::OpenOptions {
    fn custom_flags_nonblock(&mut self) -> &mut Self {
        use std::os::unix::fs::OpenOptionsExt;
        self.custom_flags(libc::O_NONBLOCK)
    }
}

/// Settings of one encoding session.
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub kbps: u32,
    /// Keyframe distance in frames (every keyframe an IDR: seekable,
    /// and Fedora's openh264 decodes nothing without them — measured).
    pub keyframe_every: u32,
}

impl Settings {
    pub fn new(width: u32, height: u32) -> Settings {
        Settings { width, height, fps: 30, kbps: 8000, keyframe_every: 60 }
    }
}

/// An H.264 encoding session on the Venus block: push tightly packed NV12
/// frames, Annex-B H.264 comes out into the sink.
pub struct Encoder {
    dev: std::fs::File,
    _ion: std::fs::File,
    out: Vec<IonBuf>,
    cap: Vec<IonBuf>,
    free: Vec<u32>,
    stride: usize,
    scanlines: usize,
    out_size: usize,
    settings: Settings,
    pub frames_in: u64,
    pub bytes_out: u64,
}

impl Encoder {
    pub fn open(device: &std::path::Path, s: Settings) -> io::Result<Encoder> {
        use std::os::fd::AsRawFd;
        let dev = std::fs::OpenOptions::new().read(true).write(true).custom_flags_nonblock().open(device)?;
        let ion = std::fs::File::open("/dev/ion")?;
        let fd = dev.as_raw_fd();
        // CODED first: it opens the firmware session (driver order).
        let mut cf = raw_fmt(BUF_TYPE_CAPTURE_MPLANE, s.width, s.height, b"H264");
        ioctl(fd, VIDIOC_S_FMT, &mut cf)?;
        let mut of = raw_fmt(BUF_TYPE_OUTPUT_MPLANE, s.width, s.height, b"NV12");
        ioctl(fd, VIDIOC_S_FMT, &mut of)?;
        ioctl(fd, VIDIOC_G_FMT, &mut of)?;
        ioctl(fd, VIDIOC_G_FMT, &mut cf)?;
        let stride = u32_at(&of, 8 + 20 + 4) as usize; // plane_fmt[0].bytesperline
        let scanlines = u32_at(&of, 8 + 20 + 8) as usize; // plane_fmt[0].reserved[0]
        let out_size = u32_at(&of, 8 + 20) as usize;
        let cap_size = u32_at(&cf, 8 + 20) as usize;
        if stride < s.width as usize || scanlines < s.height as usize || out_size == 0 || cap_size == 0 {
            return Err(io::Error::other(format!("venus: odd layout stride={stride} scanlines={scanlines}")));
        }
        let mut parm = [0u8; 204];
        parm[0..4].copy_from_slice(&BUF_TYPE_OUTPUT_MPLANE.to_le_bytes());
        parm[12..16].copy_from_slice(&1u32.to_le_bytes()); // timeperframe.numerator
        parm[16..20].copy_from_slice(&s.fps.max(1).to_le_bytes());
        let _ = ioctl(fd, VIDIOC_S_PARM, &mut parm);
        // Rate control is OFF by default downstream (≈58 Mbit/s measured).
        for (id, value) in [
            (CID_BITRATE_MODE, BITRATE_MODE_VBR),
            (CID_BITRATE, (s.kbps.saturating_mul(1000)) as i32),
            (CID_NUM_P_FRAMES, s.keyframe_every.max(1) as i32 - 1),
            (CID_IDR_PERIOD, 1),
        ] {
            let mut c = [id as i32, value];
            if let Err(e) = ioctl(fd, VIDIOC_S_CTRL, &mut c) {
                // The driver refuses to set a value it already holds
                // (VBR is often the session default) — only a real
                // difference is worth a word.
                let mut now = [id as i32, 0];
                if ioctl(fd, VIDIOC_G_CTRL, &mut now).map_or(true, |_| now[1] != value) {
                    eprintln!("[venus] control {id:#x}: {e}");
                }
            }
        }
        let mut counts = [0u32; 2];
        for (i, (kind, want)) in [(BUF_TYPE_OUTPUT_MPLANE, 4u32), (BUF_TYPE_CAPTURE_MPLANE, 6)].into_iter().enumerate() {
            let mut rq = [want, kind, MEMORY_USERPTR, 0, 0];
            ioctl(fd, VIDIOC_REQBUFS, &mut rq)?;
            counts[i] = rq[0];
        }
        // Round 68: the frames in are only WRITTEN by the CPU — uncached,
        // no cache upkeep on every queue (the decoder's lesson).
        let out = (0..counts[0]).map(|_| IonBuf::alloc_as(ion.as_raw_fd(), out_size, false)).collect::<io::Result<Vec<_>>>()?;
        let cap = (0..counts[1]).map(|_| IonBuf::alloc(ion.as_raw_fd(), cap_size)).collect::<io::Result<Vec<_>>>()?;
        let enc = Encoder {
            dev,
            _ion: ion,
            free: (0..out.len() as u32).rev().collect(),
            out,
            cap,
            stride,
            scanlines,
            out_size,
            settings: s,
            frames_in: 0,
            bytes_out: 0,
        };
        for i in 0..enc.cap.len() as u32 {
            enc.queue(BUF_TYPE_CAPTURE_MPLANE, i, 0)?;
        }
        for kind in [BUF_TYPE_CAPTURE_MPLANE, BUF_TYPE_OUTPUT_MPLANE] {
            let mut k = kind;
            ioctl(fd, VIDIOC_STREAMON, &mut k)?;
        }
        Ok(enc)
    }

    /// Bytes of one tightly packed NV12 frame.
    pub fn frame_len(&self) -> usize {
        self.settings.width as usize * self.settings.height as usize * 3 / 2
    }

    fn queue(&self, kind: u32, index: u32, used: usize) -> io::Result<()> {
        use std::os::fd::AsRawFd;
        let b = if kind == BUF_TYPE_OUTPUT_MPLANE { &self.out[index as usize] } else { &self.cap[index as usize] };
        let mut plane = V4l2Plane { bytesused: used as u32, length: b.len as u32, m: b.fd as u64, ..Default::default() };
        plane.reserved[0] = b.fd as u32;
        let mut v = V4l2Buffer {
            index,
            kind,
            bytesused: 0,
            flags: 0,
            field: 0,
            timestamp: [0; 2],
            timecode: [0; 4],
            sequence: 0,
            memory: MEMORY_USERPTR,
            planes: &mut plane,
            length: 1,
            reserved2: 0,
            request_fd: 0,
        };
        ioctl(self.dev.as_raw_fd(), VIDIOC_QBUF, &mut v)
    }

    fn dequeue(&self, kind: u32) -> Option<(u32, usize, u32)> {
        use std::os::fd::AsRawFd;
        let mut plane = V4l2Plane::default();
        let mut v = V4l2Buffer {
            index: 0,
            kind,
            bytesused: 0,
            flags: 0,
            field: 0,
            timestamp: [0; 2],
            timecode: [0; 4],
            sequence: 0,
            memory: MEMORY_USERPTR,
            planes: &mut plane,
            length: 1,
            reserved2: 0,
            request_fd: 0,
        };
        ioctl(self.dev.as_raw_fd(), VIDIOC_DQBUF, &mut v).ok()?;
        Some((v.index, plane.bytesused as usize, v.flags))
    }

    /// Take back finished input buffers and write finished packets;
    /// true = anything came (the waiting loops nap on false).
    fn reap(&mut self, sink: &mut impl Write) -> io::Result<bool> {
        let mut progress = false;
        while let Some((i, _, _)) = self.dequeue(BUF_TYPE_OUTPUT_MPLANE) {
            self.free.push(i);
            progress = true;
        }
        while let Some((i, used, _flags)) = self.dequeue(BUF_TYPE_CAPTURE_MPLANE) {
            progress = true;
            if used > 0 {
                let b = &self.cap[i as usize];
                // SAFETY: the driver filled `used` bytes of this mapping.
                sink.write_all(unsafe { std::slice::from_raw_parts(b.map, used.min(b.len)) })?;
                self.bytes_out += used as u64;
            }
            self.queue(BUF_TYPE_CAPTURE_MPLANE, i, 0)?;
        }
        Ok(progress)
    }

    fn wait(&self, ms: i32) -> io::Result<bool> {
        use std::os::fd::AsRawFd;
        let mut p = libc::pollfd { fd: self.dev.as_raw_fd(), events: libc::POLLIN | libc::POLLOUT | libc::POLLPRI, revents: 0 };
        let r = unsafe { libc::poll(&mut p, 1, ms) };
        if r < 0 {
            let e = io::Error::last_os_error();
            return if e.kind() == io::ErrorKind::Interrupted { Ok(true) } else { Err(e) };
        }
        if p.revents & libc::POLLERR != 0 {
            return Err(io::Error::other("venus: device error (POLLERR)"));
        }
        Ok(r > 0)
    }

    /// Encode one tightly packed NV12 frame (`frame_len` bytes).
    pub fn push(&mut self, nv12: &[u8], sink: &mut impl Write) -> io::Result<()> {
        self.push_rows(nv12, self.settings.width as usize, sink)
    }

    /// Round 99: encode one NV12 frame whose rows are `row` bytes apart
    /// (≥ width; the GPU converter pads rows to four bytes).
    pub fn push_rows(&mut self, nv12: &[u8], row: usize, sink: &mut impl Write) -> io::Result<()> {
        let (w, h) = (self.settings.width as usize, self.settings.height as usize);
        if row < w || nv12.len() < row * h * 3 / 2 {
            return Err(io::Error::other("venus: short frame"));
        }
        self.reap(sink)?;
        let mut tries = 0;
        while self.free.is_empty() {
            if !self.wait(2000)? {
                tries += 1;
                if tries > 2 {
                    return Err(io::Error::other("venus: encoder stalled"));
                }
            }
            if !self.reap(sink)? {
                std::thread::sleep(IDLE_NAP);
            }
        }
        let i = self.free.pop().unwrap_or(0);
        let b = &self.out[i as usize];
        // SAFETY: the mapping is `out_size` long ≥ stride*scanlines*3/2.
        let dst = unsafe { std::slice::from_raw_parts_mut(b.map, b.len) };
        for y in 0..h {
            dst[y * self.stride..y * self.stride + w].copy_from_slice(&nv12[y * row..y * row + w]);
        }
        let uv = self.stride * self.scanlines;
        for y in 0..h / 2 {
            let from = row * h + y * row;
            dst[uv + y * self.stride..uv + y * self.stride + w].copy_from_slice(&nv12[from..from + w]);
        }
        self.queue(BUF_TYPE_OUTPUT_MPLANE, i, self.out_size)?;
        self.frames_in += 1;
        Ok(())
    }

    /// Drain: wait until every pushed frame came back, write the rest.
    pub fn finish(&mut self, sink: &mut impl Write) -> io::Result<()> {
        let total = self.out.len();
        let mut idle = 0;
        while self.free.len() < total && idle < 3 {
            if !self.wait(500)? {
                idle += 1;
            }
            if !self.reap(sink)? {
                std::thread::sleep(IDLE_NAP);
            }
        }
        // A last look for packets that trail their input buffer.
        let _ = self.wait(100);
        self.reap(sink)?;
        sink.flush()
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        use std::os::fd::AsRawFd;
        for kind in [BUF_TYPE_OUTPUT_MPLANE, BUF_TYPE_CAPTURE_MPLANE] {
            let mut k = kind;
            let _ = ioctl(self.dev.as_raw_fd(), VIDIOC_STREAMOFF, &mut k);
        }
    }
}


// ------------------------------------------------------------ decoder

/// Coded formats the Venus decoder takes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Codec {
    H264,
    Hevc,
    Vp9,
}

impl Codec {
    fn fourcc(self) -> &'static [u8; 4] {
        match self {
            Codec::H264 => b"H264",
            Codec::Hevc => b"HEVC",
            Codec::Vp9 => b"VP90",
        }
    }
}

/// One decoded picture, borrowed from a hardware buffer for the length
/// of the callback: NV12 in the Venus layout (UV at `stride*scanlines`).
pub struct Frame<'a> {
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub scanlines: usize,
    pub data: &'a [u8],
    pub timestamp_us: i64,
}

impl Frame<'_> {
    pub fn luma(&self) -> &[u8] {
        &self.data[..self.stride * self.scanlines]
    }
    pub fn chroma(&self) -> &[u8] {
        let from = self.stride * self.scanlines;
        &self.data[from..(from + self.stride * self.height.div_ceil(2) as usize).min(self.data.len())]
    }
}

fn qbuf(fd: RawFd, kind: u32, index: u32, b: &IonBuf, used: usize, flags: u32, ts_us: i64) -> io::Result<()> {
    let mut plane = V4l2Plane { bytesused: used as u32, length: b.len as u32, m: b.fd as u64, ..Default::default() };
    plane.reserved[0] = b.fd as u32;
    let mut v = V4l2Buffer {
        index,
        kind,
        bytesused: 0,
        flags,
        field: 0,
        timestamp: [ts_us.div_euclid(1_000_000), ts_us.rem_euclid(1_000_000)],
        timecode: [0; 4],
        sequence: 0,
        memory: MEMORY_USERPTR,
        planes: &mut plane,
        length: 1,
        reserved2: 0,
        request_fd: 0,
    };
    ioctl(fd, VIDIOC_QBUF, &mut v)
}

/// (index, bytes used, flags, timestamp µs)
fn dqbuf(fd: RawFd, kind: u32) -> Option<(u32, usize, u32, i64)> {
    let mut plane = V4l2Plane::default();
    let mut v = V4l2Buffer {
        index: 0,
        kind,
        bytesused: 0,
        flags: 0,
        field: 0,
        timestamp: [0; 2],
        timecode: [0; 4],
        sequence: 0,
        memory: MEMORY_USERPTR,
        planes: &mut plane,
        length: 1,
        reserved2: 0,
        request_fd: 0,
    };
    ioctl(fd, VIDIOC_DQBUF, &mut v).ok()?;
    Some((v.index, plane.bytesused as usize, v.flags, v.timestamp[0] * 1_000_000 + v.timestamp[1]))
}

/// A decoding session on the Venus block (round 66). Feed whole access
/// units (one picture per packet — the driver's frame mode; for H.264
/// see `access_units`), pictures come back through the callback.
pub struct Decoder {
    dev: std::fs::File,
    ion: std::fs::File,
    input: Vec<IonBuf>,
    output: Vec<IonBuf>,
    free: Vec<u32>,
    width: u32,
    height: u32,
    stride: usize,
    scanlines: usize,
    ended: bool,
    /// The picture side streams (it must, for the firmware to start).
    capture_on: bool,
    trace: bool,
    pub packets_in: u64,
    pub frames_out: u64,
}

impl Decoder {
    /// Pictures come out at the coded size. (Round 67, measured: a smaller
    /// picture format does NOT scale on this firmware — it CROPS the top
    /// left corner. Scaling stays with the GPU.)
    pub fn open(device: &std::path::Path, codec: Codec, width: u32, height: u32) -> io::Result<Decoder> {
        use std::os::fd::AsRawFd;
        let dev = std::fs::OpenOptions::new().read(true).write(true).custom_flags_nonblock().open(device)?;
        let ion = std::fs::File::open("/dev/ion")?;
        let fd = dev.as_raw_fd();
        // CODED first (it opens the firmware session), then the picture.
        let mut of = raw_fmt(BUF_TYPE_OUTPUT_MPLANE, width, height, codec.fourcc());
        ioctl(fd, VIDIOC_S_FMT, &mut of)?;
        // Linear NV12 out of the SM8150 needs the SECONDARY stream: the
        // primary one (the reference pictures) is UBWC-only — the firmware
        // calls anything else an "unsupported bitstream" (measured).
        let mut mode = [CID_STREAM_OUTPUT_MODE as i32, 1];
        if let Err(e) = ioctl(fd, VIDIOC_S_CTRL, &mut mode) {
            eprintln!("[venus] secondary output: {e}");
        }
        let mut cf = raw_fmt(BUF_TYPE_CAPTURE_MPLANE, width, height, b"NV12");
        ioctl(fd, VIDIOC_S_FMT, &mut cf)?;
        for kind in [EVENT_PORT_SETTINGS_SUFFICIENT, EVENT_PORT_SETTINGS_INSUFFICIENT, EVENT_SYS_ERROR] {
            let mut sub = [0u32; 8];
            sub[0] = kind;
            let _ = ioctl(fd, VIDIOC_SUBSCRIBE_EVENT, &mut sub);
        }
        ioctl(fd, VIDIOC_G_FMT, &mut of)?;
        // The driver insists on its full input size (16 MB for 1080p;
        // smaller buffers: EINVAL on QBUF, measured round 67).
        let in_size = (u32_at(&of, 8 + 20) as usize).max(1 << 20);
        let mut rq = [6u32, BUF_TYPE_OUTPUT_MPLANE, MEMORY_USERPTR, 0, 0];
        ioctl(fd, VIDIOC_REQBUFS, &mut rq)?;
        let input = (0..rq[0]).map(|_| IonBuf::alloc_as(ion.as_raw_fd(), in_size, false)).collect::<io::Result<Vec<_>>>()?;
        let d = Decoder {
            dev,
            ion,
            free: (0..input.len() as u32).rev().collect(),
            input,
            output: Vec::new(),
            width,
            height,
            stride: 0,
            scanlines: 0,
            ended: false,
            capture_on: false,
            trace: std::env::var_os("MATRIX_VENUS_TRACE").is_some(),
            packets_in: 0,
            frames_out: 0,
        };
        // The firmware session starts only when BOTH sides stream
        // (msm_vidc_start_streaming) — the picture side goes up at once.
        let mut d = d;
        d.setup_output()?;
        for kind in [BUF_TYPE_OUTPUT_MPLANE, BUF_TYPE_CAPTURE_MPLANE] {
            let mut k = kind;
            ioctl(fd, VIDIOC_STREAMON, &mut k)?;
        }
        d.capture_on = true;
        if d.trace {
            eprintln!("[venus] pictures {}x{} stride {} scanlines {} ×{}", d.width, d.height, d.stride, d.scanlines, d.output.len());
        }
        Ok(d)
    }

    fn fd(&self) -> RawFd {
        use std::os::fd::AsRawFd;
        self.dev.as_raw_fd()
    }

    /// (Re)build the picture side from what the driver says now.
    fn setup_output(&mut self) -> io::Result<()> {
        use std::os::fd::AsRawFd;
        let fd = self.fd();
        let mut cf = raw_fmt(BUF_TYPE_CAPTURE_MPLANE, 0, 0, b"NV12");
        ioctl(fd, VIDIOC_G_FMT, &mut cf)?;
        self.width = u32_at(&cf, 8);
        self.height = u32_at(&cf, 12);
        self.stride = u32_at(&cf, 8 + 20 + 4) as usize;
        self.scanlines = u32_at(&cf, 8 + 20 + 8) as usize;
        let size = u32_at(&cf, 8 + 20) as usize;
        if size == 0 || self.stride == 0 {
            return Err(io::Error::other("venus: decoder reports no picture layout"));
        }
        self.output.clear();
        let mut rq = [0u32, BUF_TYPE_CAPTURE_MPLANE, MEMORY_USERPTR, 0, 0];
        let _ = ioctl(fd, VIDIOC_REQBUFS, &mut rq);
        let mut rq = [8u32, BUF_TYPE_CAPTURE_MPLANE, MEMORY_USERPTR, 0, 0];
        ioctl(fd, VIDIOC_REQBUFS, &mut rq)?;
        // Pictures stay CACHED: measured per picture, cached = 2.6 ms
        // kernel (cache upkeep) + 0.4 ms copy, uncached = 0.6 ms kernel +
        // 2.9 ms copy — and the copy runs in our thread.
        self.output = (0..rq[0]).map(|_| IonBuf::alloc_as(self.ion.as_raw_fd(), size, true)).collect::<io::Result<Vec<_>>>()?;
        for (i, b) in self.output.iter().enumerate() {
            qbuf(fd, BUF_TYPE_CAPTURE_MPLANE, i as u32, b, 0, 0, 0)?;
        }
        Ok(())
    }

    /// The picture size the decoder settled on.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Has the end of stream come out (after `finish`)?
    pub fn ended(&self) -> bool {
        self.ended
    }

    fn events(&mut self) -> io::Result<()> {
        loop {
            let mut ev = [0u8; 136];
            if ioctl(self.fd(), VIDIOC_DQEVENT, &mut ev).is_err() {
                return Ok(());
            }
            let kind = u32_at(&ev, 0);
            if self.trace {
                eprintln!("[venus] event {:#x} (capture {})", kind, if self.capture_on { "on" } else { "off" });
            }
            match kind {
                EVENT_PORT_SETTINGS_INSUFFICIENT => {
                    let mut k = BUF_TYPE_CAPTURE_MPLANE;
                    ioctl(self.fd(), VIDIOC_STREAMOFF, &mut k)?;
                    self.setup_output()?;
                    let mut k = BUF_TYPE_CAPTURE_MPLANE;
                    ioctl(self.fd(), VIDIOC_STREAMON, &mut k)?;
                    self.resume()?;
                }
                EVENT_PORT_SETTINGS_SUFFICIENT => {
                    let mut cf = raw_fmt(BUF_TYPE_CAPTURE_MPLANE, 0, 0, b"NV12");
                    if ioctl(self.fd(), VIDIOC_G_FMT, &mut cf).is_ok() {
                        self.width = u32_at(&cf, 8);
                        self.height = u32_at(&cf, 12);
                    }
                    self.resume()?;
                }
                EVENT_SYS_ERROR => return Err(io::Error::other("venus: firmware error (SYS_ERROR)")),
                _ => {}
            }
        }
    }

    /// After a port-settings event the driver waits for SESSION_CONTINUE
    /// (then, in secondary mode, it queues its picture buffers itself).
    fn resume(&self) -> io::Result<()> {
        let mut cmd = [0u32; 18];
        cmd[0] = QCOM_CMD_SESSION_CONTINUE;
        ioctl(self.fd(), VIDIOC_DECODER_CMD, &mut cmd)
    }

    /// Take back done inputs and hand out done pictures; true = anything
    /// came (round 67: the driver's poll says "ready" without anything to
    /// dequeue — the waiting loops sleep on false instead of spinning,
    /// which cost the playback thread 44 % CPU).
    fn reap(&mut self, on_frame: &mut impl FnMut(Frame<'_>)) -> io::Result<bool> {
        self.events()?;
        let mut progress = false;
        while let Some((i, _, _, _)) = dqbuf(self.fd(), BUF_TYPE_OUTPUT_MPLANE) {
            self.free.push(i);
            progress = true;
        }
        while self.capture_on {
            let Some((i, used, flags, ts)) = dqbuf(self.fd(), BUF_TYPE_CAPTURE_MPLANE) else { break };
            let Some(b) = self.output.get(i as usize) else { continue };
            progress = true;
            // Round 69: the end-of-stream buffer comes back "filled" with a
            // stale picture — every stream showed one picture too many
            // (2280 for 2279 on a real YouTube stream). It ends, it shows
            // nothing.
            if used > 0 && flags & QCOM_BUF_FLAG_EOS == 0 {
                // SAFETY: the driver filled `used` bytes of this mapping.
                let data = unsafe { std::slice::from_raw_parts(b.map, used.min(b.len)) };
                on_frame(Frame { width: self.width, height: self.height, stride: self.stride, scanlines: self.scanlines, data, timestamp_us: ts });
                self.frames_out += 1;
            }
            if flags & QCOM_BUF_FLAG_EOS != 0 {
                self.ended = true;
            }
            qbuf(self.fd(), BUF_TYPE_CAPTURE_MPLANE, i, b, 0, 0, 0)?;
        }
        Ok(progress)
    }

    fn wait(&self, ms: i32) -> io::Result<bool> {
        let mut p = libc::pollfd { fd: self.fd(), events: libc::POLLIN | libc::POLLOUT | libc::POLLPRI, revents: 0 };
        let r = unsafe { libc::poll(&mut p, 1, ms) };
        if r < 0 {
            let e = io::Error::last_os_error();
            return if e.kind() == io::ErrorKind::Interrupted { Ok(true) } else { Err(e) };
        }
        if p.revents & libc::POLLERR != 0 {
            return Err(io::Error::other("venus: device error (POLLERR)"));
        }
        Ok(r > 0)
    }

    fn take_input(&mut self, on_frame: &mut impl FnMut(Frame<'_>)) -> io::Result<u32> {
        self.reap(on_frame)?;
        let mut quiet = 0;
        while self.free.is_empty() {
            if !self.wait(1000)? {
                quiet += 1;
                if quiet > 3 {
                    return Err(io::Error::other("venus: decoder stalled"));
                }
            }
            if !self.reap(on_frame)? {
                std::thread::sleep(IDLE_NAP);
            }
        }
        Ok(self.free.pop().unwrap_or(0))
    }

    /// Decode one access unit; finished pictures go to `on_frame`.
    pub fn decode(&mut self, packet: &[u8], timestamp_us: i64, on_frame: &mut impl FnMut(Frame<'_>)) -> io::Result<()> {
        self.send(packet, timestamp_us, 0, on_frame)
    }

    /// Hand the codec configuration (H.264: SPS + PPS) as its own packet,
    /// marked the way the Android media stack marks it.
    pub fn configure(&mut self, config: &[u8], on_frame: &mut impl FnMut(Frame<'_>)) -> io::Result<()> {
        self.send(config, 0, QCOM_BUF_FLAG_CODECCONFIG, on_frame)
    }

    fn send(&mut self, packet: &[u8], timestamp_us: i64, flags: u32, on_frame: &mut impl FnMut(Frame<'_>)) -> io::Result<()> {
        let i = self.take_input(on_frame)?;
        let b = &self.input[i as usize];
        if packet.len() > b.len {
            self.free.push(i);
            return Err(io::Error::other(format!("venus: packet of {} bytes exceeds the input buffer", packet.len())));
        }
        // SAFETY: the mapping is `b.len` long ≥ packet.len().
        unsafe { std::ptr::copy_nonoverlapping(packet.as_ptr(), b.map, packet.len()) };
        qbuf(self.fd(), BUF_TYPE_OUTPUT_MPLANE, i, b, packet.len(), flags, timestamp_us)?;
        self.packets_in += 1;
        Ok(())
    }

    /// Signal the end of stream and hand out every remaining picture.
    pub fn finish(&mut self, on_frame: &mut impl FnMut(Frame<'_>)) -> io::Result<()> {
        let i = self.take_input(on_frame)?;
        let b = &self.input[i as usize];
        qbuf(self.fd(), BUF_TYPE_OUTPUT_MPLANE, i, b, 0, QCOM_BUF_FLAG_EOS, 0)?;
        let mut quiet = 0;
        while !self.ended && quiet < 4 {
            if !self.wait(500)? {
                quiet += 1;
            }
            if !self.reap(on_frame)? {
                std::thread::sleep(IDLE_NAP);
            }
        }
        Ok(())
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        let kinds: &[u32] = if self.capture_on { &[BUF_TYPE_CAPTURE_MPLANE, BUF_TYPE_OUTPUT_MPLANE] } else { &[BUF_TYPE_OUTPUT_MPLANE] };
        for &kind in kinds {
            let mut k = kind;
            let _ = ioctl(self.fd(), VIDIOC_STREAMOFF, &mut k);
        }
    }
}

/// The parameter sets (SPS, PPS — NAL 7 and 8) of an H.264 access unit,
/// with their start codes: the codec configuration to hand first.
pub fn parameter_sets(unit: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut starts = Vec::new();
    let mut i = 0;
    while i + 3 < unit.len() {
        if unit[i] == 0 && unit[i + 1] == 0 && unit[i + 2] == 1 {
            starts.push(i);
            i += 3;
        } else {
            i += 1;
        }
    }
    for (k, &s) in starts.iter().enumerate() {
        let end = starts.get(k + 1).copied().unwrap_or(unit.len());
        let end = if end < unit.len() && end > 0 && unit[end - 1] == 0 { end - 1 } else { end };
        if matches!(unit[s + 3] & 0x1f, 7 | 8) {
            out.extend_from_slice(&[0, 0, 0, 1]);
            out.extend_from_slice(&unit[s + 3..end]);
        }
    }
    out
}

/// Cut an Annex-B H.264 stream into access units at its delimiters
/// (NAL type 9 — ffmpeg writes them with `h264_metadata=aud=insert`).
/// Returns the byte ranges of the complete units in `data`; the tail
/// after the last delimiter stays for the next round (`consumed`).
pub fn access_units(data: &[u8]) -> (Vec<std::ops::Range<usize>>, usize) {
    let mut starts = Vec::new();
    let mut i = 0;
    while i + 3 < data.len() {
        if data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1 {
            if data[i + 3] & 0x1f == 9 {
                // include a leading zero of a 4-byte start code
                starts.push(if i > 0 && data[i - 1] == 0 { i - 1 } else { i });
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    let units = starts.windows(2).map(|w| w[0]..w[1]).collect();
    (units, starts.last().copied().unwrap_or(0))
}

/// Round 146: cut an Annex-B stream of `codec` into access units at its
/// delimiters — H.264 NAL 9, HEVC NAL 35 (ffmpeg
/// `hevc_metadata=aud=insert`). Like `access_units`, the tail after the
/// last delimiter is left for the next round (`consumed`); at the end of a
/// stream pass it as the last unit. VP9 has no Annex B: None.
pub fn access_units_of(codec: Codec, data: &[u8]) -> Option<(Vec<std::ops::Range<usize>>, usize)> {
    let delimiter = |b: u8| match codec {
        Codec::H264 => b & 0x1f == 9,
        Codec::Hevc => (b >> 1) & 0x3f == 35,
        Codec::Vp9 => false,
    };
    if codec == Codec::Vp9 {
        return None;
    }
    let mut starts = Vec::new();
    let mut i = 0;
    while i + 3 < data.len() {
        if data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1 {
            if delimiter(data[i + 3]) {
                starts.push(if i > 0 && data[i - 1] == 0 { i - 1 } else { i });
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    let units = starts.windows(2).map(|w| w[0]..w[1]).collect();
    Some((units, starts.last().copied().unwrap_or(0)))
}

#[cfg(test)]
mod access_unit_tests {
    use super::*;

    #[test]
    fn hevc_and_h264_cut_at_their_own_delimiters() {
        // HEVC: AUD = NAL 35 → header byte 35 << 1 = 0x46; a slice (NAL 1) between.
        let hevc = [0, 0, 0, 1, 0x46, 0x01, 0x50, 0, 0, 1, 0x02, 0x01, 0xAA, 0, 0, 0, 1, 0x46, 0x01, 0x50, 0, 0, 1, 0x02, 0x01, 0xBB];
        let (units, consumed) = access_units_of(Codec::Hevc, &hevc).unwrap();
        assert_eq!(units, vec![0..13]);
        assert_eq!(consumed, 13);
        // The same bytes are no H.264 delimiters (0x46 & 0x1f = 6).
        let (units, _) = access_units_of(Codec::H264, &hevc).unwrap();
        assert!(units.is_empty());
        // H.264 agrees with the old cutter.
        let h264 = [0, 0, 0, 1, 0x09, 0xF0, 0, 0, 1, 0x65, 0x88, 0, 0, 0, 1, 0x09, 0xF0, 0, 0, 1, 0x41];
        assert_eq!(access_units_of(Codec::H264, &h264).unwrap(), access_units(&h264));
        assert!(access_units_of(Codec::Vp9, &h264).is_none());
    }
}

/// MP4 display rotation (ffmpeg `-display_rotation`) for a canvas output
/// transform: the screencopy picture is panel-native, niri turns it.
/// Measured on the Duo: transform "90" → -90.
pub fn display_rotation(transform: &str) -> i32 {
    match transform {
        "90" => -90,
        "180" => 180,
        "270" => 90,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts_match_the_kernel_abi() {
        assert_eq!(std::mem::size_of::<V4l2Plane>(), 64);
        assert_eq!(std::mem::size_of::<V4l2Buffer>(), 88);
        assert_eq!(std::mem::size_of::<IonAlloc>(), 24);
        assert_eq!(VIDIOC_S_FMT, 0xc0d0_5605);
        assert_eq!(VIDIOC_QBUF, 0xc058_560f);
        assert_eq!(CID_BITRATE_MODE, 0x0099_09c8);
        assert_eq!(CID_BITRATE, 0x0099_09cf);
    }

    #[test]
    fn access_units_cut_at_delimiters() {
        let aud = [0u8, 0, 0, 1, 0x09, 0xf0];
        let idr = [0u8, 0, 0, 1, 0x65, 1, 2, 3];
        let mut s = Vec::new();
        s.extend_from_slice(&aud);
        s.extend_from_slice(&idr);
        s.extend_from_slice(&aud);
        s.extend_from_slice(&[0, 0, 1, 0x41, 9]);
        s.extend_from_slice(&aud);
        let (units, rest) = access_units(&s);
        assert_eq!(units.len(), 2);
        assert_eq!(units[0], 0..14);
        assert_eq!(&s[units[1].start..units[1].start + 5], &[0, 0, 0, 1, 0x09]);
        assert_eq!(rest, s.len() - 6);
        assert_eq!(std::mem::size_of::<[u8; 136]>(), 136);
        assert_eq!(VIDIOC_DQEVENT, 0x8088_5659);
    }

    #[test]
    fn rotation_follows_the_transform() {
        assert_eq!(display_rotation("90"), -90);
        assert_eq!(display_rotation("normal"), 0);
        assert_eq!(display_rotation("270"), 90);
    }
}
