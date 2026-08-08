# 0098 — Domain-gated site vetoes (NEW MECHANISM)

- **Date:** 2026-08-08
- **Tag:** 0098-v2 (baseline: 0096-v2)
- **Status:** landed — largest extractor gain since 0055

## The construction
Site-specific selector vetoes gated on the page's own domain (og:url/
canonical) — CANNOT fire cross-site, eliminating by construction the
census≠action crater class that killed every generic chrome attempt
(0074/0081/0096). Table seeded from the 0096 safe-container extraction:
25 (domain, selector) rules after dropping generic selectors and one
measured offender (.simple-row ate its own job listing).

## Results
| target | F1 | Lev | Δ |
|---|---|---|---|
| golden | **0.8507** | **0.7659** | **+0.0017 / +0.0023** |
| original dev | **0.8299** | **0.7343** | +0.0010/+0.0013 (2 golden-primary dips) |
| train | 0.8153 | 0.7168 | +0.0001/+0.0001 |

Zero craters; general plain 1000/1000; goldens pass. Golden F1 crosses
0.85 (M2's F1 threshold on the golden target).

## The lane this opens
The mechanism SCALES: every mid-band/singleton chrome doc can feed the
table via the same extract-verify-census-battery pipeline (agents
propose, battery disposes, domain gate contains all risk). This
converts the previously "unfixable heterogeneous per-site chrome"
bucket (~40+ docs) into routine work.
