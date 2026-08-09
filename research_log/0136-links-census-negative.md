# 0136: link-emission census — NEGATIVE, closes the engine family

The band-79 engine docs are dominated by one theme: gold keeps `[text](url)`
markdown on SELECTED anchors (746 instances across 216/1000 docs) while our
frozen config (links=False since 0009) emits bare text. Probe on 150 docs:
blanket links=True scores **0.8171/0.7022 vs 0.8727/0.7955** — emitting all
anchors is catastrophically worse than none, because gold's annotator kept
only content-area links (attachments, in-article citations, image links).
Per-anchor selection is the identical wall as image emission (0011: 3.4%
keep, no DOM rule reaches precision; 0113 re-confirmed at 13% for
wp-image). The engine family is thus measured-closed: its docs need
learned per-node link/image selection, not rules.

No code changed. Remaining autonomous surface after 33 cycles:
hinted-wave grinding at ~+0.0005/800k tokens with decay — mathematically
insufficient for the −0.0100 gap without the owner-gated items.
