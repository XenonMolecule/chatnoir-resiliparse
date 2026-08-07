# 0056 — NEGATIVE: v6 positional features

- **Date:** 2026-08-07
- **Tags:** 0056-v6/v45k60/v40k65/v35k60 (vs 0055-ship); reverted

## Hypothesis
Block position and distance-to-text-mass-center should separate edge
chrome from central content (boilerplate lives at the edges).

## Result: AUC up, live metrics down — reverted
AUC 0.8592 → **0.8628**, but every live sweep point measured below v5
(best 0.8220/0.7252 vs 0.8223/0.7260). Position correlates with
containment on average, but ANTI-correlates on comment sections — they
sit at the page bottom, far from the text-mass center, exactly where
this gold keeps content. First formal AUC/live divergence of the model
program: held-out containment AUC is no longer a sufficient go-signal;
live dev is the only arbiter for feature families that interact with
comment placement.

## Kept
The v6 emission in `collect_block_features` (training-side only) and
the 68-dim exporter stay — harmless, and positional features may return
gated (e.g. position × has-comment-markers interaction). Model reverted
to v5 (verified byte-identical to 0055-ship).
