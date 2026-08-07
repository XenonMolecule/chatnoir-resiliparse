# 0006 — Widget/meta/hide veto + error-stub guard (stronger list exemption)

- **Date:** 2026-08-07
- **Tag:** 0006-widget2 (baseline compared against: 0005-ship3/0005-ship)
- **Status:** landed (with one explicitly logged trade-off, see Results)

## Hypothesis

Blogger/WordPress widget containers (`widget BlogList`, `widget-content`, …)
are a platform-generic junk signal; adding a widget veto to the list-exemption
path should block 0005's junk admissions and let the exemption drop the
win-killing "≤1 link per item" guard, unlocking ~+12 docs.

## What changed

- `has_widgetish_ancestry`: veto the list exemption when the `<ul>` or any
  ancestor has a `widget(s)`/`hide`/`meta` class word (checked only inside the
  exemption path — bounded blast radius). The `hide`/`meta` words come from
  the 0005 FAQ-index doc, whose junk list is literally `<ul class="meta">`
  with `<li class="hide">`.
- Dropped the "≤1 link per item" guard (it killed 9 of 12 wins).
- Error-stub veto for both rescue tiers: a <300B extraction matching
  "we're sorry / error has occurred / not found / out of stock / …" is the
  page's true content — never rescue it.
- Two robustness fixes found by the 10k train run: a UTF-8 char-boundary
  panic in the duplication guard (advance by probe length, not +1 byte), and
  the harness worker now catches `BaseException` (pyo3 `PanicException` is
  not an `Exception` and un-picklable across the pool).

## Results

vs. 0005 (all per-doc zero-regression):

| set | 0005 F1 | 0006 F1 | per-doc |
|---|---|---|---|
| lpv11 dev | 0.7789 | **0.7801** | +3 / −0 |
| general dev | 0.8113 | **0.8138** | +4 / −0 |
| general dev2 | 0.8136 | **0.8144** | +2 / −0 |

**Train (10k) generalization — cumulative rescues (0004–0006) vs 0001
baseline: +0.0156 mean F1, 312 improved / 50 regressed (6.2:1).** Speed ratio
vs frozen Cython: 1.20× (1.23× at 0002 → all rescue machinery costs ~2.5%
relative). Golden tests pass; 0 errors on 10,999 docs.

**Logged trade-off decision:** the 50 residual train regressions are pages
whose *true content is tiny* (photo captions, one-paragraph news, short forum
posts) wrapped in heavy shells — from structure alone, indistinguishable from
the classifier-wipeout pages the rescues exist to fix. Two separating signals
were tried and measured as NEGATIVE: body link-ratio (regressed median 0.17
vs improved 0.14 — no separation) and output prose-ratio (0.36 vs 0.49 —
heavy overlap). The dev guardrails are all zero-regression; per playbook §6
the 6:1 train ratio at +0.0156 mean ships with this note. Revisit if a
content-aware confidence signal appears.

## Insights

- The 0005→0006 arc is the playbook's tight-gate discipline working: each
  junk admission (blogroll, FAQ index, error stubs) became a named,
  independently-gated veto rather than a threshold tweak.
- `class="hide"` is invisible to the display regex (which matches `hidden`
  but not `hide`) — possible future global classifier fix, kept scoped for
  now.
- Train-scale runs are where rare-input crashes live: the UTF-8 panic
  appeared on doc 6,621 of 9,999. Worth running train after every
  string-manipulating change, not just at milestones.
- 10× data (train) turns "zero regressions" into "0.5% regressions" — the
  honest generalization number for gated heuristics tuned on dev.

## Next

- Article-teaser rule repair (8-doc bucket, 0003) — next rule cycle.
- landmark_id/footer regex tightening; forum over-extraction track.
- Markdown output mode remains queued (0003 showed modest Lev headroom, but
  correctness value stands).
