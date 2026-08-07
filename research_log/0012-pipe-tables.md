# 0012 — Markdown pipe tables

- **Date:** 2026-08-07
- **Tag:** 0012-final2 (baseline compared against: 0011-reverted = 0010)
- **Status:** landed

## Hypothesis

Gold renders data tables as pipe tables (61 dev docs with separator rows);
our output had *already lost cell boundaries* (cells glue with spaces — no
per-cell nodes). Per-cell nodes + a data-table gate (jusText-0051 uniform-
table lesson) + a pipe-row serializer should recover table structure, mostly
on Lev, and must pass the render check (playbook §6).

## What changed

- Markdown mode creates ExtractNodes per `<td>/<th>`; a serializer pre-pass
  marks *eligible* tables (≥2 rows, ≥2 cells in a row, no nested table, no
  cell over 300 content bytes — cell text accumulated across nested block
  nodes, or IMDB-style layout-table bios slip through) and pipe-serializes
  them: `| cell | cell |` rows, `| --- |…` separator after row 0, margins
  inside rows flushed as spaces (a `<br>` inside a cell must not break the
  row), blank lines around the table.
- Rescue-gate hardening this surfaced (three iterations, all measured):
  1. Gates now measure content length excluding `|#*` — raw-length gates let
     table *syntax* push a page across the near-empty boundary. (Excluding
     whitespace/dashes recalibrated every gate and regressed guardrails —
     reverted to the minimal charset.)
  2. Duplication-guard probes abstain when the probe is table content
     (calendars legitimately repeat rows). A probe-selection rewrite that
     preferred prose lines changed plain-mode behavior and regressed a
     guardrail doc — reverted for the minimal abstention instead.

## Results

| set | 0010/0011 | 0012 | per-doc |
|---|---|---|---|
| lpv11 dev F1 | 0.7811 | **0.7819** | 15↑ / 12↓ (worst −0.031) |
| lpv11 dev Lev | 0.6705 | **0.6712** | has_table subset: F1 +0.002, Lev +0.001 |
| lpv11 train F1 | 0.7756 | **0.7763** | 43 > +0.05 vs 4 < −0.05 |
| general dev / dev2 | — | — | **0 / 0 changed docs** (plain mode untouched) |

**Render check: 105/109 predictions with pipe tables render as `<table>`**
via python-markdown (was 27/124 before the blank-line/multi-line-cell fixes;
residual 4 are empty-cell calendar grids and renderer strictness). Speed
1.343 ms/doc (≈0009 level). Golden tests pass.

Accepted casualty (logged): one train calendar page (metrorock, −0.83) whose
old score came from a rescue that swapped in nav junk; under content-length
gates its rescue no longer fires. Calendar grids are junk tables — a future
cycle should veto mostly-empty-cell tables at eligibility time.

## Insights

- **Formatting features now interact with rescue gates**: any change that
  adds output bytes can move a page across a size-gated boundary. The
  content-length gate decouples this permanently — worth remembering for
  fences/links.
- The blank-line-before-table and no-newlines-inside-rows requirements are
  where naive table emitters die; the render check caught both immediately
  (27% → 96%).
- Minimal-blast-radius discipline paid off twice: both "smarter" variants
  (whitespace-excluding gates, prose-probe selection) regressed guardrails
  and were replaced by narrower fixes with identical intended effect.

## Next

- Code fences (30 gold docs) — markdown v2 remainder.
- Calendar/empty-cell veto for table eligibility.
- Forum chrome family (48 OVER docs) — still the largest untouched bucket.
