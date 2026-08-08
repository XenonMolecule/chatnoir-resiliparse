# 0081 — NEGATIVE ×2: dup-content family (text + DOM approaches)

- **Date:** 2026-08-08
- **Tags:** prototype + 0081-v2; both reverted/unshipped

## Mid-band clustering context
242 mid-band docs fleet-diagnosed into families; dup-content (15 docs)
attacked first.

## Attempt 1: containment dedup (jusText 0030 port, text-level)
Best-algorithm version of the 0031 idea: dev +0.9 F1-milli sum but
train **−9.2/−14.1** across 503 both-down docs. Gold keeps rendered
duplicates on a large train family. Wall #5 SEALED for any text-level
dedup — third and final attempt.

## Attempt 2: DOM-gated .entry-summary veto (81% gold consistency!)
Offline census said 22/27 docs dedup the teaser — but live battery:
dev crater −0.57 (recordnet: the summary IS the article; entry-content
holds junk), train −0.89 ×2 (pages whose entire body renders inside
.entry-summary). The 81% census counted docs where BOTH classes carry
the same text; the veto also fires where they DIFFER — the live
distribution is the census's blind side. Reverted.

## Lesson
An offline gold-consistency census on the TRIGGER condition is not a
measurement of the ACTION's effect — the action fires on a wider set
than the census counted (fourth instance: 0047 bold, 0069 titles,
census-vs-action divergences). Only the full battery decides.
