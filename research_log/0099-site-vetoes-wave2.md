# 0099 — Site-veto wave 2: 271 rules

- **Date:** 2026-08-08
- **Tag:** 0099-v2 (baseline: 0098-v2)
- **Status:** landed — largest single extractor gain of the ENTIRE journey

## What changed
Wave-2 extraction (3 agents, 67 docs, multi-selector per multi-region
doc) → 275 verified-chrome selectors filtered → domain-gated table
extended to ~296 rules → battery → 4 offender rules bisected out
(wheretoget widget its gold keeps; ljworld). Zero craters, zero golden
both-down.

## Results
| target | F1 | Lev | Δ |
|---|---|---|---|
| golden | **0.8562** | **0.7727** | **+0.0055 / +0.0068** |
| original dev | **0.8345** | **0.7400** | +0.0046 / +0.0057 |
| train | 0.8153 | 0.7168 | +0.0000/+0.0000 (dev-domain rules) |

General plain 1000/1000; goldens pass. Golden Lev crosses 0.77;
original dev crosses M1's Lev at last (0.7400 vs 0.72 — and F1 0.8345).

## Lane status
Two waves: +0.0072 F1 / +0.0091 Lev golden combined. The remaining
sub-0.5 pool and train-side chrome docs can feed further waves; per-doc
yields will decay but the mechanism is now the single most productive
lane in the project.
