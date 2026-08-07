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
- **Markdown output mode** — the planned big lever (playbook §8.4); scheduled
  after the 0002 spike resolves which implementation iterates.
- **Article-teaser rule repair** (from 0003): 8-doc bucket; `<article>` with
  ≤500 chars + links rejected as teaser even when it's the real (short) story.
  Same rescue-tier pattern as 0005.
- **landmark_id / footer regex tightening** (from 0003): `id="footer"` wipes
  pages whose whole layout sits in a footer-named div; Weebly's
  `wsite-not-footer` literally matches the footer regex.
- **Forum over-extraction track** (from 0003): 131 OVER docs — reply forms,
  whole-thread dumps where gold keeps one post, tag/archive pages.
