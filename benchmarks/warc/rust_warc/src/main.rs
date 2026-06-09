use flate2::read::MultiGzDecoder;
use rust_warc::WarcReader;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};

const BUFFER_SIZE: usize = 4096 << 10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} WARCFILE", args[0]);
        return Ok(());
    }

    let path = &args[1];
    let start = Instant::now();
    let mut last_timer = start;
    let mut last_count = 0usize;
    let mut last_bytes = 0usize;
    let mut total_count = 0usize;
    let mut total_bytes = 0usize;

    println!("Reading WARC file: {}", path);
    let file = File::open(&path)?;
    let reader: Box<dyn BufRead> = if path.ends_with(".gz") {
        Box::new(BufReader::with_capacity(
            BUFFER_SIZE,
            MultiGzDecoder::new(BufReader::with_capacity(BUFFER_SIZE, file)),
        ))
    } else {
        Box::new(BufReader::with_capacity(BUFFER_SIZE, file))
    };
    let warc = WarcReader::new(reader);

    for item in warc {
        let Ok(record) = item else {
            continue;
        };
        let content_length = record.content.len();
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
    }

    let total_elapsed = start.elapsed().as_secs_f64();
    println!(
        "Summary: {:.1}s, {:.0} records/s, {:.1} MiB/s, {:.1} KiB/rec ({} total, {:.1} MiB)",
        total_elapsed,
        total_count as f64 / total_elapsed,
        total_bytes as f64 / total_elapsed / 1024.0 / 1024.0,
        total_bytes as f64 / total_count.max(1) as f64 / 1024.0,
        total_count,
        total_bytes as f64 / 1024.0 / 1024.0
    );
    Ok(())
}
