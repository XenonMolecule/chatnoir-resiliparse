use fastwarc::warc::iter::{ArchiveIterator, SharedWarcRecord};
use std::io::BufReader;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    let start = Instant::now();
    let mut last_timer = start;
    let mut last_count = 0usize;
    let mut last_bytes = 0u64;
    let mut total_count = 0usize;
    let mut total_bytes = 0u64;

    let file = std::fs::File::open(path).unwrap();
    let reader = BufReader::with_capacity(1 << 20, file);

    println!("Reading WARC file: {}", path);
    for record in ArchiveIterator::new(reader) {
        if record.is_err() {
            continue;
        }
        record.unwrap().with_mut(|record| {
            let content_length = record.content_length();
            last_count += 1;
            last_bytes += content_length;
            total_count += 1;
            total_bytes += content_length;

            let elapsed = last_timer.elapsed();
            if elapsed >= Duration::from_millis(500) {
                println!(
                    "{:.0} records/s, {:.1} MiB/s, {:.1} KiB/rec ({} total, {:.1} MiB)",
                    last_count as f64 / elapsed.as_secs_f64(),
                    last_bytes as f64 / elapsed.as_secs_f64() / 1024.0 / 1024.0,
                    last_bytes as f64 / last_count.max(1) as f64 / 1024.0,
                    total_count,
                    total_bytes as f64 / 1024.0 / 1024.0
                );
                last_count = 0;
                last_bytes = 0;
                last_timer = Instant::now();
            }
        });
        // if total_count > 20000 {
        //     break;
        // }
    }
    println!("Time elapsed: {:.1}s", (Instant::now() - start).as_secs_f64());
}
