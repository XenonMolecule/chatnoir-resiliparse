# 0165: corpus-verified zero-gold lexicon — golden 0.89457, train up 513

## Method change worth keeping
Previous lexicon batches came from taxonomies or inspection — hand-picked,
then battery-checked (and several were wrong: "Cite This Source" 0114,
"survey" 0161, the » family 0146). This cycle inverted it: compute every
short line we emit in >=3 docs that appears **ZERO times across all 1000
golds**. 31 candidates, all corpus-verified before writing any code.

Shipped 18: close, follow, pin it, subscribe, sign up, submit, rss feed,
archives, similar stories, join the discussion, email this article,
comments are closed., search:, i am looking for:, jump to: navigation
search, avatar, latest news, apply now.

## Battery (`0165-w1`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | 0.89438 → **0.89457** | 0.81476 → **0.81507** | up 63 / down 5 (1 both) |
| dev | 0.85700 → **0.85755** | 0.76617 → **0.76667** | up 58 / down 14 |
| train | 0.81632 → **0.81636** | 0.71815 → **0.71824** | **up 513** / down 171 |

Plain 1000/1000, tests 7/7. The train up-count is the largest of any
cycle this session — a genuinely generic lane.

## The one crater, and its mechanism
thepage.time.com lost 0.293: the strips shortened its output enough to
cross a RESCUE gate, and the fallback that fired is worse. Post-passes run
INSIDE extraction so gates measure stripped length (0035) — meaning any
line-strip can flip a rescue decision. This is the 0012/0036 instability
class resurfacing; it caps how far line-level work can be pushed without
making the gates strip-aware.
