# 0001 — Cython baseline (quality + speed reference)

- **Date:** 2026-08-06
- **Tag:** v1.0.9-cacde72 (no prior baseline — this IS the reference tag)
- **Status:** landed
- **Machine:** Apple M2 (MacBook Air-class, 8 cores, 24 GB), macOS 15/Darwin 24.6.0,
  Python 3.12.13 (conda env `resiliparse`), single worker for official timing.

## Hypothesis

Stock `extract_plain_text(html, main_content=True)` (Cython, resiliparse 1.0.9)
establishes the quality floor on lpv11 dev and the ms/doc reference for
everything that follows. Preregistered expectations: (a) no markdown structure
while the gold is markdown-flavored → Levenshtein depressed more than F1; (b)
defaults near-optimal among stock knobs; `minimal_html` might beat plain on Lev
because the gold has structural markup. This run's cached predictions double as
the parity oracle for the 0002 Rust port spike.

## What changed

Nothing (measurement-only cycle).

## Results

**PRIMARY — lpv11 dev, raw HTML, stock defaults (`main_content=True`):**

| metric | value |
|---|---|
| ROUGE-L F1 (mean / median) | **0.7619** / 0.8655 |
| ROUGE-L P / R | 0.7902 / 0.8107 |
| Levenshtein similarity | **0.6540** |
| ms/doc mean (p50 / p95), best of 3 | **1.64** (0.97 / 4.75) |
| docs/s (single worker) | ~600 |

Timing noise is real on this machine (identical runs ranged 1.6–8.6 ms/doc
mean; first-ever run pays cold-cache costs). **Methodology going forward:
official ms/doc = best of 3 consecutive single-worker runs; A/B speed claims
from interleaved runs, same session.** Criterion benches will be the precise
instrument if the Rust path wins.

**lpv11 dev, preprocessed variant** (tag `v1.0.9-cacde72-preproc`): F1 0.7080 /
Lev 0.5983 — **5.4 F1 points BELOW raw HTML.** Surprising and worth a diagnosis
cycle: the lpv11 preprocess transform apparently removes signals the
main-content heuristics rely on (or strips content wholesale). Raw is the
production-faithful priority variant anyway.

**Stock knob sweep** (lpv11 dev raw, Δ vs. defaults):

| config | F1 | Lev | verdict |
|---|---|---|---|
| defaults (`preserve_formatting=True, list_bullets=T, alt_texts=T`) | 0.7619 | 0.6540 | **baseline** |
| `--no-alt-texts` | 0.7684 | 0.6627 | best aggregate, but 89 docs regress (worst −0.67); not shipped, queued |
| `--no-alt-texts --no-list-bullets` | 0.7685 | 0.6641 | ditto |
| `--no-list-bullets` | 0.7620 | 0.6555 | Lev +0.0015, noise-level |
| `--preserve-formatting false` | 0.7620 | 0.6515 | formatting helps Lev |
| `--preserve-formatting minimal_html` | 0.7205 | 0.6175 | HTML tags ≠ markdown; worse on both |
| `--links` | 0.6933 | 0.5571 | gold has no link targets |
| `--no-comments` | 0.7409 | 0.6333 | gold keeps comment sections |
| `--no-post-meta` | 0.7504 | 0.6451 | gold keeps post meta |
| `--hidden-elements` | 0.7625 | 0.6538 | noise |
| `--noscript` | 0.7613 | 0.6532 | noise |
| `--form-fields` | 0.7554 | 0.6364 | worse |

**Decision:** stock defaults stay the baseline config. The `no-alt-texts`
aggregate win fails the zero-regression rule (jusText-0079 pattern) and alt
texts are where LaTeX-as-image math transcriptions live — the right capture is
*selective* alt-text filtering (queued), not a blanket drop.

**SECONDARY — old-benchmark guardrails (defaults):**

| dataset/split | F1 | Lev |
|---|---|---|
| general dev (rawhtml) | 0.7983 | 0.7114 |
| general dev2 (rawhtml) | 0.8013 | 0.7152 |
| general dev (preproc) | 0.7955 | 0.7101 |
| math dev (preproc) | 0.7998 | 0.6937 |
| code dev (preproc) | 0.7563 | 0.6708 |
| science dev (preproc) | 0.6390 | 0.6165 |
| table dev (preproc) | 0.6526 | 0.3673 |

(Domain splits are tiny — direction, not decimals. For reference: the tuned
jusText fork reaches 0.886 on general dev; stock resiliparse at 0.798 is a
strong heuristic baseline.)

**Failure-tag distribution, lpv11 dev:** PARTIAL 511 · GOOD 243 ·
OVER_EXTRACT 134 · UNDER_EXTRACT 97 · EMPTY_PRED 13 · ERROR 0. No crashes on
1,000 raw-HTML docs.

## Insights

- The markdown gap is visible exactly as predicted: F1 0.76 vs Lev 0.65, and
  `minimal_html` (structure, wrong syntax) *loses* Lev — the gold wants
  markdown syntax specifically, not just any structure. Markdown output mode
  remains the biggest single lever on the board.
- lpv11 gold **keeps** comments and post meta (dropping either costs 1–2 F1
  points) — opposite of classic boilerplate-removal intuition; do not "clean"
  these away in future cycles.
- Raw HTML beating the preprocessed variant by 5.4 F1 is the most surprising
  result of the cycle; diagnosis queued.
- OVER_EXTRACT (134) outnumbers UNDER_EXTRACT (97): precision (boilerplate
  leaking in) is the bigger main-content problem at baseline.
- Speed timing on this laptop needs the best-of-3/interleaved discipline; raw
  per-run means can differ 2× on identical code.

## Next

- 0002: Rust port spike per playbook §8.2 (parity oracle = this tag's cached
  predictions; gates: feasibility at ~4h timebox, then speed verdict).
- Queued: selective alt-text filtering; preprocessed-variant deficit diagnosis;
  markdown output mode; EMPTY_PRED/UNDER_EXTRACT rescue patterns.
