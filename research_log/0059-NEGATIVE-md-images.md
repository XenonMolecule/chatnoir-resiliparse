# 0059 — NEGATIVE: markdown image emission (second attempt, golden target)

- **Date:** 2026-08-07
- **Tags:** 0059-images (vs 0058-base golden / 0055-ship original); reverted

## Hypothesis
The 0011 image revert was measured against noisy gold and predates the
selection stack; the golden devset keeps 426 `![alt](src)` images across
164 docs, so emission gated by the walk + a src blocklist should pay.

## Result: strongly negative on BOTH targets
golden −0.0181 F1 / −0.0270 Lev; 504 docs regressed. Root measurement
(the real contribution of this cycle): within image-bearing golden docs
the walk sees 4,955 images and gold keeps 433 (9%); best single
discriminators — figure-parent 49% precision @ 6% recall, wp-image 43%,
wide>=300 42% @ 29%. No gate combination approaches break-even when a
wrong image costs 50–100 Lev chars.

## Disposition
Image selection requires a learned per-image scorer (dims, DOM position
relative to article start, sibling text, host patterns) — queued as a
model-axis item, not a rule. Family stays blocked for rules FOREVER;
this is the second and final rule-based attempt.
