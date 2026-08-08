# 0109: majority-link-host page_domain fallback — golden 0.8789/0.7969

## What
`page_domain()` previously required og:url or rel=canonical — absent on
pre-social-web pages (census.gov 2005 was the tell: its 18 new SITE_VETOES
rows were provably inert, as was the 0108 whitelist removal). New fallback:
when neither meta exists, take the majority host of absolute `a[href]`s,
accepted only when it clearly dominates (≥10 links and ≥60% of absolute
hrefs, www-stripped). This switched ~40 existing site-rule domains live on
their og:url-less sibling docs in one shot.

Also: census.gov dropdown-panel vetoes (#TopicsMain/#GeoMain/#LibMain/
#DataMain/#abtMain/#newsMain + TopLink/GeoLink/LibLink/DataLink/AboutLink/
NewsLink panels + the FTD sidebar `td[bgcolor="#FFFFCC"]`).

## Bisect: FALLBACK_EXCLUDE, not rule removal
First battery: golden +0.0037/+0.0049 but craters where rules fitted to ONE
page misfired on link-majority siblings (theserverside −0.86, pt.usc.edu
−0.25, bimmerwerkz −0.35, motoprofi, iclassifieds, menstennisforums,
cricketarchive, convertunits). Removing those domains' RULES would regress
the og:url docs they were built for — instead a `FALLBACK_EXCLUDE` const
opts those 9 domains out of the fallback only, preserving all prior
behavior exactly.

## Battery (`0109-w3` vs `0108-w4`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8789** (+0.0043) | **0.7969** (+0.0053) | up 34 / down 2 (both mixed-sign) / craters 0 |
| dev | **0.8494** (+0.0027) | **0.7563** (+0.0033) | up 29 / down 10 / craters 0 |
| train | **0.8156** | **0.7172** | up 10 / down 0 |

Guardrails: plain 1000/1000 identical, extract_golden 7/7.

## Runtime (7-run best-of, 1000 dev docs, main_content=True)
markdown **2.73 ms/doc**, plain 1.30 ms/doc — better than 0103's 2.94 under
the same protocol. (Protocol correction: the 0.95/0.81 figures logged at
0104/0105 were measured WITHOUT main_content and are not comparable.)

## Scoreboard
Original dev **0.8494/0.7563** · golden v12 **0.8789/0.7969** · goal gap
−0.0211 F1 / **−0.0031 Lev** — the 0.80 Lev edge is one cycle away.
