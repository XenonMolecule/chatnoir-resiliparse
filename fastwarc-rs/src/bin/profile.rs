use fastwarc::warc::iter::{ArchiveIterator, SharedWarcRecord};
use std::io::BufReader;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    let start = Instant::now();
    let mut last_timer = start;
    let mut last_count = 0usize;
    let mut total_count = 0usize;
    println!("Reading WARC file: {}", path);

    for record in ArchiveIterator::new(BufReader::with_capacity(2usize.pow(10), std::fs::File::open(path).unwrap())) {
        if record.is_err() {
            continue;
        }
        record.unwrap().with_mut(|_| {
            last_count += 1;
            total_count += 1;

            let elapsed = last_timer.elapsed();
            if elapsed >= Duration::from_millis(500) {
                println!("{:.1} records/s ({} total)", last_count as f64 / elapsed.as_secs_f64(), total_count);
                last_count = 0;
                last_timer = Instant::now();
            }
        });
        // if total_count > 20000 {
        //     break;
        // }
    }
    println!("Time elapsed: {:.1}s", (Instant::now() - start).as_secs_f64());
}
