# 0058 — Golden-target recalibration checks

- **Date:** 2026-08-07
- **Tags:** 0058-* (baseline: 0055-ship scored on dev_golden = 0058-base)
- **Status:** two bounded results, no ship

## 1. Threshold re-sweep on golden: no move
Hypothesis: noisy gold penalized chrome vetoing, so golden should favor
more aggression. FALSE — veto 0.45/0.50 measure worse on golden too;
0.40/0.65 vs 0.40/0.60 split conflictingly (+0.0004 golden,
−0.0028 original) at noise level. The 0.40/0.60 point is robust across
both targets. The model's chrome removal was never being punished by
the noise; the noise sat in docs the model already couldn't win.

## 2. NEGATIVE: empty-base unfiltered fallback
The gascu.org total-veto case (0B output, rescue killed by its own
model vetoes) suggested dropping model vetoes from the tier-1 fallback
when base content < 50B. Measured: −0.0025/−0.0031 golden with 11
craters (−0.55..−0.66) — tag/listing pages where the empty base IS the
right answer and the restored shell is pure junk. One doc gained.
Reverted. Third confirmation that rescue-vs-veto arbitration on
empty-ish pages needs page-type awareness beyond local signals; gascu
stays on the deficiency list as an acknowledged single-doc loss.

## Standing scoreboard
original dev 0.8223/0.7260 · golden dev 0.8330/0.7450 (0055 build).
Extractor-deficiency ledger from the fleet: 46 clean-gold docs at
F1<0.6 holding 0.029 golden F1 — the highest-value target pool for
upcoming cycles.
