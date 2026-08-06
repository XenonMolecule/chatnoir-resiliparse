# 0003 — Failure taxonomy on lpv11 dev

- **Date:** 2026-08-06
- **Tag:** v1.0.9-cacde72 (analysis of the 0001/0002 baseline; no code change)
- **Status:** landed (analysis-only cycle)
- **Machine note:** mid-cycle the session teleported to a cloud VM
  (GCP e2-standard-4, 4 vCPU x86, Debian 12). Parity re-verified there:
  golden tests pass and `--impl rust` is byte-identical to the cached Cython
  predictions on all 1,000 dev docs **on Linux/x86 too**. New speed baseline on
  this machine (interleaved, single process): Cython 2.654 / Rust 2.152 ms/doc
  (1.23×, consistent with the M2's 1.27×). M2 timings are not comparable to VM
  timings; all future A/B numbers are VM-vs-VM.

## Hypothesis

Reading the worst ~20 lpv11 dev docs will reveal a small number of shared
pathologies that account for most of the catastrophic tail, and rank the
roadmap better than aggregates. Preregistered guesses: main-content classifier
false negatives wiping whole subtrees, and the markdown gap dominating
mid-range PARTIAL docs.

## What changed

Nothing (analysis-only). Tools: ancestor-chain "blame" simulation of every
`_is_main_content_node` rule (regex + structural) against the DOM node holding
the gold text, run over all catastrophic docs; word-coverage check of gold
against DOM text to separate extraction failures from data-capture failures.

## Results

**1. The catastrophic tail is main-content classifier false negatives, and it
is fixable.** Of 110 catastrophic docs (EMPTY_PRED or pred/gold < 0.5):

- **98 have ≥70% of gold words present in the DOM** — extraction failures.
- **12 are data-capture failures** (JS-rendered SPAs etc.; word coverage ≤0.7)
  — route-around bucket, not extraction bugs.
- All 20 worst-F1 docs recover most content with `main_content=False`
  (e.g. gold 4,190 chars → mc 0 chars → no-mc 8,342 chars).

**2. Rule-level blame histogram** (first rule that fires on the gold-content
ancestor chain, over the catastrophic docs):

| rule | docs | example pathology |
|---|---|---|
| `<ul>` rule (depth<4 OR link-ratio>0.2) | 13 | obituaries/news/docs where main content *is* a list (`<ul class="clr">`) |
| `<article>` teaser link-cluster | 8 | article bodies ≤500 chars with a few links |
| nav class regex | 4 | `submenu_container` etc. on content wrappers |
| landmark_id (`id="footer"` etc.) | 3 | whole page inside `<div id="footer">` |
| footer class regex | 2+ | Weebly's `wsite-not-footer` *matches the footer regex* |
| aside/time tags, sidebar/social/search/display/post_meta/itemprop/aria, link-cluster divs | ~10 | one-offs |
| unattributed (blame walk inconclusive) | 19 | needs per-doc debugging |

**3. OVER_EXTRACT is forums + tag/archive pages.** 131 docs with ratio>1.5 &
F1<0.8; the worst are forum reply forms (25× gold size), whole-thread dumps
where gold keeps one post, and tag/index pages. This is a *precision* problem
mostly orthogonal to the classifier false negatives.

**4. Markdown syntax is NOT the big Lev lever (hypothesis refuted).**
Re-scoring baseline predictions against markdown-stripped gold moves Lev only
+0.006 overall, and only +0.0057 even on content-correct docs (F1≥0.9, Lev
0.894 there). The residual 0.10 on good docs is whitespace/newline structure,
bullet-style (`•` vs `-`), and small content diffs — not missing `#`/`**`
tokens. A markdown mode is still worth building (its Lev value includes fixing
bullet/heading *line structure*, which the strip test can't see), but the
expectation is now "modest Lev + correctness win," not a step change; the
classifier tail is the bigger prize.

## Insights

- Both preregistered guesses were half-right: classifier false negatives
  dominate (confirmed, and they're rule-attributable), but the markdown gap
  does NOT dominate the mid-range — content selection error dwarfs formatting
  even there.
- The `<ul>` blacklist rule is the single worst offender. It embodies the
  assumption "shallow or link-heavy lists are chrome," which breaks precisely
  on list-structured main content (docs, obits, briefs).
- A **self-correcting rescue** (jusText pattern: if main-content extraction is
  near-empty vs. page text, fall back to `main_content=False`, keep only if it
  yields much more) would neutralize all 13 EMPTY_PRED docs and the worst
  UNDERs in one gated, regression-safe change — highest-leverage next cycle.
- Per-rule fixes (ul, article-teaser, landmark) are the follow-up cycles, each
  gated on a train-wide fire count per the playbook's engine-handler discipline.

## Next

- 0004: near-empty rescue (fallback to `main_content=False` when the
  main-content result is a tiny fraction of body text; ship only if per-doc
  zero-regression holds).
- 0005+: relax/repair the `<ul>` rule; then `<article>` teaser threshold; then
  landmark/footer regex tightening (e.g. require *exact* footer semantics, fix
  `wsite-not-footer`).
- Separate track: forum over-extraction (reply forms, quoted threads).
- Route-around: needs-JS detector for the 12 data-capture docs (low priority).
