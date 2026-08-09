# 0146: trailing-» strip — NEGATIVE, reverted

Char census (0145) showed '»' in 47 of our docs vs 11 golds, and a
pred-only line census looked unambiguous: "Next »", "more »", "Read Full
Post »", "Apply for this Job »" — UI affordances. Gold has exactly ONE
such line corpus-wide. Stripped short lines ending in '»'.

Battery killed it: golden 4 both-downs, dev F1 0.85686 → 0.85669, train
F1 0.81612 → 0.81599 with **77 both-downs and a −0.32 crater**
(chinadigitaltimes). Cause: **'»' is a closing quotation mark in French,
German and Russian typography** — the rule cut the last line of quoted
prose on non-English pages (partitionsdechansons, texasmonthly quote
blocks, chinadigitaltimes).

Lesson for the ledger: punctuation-shaped chrome rules must be checked
against non-English typography before shipping; an English-only reading of
a character is a hidden locale assumption. Reverted, parity verified.
