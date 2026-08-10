# 0177: checkbox-widget veto — table dev 0.6983 → 0.8870, all domains ≥0.82

## What
Hard-blacklist (markdown config) for containers with 4+ direct-child
checkbox/radio inputs, guarded to fire only when every element child is
widget furniture (input/label/br). Kills picker widgets whose bare text
nodes between the inputs ("Alpine-91901<br>…") are emitted as prose —
discovernorthcounty's ZIP-area search widget was 63% of that doc's output.

## Why a blacklist, not an is_main_content_node veto (two dead ends)
1. The block model WHITELISTS the label run (plain, link-free text scores
   as content) and the traversal's whitelist check short-circuits
   is_main_content_node.
2. The near-empty rescue tiers re-enter extraction with
   main_content=false, skipping is_main_content_node entirely — the target
   doc's output came through a rescue tier. blacklisted_nodes is honored
   by every path.

## The battery-found guard (iconics crater, −0.61)
CSS-only tab widgets drive panes with radio state hacks: iconics'
`.tabs-wrapper` holds 4 `<input type=radio class=tab-head>` + label tab
heads + the DIV content panes as siblings. Blacklisting that parent wiped
the article body. Guard: veto only when ALL element children are
input/label/br — a pure picker qualifies, a tabs-wrapper (div panes)
never does. Post-guard: iconics byte-identical to 0175, target doc still
clean.

## Battery (`0177-w3` vs `0175-w1/w2`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| table/dev | **0.8870** (+0.1887) | 0.4620 (+0.1782) | target doc fixed |
| lpv11 dev_golden | 0.8948 (±0) | 0.8156 (±0) | up 0 / down 0 |
| lpv11 dev | 0.8577 (±0) | — | up 0 / down 0 |
| lpv11 train | 0.8166 (±0) | — | up 8 / down 2 / craters 0 (worst −0.018) |
| general dev (pf=true) | 0.8139 (±0) | — | up 0 / down 0 |
| code/math/science dev | unchanged | | |

Guardrails: plain parity 1000/1000 (min F1 = 1.0), extract_golden 7/7.

## First-ever specialized TEST split runs (all domains now measured)
code 0.7915 · math 0.5620 · science 0.5132 · table 0.8259 (F1). The low
math/science numbers are 2-doc splits dominated by LLM-gold rewriting:
27/32 (science) and 26/35 (math) gold paragraphs do not exist in the
source HTML at all. Recall of source-present content verified by direct
containment: math variance doc — everything present incl. all LaTeX
(gold's rewrite broke the probe, false alarm); GeneCards ATM — aliases
and summaries present. Only true miss: the publications reference list
(link-formatted `<ol>` of paper-title anchors) is taken by the list
link-cluster veto. Logged as known limitation — relaxing link-dense list
vetoes for reference appendices is the 0005 crater family; not worth it
for ~3 appendix lines on one doc.

## Config forensics (baseline hygiene)
0175-w1's general/dev was recorded with preserve_formatting=True (the
historical general config), NOT markdown — a markdown-config comparison
shows ~990/1000 phantom diffs (+0.04 F1 artifact). 0177 battery ran
general with the historical config; per-doc parity is exact. Memory
updated (eval-testing-traps): check config.extract_kwargs before
trusting cross-tag diffs; ad-hoc tests must import resiliparse._extract_rs.

## Scoreboard
lpv11 golden **0.8948/0.8156** (unchanged best) · domains dev: code
0.8445 · math 0.8240 · science 0.9333 · table **0.8870** — all ≥0.8.
