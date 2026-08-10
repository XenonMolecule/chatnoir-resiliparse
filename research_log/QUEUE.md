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

## New lane: SPA state mining (opened post-0179; PoC DE-RISKED same night)
Target case: marin khanacademy conservation-of-energy (13d6d6eecd),
DOM-only F1 0.204 (P 0.984 / R 0.114). Corrected diagnosis after
inspection: the page is React 15 SSR — THIS video's transcript IS in the
DOM and we extract it fully; the missing 78k of gold is the tutorial's
SIBLING content, present in the page bytes only as state JSON in two
encodings: (a) caption-cue arrays `{"startTime":…, "text": "…"}` for the
other videos' transcripts (cue boundaries split sentences — containment
probes must join cues before matching), and (b) escaped-JSON markdown
strings (`\"content\":\"**Exercise 1a:** …\"`) for tutorial articles.
The other six golded KA docs score 0.92–0.995 DOM-only.

**PoC measured (40-line Python, regex/scan only, no JS execution):**
DOM + naive mining of those two shapes = **F1 0.785 / P 0.785 / R 0.786**
(876 cues + 29 content fields). Remaining gap is ordering/interleaving,
cue-joining, and prose under other keys — engineering, not research risk.

Two-tier architecture (owner's framing: detect React → route to a
slightly more expensive path):
- Tier gate (near-free): data-reactid / __NEXT_DATA__ / __NUXT__
  substrings, or >50k JSON-shaped script blob. Prevalence measured:
  4.4% of marin html, 5.9% of WMB, 22 Next + 8 Nuxt pages in marin.
- Expensive path (gated pages only): mine ordered cue arrays and
  content/markdown fields; unescape one level; dedupe against DOM
  output; append in state order. A few ms on a 450k page; amortized
  corpus cost ≈ nil.
- Safety: the tier CANNOT fire on non-SPA pages by construction —
  stronger than the original "rescue" framing. Still verify gate-fire
  count ≈ 0 across lpv11/general/specialized batteries before shipping
  (0108: app-state echoes on normal pages are chrome, removed from gold).
- Yield: this doc 0.204→0.785 immediately; the strategic case is the
  modern web — React-era pages are a growing share of any fresh crawl.

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
