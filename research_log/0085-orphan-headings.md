# 0085 — Orphan chrome-heading strip

- **Date:** 2026-08-08
- **Tag:** 0085-orphan (baseline: 0084-v2)
- **Status:** landed

## What changed
`strip_orphan_headings` post-pass: censused chrome headings whose
section is EMPTY (next heading/EOF immediately follows) — "No
comments:" (1% kept as orphan, 143 train instances), "0 comments:",
"archives", "personal tools", "search", "pages", "categories". The
orphan gate supplies the precision the bare-line census lacked (the
same headings WITH content stay).

## Results
golden **0.8482/0.7603** (28 up / 3 Lev-only down) · original dev
**0.8288/0.7328** · train 0.8152/0.7167 (229 up / 94 down, mostly
F1-up-Lev-down mixed-sign on Blogspot docs; zero craters, zero
both-down on golden). General plain 1000/1000; goldens pass.
