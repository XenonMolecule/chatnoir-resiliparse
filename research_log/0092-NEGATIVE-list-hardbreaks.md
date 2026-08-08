# 0092 — NEGATIVE (census-only): list-item hard-break convention

- **Date:** 2026-08-08

Golden list items are 46% hard-break-terminated (we emit 4%) — a large
Lev residue. Per-doc it IS consistent (83 all-hb / 74 all-bare / 22
mixed) but NO source signal separates the camps: li>br presence splits
15/68 vs 15/59 (identical rates), position (last vs mid) is 49%/38%.
Pure per-doc annotator variance — wall #12 family, now with its largest
member measured. Emitting hb on all items would win 83 docs and lose
74; near-zero expected, high churn. Not built.

This measurement likely accounts for a large share of the >=0.85
bucket's 77 Lev-milli residue identified in the 0080 ceiling analysis.
