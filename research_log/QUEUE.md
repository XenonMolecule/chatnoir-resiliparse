# Queue (updated cycle 0178)

State: golden **0.89482 F1 / 0.81562 Lev** (0177 basis: 0.8948/0.8156 on
the w3 battery). Lev goal (0.80–0.85) met since 0110 and holding. F1 gap
to 0.90: **−0.0052**. All 4 specialized domains ≥0.82 dev (0177); all 5
external benchmarks won vs upstream AND vs Dripper on WMB fine-grained
(0176).

## Model lane status after 0178 (both 120k variants live-negative)
- **MEASURED CLOSED as-trained**: v7-120k (title features, AUC 0.8657)
  golden −0.0064, 28 craters; v5ctrl-120k (66f, more data only, AUC
  0.8581) golden −0.0019, 14 craters. AUC-is-not-a-go-signal at its
  starkest. Shipped model stands; parity re-verified.
- **CLOSED 0179**: threshold recalibration tested (13-point grid). Best
  point (0.40/0.10/keep 0.65) recovers ⅓ of the deficit via stricter
  keep (the predicted tag-page mechanism) but stays −0.0014 below
  shipped; both axes unimodal around incumbents, big axis inert. The
  120k data lever is measured-closed in all three forms. Surviving idea
  (mechanism change, not calibration): per-page adaptive thresholds
  (quantile-of-page-scores) — park behind the next feature idea.
- 0027-vs-0170 contradiction resolved in 0027's favor at 120k scale:
  more data alone does not transfer under frozen thresholds.
- **Unused labelled corpora** (general/train 10k, dev2 1k, dev3 2k)
  remain untouched by training; only worth revisiting together with the
  threshold sweep above. Exclude general/dev and general/test (guardrail
  backing).

## New lane: SPA state mining (opened post-0179, owner-requested)
The marin devset's khanacademy conservation-of-energy doc (13d6d6eecd,
F1 0.204 with P 0.984 / R 0.114) fails because the content was never
HTML: gold = 87.8k of video transcript + Q&A discussion, and of its 255
paragraphs only 7 exist in the visible DOM — 153 verified ONLY inside
<script> React-state JSON (269k of the page's 447k is script), the rest
behind \uXXXX escaping. No DOM walker can score this doc. The other six
golded khanacademy docs score 0.92–0.995 (older snapshots server-render
the transcript), so this is snapshot-era-specific, not site-specific.

Design constraints for any attempt (this is a NEW PARADIGM, not a tweak):
- **Hard-gated rescue only.** Script JSON on normal pages is duplication
  and chrome — 0108 removed complex.com's app-state echoes FROM GOLD.
  Ungated mining would crater lpv11 and every benchmark we win.
- Proposed gate (all three): (1) base output tiny relative to visible
  page text AND relative to total page bytes; (2) a single large
  (>50–100k) JSON blob in script; (3) the blob contains ARRAYS of long
  human-text strings (sentence-cased, multi-word, low markup density) —
  mine only those strings, in document order.
- Dedup against the DOM output (the 7 visible paras also live in JSON).
- Battery risk is asymmetric: the gate must provably never fire on any
  lpv11/general/specialized doc (verify: count gate-fires across all
  batteries = expected ~0 before shipping).
- Yield: 1 golded marin doc immediately; the real prize is generic SPA
  coverage (React/Next/Nuxt hydration payloads) for real-world corpora.

## Runtime ledger (local series)
0178 clean measure: markdown main_content **3.00 ms/doc**, plain 1.32
(0116: 2.76/1.30). Cycles 0161–0177 cost +8.7% markdown. Perf pass
queueable if drift continues (known adds: 0172 title-restore h1 query +
full-output normalization; 0177 input query).

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
