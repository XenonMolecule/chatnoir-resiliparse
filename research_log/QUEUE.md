# Queue (updated cycle 0050)

## In flight
- **big-data v3 model retrain** (background): 100k-doc big_train, v3
  60-dim features, GBR n60d6 → `gbm_v3big_n60d6.joblib`. On completion:
  export via export_gbm_rust.py, re-eval at veto 0.25 / keep 0.85, and
  re-sweep thresholds (a better model shifts the optimum).

## Next candidates
- M1 `--allow-test` checkpoint once Lev ≥ 0.72 (F1 side crossed at 0.8120).
- Lithium platform comments (shopify/mcafee/lenovo: 2 dev / many train).
- Drupal comments (uxmag, thefreshloaf, rationalresponders: ~4 dev).
- phpBB2 murga-family: needs COLUMN-aware pairing (doc-order is an
  anti-pattern — 0040/0050 both negative).
- legalinsurrection-style camelCase content classes (postContent):
  root-restriction interplay, one-doc forensic parked.
- Criterion speed profiling (~10% markdown-config cost reclaim).
- Gold-noise exclusion list for sweep hygiene (no official-metric gain).

## Walls (do not build without new evidence)
1. nav-on-catalog / 2. related-modules / 3. listing pages / 4. generic
post-streams / 5. dedup / 6. CSS-visual bold / 7. Tags-lines / 8.
byline-keeping on listing pages / 9. post-byline format / 10.
comment-convention per-doc variance (Highlander 41/27, old-Blogger) /
11. blockquote `> ` prefix (41/59) / 12. em-vs-en dash (61/39) /
plus: hard-breaks beyond BR (bluffcountry), U+2011/U+202F typography,
md-links/images (base-rate), tweet-wall golds.

## Profiling result (cycle 0055 window)
Markdown-config overhead (~75% over plain) is DIFFUSE: disabling the
model entirely makes it SLOWER (3.04 vs 2.61 ms/doc) — vetoes shrink
downstream serialization/rescue work and pay for themselves. No single
hot lever; criterion micro-profiling deprioritized. Floor-10 quality
margin stays blocked unless a structural change (e.g., feature reuse
between tpl scan and model) lands.

## Strategic map after the golden reset (post-0059)
Scoreboard: original 0.8223/0.7260 · golden 0.8330/0.7450.
Local rules are exhausted (deficiency ledger = singleton page
structures; empty-section list family = 3 docs; yahoo-mb = 1).

**Next big swing: MODEL-PRIMARY SELECTION.** Replace the rule cascade
as the primary keep/drop decision: score EVERY block (floor ~40B),
keep iff score >= tau (sweep on golden + original-cross-check), rules
demoted to tie-breakers; engine handlers/rebuilds still run first;
rescue ladder becomes the safety net for model wipes. Justification:
AUC 0.86 on 12M blocks, five straight wins from raising model
authority, walls doctrine (12 confirmations) says only global judgment
moves the remaining mass. Implementation sketch: new
SELECTION_MODE=ModelPrimary in tpl_vetoes -> emit veto set = blocks
below tau (no is_main_content_node gating), whitelist = everything
else; sweep tau in {0.3..0.6}.

Also queued: learned per-image selection (0059 verdict); v7 features
gated on live-dev only (0056 lesson); 27 fleet flags for owner review;
empty-section list drops (3 docs); Yahoo message-board handler.

## Empty-section family diagnosis (0062 window, 3 docs)
Drupal year-sectioned lists (dartmouth pattern): each section is an
<article class="text-chunk"> whose item titles are links -> the teaser-
article rule vetoes them; tier-2 short_articles rescue is blocked by
ARTICLE_RESCUE_MAX_COUNT=3 (the page has ~15). Correct fix requires
distinguishing "many small content articles" from "teaser stream" —
page-type wall instance #13. Not chased at 3-doc scale; unlocks with
any future page-type capability.

## Next rendering probe: CSS-bold (0063 candidate)
Reuse the 0062 <style> parser but for `font-weight:bold|700+` classes →
wrap matching spans' text in ** during markdown serialization. Attacks
wall #6 (visual bold, 100+ census ops incl. course-header-title,
span.label families). Unlike 0062 this is ADDITIVE (no veto — worst
case is wrong bold, small per-instance Lev), so the crater class does
not apply; still run the full three-target battery and watch for the
unexplained 0062 parser interaction (verify lawyerfox stays 3277B).
Gold-side conventions already measured in 0035 (bold-census).
