# 0145: zero-width/BOM strip — golden 0.8915/0.8117 (17 up / 4 down)

Corpus-wide character-class census (ours vs gold, per-doc counts) after the
nbsp find. Clear over-emissions: U+FEFF in 13 of our docs and **zero**
golds; also U+200B/200E/200F. Stripped in the markdown normalization pass.

Battery: golden up 17 / down 4 (max −0.0005 Lev, F1 all flat), train up
147 / down 23. Means unchanged at 5dp. Shipped as objective noise removal —
these characters are never rendered and never appear in gold.

Census also flagged, NOT actioned:
- U+FFFD (868x in 38 docs vs 30x in 3 golds): charset misdecode. Cannot be
  repaired by substitution — gold holds the correct character, so both
  stripping and keeping lose. Needs real charset detection (parser-level).
- '»' (47 docs vs 11) and '·' (14 vs 7): chrome separators; the 0115 rule
  only fires at >=2 separators per line. Single-separator lines are mixed
  content/chrome — deferred pending evidence.
- U+2013/2014 en/em-dash deficits (gold 368/300 docs vs ours 207/195): we
  under-emit, i.e. a RECALL artifact of missing content, not a char bug.
