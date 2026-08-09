# 0123: site-rules wave 13 — golden 0.8880/0.8074; lane at true saturation

## What
Checkpointed fleet (10x20 docs, per-doc appends, all 200 done). 414 raw
rules → only **+44 vetoes / +22 whitelists** survived filtering — table
saturation is real: most verified selectors already exist from waves 1-12.

## Bisect
Removed + blocklisted: failblog.cheezburger.com (−0.76 golden crater — the
"tag page" whitelist grabbed the whole listing), cheatmasters.com,
query.nytimes.com (mixed-negative on dev/golden).

## Battery (`0123-v2` vs `0122-w1`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8880** (+0.0001) | **0.8074** (±0) | up 8 / down 2 (0 both) / craters 0 |
| dev | 0.8557 | 0.7644 | up 7 / down 4 |
| train | 0.8159 | 0.7175 | unchanged |

Guardrails: plain 1000/1000 identical, extract_golden 7/7.

## Lane verdict
Wave yields: w10 +0.0007 → w11 +0.0015 → w12 +0.0006 → w13 **+0.0001**.
The site-rule lane is DONE at this target density — ~1.2M tokens for
+0.0001 F1 is not a trade to repeat. Remaining −0.0120 F1 gap requires:
(1) owner gold rulings (0108 questions, ~+0.0015), (2) structural/emitter
work from the band taxonomy (markdown-shape fixes, inline-anchor sentence
flow), (3) the model lane with richer features. No further waves without a
material extractor change to refresh the diffs.
