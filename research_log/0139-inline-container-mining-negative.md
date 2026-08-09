# 0139: inline container mining on the worst OVER docs — NEGATIVE

Scripted (no agents, per owner token constraint): for the 8 worst
over-extraction docs not blocklisted, ranked every div/section/td/ul/span
by "contains >=2 pred-only junk lines AND zero gold lines", excluding rows
already in the tables. Result:

| doc | F1 | minable containers |
|---|---|---|
| menstennisforums | 0.046 | only .tborder/.container/.page (generic layout) |
| iclassifiedsnetwork | 0.087 | none |
| typekit | 0.243 | only #content |
| lafollettepress | 0.254 | none |
| mydd | 0.325 | none |
| theday | 0.326 | only #trueContainer |
| newhampshire | 0.359 | only bootstrap .col-* grid classes |
| eslflashcards | 0.381 | none |

Every survivor is a generic layout wrapper that also carries content on
sibling pages — exactly the class of rule the GENERIC filter has always
rejected and that produced the wave craters (hitvibz, failblog, kesq).
**The deep tail has no clean containers left**: its junk is interleaved
with content inside shared wrappers.

This closes selector mining at the doc level, inline and cheaply. It also
independently confirms the band-79 census's "unfixable 62" count.
menstennisforums (F1 0.046) is not an extractor failure at all — it is
one of the three docs awaiting an owner gold ruling.
