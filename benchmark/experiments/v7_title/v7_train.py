import numpy as np
from sklearn.ensemble import HistGradientBoostingClassifier
from sklearn.metrics import roc_auc_score
d = np.load('/tmp/v7_cache/v7.npz')
X, y = d['X'], d['y']
print(f'{X.shape[0]} blocks, {X.shape[1]} features, {y.mean():.3f} positive rate')
# doc-independent split is unavailable (rows unlabelled by doc); use a
# contiguous split so blocks from the same doc stay together.
cut = int(0.75 * len(y))
Xtr, Xte, ytr, yte = X[:cut], X[cut:], y[:cut], y[cut:]
res = {}
for name, cols in [('v5 baseline', slice(0, X.shape[1]-3)), ('v7 +title', slice(0, X.shape[1]))]:
    clf = HistGradientBoostingClassifier(max_iter=120, max_depth=6, random_state=7)
    clf.fit(Xtr[:, cols], ytr)
    p = clf.predict_proba(Xte[:, cols])[:, 1]
    auc = roc_auc_score(yte, p)
    res[name] = auc
    print(f'{name:14s} AUC {auc:.4f}')
print(f'delta from title features: {res["v7 +title"] - res["v5 baseline"]:+.4f}')
