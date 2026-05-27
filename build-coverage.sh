#!/bin/bash
# Build coverage report

set -e

rm -rf .coverage.out
rm -f .coverage
rm -f ./*.profraw

OUTDIR=.coverage.out
PYTHON="python${PYTHON_VERSION:-3}"

# Set up llvm-cov env
source <(cargo llvm-cov show-env --sh)

# Clean up previous coverage runs
cargo llvm-cov clean --workspace

# Run fastwarc tests
cargo test --package fastwarc

# Rebuild Python bindings
$PYTHON -m pip install --no-build-isolation -e fastwarc-py/

# Run Python tests
rm -rf .coverage.html
$PYTHON -m pytest \
  --cov \
  --cov-report "xml:${OUTDIR}/pytest-coverage.xml" \
  --cov-report "lcov:${OUTDIR}/pytest-coverage.info" \
  --junitxml="${OUTDIR}/pytest-report.junit.xml" \
  fastwarc-py/fastwarc-pytest

# Generate Rust coverage report
cargo llvm-cov report --lcov --output-path "${OUTDIR}/cargo-coverage.info"

# Generate HTML report
if [ -n "$WITH_HTML" ] || [ -n "$WITH_HTML_OPEN" ]; then
  genhtml -o "${OUTDIR}/html" --ignore-errors inconsistent "${OUTDIR}/pytest-coverage.info" "${OUTDIR}/cargo-coverage.info"
fi
if [ -n "$WITH_HTML_OPEN" ]; then
  if command -v xdg-open &>/dev/null; then
    xdg-open "${OUTDIR}/html/index.html"
  else
    open "${OUTDIR}/index.html"
  fi
fi

echo
echo "--------------------------------------------------------"
echo
echo "Coverage reports written to: ${OUTDIR}"
if [ -d "${OUTDIR}/html" ]; then
  echo "HTML report written to: ${OUTDIR}/html"
fi
