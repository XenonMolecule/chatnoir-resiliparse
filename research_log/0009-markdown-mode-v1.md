# 0009 — Markdown output mode v1 (headings, bold, bullets, hard breaks)

- **Date:** 2026-08-07
- **Tag:** 0009-markdown-v3 (baseline compared against: 0007-article)
- **Status:** landed — **markdown is now the goal-tracking config for lpv11**

## Hypothesis

Gold is markdown-flavored; a `FormattingOpts::Markdown` mode (`#` headings,
`**bold**`/`*italic*`, `- ` bullets, gold-matching structure) should move Lev
+0.01–0.04 with F1 ≈ flat.

## What changed

- New `FormattingOpts::Markdown` (opt-in; `preserve_formatting='markdown'`
  through PyO3 and `--preserve-formatting markdown` in the harness). Plain
  and minimal_html modes byte-unchanged (golden tests pass).
- `#`–`######` heading prefixes (via the existing element_text_prefix
  mechanism; end-tag clears a stale prefix from empty headings).
- `**`/`*` inline markers for b/strong/i/em, with three whitespace/balance
  guards: childless elements skipped (no end-tag event → would unbalance),
  closing marker placed before trailing whitespace, opener rotated past
  leading whitespace (`** word**` → ` **word**`), empty spans collapsed.
- `- ` bullets (vs `•`), top-level bullets in column 0 (nested +2 each).
- `<br>` → markdown two-space hard break (`"  \n"`) — **gated on the margin
  actually originating from a BR**; the ungated version applied it to every
  block boundary and measured *worse* (Lev −0.002 vs +0.002).

## Results

| set | 0007 (plain) | 0009 (markdown) | note |
|---|---|---|---|
| lpv11 dev F1 | 0.7805 | 0.7809 | flat as predicted; worst per-doc −0.012 (token side effects) |
| lpv11 dev Lev | 0.6677 | **0.6701** | per-doc 281↑ / 131↓ (worst −0.047: bolding where gold is plain) |
| lpv11 train F1 | 0.7753 | 0.7753 | exactly flat at 10k |
| lpv11 train Lev | 0.6611 | **0.6633** | |
| speed | — | +0.4% | free |

**Config decision:** lpv11 runs now use `--preserve-formatting markdown`
(the goal explicitly includes markdown formatting). The old-benchmark
guardrail sets (general dev/dev2) keep plain formatting — their gold is
plain text and markdown markers would be noise there. Output verified to
render as markdown (headings/bold/lists) on spot docs.

## Insights

- +0.002 Lev is the *syntax* payoff, roughly a third of the 0003 strip-test
  bound (+0.006) — the rest of gold's markdown is **unreachable from tag
  semantics alone**: gold bolds label-styled spans (`**Name:**`,
  `**BUY NOW $495**`) that are not `<b>/<strong>` in source, and renders
  some links as `[text](href)` (we drop targets). Those need
  content/CSS-signal heuristics and a links-as-markdown mode (queued).
- The doc diffs show the real Lev frontier is unchanged: content precision
  (nav junk in mid-F1 docs) and text normalization (`\u{a0}` vs `\u{202f}`
  typography, hard-break placement). Formatting was never going to be a step
  change (0003 predicted this); it's a foundation the table/fence work (v2)
  and the precision track build on.
- Blanket-vs-gated strikes again: the same hard-break feature flipped from
  negative to positive purely by gating on BR origin.

## Next

- Markdown v2: pipe tables (72 gold docs), code fences (30), links-as-
  markdown `[text](href)` experiment.
- Bold-coverage heuristics (dt/th/label/strong-styled spans) — check gold
  consistency first (playbook §7 warning).
- Precision track (OVER_EXTRACT 155 docs) — now the largest bucket.
