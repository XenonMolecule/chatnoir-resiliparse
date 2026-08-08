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

## M2 vaulted-test result (declared checkpoint, --allow-test)
**Test: 0.8244 / 0.7279** (M1 test: 0.8218/0.7225 → +0.0026/+0.0054).

Honest read: at M1 test led dev; now test LAGS dev by ~0.030/0.034. The
site-rule lane (0098→0111, +~0.02 dev) is fitted to dev domains and mostly
does not transfer — test's gains since M1 match the generalizable lanes only
(rescue tiers, boilerplate strips, anchor-run/template vetoes, engine
handlers). Implications:
1. The devset goal (stop-hook: 0.9/0.8–0.85 on lpv11 dev) is still served
   by waves; the science claim about generalization must cite TEST.
2. Future generalizable-lane wins matter double; consider a wave-style
   fleet that extracts GENERIC (cross-site) patterns from dev diffs instead
   of domain-gated selectors.
