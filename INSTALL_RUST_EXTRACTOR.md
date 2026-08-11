# Installing the Rust extractor (this fork's main-content engine)

The extraction engine this fork's benchmarks measure is the Rust
implementation (`resiliparse._extract_rs`), not upstream's Cython
extension. It does NOT build automatically with `pip install` — three
extra steps are required.

## Prerequisites
- **Rust** (stable toolchain; `rustup` is fine)
- **vcpkg on PATH** — `resiliparse-rs/build.rs` runs `vcpkg install` at
  build time to fetch lexbor into the build directory. Install vcpkg and
  make sure the `vcpkg` binary is on PATH; set `VCPKG_ROOT` to the vcpkg
  checkout (e.g. `export VCPKG_ROOT="$HOME/vcpkg"`).
- **cmake** and **libclang** (bindgen needs libclang; on macOS the Xcode
  CLT suffice, on Debian/Ubuntu `apt install cmake libclang-dev`).
- Python 3.11+ with the `resiliparse` package importable — either
  `pip install -e resiliparse-py` or put `resiliparse-py` on PYTHONPATH.

## Build
```bash
export VCPKG_ROOT="$HOME/vcpkg"   # your vcpkg checkout
cargo build --release -p resiliparse-extract-rs
```
The first build is slow (vcpkg compiles lexbor); later builds are ~10 s.

## Link the extension (REQUIRED, easy to miss)
The Python import resolves through a symlink that is gitignored, so a
fresh clone will import-fail even after a successful cargo build:
```bash
# macOS
ln -sf "$(pwd)/target/release/libresiliparse_extract_rs.dylib" \
       resiliparse-py/resiliparse/_extract_rs.so
# Linux
ln -sf "$(pwd)/target/release/libresiliparse_extract_rs.so" \
       resiliparse-py/resiliparse/_extract_rs.so
```

## Verify
```bash
python - <<'EOF'
from resiliparse._extract_rs import extract_plain_text
h = "<html><body><main><p>It works.</p></main></body></html>"
print(extract_plain_text(h, main_content=True, preserve_formatting='markdown'))
EOF
cargo test --release -p resiliparse --test extract_golden   # 7 tests
```

## Gotchas
- `from resiliparse.extract.html2text import extract_plain_text` imports
  the CYTHON reference extension, not this engine. Benchmarked behavior
  is only in `resiliparse._extract_rs` (what
  `benchmark/eval/run_eval.py --impl rust` uses).
- The eval harness re-runs `cargo build` automatically but does NOT
  create the symlink.
- The symlink targets a platform-specific dylib; re-create it when
  moving a checkout between macOS and Linux.
- Typical usage: `extract_plain_text(html, main_content=True,
  preserve_formatting='markdown')` — ~3 ms/doc on an M-series laptop.
