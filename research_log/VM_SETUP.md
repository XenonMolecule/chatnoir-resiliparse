# Running this repo on a fresh Linux VM (agent-online)

Verified 2026-08-09 on debian-12 / e2-standard-4. Parity confirmed: the
VM-built binary reproduced golden v16 (0.89468 / 0.81528) with **0 per-doc
differences** vs the local baseline.

## Transfer (before `agent-online up`)
The repo is 21G; `benchmark/runs` alone is 17G of historical eval output and
`benchmark/experiments` 2.1G of caches. `.agent-online/config` excludes both;
`.agent-online/sync-repo` ships only the ~17M of baselines actually needed
(the 0030-respond plain-config guardrail + the current baseline tag).
`benchmark/datasets_rawhtml` (447M) MUST travel — every eval reads it.

## Toolchain (none of it travels)
```
sudo apt-get install -y build-essential cmake pkg-config curl zip unzip tar \
     git python3-pip python3-venv python3-dev clang libclang-dev llvm-dev
curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
git clone --depth 1 https://github.com/microsoft/vcpkg.git ~/vcpkg && ~/vcpkg/bootstrap-vcpkg.sh -disableMetrics
python3 -m venv ~/venv && ~/venv/bin/pip install rapidfuzz selectolax
```
**clang/libclang-dev is the non-obvious one**: bindgen needs libclang to
generate the lexbor FFI, and the first build dies at 90% without it.

## Build + wiring
```
export PATH="$HOME/.cargo/bin:$PATH" VCPKG_ROOT="$HOME/vcpkg"
cargo build --release -p resiliparse-extract-rs      # vcpkg compiles lexbor/re2/uchardet/lz4/zlib
ln -sf "$PWD/target/release/libresiliparse_extract_rs.so" resiliparse-py/resiliparse/_extract_rs.so
echo "$PWD/target/release/build/resiliparse-*/out/vcpkg_installed/x64-linux/lib" \
  | sudo tee /etc/ld.so.conf.d/lexbor.conf && sudo ldconfig
```
The checked-in symlink points at a macOS `.dylib` name — re-point it to `.so`.
Without the ldconfig entry, importing fails on `liblexbor.so.2`.

## Running evals
```
export PATH="$HOME/.cargo/bin:$PATH" VCPKG_ROOT="$HOME/vcpkg" PYTHONPATH=resiliparse-py
~/venv/bin/python benchmark/eval/run_eval.py --dataset lpv11 --split dev_golden \
    --impl rust --workers 4 --tag TAG --preserve-formatting markdown
```

## Caveat: runtime numbers are NOT comparable across machines
This VM measures 8.07 ms/doc where the local box measures 2.76. Runtime
protocol results must be taken on one machine; do not log VM timings against
the local series.
