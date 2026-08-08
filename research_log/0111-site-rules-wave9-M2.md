# 0111: site-rules wave 9 — golden 0.8850/0.8041; M2 crossed on original dev

## What
Ninth wave on 0110-v3 diffs (381 targets, 8 agents, 530 raw rules →
**+164 SITE_VETOES / +86 SITE_WHITELIST** after the standard filter).

## Bisect (golden/train evidence only, per the 0110 lesson)
Removed + blocklisted: parlinfo.aph.gov.au (−0.23 golden), tv.com (−0.03/
−0.07 golden), dictionary.reference.com (train both-down). topjobs.ch's dev
−0.29 kept: golden-neutral, the 0093 zero-sum family.

## Battery (`0111-v2` vs `0110-v3`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8850** (+0.0030) | **0.8041** (+0.0032) | up 50 / down 16 (1 both, mixed-sign) / craters 0 |
| dev | **0.8540** (+0.0022) | **0.7620** (+0.0026) | up 45 / down 21 / craters 0 |
| train | **0.8156** | **0.7172** | up 7 / down 3 (0 both) |

Guardrails: plain 1000/1000 identical, extract_golden 7/7.

## Milestones
- **M2 (0.85/0.76 on original dev) CROSSED: 0.8540/0.7620.** Vaulted test
  run follows this cycle (--allow-test, declared checkpoint).
- Golden: 0.8850/0.8041 — Lev solidly in the goal band; F1 gap −0.0150.
- Wave-lane session arc: 0.8704/0.7880 → 0.8850/0.8041 (+0.0146/+0.0161)
  across 0104–0111.
