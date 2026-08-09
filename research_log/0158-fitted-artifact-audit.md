# 0158: fitted-artifact audit round 2 — thresholds and UL gates hold

After 0156/0157 showed two fitted tables had decayed (+0.0017 combined),
re-tested the remaining ones on the current golden v16 basis.

**Model thresholds** (last swept 0126 on v12):
| veto/keep | F1 | Lev |
|---|---|---|
| **0.40/0.60 (ship)** | **0.89354** | **0.81387** |
| 0.35/0.60 | 0.89101 | 0.81056 |
| 0.45/0.60 | 0.88712 | 0.80681 |
| 0.40/0.55 | 0.89264 | 0.81181 |
| 0.40/0.65 | 0.89237 | 0.81241 |

Still dominant on both metrics — this artifact did NOT decay, so the
decay hypothesis is specific to tables that encode per-DOMAIN verdicts
(which the gold audit directly invalidated), not to global operating
points.

**UL_EXEMPT gates** (cycle 0005, never swept until now):
min-text 600 = 1000 (flat), 1500 worse; per-item 100 = 150 (flat), 200
worse; link-ratio 0.4 = 0.5 = 0.6 (flat). The shipped values sit on a
plateau whose only gradient is downward — 13 years of forum-list golds
apparently do not discriminate finer.

No changes shipped; source restored byte-identical, tests 7/7. Every
numeric constant and fitted table in the extractor now carries a
measurement against golden v16.
