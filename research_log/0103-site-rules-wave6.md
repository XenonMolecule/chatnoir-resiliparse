# 0103 — Site-rules wave 6

- **Date:** 2026-08-09
- **Tag:** 0103-v2 (baseline: 0102-v3)

410 docs → 916 filtered rules (627v/289w) → battery → 3 domains
bisected + blocklisted (greshamlodge image-family veto, covers.com,
wikipedia category rules) → clean.

| target | F1 | Lev | Δ |
|---|---|---|---|
| golden | **0.8704** | **0.7880** | +0.0021 / +0.0025 |
| original dev | **0.8450** | **0.7513** | +0.0015 / +0.0019 |
| train | 0.8154 | 0.7170 | ~0 |

Zero craters; general plain 1000/1000. Six-wave lane:
**+0.0214 F1 / +0.0244 Lev golden**. Goal gap: **−0.0296 F1 /
−0.0120 Lev**. Yield decaying (~half of wave 5) but still strong.
