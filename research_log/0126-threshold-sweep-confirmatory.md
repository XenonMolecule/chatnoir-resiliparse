# 0126: model-threshold sweep on golden v12 — CONFIRMATORY, no change

Hypothesis: veto 0.40 / keep 0.60 were tuned on the original noisy dev
(0046-0051 era); golden v12's cleaner labels might prefer a different
operating point. Swept the four ±0.05 neighbors plus one diagonal on
dev_golden:

| veto/keep | F1 | Lev |
|---|---|---|
| **0.40/0.60 (ship)** | **0.8881** | **0.8075** |
| 0.35/0.60 | 0.8859 | 0.8045 |
| 0.45/0.60 | 0.8819 | 0.8008 |
| 0.40/0.55 | 0.8874 | 0.8055 |
| 0.40/0.65 | 0.8869 | 0.8060 |
| 0.45/0.55 | 0.8813 | 0.7990 |

The shipped point dominates every neighbor on BOTH metrics — the optimum
is robust to the gold audit (golden edits removed chrome the model already
vetoes, so the decision boundary is unchanged). Model lane's cheap knob is
exhausted; what remains there is retraining with richer features/labels
(blocked on train-gold, owner-gated). Extractor untouched (parity verified,
0 changed docs); no version bump.
