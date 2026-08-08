# 0064 — Yahoo message-board handler

- **Date:** 2026-08-08
- **Tag:** 0064-yahoomb (baseline: 0055-ship / 0058-base)
- **Status:** landed — zero regressions on all three targets

## What changed
`extract_yahoo_mb` (one-off engine, 0030 pattern): `.mb-message-body`
blocks, `.mb-author-actual` author, `.mb-timestamp abbr` time,
`.mb-message-bd` body → `**author — time**  \nbody`. Near-byte-exact on
the probe doc (F1 0.07 → ~0.9).

## Results
| target | F1 | Lev | Δ |
|---|---|---|---|
| golden dev | **0.8339** | **0.7458** | +0.0009 / +0.0008 |
| original dev | **0.8232** | **0.7267** | +0.0009 / +0.0007 |
| train | 0.8142 | 0.7155 | 0 / +0.0001 |

1 doc changed per split, all up; general plain identical; goldens pass.
First scoreboard motion since 0055 — the singleton lane works.
