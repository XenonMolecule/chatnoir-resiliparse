# 0080 — Ceiling analysis on golden v6

- **Date:** 2026-08-08
- **Status:** analysis (the strategic map for the 0.90/0.80 goal)

## Loss decomposition (153.1 F1-milli total at 0.8469)
| bucket | F1 mass | nature |
|---|---|---|
| 683 docs already >=0.85 | 35.0 (Lev 77.1) | convention variance (0079: per-doc, unlearnable) — largely IRREDUCIBLE |
| 242 docs 0.5-0.85 | 67.1 | mixed over/under-extraction — the real battleground |
| 81 docs <0.5 | ~41 | walls, client-rendered, bespoke singletons |
| client-rendered/unfixable | 3.6 | needs JS layer |
| listing/tag walls | 9.4 | page-type ambiguity |

## Reading
- The Lev target (0.80) is dominated by the >=0.85 bucket's 77 Lev-milli
  of convention variance — mostly irreducible per 0079. Realistic Lev
  ceiling against this gold: ~0.79-0.81. THE LEV GOAL SITS AT THE
  CEILING; reaching it requires near-perfection on everything else.
- The F1 target (0.90) needs 53 of the 67 mid-band millis plus all
  walls — a rough static-extraction ceiling lands ~0.90-0.93, so the
  F1 goal is at/inside the ceiling but only via the mid-band.
- Mid-band profile: half low-precision (chrome kept on niche layouts),
  half low-recall (content vetoed on niche layouts) — 242 docs of
  medium-size selection errors, each individually diagnosable. This is
  neither walls nor singletons: it is a VOLUME of small selection work
  (est. 100+ cycles by hand) — or the strongest possible justification
  for the fleet-scale approaches the owner has gated.

## Recommendation (standing)
Mid-band volume is agent-fleet-shaped work: per-doc diagnose+fix at
~25 docs/agent with verified batteries. Await owner decision; grind
lane continues meanwhile.
