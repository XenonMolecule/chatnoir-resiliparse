# 0170: title-relationship model features — AUC +0.0080, live +0.0000

Run on a cloud VM (see VM_SETUP.md; binary parity vs local verified, 0
per-doc diffs). The one substantive autonomous lane left: give the block
model a feature it has never had. Cycle 0161 showed the page `<title>`
encodes PURPOSE, which is the signal four separate walls were missing
(link 0137, image 0011/0113, form 0160).

## Features
`title_overlap` = |block tokens ∩ title tokens| / |block tokens|
`title_cover`   = |block tokens ∩ title tokens| / |title tokens|
`title_len`     = ln(1 + |title tokens|)
Tokens are distinct FNV-1a hashes of whitespace-delimited lowercased runs —
**byte-identical to the block scanner's own hashing**, so train and serve
cannot drift.

## Two skews found and fixed BEFORE training the shipped candidate
1. **Entity decoding**: a regex over raw HTML sees `AT&amp;T`, the DOM sees
   `AT&T`. Measured: **17.3% of docs** have titles where decoded ≠ raw.
2. **Tokenizer**: the screens used Python `\w+`; the Rust scanner splits on
   WHITESPACE (punctuation retained). This mismatches nearly every
   sentence-final word.
Both were corrected by re-extracting (three passes total) rather than
patched around. Skew of this kind produces a model that validates well and
underperforms live — the exact failure this cycle was hunting.

## Offline result (strongest in the model program's history)
| model | AUC |
|---|---|
| v5 control, 10k docs | 0.8562 |
| **v7 +title, 10k docs** | **0.8637** (+0.0075) |
| v7 with Rust-exact tokens (screen) | 0.8717 vs 0.8637 control (+0.0080) |
Title features ranked **6th and 11th of 69** by importance.

## Live result — the controlled decomposition
| build | dev_golden F1 | Lev |
|---|---|---|
| shipped v5 (trained on 100k docs) | **0.89468** | 0.81528 |
| v5 control (same features, 10k docs) | 0.88529 | 0.80434 |
| v7 +title (10k docs) | 0.88527 | 0.80291 |

- **data budget effect** (10k vs 100k, features unchanged): **−0.00939 F1**
- **title-feature effect** (identical rows): **−0.00002 F1**

The entire loss is the training-set reduction. The title features are
live-**neutral**: +0.0080 AUC bought exactly nothing. This is the seventh
AUC/live divergence (0056 was the first) and by far the cleanest — a
perfectly controlled A/B where a large, well-founded, importance-confirmed
offline gain converts to zero.

## Two consequences
1. **The lane is not dead, it is BLOCKED ON DATA.** A 100k-doc retrain with
   these features is the obvious next experiment; the corpus lives in the
   sibling jusText repo and did not travel. Pipeline + trained model are
   preserved under `benchmark/experiments/v7_title/`.
2. **Tension with 0027**, which concluded "10× data → AUC +0.001, live
   wash — features are the frontier, not rows". Here 10× LESS data costs
   only 0.003 AUC but **0.0094 live**. Data matters live far more than AUC
   suggests, and AUC understates it in both directions. 0027's conclusion
   should be re-tested before being relied on again.

Extractor unchanged: shipped v5 model restored, parity re-verified
(0.89468/0.81528, 0 per-doc diffs), tests 7/7.
