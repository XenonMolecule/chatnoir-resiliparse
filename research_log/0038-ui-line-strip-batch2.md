# 0038 — UI-line strip batch 2 (train-census, fragment-checked)

- **Date:** 2026-08-07
- **Tag:** 0038-final (= 0038-batch2; baseline: 0037-v4)
- **Status:** landed

## Hypothesis
The train-scale pred-only census (10x dev's evidence) surfaces a second
chrome batch. This time every candidate passes BOTH the exact-keep test
(emitted-line gold keep-rate ≤3%) AND the 0037 fragment test (gold holds
no superstring — the line is not a degenerate version of kept content).

## What changed
13 lines added to `strip_ui_label_lines`: "View Single Post", both Disqus
attribution forms, "Related Posts Plugin for WordPress, Blogger...",
"Share This Page", "Events Calendar", the Stack Exchange chrome trio,
vBulletin "You may not post new threads/replies", "Subscribe to: Post
Comments (Atom)", WP.com "Get every new post delivered to your Inbox.".

**Excluded by the fragment test:** "Anonymous said...:" — gold REWRITES
Blogspot comment bylines as `**Anonymous — Feb 23, 2012 8:42 AM**`
(author + comment timestamp, same shape as our forum-handler convention).
That is a comment-byline rebuild family (~54 train docs), queued as its
own cycle, not a strip.

## Rejected variant
smartologie (train, −0.15) is a photo post whose correct 220B output fell
under the near-empty gate after stripping — the strip-before-gate
tradeoff (0035) hits again. A "base contains full byline → skip rescue"
veto was tried and measured decisively WORSE (−0.63 crater on clarkadams:
identical signal, opposite gold outcome — page-type wall, 8th
confirmation). Reverted; smartologie stands as a logged single-doc
regression against 405 improved train docs.

## Results
| split | F1 | Lev | vs 0037 |
|---|---|---|---|
| dev | **0.8063** | **0.7076** | +0.0002 / +0.0004 |
| train | 0.7968 | 0.6936 | +0.0003 / +0.0003 |

dev: 30 improved / 3 down (Lev-only, F1 up on all three). train:
405 / 83 (16 both-down, worst the logged smartologie). General dev
plain 1000/1000 identical; goldens pass; fences balanced; 1.42 ms/doc.
