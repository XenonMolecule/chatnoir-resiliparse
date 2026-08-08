# 0062 — NEGATIVE: stylesheet-hidden element vetoes (rendering-aware v1)

- **Date:** 2026-08-08
- **Tags:** 0062-csshidden/gated/narrow/final; reverted

## Hypothesis
Parse <style> blocks for `.class { display:none }` rules (skipping
@media) and veto matching elements — cheap rendering-awareness against
the hidden-chrome leakage class.

## Iterations, all measured
1. Ungated: flat aggregate; news-site craters (GateHouse pattern —
   article body display:none at load, JS-revealed; the charter's
   rendered-page target means class-hiding is NOT ground truth).
2. Chrome-name-gated (modal/popup/menu/…): dev clean +2 docs, but
   train crater lawyerfox −0.97 (directory whose content lives in
   menu-named structures).
3. Menu names dropped + 30%-page-mass guard: lawyerfox STILL −0.97,
   and forensics show NO element carries the hidden classes — the
   zeroing mechanism is not the designed veto path (suspected parser
   interaction; unresolved).

## Verdict
Upside 1–2 docs (+0.0001); one unexplained catastrophic failure mode.
Unshippable. Static CSS-class hiding is a poor proxy for rendering:
the display:none-at-load / JS-reveal idiom makes class-hiding
anti-correlated with visibility on exactly the sites that matter.
True rendering-awareness needs computed styles from a real layout
engine — noted as the entry cost for this direction. State verified
byte-identical to 0055-ship.
