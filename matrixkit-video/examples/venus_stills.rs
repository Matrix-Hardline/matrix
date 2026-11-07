//! Stills from an HEVC stream through the Venus decoder (05.11.2026):
//! `venus_stills FILE.hevc W H OUTDIR EVERY` — an Annex-B stream with
//! access-unit delimiters (ffmpeg `-c:v copy -bsf:v
//! hevc_mp4toannexb,hevc_metadata=aud=insert -f hevc`). Writes every
//! EVERY-th picture as packed NV12 (W×H luma, then W×H/2 chroma) to
//! OUTDIR/fNNNN.nv12. The way to look at iPhone recordings (HEVC) where no
//! software decoder for HEVC is installed.
use matrixkit_video::venus;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).expect("file");
    let (w, h): (u32, u32) = (a[2].parse().unwrap(), a[3].parse().unwrap());
    let out = std::path::PathBuf::from(&a[4]);
    let every: u64 = a[5].parse().unwrap();
    std::fs::create_dir_all(&out).unwrap();
    let dev = venus::decoder_device().expect("no Venus decoder");
    let mut d = venus::Decoder::open(&dev, venus::Codec::Hevc, w, h).expect("open");
    let mut n = 0u64;
    let mut written = 0;
    let mut on = |f: venus::Frame<'_>| {
        if n % every == 0 {
            let (fw, fh) = (f.width as usize, f.height as usize);
            let mut px = Vec::with_capacity(fw * fh * 3 / 2);
            for r in 0..fh {
                px.extend_from_slice(&f.luma()[r * f.stride..r * f.stride + fw]);
            }
            let c = f.chroma();
            for r in 0..fh.div_ceil(2) {
                px.extend_from_slice(&c[r * f.stride..r * f.stride + fw]);
            }
            std::fs::write(out.join(format!("f{:04}-{}x{}.nv12", n, fw, fh)), px).unwrap();
            written += 1;
        }
        n += 1;
    };
    // The kit's cutter (round 146); the tail is the last unit.
    let (mut units, tail) = venus::access_units_of(venus::Codec::Hevc, &data).expect("Annex B");
    if tail < data.len() {
        units.push(tail..data.len());
    }
    for (k, u) in units.into_iter().enumerate() {
        d.decode(&data[u], k as i64 * 16_667, &mut on).expect("decode");
    }
    d.finish(&mut on).expect("finish");
    eprintln!("{n} Bilder, {written} geschrieben");
}
