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
