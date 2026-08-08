# 0082 — Forum postbit stat-line strips

- **Date:** 2026-08-08
- **Tag:** 0082-postbit (baseline: 0077-legacy)
- **Status:** landed — zero dev regressions

## What changed
Prefix-family strips in strip_ui_label_lines: "Joined:"/"Join Date:",
"Posts: N" (digit-gated), "Rep Power: N", "Thanks: N", "N posts" —
censused at 0-6% keep across 3,558 emitted lines (Location: excluded
at 13% — content on some pages). Line-strips remain the one action
class whose census matches its effect (0081 lesson holds: the failures
were section/veto actions).

## Results
| target | F1 | Lev | Δ |
|---|---|---|---|
| golden | **0.8473** | **0.7591** | +0.0004/+0.0004 |
| original dev | **0.8287** | **0.7325** | +0.0004/+0.0004, 15 up 0 down |
| train | 0.8150 | 0.7164 | +0.0003/+0.0003, 208 up 17 down |

General plain 1000/1000; goldens pass.
