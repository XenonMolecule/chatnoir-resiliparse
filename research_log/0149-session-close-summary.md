# 0149: session close — arc, walls, and the shortest path to 0.90

## Arc (cycles 0104–0148)
| basis | start (0104) | end (0148) | delta |
|---|---|---|---|
| golden dev | 0.8704 / 0.7880 (v11) | **0.8915 / 0.8117** (v15) | +0.0211 / +0.0237 |
| original dev | 0.8450 / 0.7513 | 0.8569 / 0.7660 | +0.0119 / +0.0147 |
| vaulted test | 0.8218 / 0.7225 (M1) | 0.8244 / 0.7279 (M2) | +0.0026 / +0.0054 |

**Lev goal met** (0.80–0.85 band, reached at 0110). F1 gap: **−0.0085**.
Runtime steady at 2.76 ms/doc markdown, 1.33 plain.

## What produced the gains
1. Domain-gated site rules, waves 7–15 (~2,300 rows): +~0.010 golden F1
   before saturating at +0.0001/wave. Dev-local — does NOT transfer (M2
   test showed test lagging dev by 0.03 after this lane).
2. Generic line-level lexicon (0113/0114/0115/0119/0125/0133/0141/0147/
   0148): the transfer lane — train up-counts of 80–185 per cycle.
3. Gold-noise batches 1–4 + owner rulings (0108/0127/0134/0143): +~0.006
   golden. The owner's three rulings alone moved menstennisforums
   0.047→0.948.
4. Structural fixes: rooted near-empty rescue (0106), page_domain
   majority-host fallback (0109, +0.0043), anchor-run veto (0105).
5. Character normalization (0144/0145/0147/0148): nbsp, BOM/zero-width,
   separator lines. Found only via character-exact diffing after the owner
   review pointed at the family.

## Walls, each with a measurement (not a guess)
| wall | evidence |
|---|---|
| computed-CSS formatting | bold-vs-heading maps to source tag but bolding comes from external CSS (0130); inline font-weight is honored by gold only 11% (0131) |
| per-anchor link selection | 614k-anchor GBM, AUC 0.865, precision 0.70 only at 2.5% recall (0137) |
| image emission | 3.4% gold keep-rate (0011), 13% for wp-image (0113) |
| convention unification | zero-sum at three granularities (0093, 0129) |
| gold mirrors source fusion | emitter "hygiene" is anti-progress (0124) |
| upstream charset corruption | all 38 U+FFFD docs corrupt in stored HTML (0147) |
| JS-rendered content | unreachable by static extraction |
| deep-tail containers | no clean selectors remain (0132, 0139) |

## Shortest path to 0.90 from here
1. **Train-gold authorization** (owner-gated). The four charter rules from
   the 0143 rulings would clean train labels; the block model retrains on
   them. Highest expected value of anything remaining.
2. **Owner spot-check pass** over the ~300 unlabeled docs. Three of the
   last four shipped fixes came from ~15 minutes of owner review, after
   13 automated waves had missed them.
3. Engine handlers for the ~20 unmatched forum templates (0148 confirmed
   the en-dash deficit is recall from missing post headers, not a
   character bug) — real work, bounded, autonomous.
4. Research-scale: computed-style formatting head, JS rendering.

## Protocol changes worth keeping
- **Checkpointed fleets**: per-doc JSONL appends survived 3 crashes
  (2 session limits, 1 network switch), ~1.5M tokens of rework avoided.
- **Bisect against HEAD**, never regex-nuke a domain (0104, 0132).
- **Per-doc rescore gate on every gold edit** (0134 caught a census verdict
  that would have cost −0.535 on its doc).
- **Battery is the arbiter**: 8 measured negatives this session, several
  of which looked airtight beforehand (» strip, dedupe, bold labels).
