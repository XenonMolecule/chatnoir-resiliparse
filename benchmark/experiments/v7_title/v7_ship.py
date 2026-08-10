import numpy as np, joblib
from sklearn.ensemble import GradientBoostingRegressor
from sklearn.metrics import roc_auc_score
d = np.load('/tmp/v7_cache/rustexact.npz'); X, y = d['X'], d['y']
cut = int(0.8*len(y)); ybin = (y[cut:] >= 0.8).astype(int)
m = GradientBoostingRegressor(n_estimators=120, max_depth=6, random_state=7)
m.fit(X[:cut], y[:cut])
print('v7 ship-candidate AUC', round(roc_auc_score(ybin, m.predict(X[cut:])), 4), flush=True)
joblib.dump(m, '/tmp/v7_cache/gbr_v7_ship.joblib')
# control on identical rows, for attribution if the battery fails
c = GradientBoostingRegressor(n_estimators=120, max_depth=6, random_state=7)
c.fit(X[:cut, :66], y[:cut])
print('v5 control AUC', round(roc_auc_score(ybin, c.predict(X[cut:, :66])), 4), flush=True)
joblib.dump(c, '/tmp/v7_cache/gbr_v5_ctrl.joblib')
print('BOTH_DONE')
