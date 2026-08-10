# 0171: table emitter correctness — empty cells, delimiters, colspan

New phase: specialized-domain fidelity (code/math/science/table sets from
the jusText benchmark) + 5 external benchmarks. Goal: F1>0.8 everywhere,
priority marin devset + lpv11, NO major errors.

## Domain audits (4 Opus agents, 18 dev docs; full reports in task outputs)
- **code: CODE_INTACT 11/11** — 157 fenced blocks, zero truncation/mangling;
  we are repeatedly MORE faithful than the LLM golds (which silently fix
  page bugs). Open items: highlighter label leaks into fences (shiki),
  blank lines inside per-line-div code dropped, Q&A vote/user-card chrome.
- **math: MATH_INTACT 2/2** — 26/26 LaTeX spans byte-exact; found the
  StackExchange out-of-container H1 title loss (confirmed, reproducible).
- **science: SCIENCE_INTACT 3/3** — zero deletions; gold copy-edits, we
  don't. Found empty-bold `****` artifact; sup/sub flattening flagged.
- **table: TABLE_INTACT (data) 2/2, syntax BROKEN** — 530 cells, zero value
  loss, but trailing-empty-cell pipe pop truncated rows and killed the
  |---| delimiter for whole tables (exeter unparseable as GFM).

## Fix shipped (this cycle)
`</tr>` handler: only fully-empty rows are dropped; last-cell-empty rows
keep shape (`| Cooling: |  |`); the |---| delimiter attaches to the first
REAL row; `colspan` expands to empty cells (cap 12) so column geometry is
true. Interaction caught by battery: 0107's "Similar Threads" strip
matched only the exact single-cell form — now matches the expanded row
(bimmerwerkz −0.16 recovered). Perf: 0161 form gate reordered so title
vocab gates the DOM queries.

## Battery (vs 0168 baseline)
golden F1 **0.89468** (exact), Lev 0.81520 (−0.0001, pure-Lev byte cost of
now-correct trailing empty cells); dev/golden zero both-downs; train F1
0.81643 (−0.00002), one −0.096 rescue-gate flip (viz.com, documented
instability class). Plain 1000/1000 identical; tests 7/7. Exeter league
table now emits a valid 20-column GFM table with delimiter.

Runtime: measurements currently contaminated by the 120k training pinning
a core; re-measure when it completes. The changes are O(cells).
