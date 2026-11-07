//! Round 99: RGB → NV12 on the GPU.
//!
//! The Duo's video block (Venus) encodes H.264 in hardware but takes only
//! NV12 — no RGB. wf-recorder converted every screen pixel on the CPU
//! (swscale, 73 Mpx/s for 1350×1800 at 30 fps) and the system stuttered
//! while filming. A compute shader does it here: 32-bit compositor pixels
//! in, NV12 (BT.709, limited range) out. One invocation writes four luma
//! bytes as one word and, on the upper half of the rows, the four chroma
//! bytes (U V U V) of the 4×2 block below it.

//!
//! Round 100: the converter can IMPORT a dmabuf (Vulkan external memory)
//! and read the compositor's picture where the compositor rendered it — the
//! CPU never touches the pixels. Where the driver cannot, `import` fails and
//! the caller keeps uploading.

use crate::block_on;
use ash::vk;

const SHADER: &str = r#"
struct Params { w: u32, h: u32, stride_words: u32, quads: u32, swap: u32, invert: u32 }
@group(0) @binding(0) var<storage, read> src: array<u32>;
@group(0) @binding(1) var<storage, read_write> dst: array<u32>;
@group(0) @binding(2) var<uniform> p: Params;

fn rgb(x: u32, y: u32) -> vec3<f32> {
    let cx = min(x, p.w - 1u);
    var row = min(y, p.h - 1u);
    if (p.invert == 1u) { row = p.h - 1u - row; }
    let v = src[row * p.stride_words + cx];
    let a = f32((v >> 16u) & 255u);
    let g = f32((v >> 8u) & 255u);
    let c = f32(v & 255u);
    if (p.swap == 1u) { return vec3<f32>(c, g, a); }
    return vec3<f32>(a, g, c);
}

fn luma(c: vec3<f32>) -> u32 {
    return u32(clamp(16.0 + 0.1826 * c.r + 0.6142 * c.g + 0.0620 * c.b + 0.5, 16.0, 235.0));
}

fn chroma(c: vec3<f32>) -> vec2<u32> {
    let u = 128.0 - 0.1006 * c.r - 0.3386 * c.g + 0.4392 * c.b + 0.5;
    let v = 128.0 + 0.4392 * c.r - 0.3989 * c.g - 0.0403 * c.b + 0.5;
    return vec2<u32>(u32(clamp(u, 16.0, 240.0)), u32(clamp(v, 16.0, 240.0)));
}

@compute @workgroup_size(16, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let q = id.x;
    let y = id.y;
    if (q >= p.quads || y >= p.h) { return; }
    let x = q * 4u;
    let word = luma(rgb(x, y)) | (luma(rgb(x + 1u, y)) << 8u)
             | (luma(rgb(x + 2u, y)) << 16u) | (luma(rgb(x + 3u, y)) << 24u);
    dst[y * p.quads + q] = word;
    if (y < p.h / 2u) {
        let r0 = y * 2u;
        let left = (rgb(x, r0) + rgb(x + 1u, r0) + rgb(x, r0 + 1u) + rgb(x + 1u, r0 + 1u)) * 0.25;
        let right = (rgb(x + 2u, r0) + rgb(x + 3u, r0) + rgb(x + 2u, r0 + 1u) + rgb(x + 3u, r0 + 1u)) * 0.25;
        let a = chroma(left);
        let b = chroma(right);
        dst[p.h * p.quads + y * p.quads + q] = a.x | (a.y << 8u) | (b.x << 16u) | (b.y << 24u);
    }
}
"#;

/// A dmabuf the converter reads in place.
struct Imported {
    buffer: wgpu::Buffer,
    group: wgpu::BindGroup,
    memory: vk::DeviceMemory,
}

/// A converter for one picture size.
pub struct Nv12 {
    /// The Vulkan device with external-memory extensions (None: no import).
    vk: Option<ash::Device>,
    vk_fd: Option<ash::khr::external_memory_fd::Device>,
    imported: Option<Imported>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    group: wgpu::BindGroup,
    input: wgpu::Buffer,
    output: wgpu::Buffer,
    readback: wgpu::Buffer,
    params: wgpu::Buffer,
    width: u32,
    height: u32,
    stride: u32,
    quads: u32,
    out: Vec<u8>,
}

