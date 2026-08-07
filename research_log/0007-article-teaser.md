# 0007 — Article-teaser rule repair (short-story exemption)

- **Date:** 2026-08-07
- **Tag:** 0007-article (baseline compared against: 0006-widget2)
- **Status:** landed

## Hypothesis

The `<article>` teaser rule (reject when body_depth>2, ≤500 chars, >20% link
text) drops real-but-short stories; teasers come in streams (many sibling
`<article>`s), so gating a relaxation on "few `<article>` elements on the
page" should recover the 0003 bucket without readmitting streams.

## What changed

- Generalized the tier-2 rescue's single `ul_exemption` flag into
  `RelaxFlags { text_heavy_lists, short_articles }`; the primary walk now
  records dropped `<article>` nodes alongside dropped `<ul>`s.
- `short_articles` relaxation: skip the teaser check on retry — enabled only
  when a dropped `<article>` would survive with it AND the page has ≤3
  `<article>` elements total. Same keep criteria as 0005 (>2× larger, no
  duplication, no error-stub).

## Results

| set | 0006 F1 | 0007 F1 | per-doc |
|---|---|---|---|
| lpv11 dev | 0.7801 | **0.7805** | +1 / −0 |
| general dev | 0.8138 | 0.8138 | 0 / 0 |
| general dev2 | 0.8144 | 0.8144 | 0 / 0 |
| lpv11 train | 0.7747 | **0.7753** | **+12 / −1** (−0.015, noise-level) |

Speed ratio vs frozen Cython: 1.19× (vs 1.20× at 0006 — within run noise;
the added work is one tag-name collection behind the same gate). Golden
tests pass.

Smaller on dev than the 0003 taxonomy suggested (8-doc bucket) because
tier-1 (0004) had already recovered most of those docs — the buckets
overlap. Train shows the true incremental value (+12).

## Insights

- Rule-repair cycles after a general rescue have shrinking dev-visible
  returns; train (10×) is where their real effect is measurable. Keep using
  train as the per-cycle confirmation for rescue-family changes.
- The RelaxFlags structure makes the next rule-relaxation cycles
  (landmark/footer) mechanical: record dropped node kind → eligibility test
  → shared retry gate.

## Next

- landmark_id/footer regex tightening (or a `landmark` RelaxFlag) — the
  remaining named 0003 bucket.
- Forum over-extraction track (131 OVER docs) — different failure family,
  needs its own diagnosis pass.
- Then revisit markdown output mode.
