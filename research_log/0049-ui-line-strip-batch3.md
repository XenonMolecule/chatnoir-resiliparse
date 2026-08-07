# 0049 — UI-line strip batch 3 (vBulletin footer band + WP likes)

- **Date:** 2026-08-07
- **Tag:** 0049-batch3 (baseline: 0048-v2)
- **Status:** landed

## What changed
18 lines: "You cannot vote in polls" (0% keep, the 6th phpBB permission
line), "LikeLiked by N person/people" (WP like widget, 0%), "Rate/Search
this Thread", "Display Modes", and the vBulletin footer band ("You may
not post attachments/edit your posts", "Smilies are On", "[IMG]/BB/HTML
code is On/Off", "Posting Rules", "Thread Tools") at 7–11% keep-rate —
the keeps are 3 recurring form pages (newreply.php) whose gold retains
the whole footer; logged decision, 34:1 doc-improvement ratio.

## Results
dev 0.8120/0.7153 (7 up / 1 mixed); train **0.8028/0.7012**
(+0.0002/+0.0002; 102 up / 6 down, worst −0.016 on the newreply form
family). Goldens pass.
