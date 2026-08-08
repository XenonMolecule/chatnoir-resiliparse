# 0097 — NEGATIVE (census-only): JSON-LD description rescue

- **Date:** 2026-08-08

The dittrickswines gold IS its JSON-LD description verbatim — but the
family census kills the rescue: gold contains the JSON-LD description
in only 27-38% of candidate docs (usually because the description
ALSO renders as page text, which we already extract), and among
gold-containing docs the pred lacks it in exactly ONE doc per split
(dittrickswines itself + one train sibling). A rescue rule would fire
on ~150 docs to fix 2 — far below any precision bar. Single-doc
territory; not built. The embedded-JSON vein (incl. complex.com's
escaped-HTML payload, a 1-doc bespoke parser) is dispositioned to the
client-rendered bucket for good.
