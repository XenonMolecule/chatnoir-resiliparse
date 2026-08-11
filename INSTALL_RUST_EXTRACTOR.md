# Installing the Rust extractor (this fork's main-content engine)

The extraction engine this fork's benchmarks measure is the Rust
implementation (`resiliparse._extract_rs`), not upstream's Cython
extension. It is not installed by `pip install resiliparse` — build it
from this repo with the four steps below.

Verified end to end from a clean clone (macOS/arm64, ~1 min build).

## 1. Prerequisites
- **Rust** (stable; `rustup` is fine)
- **vcpkg on PATH** — the build script runs `vcpkg install` to fetch
  lexbor. Install vcpkg, then `export VCPKG_ROOT="$HOME/vcpkg"`
  (point it at your checkout).
- **cmake** and **libclang** — bindgen needs libclang. macOS: Xcode
  command line tools. Debian/Ubuntu: `apt install cmake libclang-dev`.
- **Python 3.11+**

## 2. Build
```bash
git clone https://github.com/XenonMolecule/chatnoir-resiliparse.git
cd chatnoir-resiliparse
export VCPKG_ROOT="$HOME/vcpkg"
cargo build --release -p resiliparse-extract-rs
```
First build takes ~1 minute (vcpkg compiles lexbor); later builds ~10 s.

## 3. Link the extension module (required)
Python imports the compiled library through this path, which is
gitignored — a fresh clone has to create it:
```bash
# macOS
ln -sf "$(pwd)/target/release/libresiliparse_extract_rs.dylib" \
       resiliparse-py/resiliparse/_extract_rs.so
# Linux
ln -sf "$(pwd)/target/release/libresiliparse_extract_rs.so" \
       resiliparse-py/resiliparse/_extract_rs.so
```

## 4. Put the Python package on the path
The lightest option — no compilation, since the Rust library is already
built:
```bash
export PYTHONPATH="$(pwd)/resiliparse-py:$PYTHONPATH"
```
`pip install -e resiliparse-py` also works, but it builds *upstream's
Cython* extensions from source (needs Cython + its own vcpkg step) and
is not required to use the Rust extractor.

## 5. Verify
```bash
python - <<'EOF'
from resiliparse._extract_rs import extract_plain_text
html = "<html><body><main><h1>Title</h1><p>It works.</p></main></body></html>"
print(extract_plain_text(html, main_content=True, preserve_formatting="markdown"))
EOF
# expected: "# Title" then "It works."

cargo test --release -p resiliparse --test extract_golden   # 7 passed
```

## Usage
```python
from resiliparse._extract_rs import extract_plain_text

# markdown main-content extraction (what the benchmarks measure)
text = extract_plain_text(html, main_content=True, preserve_formatting="markdown")

# plain text, whole document
text = extract_plain_text(html, main_content=False)
```
Roughly 3 ms/doc for markdown main-content extraction on an M-series
laptop, single-threaded.

## Gotchas
- **`resiliparse.extract.html2text` is upstream's Cython engine**, not
  this one. Every benchmark result in the README comes from
  `resiliparse._extract_rs`. Importing the wrong one silently gives you
  upstream behavior.
- **`Library not loaded: @rpath/liblexbor.*` on import** means the
  extension was built before the rpath fix, or the vcpkg lexbor tree was
  deleted. Rebuild: `cargo build --release -p resiliparse-extract-rs`.
- The symlink points at a platform-specific library — re-create it when
  a checkout moves between macOS and Linux.
- `benchmark/eval/run_eval.py --impl rust` rebuilds the Rust library
  automatically, but does not create the symlink for you.
