# 0144: U+00A0 normalization — golden 0.8915/0.8117, 18 up / 0 down

Found while validating the owner's charter rules: our forum post headers
render `**SC Cy – 12-01-2010,\u{a0}12:30 PM**` while gold has a plain
space. Census: 22 dev docs carry U+00A0 in our output, 39 in gold, but
only **1 doc in both** — the annotator's pipeline collapses nbsp to space
in 21 of our 22 (and the source HTML carries `&nbsp;` in 20 of them, so
this is their normalization, not ours to preserve).

Markdown-mode `normalize_nbsp` pass. Battery: golden **up 18 / down 0**,
dev up 18 / down 0, train **up 159 / down 1** (−0.0002 Lev). Plain config
1000/1000 identical, tests 7/7.

Worth noting how this was found: the owner's ruling review surfaced two
defects directly, and validating his charter rules across the corpus
surfaced a third that no census had caught — because it is invisible at
the token level and only shows up in a character-exact diff of a family
the rulings pointed at.

## Charter-rule census (rules #1/#2 from 0143)
- Duplicated-stub golds: 1 candidate (beckett), and its repetition is
  faithful — the signature block genuinely occurs 6x in source. No edit.
- Golds omitting rendered posts: the 10 flagged docs were header-FORMAT
  mismatches (this nbsp bug), not omissions. jeepforum was the only true
  instance and is already ruled.
Both families are singletons; the rules stand for future gold work.

## Scoreboard
Original dev **0.8569/0.7660** · golden v15 **0.8915/0.8117** · gap
**−0.0085 F1**.
