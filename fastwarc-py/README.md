# FastWARC (Python)

FastWARC is a high-performance WARC parsing library written in Rust with bindings for Python.
FastWARC supports compressed and uncompressed WARC/1.0 and WARC/1.1 streams. Supported compression algorithms are Gzip,
Zstd, and LZ4.

This package contains the Python bindings for FastWARC.

FastWARC belongs to the [ChatNoir Resiliparse toolkit](https://github.com/chatnoir-eu/chatnoir-resiliparse/) for fast
and robust web data processing.

## Installing FastWARC

Pre-built FastWARC binaries for Python can be installed from [PyPi](https://pypi.org/project/FastWARC/):

```bash
pip install fastwarc
```

## Building Python Bindings From Source

To be able to build FastWARC, you need the Rust toolchain installed (see above). Then to build directly from the GitHub
repository, follow these steps:

```bash
# Clone repository

git clone https://github.com/chatnoir-eu/chatnoir-resiliparse.git
cd chatnoir-resiliparse

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

You can also cite our [ECIR 2018 demo paper](https://webis.de/downloads/publications/papers/bevendorff_2018.pdf):

```bibtex
@InProceedings{bevendorff:2018,
  address =             {Berlin Heidelberg New York},
  author =              {Janek Bevendorff and Benno Stein and Matthias Hagen and Martin Potthast},
  booktitle =           {Advances in Information Retrieval. 40th European Conference on IR Research (ECIR 2018)},
  editor =              {Leif Azzopardi and Allan Hanbury and Gabriella Pasi and Benjamin Piwowarski},
  month =               mar,
  publisher =           {Springer},
  series =              {Lecture Notes in Computer Science},
  site =                {Grenoble, France},
  title =               {{Elastic ChatNoir: Search Engine for the ClueWeb and the Common Crawl}},
  year =                2018
}
```
