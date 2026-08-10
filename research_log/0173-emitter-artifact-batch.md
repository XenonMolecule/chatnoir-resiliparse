# 0173: emitter artifact batch (audit findings) — golden 0.89481/0.81561

Four fixes from the domain audits, batteried as a batch then individually:

1. **Shiki language-label leak** (code audit: 42/45 fences on the TS release
   notes had "ts" as a content line). `pre .language-id` divs blacklisted;
   their text becomes the fence language (```` ```ts ````) via
   fence_language.
2. **Orphaned separator runs** (math audit): dropped rel=tag anchors left
   "**Topics:** , ," — trailing comma-only runs stripped.
3. **Malformed-emphasis lines** (science audit): bold elements spanning
   block boundaries strand markers in empty blocks -> bare "****" lines
   (8 on jpost). Lines of only asterisks stripped; "***" (hr) preserved.
4. **Citation-marker sup fusion** (my GeneCards finding): <sup><a>1</a></sup>
   reference markers fused onto identifiers (TEL1 -> "TEL11", 28 tokens on
   the ATM gene page). Only sup-wrapping-short-numeric-anchor is vetoed;
   bare <sup> stays (0124: gold mirrors source rendering — verified: PubMed
   "Mayhew PJ1" unchanged).

## Battery
golden **0.89481/0.81561** (Lev +0.0002 over 0172; artifact strips up 45 /
down 6, sup change zero-effect on golden), train up 487 / down 139 (pure-Lev
docs whose original golds keep asterisk separators; zero both-down), science
dev Lev +0.0015, code/math small Lev gains, plain 1000/1000, tests 7/7.

## Known limitations logged, deliberately not attempted
- Blank lines inside per-line-div highlighter code (shiki/gist) are still
  collapsed — fixing requires margin-machinery changes inside <pre>; the
  content itself is intact.
- <sub>/<sup> flatten without markers on plain pages (matches lpv11 gold
  convention; chem-heavy pages untested — science audit's reservation).
