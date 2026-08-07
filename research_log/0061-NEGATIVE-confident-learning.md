# 0061 — NEGATIVE/NEUTRAL: confident-learning label cleaning

- **Date:** 2026-08-07
- **Tags:** 0061-conf/c35k65/c45k55; reverted to v5

## Hypothesis
big_train labels carry the same ~18% gold noise the fleet documented on
dev; dropping training rows where the v5 model and label disagree by
>0.6 (12–13% of rows) removes label noise and yields a better model.

## Result
Best re-swept point ties the baseline F1 exactly (0.8330) with Lev
−0.0008. The GBM was already averaging over the label noise; removing
the disagreement tail changes calibration, not knowledge.

## MODEL PROGRAM CLOSED (at this feature/label regime)
Ledger: capacity (n60→n120: +0.0008 then plateau) · data (0.9M→3M:
plateau) · features (page-type v4/v5: +0.0030 total; positional v6:
AUC-up/live-down) · authority (0046/0051/0052: +0.0100, the big wins)
· architecture (model-primary: bounded below hybrid) · labels
(confident-learning: neutral). The hybrid at v5/0.40/0.60/floor-40 is
the settled optimum of this program. Materially better selection needs
a different signal class (rendering, layout geometry, cross-page
priors, or inference-time LLM assistance).