impl Nv12 {
    /// `stride` = bytes per source row (from screencopy). `width`/`height`
    /// must be even.
    pub fn new(width: u32, height: u32, stride: u32) -> Result<Nv12, String> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .map_err(|e| format!("Adapter: {e}"))?;
        let desc = wgpu::DeviceDescriptor {
            label: Some("matrixkit-nv12"),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..Default::default()
        };
        // Vulkan with the dmabuf extensions where the driver has them.
        let (device, queue, vk, vk_fd) = match open_importing(&adapter, &desc) {
            Some((d, q, raw, fd)) => (d, q, Some(raw), Some(fd)),
            None => {
                let (d, q) = block_on(adapter.request_device(&desc)).map_err(|e| format!("Gerät: {e}"))?;
                (d, q, None, None)
            }
        };
        let quads = width.div_ceil(4);
        let in_len = stride as u64 * height as u64;
        let out_len = quads as u64 * 4 * (height as u64 + height as u64 / 2);
        let usage_in = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let input = device.create_buffer(&wgpu::BufferDescriptor { label: Some("nv12-in"), size: in_len, usage: usage_in, mapped_at_creation: false });
        let output = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nv12-out"),
            size: out_len,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nv12-readback"),
            size: out_len,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nv12-params"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nv12"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("nv12"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nv12"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: input.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: output.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: params.as_entire_binding() },
            ],
        });
        Ok(Nv12 { vk, vk_fd, imported: None, device, queue, pipeline, group, input, output, readback, params, width, height, stride, quads, out: Vec::new() })
    }

    /// Can this converter read dmabufs in place?
    pub fn can_import(&self) -> bool {
        self.vk.is_some()
    }

    /// Read the pictures from this dmabuf from now on (`len` = its size).
    /// The fd stays the caller's (it is duplicated for Vulkan).
    pub fn import(&mut self, fd: std::os::fd::BorrowedFd<'_>, len: u64) -> Result<(), String> {
        use std::os::fd::AsRawFd;
        self.drop_import();
        let (Some(raw), Some(fd_ext)) = (&self.vk, &self.vk_fd) else { return Err("kein Import".into()) };
        let size = self.stride as u64 * self.height as u64;
        if len < size {
            return Err("dmabuf zu klein".into());
        }
        let handle = vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT;
        // SAFETY: plain Vulkan calls on our own device; every object made
        // here is destroyed on failure or in drop_import.
        unsafe {
            let mut ext = vk::ExternalMemoryBufferCreateInfo::default().handle_types(handle);
            let info = vk::BufferCreateInfo::default()
                .size(size)
                .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .push_next(&mut ext);
            let buffer = raw.create_buffer(&info, None).map_err(|e| format!("vkCreateBuffer: {e}"))?;
            let req = raw.get_buffer_memory_requirements(buffer);
            let mut props = vk::MemoryFdPropertiesKHR::default();
            if let Err(e) = fd_ext.get_memory_fd_properties(handle, fd.as_raw_fd(), &mut props) {
                raw.destroy_buffer(buffer, None);
                return Err(format!("dmabuf-Eigenschaften: {e}"));
            }
            let bits = req.memory_type_bits & props.memory_type_bits;
            if bits == 0 {
                raw.destroy_buffer(buffer, None);
                return Err("kein passender Speichertyp".into());
            }
            // Vulkan takes ownership of the fd on success: hand it a copy.
            let dup = match fd.try_clone_to_owned() {
                Ok(d) => std::os::fd::IntoRawFd::into_raw_fd(d),
                Err(e) => {
                    raw.destroy_buffer(buffer, None);
                    return Err(format!("dup: {e}"));
                }
            };
            let mut import = vk::ImportMemoryFdInfoKHR::default().handle_type(handle).fd(dup);
            let mut dedicated = vk::MemoryDedicatedAllocateInfo::default().buffer(buffer);
            let alloc = vk::MemoryAllocateInfo::default()
                .allocation_size(len.max(req.size))
                .memory_type_index(bits.trailing_zeros())
                .push_next(&mut import)
                .push_next(&mut dedicated);
            let memory = match raw.allocate_memory(&alloc, None) {
                Ok(m) => m,
                Err(e) => {
                    drop(<std::os::fd::OwnedFd as std::os::fd::FromRawFd>::from_raw_fd(dup));
                    raw.destroy_buffer(buffer, None);
                    return Err(format!("dmabuf-Import: {e}"));
                }
            };
            if let Err(e) = raw.bind_buffer_memory(buffer, memory, 0) {
                raw.free_memory(memory, None);
                raw.destroy_buffer(buffer, None);
                return Err(format!("Binden: {e}"));
            }
            // wgpu owns (and destroys) the VkBuffer; the memory stays ours.
            let hal = wgpu::hal::vulkan::Buffer::from_raw(buffer);
            let buffer = self.device.create_buffer_from_hal::<wgpu::hal::api::Vulkan>(
                hal,
                &wgpu::BufferDescriptor { label: Some("nv12-dmabuf"), size, usage: wgpu::BufferUsages::STORAGE, mapped_at_creation: false },
            );
            let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("nv12-dmabuf"),
                layout: &self.pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: self.output.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: self.params.as_entire_binding() },
                ],
            });
            self.imported = Some(Imported { buffer, group, memory });
        }
        Ok(())
    }

    fn drop_import(&mut self) {
        if let Some(imp) = self.imported.take() {
            drop(imp.group);
            drop(imp.buffer);
            let _ = self.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: Some(std::time::Duration::from_secs(2)) });
            if let Some(raw) = &self.vk {
                // SAFETY: the GPU is idle and wgpu let go of the buffer.
                unsafe {
                    let _ = raw.device_wait_idle();
                    raw.free_memory(imp.memory, None);
                }
            }
        }
    }

    /// Convert the picture that sits in the imported dmabuf right now.
    pub fn convert_imported(&mut self, swap: bool, invert: bool) -> Result<&[u8], String> {
        if self.imported.is_none() {
            return Err("nichts importiert".into());
        }
        self.run(None, swap, invert)
    }

    /// Bytes per row of the NV12 result (both planes): width rounded up to 4.
    pub fn row_bytes(&self) -> usize {
        self.quads as usize * 4
    }

    /// Convert one picture. `swap`: the source is XBGR/ABGR (red in the low
    /// byte); `invert`: its rows run bottom-up. Returns the luma plane
    /// followed by the interleaved chroma plane, rows of `row_bytes()`.
    pub fn convert(&mut self, pixels: &[u8], swap: bool, invert: bool) -> Result<&[u8], String> {
        let len = self.stride as usize * self.height as usize;
        if pixels.len() < len {
            return Err("Bild zu kurz".into());
        }
        self.run(Some(&pixels[..len]), swap, invert)
    }

    /// One pass: `pixels` uploaded, or None = read the imported dmabuf.
    fn run(&mut self, pixels: Option<&[u8]>, swap: bool, invert: bool) -> Result<&[u8], String> {
        let words = [self.width, self.height, self.stride / 4, self.quads, swap as u32, invert as u32, 0, 0];
        let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
        self.queue.write_buffer(&self.params, 0, &bytes);
        if let Some(px) = pixels {
            self.queue.write_buffer(&self.input, 0, px);
        }
        let group = match (pixels, &self.imported) {
            (None, Some(imp)) => &imp.group,
            _ => &self.group,
        };
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("nv12") });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("nv12"), timestamp_writes: None });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, group, &[]);
            pass.dispatch_workgroups(self.quads.div_ceil(16), self.height.div_ceil(8), 1);
        }
        enc.copy_buffer_to_buffer(&self.output, 0, &self.readback, 0, self.output.size());
        self.queue.submit([enc.finish()]);
        let slice = self.readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device
            .poll(wgpu::PollType::Wait { submission_index: None, timeout: Some(std::time::Duration::from_secs(2)) })
            .map_err(|e| format!("GPU: {e}"))?;
        rx.recv().map_err(|e| e.to_string())?.map_err(|e| format!("Abbildung: {e}"))?;
        {
            let view = slice.get_mapped_range();
            self.out.clear();
            self.out.extend_from_slice(&view);
        }
        self.readback.unmap();
        Ok(&self.out)
    }
}

