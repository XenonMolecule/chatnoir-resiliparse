# 0142: forum quote block-break (owner-requested readability)

Owner on menstennisforums: "there is no space between the quote and the new
comment, so that makes it hard to parse." Fix: blockquote elements and
containers matching QUOTE_CLS (bbcode_quote / quoteheader / quotecontent /
post_quote / quotebox) now force a paragraph break in markdown mode.

Metrics (5dp): golden F1 0.89016 → 0.89016, Lev 0.81064 → 0.81062;
train F1 unchanged, Lev −0.00001. Counts churn (golden up 19 / down 25;
train up 206 / down 265) with zero both-down docs — pure whitespace
redistribution. Shipped on the owner's explicit readability requirement;
the −2e-5 Lev is below noise and no doc regressed on both metrics.
Plain config unaffected (1000/1000 identical), tests 7/7.
