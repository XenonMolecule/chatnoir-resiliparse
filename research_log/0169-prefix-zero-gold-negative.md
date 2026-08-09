# 0169: zero-gold at PREFIX level — NEGATIVE, and why the method breaks

Extended the (twice-successful) zero-gold test from exact lines to 3-word
prefixes: 9 families we emit in >=3 docs whose opening never opens any gold
line. Top: "posted on by" (15 docs). Copyright prefixes were excluded by
hand — the owner ruled copyright is content.

Result: golden F1 flat, Lev 0.81528 → 0.81523, **10 both-downs**; dev and
train both slightly negative. Reverted, parity verified.

## Why the extension fails where the exact test succeeds
"Posted on by" is a DEGENERATE rendering of gold's "Posted on March 3 by
Jane" — author and date dropped. The prefix is absent from gold, but its
TOKENS are present in gold's fuller line, so ROUGE-L already credits them.
Deleting the line destroys real token matches.

The exact-line test is safe precisely because an exact line either matches
gold or contributes nothing. A prefix can be a TRUNCATION of a gold line,
which is the common failure mode of an extractor and exactly what we
should not delete. **Zero-gold evidence licenses deletion only at the
granularity it was measured.**

This closes the lexicon lane: exact-line (0165/0166/0168, all shipped),
block-level (empty), inverse (0167, unactionable), prefix (negative).
