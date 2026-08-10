# 0178: 120k model lane — both variants live-negative; shipped model stands

## What was tested
The queue's top lever: retrain the block model on the 120k corpus
(12.2M blocks from the 100k big_train + lpv11 + general rust-exact
caches; 14.0M train rows). Two variants, honest control design:
- **v7-120k** (69 features = shipped 66 + title_overlap/title_cover/
  title_len, via rust_title_plumbing.patch): holdout AUC **0.8657**
- **v5ctrl-120k** (identical 66 features to shipped, only more data):
  holdout AUC **0.8581**

Both beat the shipped model's AUC series. Export → patch → build →
full battery each (`0178-v7`, `0178-ctrl` vs `0177-w3`).

## Live results: both NO-GO
| variant | golden ΔF1 | golden per-doc | dev ΔF1 | train ΔF1 |
|---|---|---|---|---|
| v7-120k | **−0.0064** | up 117 / down 204 / **28 craters** (one −1.00 wipe) | −0.0067 | +0.0015 |
| v5ctrl-120k | **−0.0019** | up 54 / down 73 / **14 craters** | −0.0015 | −0.0001 |

Zero-regression policy: decisive fail for both. Reverted to the shipped
model; parity re-verified (0 changed docs on dev_golden vs 0177-w3,
extract_golden 7/7, plain parity 1000/1000).

## Reading
1. **AUC is not a go-signal** — now at its starkest: +0.0076 AUC over a
   control that itself beats shipped, and BOTH lose live.
2. The live losses concentrate in tag/listing pages (jalopnik/tag,
   blackamericaweb/tag, kowb1290/tags — the model-veto/keep frontier).
   The fixed thresholds (MODEL_VETO_THRESHOLD / BIG / KEEP) were
   calibrated to the SHIPPED model's score distribution; a 120k-trained
   model shifts that distribution, so every fixed operating point moves.
   **Queue note: a threshold recalibration sweep on the new model is the
   untested rescue** — grid the three thresholds against golden with the
   120k control model before declaring the data lever dead.
3. This also resolves the 0027-vs-0170 contradiction in 0027's favor at
   this scale: more data alone (control) does not transfer live under
   frozen thresholds.

## Runtime ledger (clean, post-training, idle CPU)
7-run best-of, 1000 lpv11 dev docs, local series: **markdown
main_content 3.00 ms/doc · plain 1.32 ms/doc** (0116 reference: 2.76 /
1.30). Cycles 0161–0177 cost +0.24 ms/doc (+8.7%) on markdown. Known
per-doc additions: 0172 title-restore (h1 query + full-output
normalization), 0177 input query. A perf pass is queueable if the drift
continues; the 3.03 reading logged during training was ~honest.

## Artifacts
Models: benchmark/experiments/v7_title/gbr_{v7,v5}_120k.joblib.
Training: v7_120k_train.py (AUC lines in /tmp/v7_120k.log). Patch still
applies cleanly post-0177. Batteries: benchmark/runs/0178-{v7,ctrl}.
