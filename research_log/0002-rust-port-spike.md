# 0002 — Rust port spike (timeboxed, two gates)

- **Date:** 2026-08-06
- **Tag:** 0002-parity-check / 0002-speed-rs (parity oracle: v1.0.9-cacde72)
- **Status:** landed — **both gates passed; Rust is the iteration base from here**
- **Timebox:** ~4h budgeted; port + parity + speed verdict landed in ≈2.5h.

## Hypothesis

`_extract_plain_text_impl` + helpers (~930 lines Cython/C++ over raw lexbor
pointers) can be ported 1:1 onto the raw lexbor bindings inside
`resiliparse-rs`, exposed via PyO3 like `parse/_html_rs`, reaching
byte-identical output vs. the Cython reference. Preregistered risk: the Cython
is already nogil C++ over the same lexbor, so a speed wash was a real
possibility.

## What changed

- New `resiliparse-rs/src/extract/html2text.rs` (~1,000 lines): direct port of
  the traversal (`next_node`), extraction callback, serializer, and the
  main-content classifier (21 RE2 regexes → `regex::bytes`, case-insensitive
  ASCII, matching RE2 semantics; `other_junk_cls_regex` kept case-SENSITIVE as
  in the reference). Works on raw `lxb_dom_node_t` pointers — the safe Rust DOM
  wrapper (mutex per access) is deliberately bypassed in the hot walk.
- Reference quirks preserved on purpose (documented in-code): C `isspace`
  incl. `\v`; the `b'embed' b'img'` → `"embedimg"` skip-selector concatenation
  bug; `rel_attr == "main"` where `role_attr` was plausibly intended; the
  `keep_hidden` operator-precedence quirk; final `.decode(errors='ignore')`
  semantics (invalid UTF-8 dropped, not replaced).
- New PyO3 crate `resiliparse-py/resiliparse/extract/_extract_rs`
  (workspace member; `cargo build --release -p resiliparse-extract-rs`;
  symlinked as `resiliparse/_extract_rs.so`), signature-compatible with the
  Cython `extract_plain_text`, GIL released during extraction.
- 5 lpv11 dev docs frozen as golden fixtures + tests in
  `resiliparse-rs/tests/extract_golden.rs` (expected = byte-exact Cython
  output); criterion bench `benches/extract_bench.rs` over the same pages.
- Harness: `--impl rust` now rebuilds `-p resiliparse-extract-rs` before every
  run (never scores a stale binary).

## Results

**Gate 1 — parity (oracle = cached Cython predictions, not gold):**

| check | result |
|---|---|
| lpv11 dev (1,000 docs) | **1000/1000 byte-identical** (parity Lev sim = 1.0000) |
| lpv11 train (9,999 docs) | **9999/9999 byte-identical** |
| 11-config option matrix × 250 docs (links, minimal_html, form_fields, skip_elements, …) | **0 mismatches** (2,750 comparisons) |
| gold-scored quality (must equal baseline) | F1 0.7619 / Lev 0.6540 — identical to 0001 |

No residual diffs to document — parity is exact, not approximate.

**Gate 2 — speed (same docs/machine as 0001, single worker):**

Wall-clock harness runs are noisy on this laptop (±2× run-to-run), so the
verdict instrument is per-doc interleaved timing in a single process
(alternating call order per doc, warmed up; drift cancels):

| metric | Cython | Rust | ratio |
|---|---|---|---|
| mean ms/doc | 1.635 | 1.284 | **1.27×** |
| p50 ms/doc | 0.952 | 0.821 | 1.16× |
| p95 ms/doc | 4.783 | 3.649 | **1.31×** (better, not worse) |

Reproduced on a second full pass: 1.270× (vs 1.273×). Six interleaved
harness-level rounds agreed directionally (Rust won all 6 pairs). Criterion
baselines on the golden fixtures (for future micro-opt cycles): plain 313 µs ·
list 337 µs · table 866 µs · code 1.57 ms · big 3.07 ms.

**Verdict per the preregistered rule (≥1.2× mean, no worse p95): adopt Rust.**
All quality iteration happens in `resiliparse-rs/src/extract/` from now on;
the Cython module is frozen as the reference implementation.

## Insights

- The port was far smoother than the preregistered risks suggested: lexbor
  bindings already expose everything needed (`_noi` variants for inlines), and
  the Cython code's C++ structure translated almost mechanically. Zero
  mismatches on first full run — the up-front quirk inventory (isspace/\v,
  embedimg, rel/role, decode-ignore) is what made that happen; every one of
  them was a real divergence risk identified by reading, not debugging.
- Rust wins ~1.27× despite both being lexbor underneath. The margin is in the
  walk/serialize layer (no C++ shared_ptr per node, no std::string temporaries
  in the same places), not parsing. Bigger wins will need profiling (criterion
  is set up) — e.g. the per-node `get_node_text` allocations in
  `is_link_cluster` look ripe.
- Build ergonomics answer (per user note on subtle blockers): incremental
  `cargo build --release -p resiliparse-extract-rs` is ~2–15 s — cheap enough
  to always measure release; no dev-profile workflow needed.
- macOS wall-clock benchmarking needs the interleaved-in-one-process
  discipline; harness-level best-of-3 was still ±30% where the interleaved
  instrument reproduced to ±0.3%.

## Next

- 0003: failure taxonomy on the worst lpv11 dev docs (playbook §8.3) — now
  iterating on the Rust extractor.
- Markdown output mode (§8.4) — extend the ported `FormattingOpts` with a
  markdown flavor; expect the first big Lev win.
- Merge `upstream/develop` periodically (upstream is actively rewriting in
  Rust; keep the port mergeable/upstreamable).
- Speed cycle candidate: profile `is_link_cluster` text materialization.
