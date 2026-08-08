# 0078 — Runtime checkpoint + engine-pool closure

- **Date:** 2026-08-08

## Runtime (7-run best-of protocol, tight variance 2.90-2.92)
markdown 2.90 ms/doc (2.74 at the 0064 checkpoint: +6% across five
handler cycles ≈ 1.2%/cycle, each under the per-decision gate; logged
as accepted cumulative cost). plain 1.45 ms/doc — untouched, still
~1.3x Cython. Queued speed item: raw-html substring pre-gates before
handler selector queries would reclaim most of the +6%.

## Engine pool CLOSED
english-subtitles probe: gold text absent from raw html (client-
fetched metadata) — third triage misdiagnosis (huskers stale+invented
selectors, widgets est_gain wrong-sign, this). Triage est_gains
require per-doc verification before build — now standing policy.
Remaining ledger: bespoke sub-0.5-milli or unfixable.

## Scoreboard
original dev 0.8283/0.7321 · golden v6 **0.8469/0.7587**
