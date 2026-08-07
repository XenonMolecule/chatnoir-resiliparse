# Idea queue / deferred backlog

Deferred ideas, parked experiments, and known-but-not-yet-scheduled fixes.
Move an item into a numbered cycle when it becomes the active hypothesis;
delete it when a cycle rules it out (and link the cycle).

- **Selective alt-text filtering** (from 0001): blanket `--no-alt-texts` gains
  +0.0065 F1 aggregate but regresses 89 docs (worst −0.67). Capture the win by
  dropping decorative/icon/thumbnail alts while keeping informative ones
  (figures, LaTeX-as-image transcriptions — those matter for math later).
- **Preprocessed-variant deficit diagnosis** (from 0001): raw HTML scores 5.4 F1
  above the lpv11 preprocessed variant (0.762 vs 0.708) with the same extractor.
  Understand what `preprocess_html_for_extraction` removes that the main-content
  heuristics need; may reveal fragile signals worth hardening.
- **landmark_id / footer regex tightening** (from 0003): `id="footer"` wipes
  pages whose whole layout sits in a footer-named div; Weebly's
  `wsite-not-footer` literally matches the footer regex.
- **Forum over-extraction track** (from 0003): 131 OVER docs — reply forms,
  whole-thread dumps where gold keeps one post, tag/archive pages.
- **Markdown v2** (from 0009): pipe tables (72 gold docs), code fences (30),
  links-as-markdown `[text](href)`. Bold-coverage heuristics for label-styled
  spans (`**Name:**` from dt/th/label) — verify gold consistency first.
- **Learned image selector** (from 0011): `![alt](src)` emission is right in
  form but needs learned keep/drop (3.4% base rate, best rule ~40% precision,
  net-negative). Features measured in 0011; revisit in the classifier era.
- **Rescue-shape veto** (from 0010): tier-1 rescue swaps shells onto tiny
  image-blog pages whose heading+image output was already gold-shaped.
