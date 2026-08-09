# 0129: line-diff census + bold-label rewrite — NEGATIVE, reverted

Aggregate line-diff census over all 1000 golden docs (free, scripted): the
single biggest residual family is section-label SHAPE — gold renders
`**Comments**` in 42 docs while we emit `Comments`/`#..### Comments` in
~41; same for `**Tags**`. Tried the majority-convention rewrite (all our
variants → bold form): golden up 16 / down 28, train up 136 / down 279 —
all pure-Lev micro-moves. The counts don't align pairwise: docs wanting
`## Comments` receive our rewrite too, and lose. This is the 0093
convention-unification zero-sum law measured a third way, now at
single-label granularity. Reverted (build + tests verified; plain
1000/1000 held throughout).

Census take-away: the residual mid-band loss is dominated by per-doc
formatting-convention variance that no deterministic emitter can match —
only per-doc signals (template detection beyond generator meta, or a
learned formatting head) could. Logged as the wall the 0.9-F1 push now
sits against, alongside the owner-gated items.
