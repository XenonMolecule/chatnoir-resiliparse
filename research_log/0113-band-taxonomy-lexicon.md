# 0113: mid-band taxonomy + nav-lexicon strips — golden 0.8858/0.8049

## Band taxonomy (F1 0.5-0.7, 67 docs — the next tail after 0105's <0.5 pass)
6 classifiers + 1 synthesizer: **over_chrome 32 · under_missed 22 ·
engine_missing 7 · gold_noise 3 · other 3.** Synthesized pattern ranking
(doc_count/risk): link+media serialization 13/low, adjacent-dup dedupe
12/low, nav lexicon 9/low, repeated-line families 9/low, markdown shape
9/low, link-density rescues 11/med, trailing boilerplate 12/med, hidden
widget veto 9/med, teaser rails 8/med, forum vocab 6/low. Full report:
`research_log/0113-band57-taxonomy-full.json`.

## Shipped (line-level lexicon, the safe action class)
- LABELS: Email/Print this page, skip-nav variants, "Back to Article",
  "Save | Post a comment |".
- `RENDER_TIMER` regex strip: "generated in 0.010506 seconds" footers.

## Two negative results (census≠action instances #5 and #6)
1. **Verbatim block dedupe** (≥200-char normalized blocks, emit-once):
   global version cratered forums broadly (−0.13..−0.23 Lev train) — gold
   KEEPS "Originally Posted by" quote blocks that verbatim-repeat earlier
   posts. Near-adjacent-only version still lost (fordcontour: gold keeps a
   quote of the immediately preceding post; blogspot repeated paragraphs).
   Duplication is not chrome. Reverted fully.
2. **Image `![alt](src)` emission** re-checked per taxonomy suggestion:
   wp-image-* content images are kept by gold only 13% (59/454) — the 0011
   verdict stands; needs learned per-image selection, not a rule.

## Battery (`0113-w3` vs `0112-v2`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8858** | **0.8049** | up 4 / down 0 |
| dev | 0.8545 | 0.7627 | up 3 / down 1 (rosinstrument, golden-primary zero-sum) |
| train | 0.8156 | 0.7172 | up 42 / down 6 (all ≤0.02, Email/Print-page family original-gold conflicts) |

Guardrails: plain 1000/1000 identical, extract_golden 7/7.

## Scoreboard
Original dev **0.8545/0.7627** · golden v12 **0.8858/0.8049** · goal gap
−0.0142 F1 (Lev in-band).
