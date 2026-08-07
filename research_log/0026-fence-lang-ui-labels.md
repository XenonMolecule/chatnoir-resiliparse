# 0026 — Fence language tags + dangling UI-label filter

- **Date:** 2026-08-07
- **Tag:** 0026-uilabels2 (baseline compared against: 0025-final)
- **Status:** landed

## What changed
- **Fence languages**: ```lang from `language-x`/`lang-x`/`brush: x` class
  hints on `<pre>`/first `<code>` child (hint-only, no sniffing); fence-open
  newline guard rewritten as a serializer flag. Metric-flat, quality-correct.
- **Dangling UI labels** (jusText-0084 family): the Lev-residual audit on
  content-correct docs showed the largest bucket (42% of diff mass) is
  per-comment widget chrome. A curated exact-match line filter (Reply / Like
  / Report / Post a Comment / notify-me boilerplate…, list-marker-tolerant,
  "Author"/"Comments"/"Quote" deliberately excluded) drops them post-
  serialization, with blank-run collapse.

## Results
dev **+0.0003 F1 / +0.0005 Lev, 109↑ / 15↓ (worst −0.016)**; train
+0.0003/+0.0004 (worst −0.119, 3 docs < −0.05). Goldens pass; markdown-only.

## Residual-audit map (banked for next Lev cycles)
Diff mass on F1≥0.95 docs: small-text chrome 42% (this cycle), md-syntax/ws
20%, content chunks 20%, links/urls 9%, typography 8%.

## Next
- Model v2 (big_train, 12M blocks) training in background.
- Typography family (curly quotes/dashes) and md-links remain from the map.
