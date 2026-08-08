# 0069 — NEGATIVE (census-only): dropped-title rescue

- **Date:** 2026-08-08
- **Status:** measured, not built

## Finding
99/678 gold-titled dev docs miss their `# title` in pred (49 from
header-region h1s our selection vetoes, 25 from <title>-only pages).
Cross-cutting family discovered via the pictureyear format diff — but
gate sweeps cap at **57% gold-keep** (no-#-in-pred + unique h1 +
title-tag confirmation; 25%→45%→50%→57% across gates). Gold keeps
page titles inconsistently: same signal, opposite conventions per doc.
Wall #14. No rule ships below ~75% under the zero-regression policy.

## Also parked from the format bucket
pictureyear at scale = en-dash (per-doc variance, wall #12) + non-BR
hard breaks (wall). mydd/bio-medicine = teaser+full dedup (wall #5).
greshamlodge/ukdefencejournal = image emission (0059: learned-only).
Format bucket effectively wall-bound; selection-other (11 docs) is the
remaining open pool.
