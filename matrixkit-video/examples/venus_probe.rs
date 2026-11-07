//! Probe of the Venus decoder (round 66): `venus_probe FILE.h264 W H [PGM]`
//! — an Annex-B stream with access-unit delimiters (ffmpeg
//! `-bsf:v h264_mp4toannexb,h264_metadata=aud=insert -f h264`). Decodes
//! everything, reports pictures and speed, writes the luma of picture 30.
use matrixkit_video::venus;
use std::io::Write;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).expect("file");
    let (w, h): (u32, u32) = (a[2].parse().unwrap(), a[3].parse().unwrap());
    let dev = venus::decoder_device().expect("no Venus decoder");
    let mut d = venus::Decoder::open(&dev, venus::Codec::H264, w, h).expect("open");
    let (mut units, _) = venus::access_units(&data);
    if let Some(last) = units.last().map(|r| r.end) {
        units.push(last..data.len());
    }
    let t0 = std::time::Instant::now();
    let pgm = a.get(4).cloned();
    let mut n = 0u64;
    let mut sum = 0u64;
    let mut copy_ns: u128 = 0;
    let mut scratch: Vec<u8> = Vec::new();
    let mut on = |f: venus::Frame<'_>| {
        let t = std::time::Instant::now();
        scratch.clear();
        scratch.extend_from_slice(f.data);
        copy_ns += t.elapsed().as_nanos();
        n += 1;
        sum += f.luma()[f.stride * (f.height as usize / 2)] as u64;
        if n == 30 {
            if let Some(p) = &pgm {
                let mut o = std::fs::File::create(p).unwrap();
                write!(o, "P5\n{} {}\n255\n", f.width, f.height).unwrap();
                for y in 0..f.height as usize {
                    o.write_all(&f.luma()[y * f.stride..y * f.stride + f.width as usize]).unwrap();
                }
            }
            eprintln!("picture {}x{} stride {} scanlines {}", f.width, f.height, f.stride, f.scanlines);
        }
    };
    if std::env::var_os("VENUS_CONFIG").is_some() {
        if let Some(first) = units.first() {
            let cfg = venus::parameter_sets(&data[first.clone()]);
            eprintln!("config {} bytes", cfg.len());
            d.configure(&cfg, &mut on).expect("configure");
        }
    }
    for (k, u) in units.iter().enumerate() {
        if let Err(e) = d.decode(&data[u.clone()], k as i64 * 33_333, &mut on) {
            eprintln!("decode {k}: {e}");
            break;
        }
    }
    if let Err(e) = d.finish(&mut on) {
        eprintln!("finish: {e}");
    }
    let s = t0.elapsed().as_secs_f64();
    println!("units {} pictures {} in {:.2} s = {:.0} fps (check {sum})", units.len(), n, s, n as f64 / s);
    println!("copy out of the decoder: {:.2} ms per picture", copy_ns as f64 / 1e6 / n.max(1) as f64);
}
