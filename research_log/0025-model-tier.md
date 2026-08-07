# 0025 — Learned block model in the loop (first model tier)

- **Date:** 2026-08-07
- **Tag:** 0025-final (baseline compared against: 0024-dl)
- **Status:** landed — first model-in-the-loop cycle

## What shipped
- `block_model.rs`: 60-tree depth-5 GBM (AUC 0.837 held-out) exported as
  generated Rust (7.7K lines, zero runtime deps), trained on 1.15M blocks
  from lpv11 train with features computed by the SAME bottom-up scan used at
  inference (parity by construction).
- Two tiers on classifier decision points ≥150 bytes, fused into the single
  tpl scan: **veto** (score < 0.10 → skip set) and **whitelist** (score >
  0.90 → overrides template vetoes and the rule classifier in the walk).
  Whitelist is the first mechanism that *returns* content the four measured
  walls were dropping.

## Results
| set | Δ F1 | Δ Lev | per-doc |
|---|---|---|---|
| lpv11 dev | **+0.0015** | **+0.0016** | **7↑ / 0↓** (150B floor removed all small-block noise) |
| lpv11 train | +0.0009 | +0.0009 | 26 > +0.1 vs 7 < −0.1, 1 < −0.3 |
| general dev | 0 | 0 | untouched |

Goldens pass. **Speed (logged decision):** model tier costs ~10% relative in
markdown config (ratio vs frozen Cython 1.05× → 0.95×); plain-text default
path unchanged at 1.19×. Optimizations applied: fused single scan, 150-byte
scoring floor, RegexSet feature bits; further profiling queued.

## Insights
- The corrected shared-scan features recalibrated the tiers: the veto side
  collapsed to 0.1% coverage (rules already take confident junk) — the
  model's real value is the WHITELIST, exactly as the four walls predicted.
- Dev per-doc 7↑/0↓ with a 150-byte floor vs 34↑/12↓ without: acting only on
  substantial blocks converts a noisy win into a clean one.
- Ceiling: this v1 model uses 22 features and block-local signals. Headroom:
  page-type features, big_train (100k docs), calibrated thresholds per tier,
  and using scores INSIDE rules (soft evidence) rather than binary tiers.

## Next
- Model v2 on big_train + page-type features; threshold sweeps per tier.
- Criterion profile of the scan+scoring path.
