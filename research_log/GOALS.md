# Goals — set 2026-08-07

**Target (user-set): lpv11 dev ≥ 0.90 ROUGE-L F1 and 0.80–0.85 Levenshtein
similarity**, including markdown formatting in the extraction. Speed guardrail
unchanged: stay ≥ ~1.15× vs the frozen Cython reference, no p95 blowups.

Current (0007): **0.7805 F1 / 0.6677 Lev**. Gap: **+0.12 F1, +0.13–0.18 Lev.**

## Why this is believed attainable (the jusText precedent)

jusText closed an almost identical gap (general dev 0.762 → 0.886 F1, 0.682 →
0.821 Lev) in ~100 logged cycles. Its gains decompose roughly as:

| lever family | jusText gain | resiliparse analog |
|---|---|---|
| recall/threshold repairs + rescues | +0.047 | rule repairs & rescue tiers (banked +0.019 so far) |
| learned classifier → fastText on 100k | +0.034 → +0.061 | learned block classifier on lpv11 big_train (100k) — biggest single lever |
| segmentation/structure (rows/lists) | +0.01–0.02 | markdown structure mode; block splitting |
| ~50 quality/engine-handler cycles | ~+0.015 agg (large per-domain) | forum/comment handlers — lpv11 gold uses `**user** (date):` |
| dedup, hygiene (mojibake/FFFD/entities), spacing | ~+0.01 F1, large Lev | port the same fixes (mostly language-agnostic) |
| JS-blob/data rescues, needs-JS routing | long tail | same pattern, same gates |

Note we can benchmark directly against jusText on the OLD general dev (same
gold): resiliparse 0.8138 vs jusText 0.886 — a −0.07 gap on identical data is
strong evidence the techniques transfer and the room is real.

## Where today's remaining F1 lives (0007 bucket analysis, lpv11 dev)

| F1 bucket | docs | value of lifting to 0.95 |
|---|---|---|
| < 0.3 | 58 | +0.045 |
| 0.3–0.6 | 130 | +0.063 |
| 0.6–0.8 | 188 | +0.044 |
| 0.8–0.9 | 201 | +0.019 |
| 0.9–0.95 | 178 | +0.004 |

Reaching 0.90 requires effectively solving everything below 0.6 (+0.108) plus
most of the 0.6–0.8 band. Tags: 155 OVER_EXTRACT (precision: forums/tag/
archive pages) now slightly outnumber the under-extraction tail — precision
work and engine handlers are where the mass is.

**Lev specifically:** empirically Lev ≈ F1 − 0.11 at our operating point; at
F1 0.90 that projects Lev ≈ 0.79 *before* formatting work. The markdown mode
(headings `#`, `**bold**`, `- ` bullets vs our `•`, pipe tables, fences) plus
whitespace conventions must close the remaining 0.01–0.06. Syntax tokens
alone measured only ~+0.006 (0003), so the Lev payoff comes from line
structure + the F1 climb together.

## Ceiling calibration (cycle 0008)

Greedy selection-oracle over the unfiltered extraction: **0.7729 F1 at
`\n\n`-block granularity** (below current output — blocks glue boilerplate to
content) and **0.8582 at line granularity** (~0.87–0.89 for a true optimum).
Implication: selection improvements (rules, learned classifier at line/node
granularity) can carry ~0.85–0.87; **the last stretch to 0.90 requires output
transformation** — text normalization, content recovery (JS blobs, alt/math),
and restructured lines. Both tracks are necessary; neither is sufficient.
The ceiling itself is movable (jusText's rose 0.893 → 0.944 with structure
fixes).

## Milestones

- **M1 (~cycles 0008–0020): 0.81 F1 / 0.72 Lev.** Ceiling measurement;
  markdown output mode v1 (headings/bold/lists, `- ` bullets, gold-matching
  line structure); remaining catastrophic tail (landmark/footer, unattributed
  19); hygiene ports (entities, spacing at inline boundaries).
- **M2 (~0020–0045): 0.85 / 0.76.** Forum & comment engine handlers
  (vBulletin/phpBB/XenForo/SMF/WordPress-comments with `**user** (date):`
  formatting); over-extraction precision family (tag/archive pages, reply
  forms); dedup (quote-normalized, code-safe); pipe tables + code fences;
  selective alt-text/math transcription.
- **M3 (~0045+): 0.90 / 0.80–0.85.** Learned block classifier on big_train
  (fastText-style features over DOM blocks, shipped as a gated tier like the
  rescues); long-tail engine handlers; JS-blob rescues; needs-JS routing;
  gold-gap grind guided by fresh dev2/dev3-style samples to avoid overfitting
  dev.

Milestone reviews at M1/M2/M3 are the declared `--allow-test` points
(vaulted-test policy): run test once per milestone, record in the log.

## M1 checkpoint — RECORDED 2026-08-07 (cycle 0051)
Condition: dev F1 ≥ 0.81 AND Lev ≥ 0.72 → met at 0051 (0.8174 / 0.7211).
**Vaulted test (first authorized measurement): F1 0.8218 / Lev 0.7225.**
Test ≥ dev on both metrics — no dev overfitting; the sweep-heavy cycles
(0046/0051 threshold tuning) generalize. Next milestone M2: 0.85 / 0.76.
