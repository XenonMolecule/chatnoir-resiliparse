# 0151: corpus-wide source-absence dedupe — golden v16 = 0.8917/0.8120

First systematic sweep of an APPROVED edit family across all 1000 golds
(previous batches only touched taxonomy-surfaced docs). Criterion: a
>=100-char gold block repeated 2+ times whose text occurs ONCE in the
tag-stripped source. Forum threads excluded up front — quoted posts repeat
legitimately (0113 negative).

11 candidates → 4 passed the per-doc rescore gate → **2 shipped** after
per-doc verification:
- **devilslakejournal** (GateHouse page-2 duplication, 2,579B): 0.738 →
  **0.974**
- **art-harun.blogspot** (315B): 0.992 → 0.997
- oracle blogs and TWiki were REVERTED: our extractor also emits those
  duplicates, which is independent DOM evidence the page really repeats
  them (my flattened-text probe missed it). Rejecting on that signal keeps
  the gate non-circular — the extractor reads the DOM, not the gold.
- guidinglighths / pragmandt skipped by the gate.

Golden basis v15 → **v16: 0.89173 / 0.81203** (+0.0002/+0.0003), 0 down docs.
Extractor untouched.
