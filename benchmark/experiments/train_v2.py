#!/usr/bin/env python
import glob, numpy as np, joblib
from sklearn.metrics import roc_auc_score
Xs, ys, gs = [], [], []
for f in sorted(glob.glob("benchmark/experiments/big_cache/chunk_*.npz")):
    d = np.load(f)
    Xs.append(d["X"]); ys.append(d["y"]); gs.append(d["groups"])
X = np.concatenate(Xs); y = np.concatenate(ys); g = np.concatenate(gs)
print(f"blocks={len(X)} docs~{len(np.unique(g))}", flush=True)
cut = np.percentile(g, 80)
tr = g < cut; te = ~tr
yb = (y > 0.5).astype(int)

from sklearn.ensemble import HistGradientBoostingRegressor
h = HistGradientBoostingRegressor(max_iter=300, max_depth=6, learning_rate=0.08)
h.fit(X[tr], y[tr])
ph = h.predict(X[te])
print(f"HistGBM-300 (ceiling): AUC {roc_auc_score(yb[te], ph):.4f}", flush=True)
for thr in (0.05, 0.1):
    v = ph < thr
    if v.sum(): print(f"  veto@{thr}: {v.mean():.2%} cov, {yb[te][v].mean():.2%} fv", flush=True)
for thr in (0.85, 0.9):
    k = ph > thr
    if k.sum(): print(f"  keep@{thr}: {k.mean():.2%} cov, {(yb[te][k]==0).mean():.2%} fk", flush=True)

# exportable: GBR on 3M subsample of train side
from sklearn.ensemble import GradientBoostingRegressor
rng = np.random.RandomState(0)
idx = np.where(tr)[0]
sub = rng.choice(idx, size=min(3_000_000, len(idx)), replace=False)
m = GradientBoostingRegressor(n_estimators=60, max_depth=5, learning_rate=0.15, subsample=0.7)
m.fit(X[sub], y[sub])
pm = m.predict(X[te])
print(f"GBR n60d5 @3M: AUC {roc_auc_score(yb[te], pm):.4f}", flush=True)
for thr in (0.05, 0.1):
    v = pm < thr
    if v.sum(): print(f"  veto@{thr}: {v.mean():.2%} cov, {yb[te][v].mean():.2%} fv", flush=True)
for thr in (0.85, 0.9):
    k = pm > thr
    if k.sum(): print(f"  keep@{thr}: {k.mean():.2%} cov, {(yb[te][k]==0).mean():.2%} fk", flush=True)
joblib.dump(m, "benchmark/experiments/gbm_v2_n60d5.joblib")
print("saved", flush=True)
