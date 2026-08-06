// Criterion benches for extract_plain_text on the frozen golden fixture pages
// (the same documents used by tests/extract_golden.rs). These are the precise
// per-page instrument for micro-optimization on the Rust path; cross-impl
// comparisons vs. Cython run through the Python-level interleaved harness
// (see research_log/0002-rust-port-spike.md).

use criterion::{Criterion, criterion_group, criterion_main};
use resiliparse::extract::html2text::{ExtractOpts, extract_plain_text};
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;

fn bench_extract(c: &mut Criterion) {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/extract_golden");
    let opts = ExtractOpts {
        main_content: true,
        ..Default::default()
    };
    let mut group = c.benchmark_group("extract_plain_text");
    for name in ["plain", "list", "table", "code", "big"] {
        let html = fs::read_to_string(fixture_dir.join(format!("{name}.html"))).unwrap();
        group.bench_function(name, |b| b.iter(|| extract_plain_text(black_box(&html), black_box(&opts))));
    }
    group.finish();
}

criterion_group!(benches, bench_extract);
criterion_main!(benches);
