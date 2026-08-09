# 0153: phpBB handler gate widened — golden 0.89184/0.81218, 0 downs

Second mis-gated-handler find (after 0150). `extract_phpbb` required
`body#phpbb` — the stock theme's marker. Census: **29 of the 36 phpBB-markup
docs in dev are custom themes without it**, so the handler never ran on
them. Added an alternative entry condition: >=2 `.postbody` AND >=2
`.postauthor/.postdetails/.postprofile`.

Battery: golden **up 1 / down 0** (canadiangardening +0.069), train **up 3 /
down 0** (oscarfish +0.071, textkit +0.062, 14ers +0.022). Plain 1000/1000,
tests 7/7.

Most of the other 28 gated-out docs are claimed earlier in the handler
chain (vBulletin) or lack two profile blocks — the gate widening is
deliberately conservative.

## Pattern worth generalizing
Twice now a correct, existing handler was simply not reached: WP comment
rebuild (mis-gated on missing authors, 0150) and phpBB (mis-gated on a
theme marker, 0153). Handler ENTRY CONDITIONS are a systematically
under-tested surface — worth auditing the rest.
