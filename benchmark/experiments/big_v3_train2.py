#!/usr/bin/env python
"""Round 2 (cycle 0052): n120d6 on 5M samples from the v3 big cache."""
import os, numpy as np
OUT = "benchmark/experiments/big_v3_cache"
Xs, ys, gs = [], [], []
for fn in sorted(os.listdir(OUT)):
    if not fn.startswith("chunk_"): continue
    d = np.load(f"{OUT}/{fn}")
    Xs.append(d["X"]); ys.append(d["y"]); gs.append(d["groups"])
X = np.concatenate(Xs); y = np.concatenate(ys); groups = np.concatenate(gs)
print(f"total blocks={len(X)}", flush=True)
rng = np.random.default_rng(1)
cut = int(groups.max() * 0.9)
tr_idx = np.flatnonzero(groups < cut)
te_mask = groups >= cut
if len(tr_idx) > 5_000_000:
    tr_idx = rng.choice(tr_idx, 5_000_000, replace=False)
from sklearn.ensemble import GradientBoostingRegressor
from sklearn.metrics import roc_auc_score
m = GradientBoostingRegressor(n_estimators=120, max_depth=6, learning_rate=0.1,
                              subsample=0.7, verbose=1)
m.fit(X[tr_idx], y[tr_idx])
pred = m.predict(X[te_mask])
yb = (y[te_mask] > 0.5).astype(int)
print(f"GBR n120d6 v3 @{len(tr_idx)}: AUC {roc_auc_score(yb, pred):.4f}", flush=True)
import joblib
joblib.dump(m, "benchmark/experiments/gbm_v3big_n120d6.joblib")
print("saved", flush=True)
