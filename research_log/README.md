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

lpv11/dev (raw HTML, markdown config since 0009): F1 **0.7959** / Lev **0.6941** (0022) · train 0.7911. Goal: 0.90 / 0.80–0.85 (GOALS.md). Baseline 0.7619/0.6540 (0001).
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
