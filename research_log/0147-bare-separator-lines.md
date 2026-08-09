# 0147: bare separator lines + U+FFFD verdict

**Shipped**: lines composed only of middots/bullets and whitespace
("· ·", 42x in dev) are stripped. Deliberately narrow — '-', '*', '|' are
excluded because gold uses them for horizontal rules, emphasis and tables.
Battery: golden net-zero (up 1 / down 1, means flat), train up 13 / down 5
(+1e-5 Lev). Shipped for the train-side generalization; harmless on golden.

**U+FFFD verdict (closes the census item)**: all 38 docs where we emit the
replacement character ALREADY contain it in the stored source HTML — the
mojibake is upstream in the dataset, and the gold was annotated from
correctly-decoded bytes. No extractor change can recover the lost
characters; charset repair would have to happen at dataset build time.
Documented as an external wall, not an extractor defect.

**Not actioned**: "Reply · Report · <name> on <date>" comment-UI lines mix
chrome with author/date that gold may want — needs a prefix-trim, not a
line-drop; deferred as its own experiment.
