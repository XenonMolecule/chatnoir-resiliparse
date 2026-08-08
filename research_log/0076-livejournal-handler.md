# 0076 — LiveJournal single-post handler

- **Date:** 2026-08-08
- **Tag:** 0076-lj (baseline: 0075-v3)
- **Status:** landed

## What changed
`extract_livejournal`: `.b-singlepost` pages → `**username — date**`
(data-ljuser scoped to the author block — the page-first ljuser is the
community), display name, `**Tags:**`, `---`, `# title`, body.
Probe doc 0.19 → **0.982/0.979**.

## Results
golden **0.8462/0.7580** (+0.0008/+0.0008, 1 up 0 down). original dev
0.8283/0.7322 (the probe +0.23 F1 with −0.006 Lev vs original gold's
different tag order — golden-primary). train no LJ singlepost docs.
General plain 1000/1000; goldens pass.
