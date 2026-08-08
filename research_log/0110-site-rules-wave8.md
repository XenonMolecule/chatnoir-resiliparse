# 0110: site-rules wave 8 — golden 0.8820/0.8009, Lev enters the goal band

## What
Eighth wave, first on golden v12 + the 0109 page_domain fallback (rules now
reach og:url-less pages). 409 targets, 8 agents on fresh 0109-w3 diffs.
Raw 485 rules → filtered **+214 SITE_VETOES, +104 SITE_WHITELIST**.

## Bisect — and an over-bisect lesson
First battery: golden +0.0031/+0.0040, ZERO craters (the wave discipline +
cleaner gold pays off). Initial bisect removed six domains flagged from DEV
losses — wrong basis: stampedia's rules were net-positive on golden (its dev
dip is the 0093 zero-sum family) and removal made it WORSE (−0.13). Restored
all but the true losers:
- **ethicalhacker.net** (down on all three splits) — rows removed, blocklisted.
- **cafemom.com** (pure train loss) — rows removed, blocklisted.
**Bisect on the golden/train evidence, never on dev-only dips.**

## Battery (`0110-v3` vs `0109-w3`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8820** (+0.0031) | **0.8009** (+0.0040) | up 70 / down 12 (1 both, all <0.03) / craters 0 |
| dev | **0.8518** (+0.0024) | **0.7594** (+0.0031) | up 63 / down 21 / craters 0 |
| train | **0.8156** | **0.7172** | up 7 / down 2 / craters 0 |

Guardrails: plain 1000/1000 identical, extract_golden 7/7.

## Milestones
- **Golden Lev 0.8009 — inside the 0.80–0.85 goal band.** Remaining goal gap
  is F1 only: −0.0180 (golden basis).
- Original-dev M2 (0.85/0.76): F1 0.8518 ✓, Lev 0.7594 — 0.0006 short.
  Declare M2 and run the vaulted test at the next Lev uptick.
