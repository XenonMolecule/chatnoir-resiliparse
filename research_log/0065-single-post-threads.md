# 0065 — Single-post thread relaxation + golden v2

- **Date:** 2026-08-08
- **Tag:** 0065-singlepost (baselines: 0064 / golden-v2)
- **Status:** landed

## Golden v2 (owner review round 1)
Owner labeled 75 specs (62 agree / 9 disagree / 4 unsure); review of all
13 contested: 10 specs dropped (chrome-shaped content: hours/nearby/docs
-index/class-grids; plus an unsound spec with bare-word patterns), 2
narrowed (legal line restored per charter C4; travel links kept).
Golden v2: 0.8348/0.7469. Owner-flavor re-review of the unlabeled ~290
specs running as background agents (four rejection flavors codified).

## Extractor: single-dated-post threads
phpBB2 + subSilver2 gates required >=2 posts; single-post threads
(robinsnest, nwnprod — ledger triage family #1) fell to the generic
walk. Now >=1 WITH a parsed date qualifies (zip pairing intact — the
0050 doc-order anti-pattern is not involved); dated single posts exempt
from the coverage guard; phpBB2 postdetails date lookup now filters to
"Posted:"-bearing spans (profile blocks were shadowing dates at the
index heuristic).

## Results
| target | F1 | Lev | Δ vs prior |
|---|---|---|---|
| golden dev | **0.8363** | **0.7485** | +0.0015 / +0.0016 |
| original dev | **0.8248** | **0.7284** | +0.0016 / +0.0017 |
| train | 0.8144 | 0.7157 | +0.0002 / +0.0002 (8 up / 1 down −0.04) |

General plain 1000/1000 identical; goldens pass.
