# 0164: tier-1b rescue band — NEGATIVE, reverted

greshamlodge sits at 259B of output against a 7.2KB body (ratio 28) — just
outside BOTH tier-1 gates (needs <200B AND ratio >30). Added a second band:
200-450B with a stricter ratio (>25).

Result: golden 0.89438 → 0.89220, **4 both-downs**, and — decisively —
**greshamlodge itself lost 0.228**. Train had 11 both-downs including
−0.87 and −0.76 craters.

Why: above ~200B the filtered output is already the article, and the
unfiltered fallback is the whole site shell. The 200B gate is not
arbitrary — it is the point below which "we produced almost nothing" is
better evidence than "the shell contains more text". A doc can be
clean-but-truncated (greshamlodge) without the fallback being the fix; its
missing content needs a targeted container, which is unreachable because
the domain does not resolve (0163).

Reverted; parity verified. This closes the rescue-ladder lane: both gates
have now been swept (0140) and extended (0164) with measurements.