impl Drop for Nv12 {
    fn drop(&mut self) {
        self.drop_import();
    }
}

/// Open the device with VK_KHR_external_memory_fd + VK_EXT_external_memory_dma_buf
/// (None where the backend is not Vulkan or the driver lacks them).
fn open_importing(adapter: &wgpu::Adapter, desc: &wgpu::DeviceDescriptor<'_>) -> Option<(wgpu::Device, wgpu::Queue, ash::Device, ash::khr::external_memory_fd::Device)> {
    let want: [&'static std::ffi::CStr; 2] = [ash::khr::external_memory_fd::NAME, ash::ext::external_memory_dma_buf::NAME];
    // SAFETY: the hal adapter is used only within this function.
    let open = unsafe {
        let hal = adapter.as_hal::<wgpu::hal::api::Vulkan>()?;
        let instance = hal.shared_instance().raw_instance();
        let have = instance.enumerate_device_extension_properties(hal.raw_physical_device()).ok()?;
        let present = |n: &std::ffi::CStr| have.iter().any(|e| e.extension_name_as_c_str().is_ok_and(|x| x == n));
        if !want.iter().all(|n| present(n)) {
            eprintln!("[nv12] Treiber ohne dmabuf-Import");
            return None;
        }
        hal.open_with_callback(
            wgpu::Features::empty(),
            &wgpu::MemoryHints::MemoryUsage,
            Some(Box::new(move |args: wgpu::hal::vulkan::CreateDeviceCallbackArgs<'_, '_, '_>| {
                for n in want {
                    if !args.extensions.contains(&n) {
                        args.extensions.push(n);
                    }
                }
            })),
        )
        .ok()?
    };
    // SAFETY: a fresh hal device of this adapter.
    let (device, queue) = unsafe { adapter.create_device_from_hal(open, desc) }.ok()?;
    // SAFETY: borrowing the raw handles of the device we just made; ash's
    // Device is a cheap clone of the function table.
    let (raw, fd) = unsafe {
        let hal = device.as_hal::<wgpu::hal::api::Vulkan>()?;
        let raw = hal.raw_device().clone();
        let fd = ash::khr::external_memory_fd::Device::new(hal.shared_instance().raw_instance(), &raw);
        (raw, fd)
    };
    Some((device, queue, raw, fd))
}

