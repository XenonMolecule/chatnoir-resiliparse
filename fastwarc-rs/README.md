# FastWARC

FastWARC is a high-performance WARC parsing library written in Rust with bindings for Python.
FastWARC supports compressed and uncompressed WARC/1.0 and WARC/1.1 streams. Supported compression algorithms are GZip,
Zstd, and LZ4.

FastWARC belongs to the [ChatNoir Resiliparse toolkit](https://github.com/chatnoir-eu/chatnoir-resiliparse/) for fast
and robust web data processing.

## Building FastWARC

To build FastWARC, you need Rust and Cargo installed. If that is not the case, install `rustup` first and follow the
instructions:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

To build FastWARC, simply run `cargo build` from the root of this repository.

## Using FastWARC as a Library

To use FastWARC in your Rust project, add it as a dependency to your `Cargo.toml`:

```toml
[dependencies]
fastwarc = "*" # Specify the correct version here
```

Then build your project like normal with `cargo build`.

### Building Python Binaries From Source

You can build the FastWARC Python bindings from source as follows:

```bash
# Install Rust toolchain (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Create a fresh venv first (recommended)
python3 -m venv venv && source venv/bin/activate

# Option 1: Build and install in editable mode (best for development)
python3 -m pip install -e ./fastwarc-py

# Option 2 (alternative): Build and install wheels in separate steps (best for redistribution)
python3 -m pip wheel -w build ./fastwarc-py
ls ./build/*.whl | xargs python3 -m pip install
```

## Usage Instructions

For detailed usage instructions, please consult
the [FastWARC User Manual](https://resiliparse.chatnoir.eu/en/latest/man/fastwarc.html).

## Cite Us

If you use FastWARC, please consider citing our [OSSYM 2021 abstract paper](https://arxiv.org/abs/2112.03103):

```bibtex
@InProceedings{bevendorff:2021,
  author =                {Janek Bevendorff and Martin Potthast and Benno Stein},
  booktitle =             {3rd International Symposium on Open Search Technology (OSSYM 2021)},
  editor =                {Andreas Wagner and Christian Guetl and Michael Granitzer and Stefan Voigt},
  month =                 oct,
  publisher =             {International Open Search Symposium},
  site =                  {CERN, Geneva, Switzerland},
  title =                 {{FastWARC: Optimizing Large-Scale Web Archive Analytics}},
  year =                  2021
}
```
