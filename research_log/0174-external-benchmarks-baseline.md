# 0174: five external benchmarks — first baseline of OUR extractor

Runner: `benchmark/external/run_external.py` + `run_unit_tests_ours.py`,
scoring with small-rephraser's own metrics (token_f1 4-gram shingle;
trafilatura with/without containment; unit-test normalize). Scorer
validated: reproduces their stored upstream-resiliparse zyte score
(0.8806 vs stored 0.8796).

| benchmark | ours | reference points | target ≥0.8 |
|---|---|---|---|
| **marin devset** (420 gold, PRIORITY) | **F1 0.9046** (P .867/R .946), Lev 0.838 | — | ✅ |
| zyte (181) | **F1 0.8899** | upstream resiliparse 0.8796 · marin-d5 0.8752 · dripper 0.9453 · trafilatura 0.9508 | ✅ |
| WebMainBench (en/dev 200) | **F1 0.8633**, Lev 0.792 | — | ✅ |
| trafilatura evaldata (960) | **F1 0.8421** (P .766/R .935, acc .825) | — | ✅ |
| unit tests (100 devset-scored of 108) | **94/100** | upstream resiliparse 98/108 · marin v1.1 96/108 · trafilatura 78/108 | ✅ |

Unit-test detail: **code 13/13, math 10/10, table 7/7, attribution 12/12,
structure 5/5** — perfect on every content category. Failures: 4 negatives
(chrome kept: slide counters, 6ya upsell, Korean footer, "Brought to you
by") + 2 prose keeps. 8 tests skipped (page:/wmb docs outside the devset).

Reading: our fork beats upstream resiliparse on zyte (+0.010) and is above
the 0.8 goal on all five. The recurring shape everywhere: recall high
(0.93-0.99), precision the loss — same chrome families the domain audits
named. Trafilatura/dripper lead zyte on precision (they cut harder; the
audits showed gold-side abridgement also inflates their advantage).

Next levers (precision, no recall risk): Q&A/forum furniture (code-audit
list), the 4 named negative spans, masthead/footer boilerplate.
