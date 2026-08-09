# 0116: site-rules wave 11 — golden 0.8872/0.8068

## What
Eleventh wave on 0115-w1 diffs (356 targets, 8 agents, 456 raw rules →
**+82 SITE_VETOES / +40 SITE_WHITELIST** after filtering — the append rate
is falling as the tables saturate: most verified selectors already exist).

## Bisect
hitvibz.com removed + blocklisted (single `#ag-terms` veto, −0.48 crater:
the "terms" container holds the article body on that template).

## Battery (`0116-v2` vs `0115-w1`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8872** (+0.0011) | **0.8068** (+0.0015) | up 16 / down 4 (1 both) / craters 0 |
| dev | **0.8559** (+0.0011) | **0.7644** (+0.0014) | up 16 / down 6 / craters 0 |
| train | 0.8158 | 0.7174 | up 1 / down 0 |

Guardrails: plain 1000/1000 identical, extract_golden 7/7.

## Scoreboard
Original dev **0.8559/0.7644** · golden v12 **0.8872/0.8068** · goal gap
**−0.0128 F1** (Lev in-band at 0.8068).
