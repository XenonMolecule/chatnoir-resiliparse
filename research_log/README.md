# Research log

Chronological record of resiliparse extraction experiments on the lpv11 benchmark.
One file per idea/cycle, newest insights captured while fresh. **Keep each entry
under a 5-minute read.** Conventions replicate the jusText auto-research log
(https://github.com/XenonMolecule/jusText, `research_log/`); the full playbook is
`AUTORESEARCH.md` at the repo root.

## Convention

- Files: `NNNN-short-slug.md` (zero-padded, monotonically increasing).
- Every entry names the **run tag** it was measured with (`vX.Y.Z-<sha>`), so
  results trace back to cached runs under `benchmark/runs/<tag>/`.
- Compare against the previous version with `viz.py compare <prevTag> <thisTag>`.
- **Two first-class objectives:** quality (F1/Lev vs. the markdown-flavored lpv11
  gold) *and* speed (ms/doc mean+p50/p95, docs/s — single worker, optimized
  build, same machine). Every results table carries both.

## Iteration cycle

Each cycle is one pass of the loop below. A "cycle" maps to roughly one research
log entry and one commit — though a single hypothesis may take several
un-committed edits before it earns a commit.

1. **Backfill** — add the git commit id to the *previous* entry (now that it exists).
2. **Review** — skim prior entries + `QUEUE.md`; note anything that informs today.
3. **Preregister** — write the hypothesis for this cycle *before* coding (a stub
   entry with Hypothesis filled in).
4. **Revert (rare)** — if a past change should be undone, revert it first.
5. **Change** — make the code changes to test the hypothesis.
6. **Measure** — `python benchmark/eval/run_eval.py --dataset lpv11 --split dev`
   (auto-tags from git; `--tag` for dirty experiments). Never test mid-cycle.
   Official timing runs: `--workers 1`, optimized build, rebuilt first.
7. **Iterate** — refine and re-run freely.
8. **Log** — once the result is clear (win, quality-neutral ship, or ruled out),
   fill in Results (vs. prior tag via `viz.py compare`), Insights, Next.
9. **Commit** — code + log entry together, message `NNNN: <short description>`.
   Follow-ups: `NNNN fix:` / `NNNN polish:`. Queue-only commits: `Queue: <note>`.
10. **Repeat.**

Guardrails (see `AUTORESEARCH.md` §6): test split stays vaulted until declared
milestones (`--allow-test`); zero per-doc regressions on guardrail sets; speed
regressions >5% mean ms/doc (or new p95 blowups) block shipping like quality
regressions; the gold is imperfect — never drop visibly-good content to chase
the metric; verify markdown tables actually render.

## Entry template

```markdown
# NNNN — <title>

- **Date:** YYYY-MM-DD
- **Tag:** vX.Y.Z-<sha>   (baseline compared against: <prevTag>)
- **Status:** idea | in progress | landed | abandoned

## Hypothesis
One or two sentences: what we believe is wrong and what change should help.

## What changed
Bullet points of the actual code change.

## Results
Small table vs. the comparison tag (dev/train only; test only at milestones).
Both F1/Lev and ms/doc. Net effect in one line.

## Insights
- What we learned (whether or not it worked). Failure modes confirmed/ruled out.

## Next
- Concrete follow-ups this surfaced.
```

## Index

lpv11/dev (raw HTML, markdown config since 0009): F1 **0.8283** / Lev **0.7321** (0077) · golden **0.8469/0.7587** (v6) · train 0.8145/0.7159 — M1 crossed; test@M1 0.8218/0.7225. Goal: 0.90 / 0.80–0.85 (GOALS.md). Baseline 0.7619/0.6540 (0001).
**Iteration base: Rust** (`resiliparse-rs/src/extract/`) since 0002; Cython frozen as reference.

- [0001 — Cython baseline](0001-cython-baseline.md) — lpv11 dev 0.7619/0.6540, 1.64 ms/doc; defaults = best stock config; parity oracle for 0002
- [0002 — Rust port spike](0002-rust-port-spike.md) — **both gates passed**: 10,999/10,999 docs byte-identical, 1.27× faster (p95 1.31×); Rust adopted
- [0003 — Failure taxonomy](0003-failure-taxonomy.md) — catastrophic tail = classifier false negatives (98/110 fixable; `<ul>` rule worst); markdown-syntax Lev headroom refuted (+0.006); roadmap: rescue → rule fixes
- [0004 — Near-empty rescue](0004-near-empty-rescue.md) — **+0.0163 F1**, 29↑/0↓ dev (+0.012 both guardrails, 0 reg); gated fallback, sweep-tuned, no speed cost
- [0005 — ul-rule repair](0005-ul-rule-repair.md) — tier-2 rescue for text-heavy lists: +1 doc/set, 0 reg, ~1% speed cost; bigger variant queued (needs widget signal)
- [0006 — Widget signal + error-stub guard](0006-widget-signal.md) — widget/meta/hide veto unlocks stronger list exemption: +9 docs/0 reg on dev sets; train 312↑/50↓ (+0.0156) with logged trade-off; 2 robustness fixes
- [0007 — Article-teaser repair](0007-article-teaser.md) — RelaxFlags generalization + short-article retry: dev +1/0 reg, train +12/−1(noise); rescue buckets overlap tier-1
- [0008 — Selection ceiling](0008-selection-ceiling.md) — block-oracle 0.77 (< current!), line-oracle 0.86; transformation is load-bearing for the 0.90 goal; GOALS.md set
- [0009 — Markdown mode v1](0009-markdown-mode-v1.md) — headings/bold/bullets/hard-breaks: Lev +0.0024, F1 flat, +0.4% cost; markdown is now the lpv11 config; label-bold + md-links queued
- [0010 — Blogger chrome handler](0010-blogger-chrome.md) — meta-generator-gated skip of share/labels/pager chrome: 110 docs improved across dev sets, 0 meaningful reg; gold-consistency check kept "Posted by" untouched (28/62 wall)
- [0011 — Markdown images (NEGATIVE)](0011-markdown-images-negative.md) — `![alt](src)` reverted: gold keeps 3.4% of images, no DOM rule reaches viable precision (−0.022 ungated → −0.001 best gate); needs learned selection
- [0012 — Pipe tables](0012-pipe-tables.md) — per-cell nodes + data-table gate + pipe serializer; renders 105/109; F1 +0.0007 dev, +0.0008 train; rescue gates hardened to content-length
- [0013 — Code fences](0013-code-fences.md) — <pre> → ``` fences: 36 docs, ΣΔLev +0.039, F1 exactly 0, quality-correct ship
- [0014 — vBulletin handler](0014-vbulletin-handler.md) — engine-gated thread rebuild (**user – date** + body walk): dev +0.0031 (18↑/0↓), train +0.0059 (258↑/13↓); handler pattern established
- [0015 — phpBB3 handler](0015-phpbb-handler.md) — body#phpbb gate + author-parse variants + authored-posts fallback: dev +0.0006, train +0.0006 (47↑/8↓); forum arc cumulative +0.0037 dev
- [0017 — IPB/UBB/SMF handlers](0017-forum-handlers-ipb-ubb-smf.md) — agent-built, battery-verified: dev +0.0023 (14↑/0↓), Lev +0.0035, train +0.0013; forum arc cumulative +0.0058 F1
- [0018 — Chrome drops](0018-chrome-drops.md) — audit-backed veto, 7 hardening variants (negations, size cap, content markers): dev +0.0017/+0.0023, train +0.0015 (30:1), guardrails clean
- [0019 — Template subtraction](0019-template-subtraction.md) — structural repetition∧link-density veto: **dev +0.0041 F1 / +0.0075 Lev**, train +0.0037/+0.0062; largest cycle gain; ~10% speed cost markdown-only (logged)
- [0020 — WP comment attribution](0020-wp-comment-attribution.md) — native-first rebuild: dev +0.0009, train +0.0011; "gold mirrors the page" lesson
- [0021 — Forum gates](0021-forum-gates.md) — vB markup fallback + phpBB2 + coverage guard: dev +0.0014 (7↑/0↓), train clean (0 < −0.1)
- [0022 — Byline + related (wall)](0022-byline-related.md) — byline-context rel=author kept, ad-tech tokens only; related-family confirmed rule-uncapturable (3rd confirmation); +0.0002
- [0023 — Listing rescue (NEGATIVE)](0023-listing-rescue-negative.md) — three gate variants unstable at scale; listing/article discriminator needs page-type classifier; plumbing kept
- [0024 — dl + classifier groundwork](0024-dl-and-classifier-groundwork.md) — **label:** value dl lines (5↑/0↓); classifier v0 AUC 0.84, safe veto/whitelist tiers identified
- [0025 — Model tier](0025-model-tier.md) — exported 60-tree GBM, veto+whitelist tiers in the walk: dev +0.0015 (7↑/0↓); first model-in-loop cycle; ~10% markdown-only speed cost logged
- [0026 — Fence langs + UI labels](0026-fence-lang-ui-labels.md) — residual audit → curated chrome-line filter: 109↑/15↓ dev; fence ```lang hints
- [0027 — Model v2 scaling (NEGATIVE)](0027-model-v2-scaling-negative.md) — 10× data → AUC +0.001, live wash; features are the frontier, not rows
- [0028 — Model v3 features (in progress)](0028-model-v3-features.md) — wordbag+context features: AUC 0.8551 (+0.005); calibration pending, exact next steps logged
- [0029 — Generic post-stream (NEGATIVE)](0029-generic-poststream-negative.md) — 2 gate rounds, −0.020/−0.002; fifth discriminator-wall confirmation; per-engine gates only
- [0030 — Respond-form veto](0030-respond-form.md) — residual diagnosis → one-line fix: **+0.0035 F1 / +0.0049 Lev** (101↑/8↓); F1 > 0.80, Lev > 0.70
- [0031 — Dedup (NEGATIVE)](0031-dedup-negative.md) — naive exact-dup: dev +0.001/train −0.0004; gold keeps template repeats; containment-aware port queued
- [0032 — Seven one-off engines](0032-oneoff-engines.md) — agent-built per-engine gates: **+0.0033 F1 / +0.0040 Lev**, 7↑/0↓, train spotless; forum family essentially solved
- [0033 — Author-bio + relative guard](0033-author-bio.md) — 10%-keep bio boxes vetoed, >40%-of-page abstention added: +0.0003/+0.0005
- [0034 — Heading promotion](0034-heading-promotion.md) — gold re-levels to #-min (98%): +0.0002 Lev; ungated hard-breaks re-measured negative (gate twice-confirmed)
- [0035 — UI-line strip extension](0035-ui-line-strip-extension.md) — census-driven chrome lines + heading/bold normalization fix: +0.0009 F1 / +0.0012 Lev; bold-coverage & Tags-lines walls confirmed
- [0036 — Table separator convention](0036-table-separator-convention.md) — tight `|---|` + cell-space collapse + rescue gate hardening (dash exclusion, content-table veto): +0.0002 Lev both splits, two crater classes closed
- [0037 — Byline microdata recovery](0037-byline-microdata-recovery.md) — "Posted by at" was a degenerate byline, not chrome: itemprop author/date exemption + dual keep-test: +0.0003 F1 both splits
- [0038 — UI-line strip batch 2](0038-ui-line-strip-batch2.md) — 13 fragment-checked chrome lines: +0.0002 F1 / +0.0004 Lev dev; Blogspot comment-byline rebuild family discovered & queued
- [0039 — Blogspot comment rebuild](0039-blogspot-comment-rebuild.md) — gold rewrites "NAME said..." as `**NAME — TIME**`: +0.0014 F1 / +0.0018 Lev dev; fixed latent effective-veto-set bug in rescue retries
- [0040 — Comment-template probes](0040-comment-template-probes.md) — WP selector fixes ship clean (+0.0001 Lev); Highlander-force & old-Blogger pairing measured negative (per-doc gold variance, wall #10)
- [0041 — phpBB subSilver2 handler](0041-phpbb-subsilver2.md) — b.postauthor table skin: +0.0009 F1 / +0.0014 Lev dev, zero regressions
- [0042 — vBulletin variants](0042-vbulletin-variants.md) — vB4 markup gate + member.php author fallback: small clean win
- [0043 — XenForo handler](0043-xenforo-handler.md) — XF1+XF2, title-attr times, quote-strip per gold majority: +0.0003 F1 / +0.0006 Lev dev
- [0044 — MovableType rebuild](0044-movabletype-rebuild.md) — commentText + "Posted by: X | date" pairs: one dev doc +0.014 F1 / +0.027 Lev
- [0045 — Tab normalization](0045-tab-normalization.md) — gold is 99.6% tab-free: interior-run collapse with code-listing/fence/indent guards: +0.0002 Lev, 57 dev docs up
- [0046 — Model threshold re-sweep](0046-model-threshold-resweep.md) — veto 0.10→0.25 + rollback tier: **+0.0030 F1 / +0.0035 Lev dev**, largest single cycle; M1 F1 target crossed
- [0047 — NEGATIVE: table bold-strip](0047-NEGATIVE-table-bold-strip.md) — 97% gold convention, flat aggregate + keep-test flip crater; reverted
- [0048 — Post-pass ordering](0048-postpass-ordering.md) — tabs-before-strip fixes "Author\tMessage" evasion; handler exits post-passed: 58 docs up, zero down
- [0049 — UI-line strip batch 3](0049-ui-line-strip-batch3.md) — vB footer band + WP likes: 109 docs up, 7 down
- [0050 — NEGATIVE: phpBB2 pairing](0050-NEGATIVE-phpbb2-pairing.md) — doc-order pairing mispairs column-layout skins; anti-pattern confirmed twice
- [0051 — Big-data model + size-tiered veto](0051-bigdata-model.md) — 3M-block retrain, veto 0.30/keep 0.70, >1500B blocks need <0.10: **+0.0054 F1 / +0.0058 Lev; M1 crossed**
- [0052 — Model block-floor sweep](0052-model-block-floor.md) — 150→40B floor, speed-free: +0.0011 F1 / +0.0021 Lev dev
- [0053 — n120d6 model](0053-n120-model.md) — AUC 0.8503, re-swept 0.35/0.65: +0.0008 F1 dev, +0.0017/+0.0017 train
- [0054 — v4 page features](0054-v4-page-features.md) — forms/articles/comment-cls counts, AUC 0.8550, veto 0.40/keep 0.60: +0.0013 F1 / +0.0014 Lev dev
- [0055 — v5 page features](0055-v5-page-features.md) — nav share/generator/block count, AUC 0.8592: +0.0017 F1 / +0.0013 Lev dev; train cross-check now standard for sweeps
- [0056 — NEGATIVE: positional features](0056-NEGATIVE-positional-features.md) — AUC +0.0036 but live −0.001: position anti-correlates on comments; first AUC/live divergence
- [0057 — Golden devset v1](0057-golden-devset.md) — 34-agent fleet, 365 docs cleaned per charter: pipeline scores 0.8330/0.7450 on golden vs 0.8223/0.7260 original
- [0058 — Golden recalibration](0058-golden-recalibration.md) — thresholds robust across targets; empty-base fallback negative; 46-doc deficiency ledger opened
- [0059 — NEGATIVE: md images v2](0059-NEGATIVE-md-images.md) — golden keeps 9% of walk-visible images, best gate 49% precision: family is learned-selection-only
- [0060 — NEGATIVE: model-primary](0060-NEGATIVE-model-primary.md) — pure-model selection bounded below hybrid at every tau; architecture settled
- [0061 — NEGATIVE: confident learning](0061-NEGATIVE-confident-learning.md) — label cleaning neutral; model program closed at settled hybrid optimum
- [0062 — NEGATIVE: CSS-hidden vetoes](0062-NEGATIVE-css-hidden.md) — three variants; JS-reveal idiom breaks the static proxy + unexplained crater; needs real computed styles
- [0063 — NEGATIVE: CSS-bold](0063-NEGATIVE-css-bold.md) — signal wrong, not the code: gold bolds a curated subset of visually-bold text; wall #6 sealed both directions
- [0064 — Yahoo mb handler](0064-yahoo-mb-handler.md) — one-off engine, one doc 0.07→0.9: +0.0009 F1 both targets, zero regressions
- [0065 — Single-post threads + golden v2](0065-single-post-threads.md) — dated 1-post relaxation + owner review round 1: +0.0016 F1 dev, zero dev regressions
- [0066 — vBulletin 5 handler](0066-vbulletin5-handler.md) — li.b-post + announcements + golden v3: +0.0009 F1 / +0.0012 Lev both dev targets, zero regressions
- [0067 — Google Forms handler](0067-google-forms-handler.md) — ss-form questions + choices: +0.0005 F1 both dev targets
- [0068 — CPAN POD handler](0068-cpan-pod-handler.md) — byte-exact; engine triage lane drained: 5 cycles, +0.0055 F1 golden, zero regressions
- [0069 — NEGATIVE census: title rescue](0069-NEGATIVE-title-rescue-census.md) — 99 dropped titles, best gate 57% gold-keep; wall #14, format bucket wall-bound
- [0070 — oneComment theme](0070-onecomment-theme.md) — WP rebuild selectors: 13KB comments recovered, +0.0005 both dev targets
- [0071 — Social-plugin comments](0071-social-comments.md) — golden crosses 0.84 F1; 8th straight zero-regression ship
- [0072 — Related-section strip](0072-related-section-strip.md) — guarded post-pass, prototyped-then-ported: +0.0003/+0.0005 dev, zero craters
- [0073 — Golden v4/v5](0073-golden-v4-v5.md) — 3 gold repairs + owner timeline ruling swept fleet-wide: golden 0.8424/0.7546
- [0074 — Golden v6 + widget negative](0074-golden-v6-husk-review.md) — husk review narrows 5 (range-overshoot hazard); golden 0.8453/0.7572; widget/vB-alias reverted
- [0075 — Safe batch](0075-safe-batch.md) — strips + Blogspot widget subset: golden 9-up-0-down; first golden-primary divergence logged
- [0076 — LiveJournal handler](0076-livejournal-handler.md) — b-singlepost rebuild, probe 0.19→0.98
- [0077 — Legacy guestbook](0077-legacy-guestbook.md) — condolence rebuild, probe 0.21→0.91; golden-primary divergence #2
- [0078 — Runtime checkpoint](0078-runtime-checkpoint.md) — 2.90 ms/doc (+6%/5 cycles, logged); engine pool closed; triage-verification policy
- [0079 — NEGATIVE: domain conventions](0079-NEGATIVE-domain-conventions.md) — walls are per-doc annotator variance, not site style; last cheap arbitration idea closed
