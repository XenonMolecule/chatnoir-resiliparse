# 0160: form-field emission — NEGATIVE at every gate tried, reverted

Gold renders form content on registration/checkout pages (read718's gold is
the registration form: labels, options, `______` blanks). Our config has
`form_fields=false`.

| configuration | golden F1 | verdict |
|---|---|---|
| baseline (off) | **0.89354** | ship |
| globally on | 0.88661 | 12 docs up, **172 down** |
| gated: inputs>=6 & labels>=4 | 0.89254 | 55 golden downs, 659 train |
| gated: inputs>=20 & labels>=4 | 0.89303 | 30 golden downs, 355 train |

Winners are real and large (read718 +0.47, bimmerfest +0.14, ornaross
+0.08) but no DOM-density gate separates them: comment forms, faceted
search and newsletter widgets give ordinary article pages the same
input/label counts. docs.servicenow (−0.60) and henneth-annun (−0.60) are
article pages with big forms; read718 is a form page with big articles —
the counts are identical, only the INTENT differs.

Same shape as the link/image selection walls (0011/0113/0137): the signal
needed is "is this form the page's purpose", which DOM density does not
encode. Reverted; parity verified vs 0157.
