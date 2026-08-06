// Copyright 2026 Janek Bevendorff
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! PyO3 bindings for the Rust `extract_plain_text` port.

use pyo3::prelude::*;
use resiliparse::extract::html2text as impl_;

#[pymodule]
pub mod _extract_rs {
    use super::*;

    /// extract_plain_text(html, preserve_formatting=True, main_content=False, ...)
    ///
    /// Rust port of `resiliparse.extract.html2text.extract_plain_text`.
    /// Signature-compatible with the Cython reference (html must be str).
    #[pyfunction]
    #[pyo3(signature = (html, preserve_formatting=None, main_content=false, list_bullets=true,
                        alt_texts=true, links=false, form_fields=false, noscript=false,
                        comments=true, post_meta=true, hidden_elements=false, skip_elements=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn extract_plain_text(
        py: Python<'_>,
        html: &str,
        preserve_formatting: Option<Bound<'_, PyAny>>,
        main_content: bool,
        list_bullets: bool,
        alt_texts: bool,
        links: bool,
        form_fields: bool,
        noscript: bool,
        comments: bool,
        post_meta: bool,
        hidden_elements: bool,
        skip_elements: Option<Vec<String>>,
    ) -> PyResult<String> {
        let formatting = match &preserve_formatting {
            None => impl_::FormattingOpts::Basic,
            Some(v) => {
                if let Ok(s) = v.extract::<&str>() {
                    if s == "minimal_html" {
                        impl_::FormattingOpts::MinimalHtml
                    } else if v.is_truthy()? {
                        impl_::FormattingOpts::Basic
                    } else {
                        impl_::FormattingOpts::Off
                    }
                } else if v.is_truthy()? {
                    impl_::FormattingOpts::Basic
                } else {
                    impl_::FormattingOpts::Off
                }
            }
        };
        let opts = impl_::ExtractOpts {
            preserve_formatting: formatting,
            main_content,
            list_bullets,
            alt_texts,
            links,
            form_fields,
            noscript,
            comments,
            post_meta,
            hidden_elements,
            skip_elements: skip_elements.unwrap_or_default(),
        };
        Ok(py.detach(|| impl_::extract_plain_text(html, &opts)))
    }
}
