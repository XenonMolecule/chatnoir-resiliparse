# 0167: comment-count heading normalization — flat, reverted

The inverse census (gold lines we never emit) surfaced the biggest single
systematic gap left: **"comments" appears in 51 golds with no counterpart
in our output**. Diagnosis: sources write "23 comments:", "322 Comments :",
"Comments (23)"; gold renders them all as `**Comments**`; we emit the
source form verbatim. The comment BODIES survive — only the heading differs.

Normalizing every count-bearing comments heading to `**Comments**`:
golden F1 0.89465 → 0.89466, Lev 0.81523 → 0.81522, **up 42 / down 25**;
dev slightly negative; train up 514 / down 309.

Pure churn — the fourth measurement of the convention zero-sum law (0093,
0129, 0142, now 0167). Gold is not internally consistent here either: some
golds keep the count form. A rewrite can only trade one population for the
other.

Reverted; parity verified. This closes the inverse-census lane: its top
finding is real but unactionable, and the remaining entries (```java
fences, `______` separators, section headings like Description/Ingredients)
are the same shape — gold-side normalization we cannot predict per doc.
