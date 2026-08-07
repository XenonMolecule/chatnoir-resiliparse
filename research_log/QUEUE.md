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
- **Forum engine handlers** (0014 diagnosis, 2026-08-07): 104 forum docs on
  dev, mean F1 0.707 (corpus 0.782). Engines: vBulletin 44, phpBB 25,
  Invision 8, custom 21, XenForo/UBB/SMF 2 each. Gold post format:
  `**user – MM-DD-YYYY, HH:MM**` header (dash variants), body below, posts
  separated by blank line (sometimes `---`); "Quote: Originally Posted by
  **user**" KEPT. Worst docs are reply-form pages (newreply.php — should
  extract almost nothing) and whole-thread chrome. Build order: vBulletin
  handler (44 docs) → phpBB (25) → reply-form detector. jusText refs: 0058
  XenForo, 0063 vB4 postcontent, 0072 phpBB date-before-author, 0079
  broadening NEGATIVE (stay engine-gated).
- **Typography (measured wall, 0026)**: dash/quote flavors are per-site style
  the gold mirrors (en/em split by site; the 15.9K en-dash mass was one
  decorative outlier + recall confounds). No directional normalization fix.
- **md-links [text](href) (deferred, 0026)**: 196 gold docs / 892 instances vs
  tens of thousands of kept anchors — same base-rate problem as images
  (0011); joins the learned-selection queue.
- **Model-placeholder guard (from 0029)**: add a test asserting
  score_block returns non-constant output on two distinct feature vectors,
  so a placeholder can never silently ship again.
