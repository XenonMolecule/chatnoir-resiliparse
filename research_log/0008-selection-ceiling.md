# 0008 — Selection-oracle ceiling (goal calibration)

- **Date:** 2026-08-07
- **Tag:** analysis over cached runs (0004-nomc, 0007-article); no code change
- **Status:** landed (analysis-only)

## Hypothesis

Before committing to the 0.90 F1 / 0.80–0.85 Lev goal (GOALS.md), measure
what pure keep/drop selection over the *current* extraction output could ever
achieve (jusText-0006 precedent). If the selection ceiling is below the goal,
output transformation is load-bearing, not optional.

## What changed

Nothing. Greedy forward-selection oracle over the unfiltered
(`main_content=False`) extraction, scored against gold, 200-doc random sample
(seed 0), at two granularities.

## Results

| oracle granularity | F1 |
|---|---|
| `\n\n` blocks | 0.7729 — **below the actual 0007 output (0.7805)** |
| `\n` lines | **0.8582** |

(Greedy underestimates the true optimum; call the line-level ceiling
~0.87–0.89.)

## Insights

- **Block-level selection is bankrupt**: `\n\n` blocks glue boilerplate to
  content so hard that a per-block oracle loses to our current pipeline. Any
  learned classifier must operate at line/DOM-node granularity, not on
  current output blocks.
- **Line-level keep/drop tops out just below the 0.90 goal.** The remaining
  ~0.03–0.05 must come from words that are *not in the plain-text extraction
  at all*: text normalization (entities/mojibake — token mismatches),
  content the extractor never emits (JS-blob text, informative alt/math
  transcriptions), and restructured lines (`**user** (date):` author
  markers). This vindicates the goal's emphasis on output transformation and
  matches jusText's history (their oracle rose 0.893 → 0.944 only after
  segmentation/structure fixes — the ceiling itself is movable).
- Practical reading of the roadmap: selection improvements (rules, learned
  classifier) can carry us to ~0.85–0.87; the last stretch to 0.90 is
  transformation work. Both tracks are necessary; neither is sufficient.

## Next

- 0009: markdown output mode v1 (gold-flavored structure: `#` headings,
  `**bold**`, `- ` bullets, gold-matching line shape) — the Lev track opener.
- Learned line/node-level classifier moves up the priority list (M3 → start
  groundwork in M2) given the block-granularity finding.
