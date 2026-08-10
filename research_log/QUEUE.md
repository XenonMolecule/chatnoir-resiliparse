# Queue (updated cycle 0170)

State: golden v16 **0.89468 F1 / 0.81528 Lev**. Lev goal (0.80–0.85) met
since 0110 and holding. F1 gap to 0.90: **−0.0053**.

## Highest value — needs the sibling repo (blocked on this VM, trivial locally)
- **v7 title features on the 100k corpus.** 0170 showed the features are
  live-neutral at a 10k training budget, but that the budget itself costs
  −0.0094 live. Retrain `benchmark/experiments/v7_title/v7_ship.py` against
  `../jusText/benchmark/datasets_rawhtml/lpv11/big_train.jsonl.gz`, export,
  battery. The Rust plumbing is written and compiles (see 0170); it was
  reverted only because the 10k model lost. Recover it from the 0170 commit
  diff, not from scratch.
- **Re-test 0027** ("10× data → live wash") — 0170 measured the opposite
  direction as −0.0094 live for 10× less. One of the two is wrong; the
  100k-vs-10k pair above answers it as a by-product.

## Owner-gated (unchanged)
- bhagpuss image-in-gold ruling — generalizes to the image-representation
  family (0152).
- Train-gold audit authorization → clean labels for the model.
- Spot-check pass: owner review has produced 3 real extractor defects per
  sitting (0141/0142/0144), better than any automated wave.

## Measured-closed lanes (do not re-walk; see logs)
site rules (0123 saturation) · lexicon exact-line (0165/0166/0168 shipped;
prefix 0169 negative; block-level empty; inverse 0167 unactionable) ·
emitter hygiene (0124) · convention unification (0093/0129/0142/0167,
four measurements) · heading adjacency (0121/0122) · formatting-from-style
(0105/0129/0130/0131) · link & image selection (0011/0113/0137) · form
emission by DOM density (0160; rescued only by title purpose, 0161) ·
rescue-ladder gates (0140/0164) · fitted-artifact decay (0156/0157 paid
+0.0017; 0158/0159 showed global points and mechanism-negatives do not
decay).

## Standing protocol
- Battery is the only arbiter; AUC is not a go-signal (0056, and 0170 is
  the cleanest instance: +0.0080 AUC → +0.0000 live).
- Bisect against HEAD, never regex-nuke a domain (0104, 0132).
- Per-doc rescore gate on every gold edit (0134).
- Fleets: per-doc JSONL checkpoints, small shards (survived 3 crashes).
- Runtime numbers are machine-specific; do not mix VM and local series.
