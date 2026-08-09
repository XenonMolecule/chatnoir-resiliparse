# 0161: form emission gated on page PURPOSE — golden 0.89423/0.81453

Direct answer to the 0160 negative. DOM density cannot distinguish an
article page with a comment form from a form page with prose — but the
page **`<title>`** encodes purpose, and it is available to a static
extractor:

| | title | verdict |
|---|---|---|
| read718 | "Register A Child \| Read 718" | form page |
| ornaross | "Orna Ross \| Checkout" | form page |
| bimmerfest | "Bimmerfest — Send Page to a Friend" | form page |
| docs.servicenow | "Create vendor credit from an incident" | article |
| hardwarezone | "GeForce GTS 450 Roundup Part 2 …" | article |

Gate: >=5 `<input>` AND a title matching a small purpose vocabulary
(register/registration, sign up, checkout, contact us, apply now,
application/order form, send page/message). "survey" was in the first cut
and removed — it cost surveymonkey −0.33 on train.

## Battery (`0161-w2` vs `0157-w1`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.89423** (+0.0007) | **0.81453** (+0.0007) | **up 3 / down 0** |
| dev | 0.85700 | 0.76617 | — |
| train | **0.81632** (+0.0001) | **0.71815** | up 5 / down 8, ups far larger (lucasforums +0.344, ramforumz +0.149) |

read718 +0.470, bimmerfest +0.139, ornaross +0.081. Plain 1000/1000,
tests 7/7.

## Why this matters beyond the score
Four walls (link selection 0137, image emission 0011/0113, form emission
0160) all failed for the same reason: the decision depends on the page's
PURPOSE, which DOM structure does not encode. This cycle shows purpose is
partially recoverable from the title — a signal none of those lanes tried.
The same probe is worth applying to the link/image walls before concluding
they need a learned model.
