#!/usr/bin/env python
"""HistGBM ceiling check on the full 12M v3 cache (cycle 0052)."""
import os, numpy as np
OUT = "benchmark/experiments/big_v3_cache"
Xs, ys, gs = [], [], []
for fn in sorted(os.listdir(OUT)):
    if not fn.startswith("chunk_"): continue
    d = np.load(f"{OUT}/{fn}")
    Xs.append(d["X"]); ys.append(d["y"]); gs.append(d["groups"])
X = np.concatenate(Xs); y = np.concatenate(ys); groups = np.concatenate(gs)
cut = int(groups.max() * 0.9)
tr = groups < cut; te = ~tr
from sklearn.ensemble import HistGradientBoostingRegressor
from sklearn.metrics import roc_auc_score
for it, d in [(300, 6), (500, 8)]:
    m = HistGradientBoostingRegressor(max_iter=it, max_depth=d, learning_rate=0.08)
    m.fit(X[tr], y[tr])
    pred = m.predict(X[te])
    yb = (y[te] > 0.5).astype(int)
    print(f"Hist-{it}d{d}: AUC {roc_auc_score(yb, pred):.4f}", flush=True)
