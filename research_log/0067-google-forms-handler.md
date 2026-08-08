# 0067 — Google Forms handler

- **Date:** 2026-08-08
- **Tag:** 0067-gforms (baseline: 0066-vb5b)
- **Status:** landed — zero regressions

## What changed
`extract_gforms`: fires on `form.ss-form`; emits `# ss-form-title`,
walked ss-form-desc, then `**ss-q-title**` + `- choice  ` hard-break
lists from ul.ss-choices (gold's exact convention). ≥2 questions gate.

## Results
golden **0.8388/0.7506** (+0.0005/+0.0004) · original dev
**0.8262/0.7299** (+0.0005/+0.0003) · train unchanged (no GForms
docs). 1 doc changed, up on both dev targets; general plain identical;
goldens pass. Engine triage lane: CPAN POD remains (last family).
