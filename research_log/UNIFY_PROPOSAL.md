# Convention-Unification Proposal (prepared cycle 0093; awaiting owner)

The largest remaining loss component is per-document annotator variance
in PRESENTATION conventions (content identical, formatting coin-flips —
0079/0092 measurements). Each ruling below unifies one convention
across the golden devset mechanically (build_golden step, reversible,
docs flagged) AND pins the extractor to the same convention. Rulings
are independent — approve any subset ("yes to C1, C3").

## C1 — List-item hard breaks  [~131 docs, 3,576 chars]
Gold is 46% `- item  ` / 54% `- item` (per-doc consistent, no source
signal). PROPOSAL: unify to BARE `- item` (markdown-canonical; our
emission). Alternative: unify to hard-break — equal validity, tell me
which.

## C2 — Byline dash  [~96 docs, 1,022 bylines]
`**author — date**` vs `**author – date**` split 61/39. PROPOSAL:
unify to EM-DASH (our handler majority; more common in gold).

## C3 — Leading page title  [~64 docs]
Some golds begin with `# Title`, others with body text, for identical
page types. PROPOSAL: where the page has a real h1 AND our extraction
leads with it, gold gains the `# title` line (title-keeping becomes
the norm). Conservative alternative: leave as-is (variance stands).

## C4 — Blockquote prefix  [~95 docs]
Gold quotes 41% `> `-prefixed / 59% plain. PROPOSAL: unify to PLAIN
(majority; matches our emission). Alternative: `> ` everywhere +
extractor emits `> ` for blockquote elements.

## Estimated combined effect if all approved as proposed
Roughly +0.010-0.020 Lev and +0.003-0.008 F1 on golden (variance
converted to agreement), plus the same conventions locked into the
extractor so future comparisons stay clean. Each ruling ships with
before/after samples in the review viewer.
