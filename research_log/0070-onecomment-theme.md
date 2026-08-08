# 0070 — WP oneComment theme selectors

- **Date:** 2026-08-08
- **Tag:** 0070-onecomment (baseline: 0068-cpan)
- **Status:** landed — zero regressions

## What changed
WP comment-rebuild selector set extended for the oneComment theme
(legalinsurrection family): `div.oneComment` items, `.commentAuthorLink`
author, `.commentAuthor a` date, `.commentContent` body. Rebuild fires
under the existing native-first gate; 13KB of attributed comments
recovered on the probe doc.

## Results
golden **0.8399/0.7518** (+0.0005/+0.0005) · original dev
**0.8273/0.7311** (+0.0005/+0.0005) · train +1 doc. Zero regressions;
general plain 1000/1000; goldens pass.
