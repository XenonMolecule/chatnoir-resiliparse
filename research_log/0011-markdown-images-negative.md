# 0011 — Markdown images `![alt](src)` — NEGATIVE, reverted

- **Date:** 2026-08-07
- **Tag:** 0011-md-images{,-v3,-v4} (measured), 0011-reverted (= 0010 output)
- **Status:** abandoned (feature reverted; knowledge banked)

## Hypothesis

Gold renders body images as `![alt](src)` (436 instances / 167 dev docs, src
verbatim); emitting the same for images with alt should gain F1+Lev on
image-heavy pages (image URLs carry many tokens).

## What changed (all reverted)

Three variants, each measured on lpv11 dev (markdown config), vs 0010:

| variant | Δ mean F1 | per-doc |
|---|---|---|
| ungated (every img with alt+src) | **−0.0218** | 66↑ / 546↓ |
| src-path keywords + dimension gate | −0.0112 | 54↑ / 375↓ |
| prose-ancestry joint gate (p/em/figure, ¬a, small-dim veto) | −0.0011 | 16↑ / 57↓ |

## Insights

- **The base rate kills every rule.** Feature measurement over 300 docs:
  gold keeps 127 of 3,785 attributed images (3.4%). Best single signals:
  `parent:p` (20× likelihood ratio), `parent:em` (∞, image-blogs), `in-link`
  as a negative (0.27 vs 0.68). Even the joint rule lands near ~40%
  precision — and each false positive costs a URL's worth of noise tokens,
  so the arithmetic stays negative all the way down the gate-tightening
  path.
- The 66 ungated wins are real (image-blogs whose gold is mostly images) —
  the *form* is right, the *selection* needs to be learned. This is the
  clearest concrete case yet for the M3 learned classifier: image keep/drop
  is a per-node decision with informative features and a terrible prior.
- Methodological note: the first "gate" attempt shipped a regex with an
  unclosed group — panic → empty predictions → F1 0.45. The harness's
  BaseException guard (0006) did its job: the run completed and the failure
  was visible in numbers, not a crash.

## Next

- Bank `![alt](src)` emission behind the future learned image-selector
  (queued). Remove from markdown-v2 scope; v2 = pipe tables + code fences.
- 0012: pipe tables (72 gold docs; `has_table` flag available for a
  targeted check).
