use std::time::{Duration, Instant};

pub(crate) fn bench_cpu_single() -> u32 {
    let start = Instant::now();
    let mut acc = 1.0f64;
    for i in 1..2_000_000 {
        acc = (acc * 1.000001) + (i as f64 % 97.0);
    }
    let ms = start.elapsed().as_millis().max(1) as u32;
    (5000 / ms).min(100)
}

pub(crate) fn bench_cpu_multi() -> u32 {
    use std::thread;

    let start = Instant::now();
    let handles: Vec<_> = (0..4)
        .map(|_| {
            thread::spawn(|| {
                let mut acc = 0u64;
                for i in 0..1_500_000u64 {
                    acc = acc.wrapping_add(i * 13);
                }
                acc
            })
        })
        .collect();
    for h in handles {
        let _ = h.join();
    }
    let ms = start.elapsed().as_millis().max(1) as u32;
    (8000 / ms).min(100)
}

pub(crate) fn bench_memory() -> (u32, f64) {
    let size = 8 * 1024 * 1024;
    let mut buf = vec![0u8; size];
    let start = Instant::now();
    for i in 0..size {
        buf[i] = (i % 251) as u8;
    }
    let bw_score = (start.elapsed().as_micros() as u32 / 1000).max(1);
    let bw = (20000 / bw_score).min(100);

    let mut samples = Vec::with_capacity(100);
    for _ in 0..100 {
        let s = Instant::now();
        std::thread::sleep(Duration::from_micros(50));
        samples.push(s.elapsed().as_secs_f64() * 1_000_000.0);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let lat = samples[samples.len() / 2];
    (bw, lat)
}

pub(crate) fn bench_storage() -> (u32, u32) {
    #[cfg(windows)]
    {
        use std::env;
        use std::fs::OpenOptions;
        use std::io::{Read, Write};

        let temp = env::temp_dir().join("boostx_bench_suite.bin");
        let chunk = vec![0u8; 512 * 1024];
        let start = Instant::now();
        if let Ok(mut f) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp)
        {
            for _ in 0..128 {
                let _ = f.write_all(&chunk);
            }
            let _ = f.flush();
            let write_ms = start.elapsed().as_millis().max(1) as u32;
            let write = (8000 / write_ms).min(100);

            let start = Instant::now();
            if let Ok(mut rf) = OpenOptions::new().read(true).open(&temp) {
                let mut sink = vec![0u8; chunk.len()];
                for _ in 0..128 {
                    let _ = rf.read(&mut sink);
                }
                let read_ms = start.elapsed().as_millis().max(1) as u32;
                let read = (8000 / read_ms).min(100);
                let _ = std::fs::remove_file(temp);
                return (read, write);
            }
        }
        (50, 50)
    }
    #[cfg(not(windows))]
    {
        (55, 55)
    }
}

pub(crate) fn bench_timer_jitter() -> f64 {
    let mut samples = Vec::with_capacity(200);
    for _ in 0..200 {
        let start = Instant::now();
        std::thread::sleep(Duration::from_micros(100));
        samples.push(start.elapsed().as_secs_f64() * 1_000_000.0);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    samples[samples.len() / 2]
}
