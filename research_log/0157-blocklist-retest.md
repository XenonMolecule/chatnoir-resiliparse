# 0157: blocklist re-test on current gold — golden 0.8935/0.8139

Follow-on to 0156's finding that fitted exclusion tables decay. The
site-rule blocklist held 48 domains, most burned at golden v11/v12 — before
four gold-edit batches and the owner's rulings. Their rules were recovered
from the wave JSON files (176 vetoes + 26 whitelists across 44 domains) and
re-A/B'd wholesale, then filtered to the net-positive set.

Re-admitted (8): **foodily.com** (+0.308 golden, **+0.629 train**),
happysadlola (+0.178), encycolorpedia (+0.117), newstral (+0.045),
query.nytimes, cafemom, thorax.bmj, cheatmasters.
Still burned (36): glassdoor −0.487, failblog −0.759, detroitnews −0.681,
bikeforums −0.614, lifeconfusions −0.737, docme −0.524, hitvibz −0.483,
kesq −0.411, parlinfo −0.234, greshamlodge −0.228, omdb −0.217 …

## Battery (`0157-w1` vs `0156-w1`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.89354** (+0.0007) | **0.81387** (+0.0007) | **up 8 / down 0** |
| dev | 0.85633 (−0.0005) | 0.76552 | 5 both-down — happysadlola's ORIGINAL gold keeps the blogroll our rules cut (0093 zero-sum); golden-primary |
| train | **0.81622** (+0.0001) | **0.71810** | up 2 / down 2 |

Plain 1000/1000, tests 7/7.

## Combined stale-config lane (0156 + 0157)
golden 0.89184 → **0.89354** (+0.0017) and Lev 0.81218 → **0.81387**
(+0.0017) from re-testing two fitted tables — more than the last eight
extractor cycles combined, at near-zero cost. **Any fitted artifact should
be re-A/B'd whenever the basis it was fitted against moves.**
