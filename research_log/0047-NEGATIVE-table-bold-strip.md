# 0047 — NEGATIVE: bold-strip inside pipe rows

- **Date:** 2026-08-07
- **Tags:** 0047-tablebold (vs 0046-final)

## Hypothesis
Gold renders pipe-row cells plain in 97.2% of rows (576/20248 bolded);
we emit `| **Player** |` on th-with-`<b>` skins — stripping `**` at
TR-end should be a mechanical Lev win.

## Result: flat-to-negative, reverted
dev 0.8120/0.7152 (Lev −0.0001); train 0.8026/0.7011 (+0.0001 Lev) with
a −0.24 F1 crater (conjuguer-verbe.eu): removing `**` bytes shrank
`result.len()` and flipped the RAW arm of the dual rescue keep-test —
the 0036 instability class strikes through the other arm. The census
mass was concentrated in chrome-table docs where gold drops or reshapes
the whole table; cell-level bold was not the binding constraint.

## Lessons
- A 97/3 gold convention does not imply aggregate gain — check WHERE the
  nonconforming mass lives before building (here: in docs already lost
  to table-selection differences).
- Any change that removes bytes from small outputs must be evaluated
  against BOTH arms of the rescue keep-test. The dual-OR test is stable
  against additions, not removals.
