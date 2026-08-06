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
- **EMPTY_PRED / UNDER_EXTRACT rescues** (from 0001): 13 empty + 97
  under-extracted docs on lpv11 dev; jusText-style self-correcting rescues
  (gated on near-empty output) are the known-safe pattern.
