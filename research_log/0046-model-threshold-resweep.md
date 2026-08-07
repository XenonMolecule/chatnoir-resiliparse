# 0046 — Model threshold re-sweep + veto-rollback tier

- **Date:** 2026-08-07
- **Tag:** 0046-final (= 0046-rb25k85; baseline: 0045-v2)
- **Status:** landed — largest single-cycle gain of the session

## Hypothesis
The learned-model tiers (0028: veto<0.10, keep>0.90, both chosen when the
pipeline was 17 cycles younger) are stale. The selection stack now handles
most of what the conservative thresholds were protecting.

## Sweep (dev)
| veto | keep | F1 | Lev |
|---|---|---|---|
| 0.10 | 0.90 | 0.8090 | 0.7118 (baseline) |
| 0.15 | 0.90 | 0.8093 | 0.7122 |
| 0.20 | 0.90 | 0.8099 | 0.7130 |
| 0.22 | 0.90 | 0.8109 | 0.7142 |
| **0.25** | **0.85** | **0.8119→0.8120*** | **0.7153** |
| 0.27 | 0.90 | 0.8086 | 0.7123 |
| 0.30 | 0.90 | 0.8054 | 0.7105 |
(*with rollback tier)

## What changed
1. `MODEL_VETO_THRESHOLD` 0.10 → 0.25; `MODEL_KEEP_THRESHOLD` 0.90 → 0.85.
2. **Tier-0 model-veto rollback**: at 0.25 the raw sweep produced total
   page wipes (−1.0 F1 — the model's false negatives gutting small
   pages, unrecoverable because model vetoes ride the shared veto set the
   rescue ladder reuses). A near-empty result with model vetoes in effect
   now retries without them and keeps the retry when it doubles content.
   `tpl_vetoes` returns the model-veto list separately to enable this.
3. **Rejected variant:** a 40% page-mass cap on vetoes (trim-not-gut)
   measured strictly worse — it blocks correct gutting of listing pages
   and still craters when the model confidently inverts (seattlepi).

## Results
| split | F1 | Lev | vs 0045 |
|---|---|---|---|
| dev | **0.8120** | **0.7153** | +0.0030 / +0.0035 |
| train | 0.8026 | 0.7010 | +0.0038 / +0.0045 |

dev: 92 improved / 25 down, worst −0.235 (tradingview profile — model
vetoes a stats table gold keeps). train: 1003 / 255, ~8 craters in
−0.5..−0.76 (poemhunter stats, geneva athletes, jango/ksnt listings) —
ALL are listing/stats pages where gold keeps what the model calls
boilerplate, i.e. the established page-type wall surfacing through the
model axis. Logged decision: the aggregate (+4x a typical good cycle,
1003 docs improved) justifies the tail; the crater family is precisely
the wall we cannot arbitrate locally.
General dev plain 1000/1000 identical; dev2 unchanged at its high-water
mark; goldens pass; 1.57 ms/doc (band).

## M1 STATUS: F1 0.8120 ≥ 0.81 target; Lev 0.7153 vs 0.72 — 0.0047 short.