#[cfg(test)]
mod tests {
    /// Known colours through the shader (skipped where no GPU answers).
    #[test]
    fn white_and_red_land_on_bt709() {
        // 6×2 pixels, XRGB little-endian: three white, three red per row.
        let (w, h) = (6u32, 2u32);
        let mut px = Vec::new();
        for _ in 0..h {
            for x in 0..w {
                let v: u32 = if x < 3 { 0x00ff_ffff } else { 0x00ff_0000 };
                px.extend_from_slice(&v.to_le_bytes());
            }
        }
        let Ok(mut c) = super::Nv12::new(w, h, w * 4) else {
            eprintln!("keine GPU — übersprungen");
            return;
        };
        let row = c.row_bytes();
        assert_eq!(row, 8);
        let out = c.convert(&px, false, false).unwrap().to_vec();
        assert_eq!(&out[0..6], &[235, 235, 235, 63, 63, 63]);
        // chroma: block 0 (white) neutral, block 2 (red) U 102 V 240.
        let uv = &out[row * h as usize..];
        assert_eq!(&uv[0..2], &[128, 128]);
        assert_eq!(&uv[4..6], &[102, 240]);
        // the same picture as XBGR: red now sits in the low byte.
        let swapped: Vec<u8> = px.chunks(4).flat_map(|p| [p[2], p[1], p[0], p[3]]).collect();
        let out2 = c.convert(&swapped, true, false).unwrap().to_vec();
        assert_eq!(out, out2);
    }
}
