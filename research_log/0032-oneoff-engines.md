# 0032 — Seven one-off forum engine handlers (agent-built, verified)

- **Date:** 2026-08-07
- **Tag:** 0032-oneoff (baseline compared against: 0030-respond)
- **Status:** landed

## What shipped (per-engine gates, 0017 discipline, agent-built in worktree)
PerlMonks, Nabble (with mail-quote/signature stripping), WebBBS/vegsource,
Motley Fool (both skin generations), CafeMom (12% coverage guard), Slashdot
(single-story gate — a journal-stream −0.90 was caught and fixed pre-ship),
GLP report pages. Deliberately not shipped: Drupal comments (targets already
0.94+), use.perl Slash (gold keeps nav — wall), genealogybuff skin.

## Results (verified on current head)
| set | before | after | per-doc |
|---|---|---|---|
| lpv11 dev | 0.8013 / 0.7010 | **0.8046 / 0.7050** | **7↑ / 0↓**, 993 byte-identical |
| lpv11 train | — | +0.0002 | 9 > +0.05, **worst −0.005** |

Per-doc target gains: perlmonks 0.41→1.00, glp 0.18→0.97, fool 0.41→0.99,
webbbs 0.47→1.00, slashdot 0.63→0.99, nabble 0.53→0.87, cafemom 0.86→1.00.
Goldens pass.

## Insights
- Per-engine gates deliver what generic detection (0029) couldn't: seven
  engines, zero off-target changes. The forum family is now essentially
  solved: twelve engines handled across 0014–0032.
- Agent-built handler batches are now a reliable production pattern (second
  clean delivery; the agent even caught its own −0.90 regression pre-ship).

## Next
- M1 milestone (0.81 F1 / 0.72 Lev) is 0.006/0.015 away — the declared
  --allow-test checkpoint fires when crossed.
- Sponsor-block + author-bio residual families; gold-noise exclusion list.
