# 0094 — Per-doc presentation-normalization pilot (the fleet datapoint)

- **Date:** 2026-08-08
- **Status:** pilot shipped into golden v10; fleet decision quantified

## Design
25 highest-(F1−Lev)-gap docs; one agent; declared conventions; HARD
content invariant (alphanumeric token sequence identical — verified
independently, 0 failures). Per-doc judgment resolved exactly the
ambiguities that sank the 0093 blanket regexes (page-rendered <br>
vs not; literal mail-quote '>' vs blockquote).

## Measurement
20 normalized: mean **+0.033 Lev/doc, 19 up / 0 down, F1 delta 0.000**
(the invariant's fingerprint). 5 content-divergence (left alone).

## The honest fleet math
Extrapolated over the 129-doc high-gap pool: **~+0.0034 Lev**; over
all ~500 variance docs perhaps +0.005-0.010 Lev total. Fleet-scale
normalization is REAL but reaches ~0.77, not the 0.80 band. The
remaining Lev gap beyond it is structural (ordering, content-side
divergence) — i.e., the band requires BOTH normalization AND the
mid-band extractor fleet.

Golden v10: **0.8489/0.7631**. Scoreboard: original 0.8288/0.7328.
