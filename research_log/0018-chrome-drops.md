# 0018 — Audit-guided chrome drops (markdown config)

- **Date:** 2026-08-07
- **Tag:** 0018-chrome7 (baseline compared against: 0017-forums)
- **Status:** landed (with logged tail decision)

## Hypothesis
The gold audit gave keep-rates per chrome category: cookie 0%, share 2%,
breadcrumb 5%, footer 5%, login/search 6% — safe to drop where our regexes
miss them (single-hyphen compounds like `post-share-buttons`). Wall
categories (nav, related-posts, pagination-adjacent) excluded.

## What changed
`MD_CHROME_CLS` veto in the main-content classifier (markdown config only;
guardrails byte-unchanged), hardened over seven measured variants:
- copyright split: linkless copyright text = source-attribution (gold keeps),
  linked copyright rows = footer chrome (drop);
- pagination tokens removed entirely (`posts-pagination` on a main listing
  container → rescue swapped junk, −0.39);
- content-marker exemption (`hentry|type-post|…|signature`) — WordPress slug
  classes (`tag-…-sharing-them`) false-match hyphen-internal words (−0.96!);
- size cap ≤1500 collapsed bytes — wrapper divs named for a widget they
  contain (`place-login-pop` wrapping 45KB) must never be vetoed;
- negation gluing (`no-share` must not match `share`; regex crate has no
  lookbehind).

## Results
| set | Δ mean | per-doc |
|---|---|---|
| lpv11 dev | **+0.0017 F1 / +0.0023 Lev** | 167↑ / 22↓ (worst −0.107, catalog-page wall + one attribution collision) |
| lpv11 train | **+0.0015** | 451 > +0.01 vs 15 < −0.1 (30:1; worst −0.57) |
| general dev / dev2 | 0.0000 | 0 changed |

Logged decision: the train tail (15 docs < −0.1) ships per the 0006
precedent (30:1 ratio); several members are wrapper-class collisions the
template-subtraction signal (0019) addresses structurally.

## Insights
- Class-regex precision work has a hard ceiling: hyphenated slugs make every
  inner word boundary-delimited, wrappers borrow widget names, and negations
  invert meaning. Each needed a structural guard, not a better word list —
  strong evidence for the repetition-based signal as the next step.
- The audit's category keep-rates were accurate at scale: improvements
  concentrated exactly in share/breadcrumb/footer categories.

## Next
- 0019: template-subtraction port (subsumes several residual collisions).
