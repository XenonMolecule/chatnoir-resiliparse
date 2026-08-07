# 0060 — NEGATIVE: model-primary selection

- **Date:** 2026-08-07
- **Tags:** 0060-tau{45,50,55} (vs golden base 0.8330/0.7450); reverted

## Hypothesis
With AUC 0.86 and five straight wins from raising model authority, the
model could replace the rule cascade outright: veto=keep=tau makes every
scored block either vetoed or whitelisted, rules reduced to unscored
crumbs. Zero new code — a threshold identity.

## Result
tau=0.45: 0.8228/0.7334 · tau=0.50: 0.8244/0.7367 · tau=0.55:
0.8265/0.7411 — every point below the hybrid (0.8330/0.7450), curve
still rising into the region 0058 already showed fails on the veto side.
**The rule cascade adds irreplaceable value inside the model's
0.40–0.60 uncertainty band.** The architecture is settled: model at the
extremes, rules in the middle, handlers first, rescues beneath.

## Implication for the 0.90 goal
Neither more model authority (0058/0060) nor more capacity/features at
this data (0053-0056 diminishing, positional negative) moves the
needle further. The next real gains need: (a) a materially better
model (new label source, e.g. training on GOLDEN-cleaned containment),
(b) new content veins (learned image selection), or (c) test-time
approaches beyond this architecture.
