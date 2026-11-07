//! Leak probe of the Venus family (round 69): open a decoder, decode a few
//! access units, drop it — N times; prints the process's open files after
//! each tenth round. `venus_cycles FILE.h264 W H [N]`
use matrixkit_video::venus;

fn fds() -> usize {
    std::fs::read_dir("/proc/self/fd").map(|d| d.count()).unwrap_or(0)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).expect("file");
    let (w, h): (u32, u32) = (a[2].parse().unwrap(), a[3].parse().unwrap());
    let rounds: usize = a.get(4).and_then(|n| n.parse().ok()).unwrap_or(30);
    let dev = venus::decoder_device().expect("no Venus decoder");
    let (units, _) = venus::access_units(&data);
    println!("start: {} files", fds());
    for r in 1..=rounds {
        let mut d = venus::Decoder::open(&dev, venus::Codec::H264, w, h).expect("open");
        let mut n = 0;
        for u in units.iter().take(12) {
            d.decode(&data[u.clone()], 0, &mut |_f| n += 1).expect("decode");
        }
        // Half the rounds end mid-stream (a seek), half drain properly.
        if r % 2 == 0 {
            let _ = d.finish(&mut |_f| n += 1);
        }
        drop(d);
        if r % 10 == 0 {
            println!("round {r}: {} files, {n} pictures in the last round", fds());
        }
    }
}
