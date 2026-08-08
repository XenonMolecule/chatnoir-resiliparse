# 0087 — NEGATIVE: link-density-conditioned veto threshold

- **Date:** 2026-08-08
- **Tags:** 0087-linky{50,45}; reverted

## Hypothesis
kept-sidebar-widget (29 mid-band docs) = link-dense small blocks in
the model's 0.40-0.60 uncertainty band; bias those toward veto.

## Result
Monotonically negative: linky@0.50 → 0.8441/0.7578, @0.45 →
0.8466/0.7600, base (@0.40) 0.8482/0.7603. The model's link-density
feature already carries this signal — post-hoc thresholding on the
same feature double-counts it and vetoes link-heavy CONTENT (forum
link posts, reference lists) faster than widgets. The uncertainty
band earns its neutrality. Operating point confirmed optimal from a
fifth direction (0053 global, 0058 both-target, 0060 model-primary,
0061 labels, 0087 conditioned).
