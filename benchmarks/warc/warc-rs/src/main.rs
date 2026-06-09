use libflate::gzip::MultiDecoder as GzipReader;
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};
use warc::WarcReader;

const DEFAULT_BUFFER_SIZE: usize = 4096 << 10;

fn buffer_size() -> usize {
    std::env::var("BUFFER_SIZE")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|&value| value > 0)
        .unwrap_or(DEFAULT_BUFFER_SIZE)
}

fn profile_reader<R: std::io::BufRead>(reader: WarcReader<R>, start: Instant) {
    let mut last_timer = start;
    let mut last_count = 0usize;
    let mut last_bytes = 0usize;
    let mut total_count = 0usize;
    let mut total_bytes = 0usize;

    for item in reader.iter_records() {
        let Ok(record) = item else {
            continue;
        };
        let content_length = record.body().len();
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} WARCFILE", args[0]);
        return Ok(());
    }

    let path = &args[1];
    let start = Instant::now();
    let buffer_size = buffer_size();

    println!("Reading WARC file: {}", path);
    if path.ends_with(".gz") {
        let file = File::open(path)?;
        let file = BufReader::with_capacity(buffer_size, file);
        let gzip_stream = GzipReader::new(file)?;
        profile_reader(WarcReader::new(BufReader::with_capacity(buffer_size, gzip_stream)), start);
    } else {
        let file = File::open(path)?;
        profile_reader(WarcReader::new(BufReader::with_capacity(buffer_size, file)), start);
    }
    Ok(())
}
