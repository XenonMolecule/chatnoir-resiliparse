# 0048 — Post-pass ordering + handler-exit coverage

- **Date:** 2026-08-07
- **Tag:** 0048-v2 (baseline: 0046-final)
- **Status:** landed (hygiene; strictly positive)

## What changed
1. **Ordering bug:** plain-rendered table cells join with `\t`, and
   `normalize_tabs` ran AFTER `strip_ui_label_lines` — so chrome lines
   like `Author\tMessage` (phpBB2 header cells) evaded the exact-line
   strip and shipped as "Author Message" (49 train docs). Tabs now
   normalize first. Found by noticing "Who is online" stripped while
   "Author Message" survived in the same doc.
2. **Coverage hole:** engine-handler early returns and appended rebuild
   blocks skipped the markdown post-passes entirely; all 10 handler
   exits and the block append now route through `md_post_passes`
   (measured zero-diff today, but closes the class).
3. (Committed earlier as 0047b: model-placeholder guard test.)

## Results
dev 0.8120/0.7153, train 0.8026/0.7010 — aggregates unchanged at 4dp;
6 dev + 52 train docs strictly improved, zero both-down. Goldens pass.
