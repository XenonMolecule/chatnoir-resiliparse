# 0130: convention wall mechanism identified — formatting follows computed CSS

Follow-up to the 0129 census: cross-tabbing gold's Comments-heading
convention against source markup shows it IS source-driven — `## Comments`
docs have `<h2>` (4/5), while `**Comments**` docs have `<span>` (10/14).
But the spans' attributes are heterogeneous (bare, .commentheader,
.Graphics, even .hide): the annotator's tool bolded them from COMPUTED
font-weight, i.e. external stylesheets we cannot see by construction.

Conclusion: the residual mid-band formatting variance is not per-doc
randomness (0129's framing) but rendered-style dependence. A future
"formatting-from-computed-style" lane (CSS cascade for font-weight on
short standalone lines) is the principled attack; in-page-only CSS was
measured too sparse for content decisions (0105) and likely also for
formatting. Parked as the documented mechanism of walls #16.
