# 0101 — Domain-gated content whitelist (wave 4, recall side)

- **Date:** 2026-08-09
- **Tag:** 0101-v3 (baseline: 0100-v2)
- **Status:** landed

## The recall mirror of 0098
SITE_WHITELIST: (domain, selector) content containers our rules drop,
forced kept via the model-whitelist path (node + full subtree). Same
zero-cross-site construction. 131 rules → wiring bug found (the
tpl_vetoes return overwrote insertions — fixed to extend) → battery →
2 domains bisected. Zero craters, zero golden both-down.

| target | F1 | Lev | Δ |
|---|---|---|---|
| golden | **0.8647** | **0.7812** | +0.0037 / +0.0037 |
| original dev | **0.8399** | **0.7453** | +0.0036 / +0.0034 |
| train | 0.8153 | 0.7169 | +0/+0 |

General plain 1000/1000; goldens pass. **Golden crosses 0.86 F1 and
0.78 Lev.** Four-wave lane total: +0.0157 F1 / +0.0176 Lev golden.
Goal gap: −0.035 F1 / −0.019 Lev.
