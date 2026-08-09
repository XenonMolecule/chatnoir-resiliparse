# 0131: inline font-weight census — NEGATIVE, lane closed without code

Last computable slice of the formatting-from-style lane (0130): elements
with INLINE `font-weight:bold/600-900` (no CSS cascade needed). Census
across all 1000 golden docs: 62 such texts appear in gold across 26 docs,
but gold renders only **7 (11%)** of them bold. The annotator's bolding is
not even faithful to inline styles — it reflects the full computed cascade
plus tool-version variance. Emitting bold on inline style would be wrong
9 times in 10. No code written; formatting-from-style is closed at every
computable granularity (in-page classes 0105, majority convention 0129,
source-element mapping 0130, inline styles 0131).
