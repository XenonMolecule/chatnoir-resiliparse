// AUTO-GENERATED placeholder until export runs.
#[derive(Clone, Copy, Default)]
pub struct BlockFeatures {
    pub tag: f64, pub depth: f64, pub log_text_len: f64, pub link_density: f64,
    pub n_links: f64, pub page_ld: f64, pub frac_page: f64, pub punct: f64,
    pub digit: f64, pub upper: f64, pub avgw: f64, pub nav: f64, pub footer: f64,
    pub header: f64, pub sidebar: f64, pub social: f64, pub article: f64,
    pub chrome: f64, pub byline: f64, pub widget: f64, pub recommended: f64,
    pub comments: f64,
}
pub fn score_block(_f: &BlockFeatures) -> f64 { 0.5 }
