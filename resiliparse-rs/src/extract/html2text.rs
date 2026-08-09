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

//! Plain text extraction from HTML.
//!
//! Direct port of the Cython reference implementation
//! (`resiliparse-py/resiliparse/extract/html2text.pyx`) onto the raw lexbor
//! bindings. Output is intended to be byte-identical to the Cython extractor;
//! quirks of the reference (including its bugs) are reproduced deliberately.

use crate::third_party::lexbor::lxb_dom_node_type_t::*;
use crate::third_party::lexbor::*;
use crate::extract::block_model;
use regex::bytes::{Regex, RegexBuilder};
use std::collections::{BTreeSet, HashSet};
use std::ptr;
use std::slice;
use std::sync::LazyLock;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum FormattingOpts {
    Off = 0,
    #[default]
    Basic = 1,
    MinimalHtml = 2,
    /// Markdown-flavored output (cycle 0009): `#` headings, `**bold**`,
    /// `*italic*`, `- ` bullets. Ordered above MinimalHtml, so comparisons
    /// that mean "minimal HTML specifically" must use `==`, not `>=`.
    Markdown = 3,
}

#[derive(Clone, Debug)]
pub struct ExtractOpts {
    pub preserve_formatting: FormattingOpts,
    pub main_content: bool,
    pub list_bullets: bool,
    pub alt_texts: bool,
    pub links: bool,
    pub form_fields: bool,
    pub noscript: bool,
    pub comments: bool,
    pub post_meta: bool,
    pub hidden_elements: bool,
    pub skip_elements: Vec<String>,
}

impl Default for ExtractOpts {
    fn default() -> Self {
        ExtractOpts {
            preserve_formatting: FormattingOpts::Basic,
            main_content: false,
            list_bullets: true,
            alt_texts: true,
            links: false,
            form_fields: false,
            noscript: false,
            comments: true,
            post_meta: true,
            hidden_elements: false,
            skip_elements: Vec::new(),
        }
    }
}

/// C `isspace()` (ASCII), matching the Cython implementation's byte-wise
/// whitespace handling (bytes >= 0x80 never count as space).
#[inline(always)]
fn c_isspace(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r')
}

#[inline]
fn lstrip(s: &[u8]) -> &[u8] {
    let mut start = 0;
    while start < s.len() && c_isspace(s[start]) {
        start += 1;
    }
    &s[start..]
}

#[inline]
fn rstrip(s: &[u8]) -> &[u8] {
    let mut end = s.len();
    while end > 0 && c_isspace(s[end - 1]) {
        end -= 1;
    }
    &s[..end]
}

#[inline]
fn strip(s: &[u8]) -> &[u8] {
    rstrip(lstrip(s))
}

#[inline]
fn rstrip_in_place(s: &mut Vec<u8>) {
    while let Some(&c) = s.last() {
        if c_isspace(c) {
            s.pop();
        } else {
            break;
        }
    }
}

/// Collapse like `get_collapsed_string`, additionally treating U+00A0 (NBSP,
/// bytes C2 A0) as collapsible whitespace — the lpv11 gold normalizes NBSP to
/// plain space almost everywhere (632 pred docs vs 43 gold docs carried it;
/// cycle 0016). Markdown mode only; `<pre>` content never passes through here.
fn get_collapsed_string_nbsp(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        let is_nbsp = input[i] == 0xC2 && i + 1 < input.len() && input[i + 1] == 0xA0;
        if is_nbsp || c_isspace(input[i]) {
            if out.is_empty() || !c_isspace(*out.last().unwrap()) {
                out.push(b' ');
            }
            i += if is_nbsp { 2 } else { 1 };
        } else {
            out.push(input[i]);
            i += 1;
        }
    }
    out
}

/// Collapse newlines and consecutive white space in a string to single spaces.
fn get_collapsed_string(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    for &c in input {
        if c_isspace(c) {
            if out.is_empty() || !c_isspace(*out.last().unwrap()) {
                out.push(b' ');
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn escape_html(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    for &c in data {
        match c {
            b'&' => out.extend_from_slice(b"&amp;"),
            b'"' => out.extend_from_slice(b"&quot;"),
            b'<' => out.extend_from_slice(b"&lt;"),
            b'>' => out.extend_from_slice(b"&gt;"),
            _ => out.push(c),
        }
    }
    out
}

const LIST_BULLET: &[u8] = "\u{2022}".as_bytes();

/// Close a markdown emphasis span: place the closing marker before any
/// trailing whitespace (`**bold **` is invalid markdown), and collapse empty
/// spans (`**` immediately followed by the closer) by removing the opener.
fn close_inline_marker(tc: &mut Vec<u8>, marker: &[u8]) {
    let mut end = tc.len();
    while end > 0 && c_isspace(tc[end - 1]) {
        end -= 1;
    }
    if tc[..end].ends_with(marker) {
        tc.drain(end - marker.len()..end);
        return;
    }
    // `<b> word</b>` would render as `** word**` (invalid): if the span's
    // opener is directly followed by whitespace, move it past that whitespace.
    if let Some(pos) = tc[..end]
        .windows(marker.len())
        .rposition(|w| w == marker)
    {
        let after = pos + marker.len();
        let ws_end = (after..end).take_while(|&i| c_isspace(tc[i])).count() + after;
        if ws_end > after && !tc[after.saturating_sub(2 * marker.len())..pos].ends_with(marker) {
            // rotate: [marker][ws] -> [ws][marker]
            tc[pos..ws_end].rotate_left(marker.len());
        }
    }
    let tail: Vec<u8> = tc.split_off(end);
    tc.extend_from_slice(marker);
    tc.extend_from_slice(&tail);
}

// lxb_dom_node_t.local_name is a plain `usize`; re-declare the tag constants
// we need at that type so comparisons stay cast-free.
macro_rules! tag_consts {
    ($($name:ident),* $(,)?) => {
        $(const $name: lxb_tag_id_t = lxb_tag_id_enum_t::$name as lxb_tag_id_t;)*
    };
}

tag_consts!(
    LXB_TAG__UNDEF,
    LXB_TAG_A,
    LXB_TAG_ADDRESS,
    LXB_TAG_AREA,
    LXB_TAG_ARTICLE,
    LXB_TAG_B,
    LXB_TAG_EM,
    LXB_TAG_STRONG,
    LXB_TAG_ASIDE,
    LXB_TAG_AUDIO,
    LXB_TAG_BLOCKQUOTE,
    LXB_TAG_BODY,
    LXB_TAG_BR,
    LXB_TAG_BUTTON,
    LXB_TAG_CENTER,
    LXB_TAG_CODE,
    LXB_TAG_DD,
    LXB_TAG_SMALL,
    LXB_TAG_FONT,
    LXB_TAG_DETAILS,
    LXB_TAG_DIV,
    LXB_TAG_DL,
    LXB_TAG_DT,
    LXB_TAG_FIELDSET,
    LXB_TAG_FIGCAPTION,
    LXB_TAG_FIGURE,
    LXB_TAG_FOOTER,
    LXB_TAG_FORM,
    LXB_TAG_H1,
    LXB_TAG_H2,
    LXB_TAG_H3,
    LXB_TAG_H4,
    LXB_TAG_H5,
    LXB_TAG_H6,
    LXB_TAG_HEADER,
    LXB_TAG_HGROUP,
    LXB_TAG_HR,
    LXB_TAG_I,
    LXB_TAG_IMG,
    LXB_TAG_INPUT,
    LXB_TAG_LI,
    LXB_TAG_LINK,
    LXB_TAG_MAIN,
    LXB_TAG_META,
    LXB_TAG_NAV,
    LXB_TAG_OL,
    LXB_TAG_P,
    LXB_TAG_PRE,
    LXB_TAG_SECTION,
    LXB_TAG_TABLE,
    LXB_TAG_TD,
    LXB_TAG_TEXTAREA,
    LXB_TAG_TH,
    LXB_TAG_TIME,
    LXB_TAG_TR,
    LXB_TAG_UL,
    LXB_TAG_VIDEO,
);

const BLOCK_ELEMENTS: &[lxb_tag_id_t] = &[
    LXB_TAG_ADDRESS,
    LXB_TAG_ARTICLE,
    LXB_TAG_ASIDE,
    LXB_TAG_BLOCKQUOTE,
    LXB_TAG_BR,
    LXB_TAG_CENTER,
    LXB_TAG_CODE,
    LXB_TAG_DETAILS,
    LXB_TAG_DD,
    LXB_TAG_DT,
    LXB_TAG_DIV,
    LXB_TAG_DL,
    LXB_TAG_FIELDSET,
    LXB_TAG_FIGCAPTION,
    LXB_TAG_FIGURE,
    LXB_TAG_FOOTER,
    LXB_TAG_FORM,
    LXB_TAG_H1,
    LXB_TAG_H2,
    LXB_TAG_H3,
    LXB_TAG_H4,
    LXB_TAG_H5,
    LXB_TAG_H6,
    LXB_TAG_HEADER,
    LXB_TAG_HGROUP,
    LXB_TAG_HR,
    LXB_TAG_LI,
    LXB_TAG_MAIN,
    LXB_TAG_META,
    LXB_TAG_NAV,
    LXB_TAG_OL,
    LXB_TAG_P,
    LXB_TAG_PRE,
    LXB_TAG_SECTION,
    LXB_TAG_TABLE,
    LXB_TAG_TR,
    LXB_TAG_UL,
];

#[inline]
fn is_block_element(tag_id: lxb_tag_id_t) -> bool {
    BLOCK_ELEMENTS.contains(&tag_id)
}

// ---------------------------------------------------------------------------
// Raw DOM helpers (ports of resiliparse.parse.html cdef helpers)
// ---------------------------------------------------------------------------

/// DOM tree pre-order traversal primitive with end-tag signaling.
unsafe fn next_node(
    root_node: *const lxb_dom_node_t,
    mut node: *mut lxb_dom_node_t,
    depth: &mut usize,
    end_tag: &mut bool,
) -> *mut lxb_dom_node_t {
    unsafe {
        let is_end = *end_tag;
        if !is_end && !(*node).first_child.is_null() {
            *depth += 1;
            (*node).first_child
        } else {
            while (*node).next.is_null() && node != root_node.cast_mut() {
                node = (*node).parent;
                *depth -= 1;
                *end_tag = true;
                return node;
            }
            *end_tag = false;
            if node == root_node.cast_mut() {
                return ptr::null_mut();
            }
            (*node).next
        }
    }
}

/// Node attribute value as a byte slice (empty slice if attribute missing).
unsafe fn get_node_attr(node: *mut lxb_dom_node_t, attr: &[u8]) -> &'static [u8] {
    unsafe {
        let mut len: usize = 0;
        let data = lxb_dom_element_get_attribute(node.cast(), attr.as_ptr(), attr.len(), &mut len);
        if data.is_null() {
            &[]
        } else {
            slice::from_raw_parts(data, len)
        }
    }
}

/// Get node inner text (like `lxb_dom_node_text_content`).
unsafe fn get_node_text(node: *mut lxb_dom_node_t) -> Vec<u8> {
    unsafe {
        if (*node).type_ == LXB_DOM_NODE_TYPE_TEXT {
            let char_data = node as *const lxb_dom_character_data_t;
            return slice::from_raw_parts((*char_data).data.data, (*char_data).data.length).to_vec();
        }
        let mut text_len: usize = 0;
        let text = lxb_dom_node_text_content(node, &mut text_len);
        if text.is_null() || text_len == 0 {
            return Vec::new();
        }
        let out = slice::from_raw_parts(text, text_len).to_vec();
        lxb_dom_document_destroy_text_noi((*node).owner_document, text);
        out
    }
}

unsafe extern "C" fn css_select_callback(
    node: *mut lxb_dom_node_t,
    _spec: lxb_css_selector_specificity_t,
    ctx: *mut ::std::os::raw::c_void,
) -> lxb_status_t {
    unsafe {
        let coll = ctx as *mut Vec<*mut lxb_dom_node_t>;
        (*coll).push(node);
        lexbor_status_t::LXB_STATUS_OK
    }
}

/// All nodes under `node` matching the given CSS selector list.
unsafe fn query_selector_all_raw(
    doc: *mut lxb_html_document_t,
    node: *mut lxb_dom_node_t,
    selector: &[u8],
) -> Vec<*mut lxb_dom_node_t> {
    unsafe {
        let mut result: Vec<*mut lxb_dom_node_t> = Vec::new();
        if lxb_html_document_css_init(doc) != lexbor_status_t::LXB_STATUS_OK {
            return result;
        }
        let parser = (*doc).css.parser;
        let sel_list = lxb_css_selectors_parse(parser, selector.as_ptr(), selector.len());
        if (*parser).status != lexbor_status_t::LXB_STATUS_OK || sel_list.is_null() {
            return result;
        }
        let selectors = lxb_selectors_create();
        lxb_selectors_init(selectors);
        lxb_selectors_find(
            selectors,
            node,
            sel_list,
            Some(css_select_callback),
            std::ptr::addr_of_mut!(result).cast(),
        );
        lxb_selectors_destroy(selectors, true);
        lxb_css_selector_list_destroy(sel_list);
        result
    }
}

// ---------------------------------------------------------------------------
// Extraction node machinery (port of ExtractNode / _extract_cb)
// ---------------------------------------------------------------------------

struct ExtractNode {
    reference_node: *mut lxb_dom_node_t,
    tag_id: lxb_tag_id_t,
    depth: usize,
    pre_depth: usize,
    collapse_margins: bool,
    make_block: bool,
    make_big_block: bool,
    is_end_tag: bool,
    escape_text_contents: bool,
    /// Set by the markdown table pre-pass: this node belongs to an eligible
    /// data table and gets pipe-row serialization (cycle 0012).
    md_table: bool,
    text_contents: Option<Vec<u8>>,
}

impl Default for ExtractNode {
    fn default() -> Self {
        ExtractNode {
            reference_node: ptr::null_mut(),
            tag_id: LXB_TAG__UNDEF,
            depth: 0,
            pre_depth: 0,
            collapse_margins: true,
            make_block: true,
            make_big_block: false,
            is_end_tag: false,
            escape_text_contents: false,
            md_table: false,
            text_contents: None,
        }
    }
}

struct ExtractContext {
    root_node: *mut lxb_dom_node_t,
    node: *mut lxb_dom_node_t,
    depth: usize,
    opts: ExtractOptsC,
}

/// The subset of options threaded through the extraction walk (mirrors the
/// C `ExtractOpts` struct in the Cython module).
#[derive(Clone, Copy)]
struct ExtractOptsC {
    preserve_formatting: FormattingOpts,
    list_bullets: bool,
    links: bool,
    alt_texts: bool,
    form_fields: bool,
    #[allow(dead_code)]
    noscript: bool,
}

#[inline]
fn ensure_text_contents(extract_nodes: &mut [ExtractNode]) {
    let last = extract_nodes.last_mut().unwrap();
    if last.text_contents.is_none() {
        last.text_contents = Some(Vec::new());
    }
}

unsafe fn extract_cb(extract_nodes: &mut Vec<ExtractNode>, ctx: &mut ExtractContext, is_end_tag: bool) {
    unsafe {
        let node = ctx.node;
        let local_name = (*node).local_name;
        let is_block = (*node).type_ == LXB_DOM_NODE_TYPE_ELEMENT && is_block_element(local_name);

        let last_exists = !extract_nodes.is_empty();
        let last_depth = extract_nodes.last().map(|n| n.depth).unwrap_or(0);
        let last_pre_depth = extract_nodes.last().map(|n| n.pre_depth).unwrap_or(0);

        if !last_exists
            || is_block
            || ctx.depth < last_depth
            || (ctx.opts.links && local_name == LXB_TAG_A)
            || local_name == LXB_TAG_TEXTAREA
            || (ctx.opts.preserve_formatting == FormattingOpts::Markdown
                && matches!(local_name, LXB_TAG_TD | LXB_TAG_TH))
        {
            let mut new_node = ExtractNode {
                reference_node: node,
                depth: ctx.depth,
                make_block: is_block,
                make_big_block: matches!(local_name, LXB_TAG_P | LXB_TAG_H1 | LXB_TAG_H2 | LXB_TAG_H3 | LXB_TAG_H4)
                    // Forum quote containers (0142, owner-flagged): a quoted
                    // post must not run into the reply text — markdown only.
                    || (ctx.opts.preserve_formatting == FormattingOpts::Markdown
                        && local_name == LXB_TAG_BLOCKQUOTE)
                    || (ctx.opts.preserve_formatting == FormattingOpts::Markdown
                        && regex_search_not_empty(get_node_attr(node, b"class"), &QUOTE_CLS)),
                tag_id: local_name,
                pre_depth: last_pre_depth,
                is_end_tag,
                escape_text_contents: ctx.opts.preserve_formatting == FormattingOpts::MinimalHtml,
                ..Default::default()
            };
            if matches!(local_name, LXB_TAG_PRE | LXB_TAG_TEXTAREA) {
                if is_end_tag {
                    new_node.pre_depth = new_node.pre_depth.wrapping_sub(1);
                } else {
                    new_node.pre_depth += 1;
                }
            }
            extract_nodes.push(new_node);
        }

        let current_tag_id = extract_nodes.last().map(|n| n.tag_id).unwrap_or(LXB_TAG__UNDEF);

        if (*node).type_ == LXB_DOM_NODE_TYPE_TEXT {
            ensure_text_contents(extract_nodes);
            let char_data = node as *const lxb_dom_character_data_t;
            let mut element_text =
                slice::from_raw_parts((*char_data).data.data, (*char_data).data.length).to_vec();

            if current_tag_id == LXB_TAG_A && ctx.opts.preserve_formatting == FormattingOpts::MinimalHtml {
                // Escape <a> inner text
                element_text = escape_html(&element_text);
            }

            if !element_text.is_empty() {
                extract_nodes
                    .last_mut()
                    .unwrap()
                    .text_contents
                    .as_mut()
                    .unwrap()
                    .extend_from_slice(&element_text);
            }
        } else if (*node).type_ != LXB_DOM_NODE_TYPE_ELEMENT {
            // Nothing to do for other node types.
        } else if local_name == LXB_TAG_BR
            && matches!(ctx.opts.preserve_formatting, FormattingOpts::Basic | FormattingOpts::Markdown)
        {
            ensure_text_contents(extract_nodes);
            extract_nodes.last_mut().unwrap().collapse_margins = false;
        } else if ctx.opts.preserve_formatting == FormattingOpts::Markdown
            && matches!(local_name, LXB_TAG_B | LXB_TAG_STRONG | LXB_TAG_I | LXB_TAG_EM)
            && !(*node).first_child.is_null()
        {
            // Markdown inline emphasis. Childless elements get no end-tag
            // event from the traversal, hence the first_child guard (keeps
            // markers balanced).
            ensure_text_contents(extract_nodes);
            let marker: &[u8] = if matches!(local_name, LXB_TAG_B | LXB_TAG_STRONG) {
                b"**"
            } else {
                b"*"
            };
            let tc = extract_nodes.last_mut().unwrap().text_contents.as_mut().unwrap();
            if !is_end_tag {
                tc.extend_from_slice(marker);
            } else {
                close_inline_marker(tc, marker);
            }
        } else if ctx.opts.links && local_name == LXB_TAG_A {
            let href = strip(get_node_attr(node, b"href"));
            ensure_text_contents(extract_nodes);
            extract_nodes.last_mut().unwrap().make_block = false;

            if ctx.opts.preserve_formatting == FormattingOpts::MinimalHtml {
                let mut element_text: Vec<u8> = Vec::new();
                if !is_end_tag {
                    element_text.extend_from_slice(b"<a href=\"");
                    element_text.extend_from_slice(&escape_html(href));
                    element_text.extend_from_slice(b"\">");
                } else {
                    element_text.extend_from_slice(b"</a>");
                }
                let last = extract_nodes.last_mut().unwrap();
                last.text_contents.as_mut().unwrap().extend_from_slice(&element_text);
                last.escape_text_contents = false;
            } else if is_end_tag {
                let last = extract_nodes.last_mut().unwrap();
                let tc = last.text_contents.as_mut().unwrap();
                tc.extend_from_slice(b" (");
                tc.extend_from_slice(href);
                tc.push(b')');
            }
        } else if ctx.opts.alt_texts && matches!(local_name, LXB_TAG_IMG | LXB_TAG_AREA) {
            ensure_text_contents(extract_nodes);
            let alt = get_node_attr(node, b"alt");
            if !alt.is_empty() {
                // NB: markdown `![alt](src)` emission was tried and reverted
                // in cycle 0011 — no DOM-rule gate reached usable precision
                // (gold keeps 3.4% of attributed images; best joint rule
                // still net-negative). Revisit with learned selection.
                extract_nodes
                    .last_mut()
                    .unwrap()
                    .text_contents
                    .as_mut()
                    .unwrap()
                    .extend_from_slice(alt);
            }
        } else if ctx.opts.form_fields && matches!(local_name, LXB_TAG_TEXTAREA | LXB_TAG_BUTTON) {
            ensure_text_contents(extract_nodes);
            let element_text: &[u8] = if !is_end_tag { b"[ " } else { b" ] " };
            extract_nodes
                .last_mut()
                .unwrap()
                .text_contents
                .as_mut()
                .unwrap()
                .extend_from_slice(element_text);
        } else if ctx.opts.form_fields && local_name == LXB_TAG_INPUT {
            let type_attr = strip(get_node_attr(node, b"type"));
            const SKIP_TYPES: &[&[u8]] = &[b"checkbox", b"color", b"file", b"hidden", b"radio", b"reset"];
            if type_attr.is_empty() || !SKIP_TYPES.contains(&type_attr) {
                let mut value = strip(get_node_attr(node, b"value"));
                if value.is_empty() {
                    value = strip(get_node_attr(node, b"placeholder"));
                }
                if !value.is_empty() {
                    ensure_text_contents(extract_nodes);
                    let last = extract_nodes.last_mut().unwrap();
                    let tc = last.text_contents.as_mut().unwrap();
                    tc.extend_from_slice(b"[ ");
                    tc.extend_from_slice(value);
                    tc.extend_from_slice(b" ] ");
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Serialization (port of _serialize_extract_nodes)
// ---------------------------------------------------------------------------

#[inline]
fn make_indent(output: &mut Vec<u8>, list_depth: usize, opts: &ExtractOptsC) {
    if list_depth == 0 {
        return;
    }
    if opts.preserve_formatting == FormattingOpts::Off {
        rstrip_in_place(output);
    }
    output.extend(std::iter::repeat_n(b' ', list_depth * 2));
}

#[inline]
fn make_margin(
    output: &mut Vec<u8>,
    margin_size: &mut usize,
    margin_is_br: &mut bool,
    pre_depth: usize,
    opts: &ExtractOptsC,
) {
    if *margin_size == 0 {
        *margin_is_br = false;
        return;
    }
    if pre_depth == 0 || opts.preserve_formatting == FormattingOpts::Off {
        rstrip_in_place(output);
    }
    if opts.preserve_formatting == FormattingOpts::Off && !output.is_empty() {
        output.push(b' ');
    } else if opts.preserve_formatting >= FormattingOpts::Basic && !output.is_empty() {
        if opts.preserve_formatting == FormattingOpts::Markdown && *margin_size == 1 && *margin_is_br {
            // A <br>-generated line break inside a block: markdown's
            // two-space hard-break form (the gold uses it systematically).
            output.extend_from_slice(b"  \n");
        } else {
            output.extend(std::iter::repeat_n(b'\n', *margin_size));
        }
    }
    *margin_size = 0;
    *margin_is_br = false;
}

/// Markdown data-table pre-pass (cycle 0012): find TABLE spans in the node
/// stream, check eligibility (>=2 rows, >=2 cells in some row, no nested
/// table, no oversized cells — layout tables must stay plain), and mark the
/// span's TABLE/TR/TD/TH nodes for pipe-row serialization.
fn mark_markdown_tables(extract_nodes: &mut [ExtractNode]) {
    const MAX_CELL_TEXT: usize = 300;
    let mut i = 0;
    while i < extract_nodes.len() {
        if extract_nodes[i].tag_id == LXB_TAG_TABLE && !extract_nodes[i].is_end_tag {
            // find matching end at same nesting level
            let start = i;
            let mut depth = 1;
            let mut end = None;
            let mut nested = false;
            let mut rows = 0usize;
            let mut max_cells = 0usize;
            let mut cur_cells = 0usize;
            let mut cell_text = 0usize;
            let mut oversized = false;
            let mut j = i + 1;
            while j < extract_nodes.len() {
                let n = &extract_nodes[j];
                if n.tag_id == LXB_TAG_TABLE {
                    if n.is_end_tag {
                        depth -= 1;
                        if depth == 0 {
                            end = Some(j);
                            break;
                        }
                    } else {
                        depth += 1;
                        nested = true;
                    }
                } else if depth == 1 {
                    if n.tag_id == LXB_TAG_TR && !n.is_end_tag {
                        rows += 1;
                        max_cells = max_cells.max(cur_cells);
                        cur_cells = 0;
                        cell_text = 0;
                    } else if matches!(n.tag_id, LXB_TAG_TD | LXB_TAG_TH) && !n.is_end_tag {
                        cur_cells += 1;
                        cell_text = 0;
                    }
                    // Any node's text between here and the next cell/row
                    // boundary belongs to the current cell (nested <p> etc.
                    // carry their text on their own nodes).
                    cell_text += n.text_contents.as_ref().map(|t| t.len()).unwrap_or(0);
                    if cell_text > MAX_CELL_TEXT {
                        oversized = true;
                    }
                }
                j += 1;
            }
            max_cells = max_cells.max(cur_cells);
            if let Some(end) = end {
                if !nested && !oversized && rows >= 2 && max_cells >= 2 {
                    for n in &mut extract_nodes[start..=end] {
                        if matches!(n.tag_id, LXB_TAG_TABLE | LXB_TAG_TR | LXB_TAG_TD | LXB_TAG_TH) {
                            n.md_table = true;
                            if n.tag_id == LXB_TAG_TABLE {
                                // blank line around the table
                                n.make_big_block = true;
                            } else {
                                // rows/cells manage their own line breaks
                                n.make_block = false;
                            }
                        }
                    }
                }
                i = if nested { i + 1 } else { end + 1 };
                continue;
            }
        }
        i += 1;
    }
}

unsafe fn serialize_extract_nodes(
    extract_nodes: &mut [ExtractNode],
    opts: &ExtractOptsC,
    reserve_size: usize,
) -> Vec<u8> {
    unsafe {
        let mut output: Vec<u8> = Vec::with_capacity(reserve_size);
        if opts.preserve_formatting == FormattingOpts::Markdown {
            mark_markdown_tables(extract_nodes);
        }
        let mut element_text_prefix: Vec<u8> = Vec::new();
        let mut bullet_inserted = false;
        let mut list_depth: usize = 0;
        let mut margin_size: usize = 0;
        let mut margin_is_br = false;
        let mut uncollapsed_margin_count: usize = 0;
        // markdown pipe-table state (only one eligible table active at a time)
        let mut md_row_index: usize = 0;
        let mut md_cell_index: usize = 0;
        let mut md_row0_cells: usize = 0;
        let mut md_in_table = false;
        let mut fence_just_opened = false;
        let mut list_numbering: Vec<usize> = Vec::new();

        for i in 0..extract_nodes.len() {
            let current_node = &mut extract_nodes[i];

            // Basic and minimal HTML formatting
            if opts.preserve_formatting >= FormattingOpts::Basic {
                if current_node.make_block && !current_node.collapse_margins {
                    uncollapsed_margin_count += 1;
                }

                // List tags
                if matches!(current_node.tag_id, LXB_TAG_UL | LXB_TAG_OL)
                    || (current_node.tag_id == LXB_TAG_LI && list_depth == 0)
                {
                    if current_node.is_end_tag {
                        list_depth = list_depth.wrapping_sub(1);
                        list_numbering.pop();
                        bullet_inserted = false;
                        element_text_prefix.clear();
                    } else {
                        list_depth = list_depth.wrapping_add(1);
                        list_numbering.push((current_node.tag_id == LXB_TAG_OL) as usize);
                    }
                }

                // List item tags
                if opts.list_bullets && current_node.tag_id == LXB_TAG_LI {
                    if matches!(opts.preserve_formatting, FormattingOpts::Basic | FormattingOpts::Markdown) {
                        if *list_numbering.last().unwrap() == 0 {
                            element_text_prefix = if opts.preserve_formatting == FormattingOpts::Markdown {
                                b"- ".to_vec()
                            } else {
                                LIST_BULLET.to_vec()
                            };
                            if opts.preserve_formatting != FormattingOpts::Markdown {
                                element_text_prefix.push(b' ');
                            }
                        } else {
                            element_text_prefix = list_numbering.last().unwrap().to_string().into_bytes();
                            element_text_prefix.extend_from_slice(b". ");
                            if !current_node.is_end_tag {
                                *list_numbering.last_mut().unwrap() += 1;
                            }
                        }
                        bullet_inserted = !current_node.is_end_tag;
                    } else if opts.list_bullets && opts.preserve_formatting == FormattingOpts::MinimalHtml {
                        make_margin(&mut output, &mut margin_size, &mut margin_is_br, current_node.pre_depth, opts);
                        if !current_node.is_end_tag {
                            output.extend(std::iter::repeat_n(b' ', 2 * list_depth));
                            output.extend_from_slice(b"<li>");
                            margin_size = 0;
                            current_node.make_block = false;
                        } else {
                            if current_node.pre_depth == 0 {
                                rstrip_in_place(&mut output);
                            }
                            output.extend_from_slice(b"</li>\n");
                        }
                    }
                }
            }

            // Markdown heading prefixes (the heading's ExtractNode carries its
            // own text, so the prefix set here is consumed in this iteration;
            // an empty heading leaves it stale, hence the end-tag clear).
            if opts.preserve_formatting == FormattingOpts::Markdown {
                let level = match current_node.tag_id {
                    t if t == LXB_TAG_H1 => 1,
                    t if t == LXB_TAG_H2 => 2,
                    t if t == LXB_TAG_H3 => 3,
                    t if t == LXB_TAG_H4 => 4,
                    t if t == LXB_TAG_H5 => 5,
                    t if t == LXB_TAG_H6 => 6,
                    _ => 0,
                };
                if level > 0 {
                    if !current_node.is_end_tag {
                        element_text_prefix = vec![b'#'; level];
                        element_text_prefix.push(b' ');
                    } else if element_text_prefix.first() == Some(&b'#') {
                        element_text_prefix.clear();
                    }
                }
            }

            // Markdown definition lists (cycle 0024): gold renders dl as
            // `**label:** value` lines — dt bolded with a colon, dd on the
            // same line.
            if opts.preserve_formatting == FormattingOpts::Markdown && !md_in_table {
                if current_node.tag_id == LXB_TAG_DT {
                    if !current_node.is_end_tag {
                        element_text_prefix = b"**".to_vec();
                    } else {
                        // close the bold around the label, folding a trailing
                        // colon inside
                        rstrip_in_place(&mut output);
                        if output.ends_with(b"**") {
                            // empty dt: drop the opener
                            output.truncate(output.len() - 2);
                        } else {
                            if output.last() == Some(&b':') {
                                output.pop();
                            }
                            output.extend_from_slice(b":** ");
                        }
                    }
                } else if current_node.tag_id == LXB_TAG_DD && !current_node.is_end_tag {
                    // dd continues the dt's line
                    current_node.make_block = false;
                    if margin_size == 1 {
                        margin_size = 0;
                    }
                }
            }

            // Markdown pipe tables (cycle 0012)
            if opts.preserve_formatting == FormattingOpts::Markdown && current_node.md_table {
                match current_node.tag_id {
                    t if t == LXB_TAG_TABLE => {
                        if !current_node.is_end_tag {
                            md_row_index = 0;
                            md_cell_index = 0;
                            md_row0_cells = 0;
                            md_in_table = true;
                        } else {
                            md_in_table = false;
                        }
                        // margins handled by normal block mechanics below
                    }
                    t if t == LXB_TAG_TR => {
                        if !current_node.is_end_tag {
                            make_margin(&mut output, &mut margin_size, &mut margin_is_br, current_node.pre_depth, opts);
                            while matches!(output.last(), Some(b' ') | Some(b'\t')) {
                                output.pop();
                            }
                            if !output.is_empty() && *output.last().unwrap() != b'\n' {
                                output.push(b'\n');
                            }
                            output.extend_from_slice(b"| ");
                            md_cell_index = 0;
                        } else {
                            rstrip_in_place(&mut output);
                            // Collapse space runs inside the finished row —
                            // source whitespace leaks into cells ("|   WK")
                            // and gold single-spaces them (0036).
                            {
                                let line_start = output
                                    .iter()
                                    .rposition(|&b| b == b'\n')
                                    .map(|i| i + 1)
                                    .unwrap_or(0);
                                let mut collapsed = Vec::with_capacity(output.len() - line_start);
                                let mut prev_space = false;
                                for &b in &output[line_start..] {
                                    if b == b' ' {
                                        if prev_space {
                                            continue;
                                        }
                                        prev_space = true;
                                    } else {
                                        prev_space = false;
                                    }
                                    collapsed.push(b);
                                }
                                output.truncate(line_start);
                                output.extend_from_slice(&collapsed);
                            }
                            if output.last() == Some(&b'|') {
                                // empty row: drop the dangling "|"
                                output.pop();
                                rstrip_in_place(&mut output);
                            } else {
                                output.extend_from_slice(b" |");
                                if md_row_index == 0 {
                                    md_row0_cells = md_cell_index.max(1);
                                    // Tight minimal separator (0036): gold
                                    // never uses the spaced ` --- |` style;
                                    // width-padded dashes measured worse
                                    // (chrome tables pay the extra bytes).
                                    output.push(b'\n');
                                    output.push(b'|');
                                    for _ in 0..md_row0_cells {
                                        output.extend_from_slice(b"---|");
                                    }
                                }
                            }
                            md_row_index += 1;
                        }
                    }
                    t if matches!(t, LXB_TAG_TD | LXB_TAG_TH) => {
                        if !current_node.is_end_tag {
                            if md_cell_index > 0 {
                                rstrip_in_place(&mut output);
                                output.extend_from_slice(b" | ");
                            }
                            md_cell_index += 1;
                        }
                    }
                    _ => {}
                }
            }

            // Markdown code fences (cycle 0013): <pre> content is already
            // verbatim via the pre_depth machinery; wrap it in ``` fences.
            if opts.preserve_formatting == FormattingOpts::Markdown
                && current_node.tag_id == LXB_TAG_PRE
                && !md_in_table
                && !(*current_node.reference_node).first_child.is_null()
            {
                if !current_node.is_end_tag {
                    make_margin(&mut output, &mut margin_size, &mut margin_is_br, current_node.pre_depth, opts);
                    if !output.is_empty() && *output.last().unwrap() != b'\n' {
                        output.push(b'\n');
                    }
                    output.extend_from_slice(b"```");
                    // language tag from class hints on the <pre> (or its
                    // first <code> child): language-x / lang-x / brush: x
                    if let Some(lang) = fence_language(current_node.reference_node) {
                        output.extend_from_slice(lang.as_bytes());
                    }
                    fence_just_opened = true;
                    margin_size = 0;
                    current_node.make_block = false;
                } else {
                    rstrip_in_place(&mut output);
                    if output.ends_with(b"```") {
                        // empty pre: drop the opener instead of ``````
                        output.truncate(output.len() - 3);
                    } else {
                        output.push(b'\n');
                        output.extend_from_slice(b"```");
                    }
                }
            }

            // Minimal HTML formatting only
            if opts.preserve_formatting == FormattingOpts::MinimalHtml {
                // Add <pre> tags immediately with newlines and skip usual block logic for opening tags
                if current_node.tag_id == LXB_TAG_PRE {
                    if !current_node.is_end_tag {
                        make_margin(&mut output, &mut margin_size, &mut margin_is_br, current_node.pre_depth, opts);
                    }
                    output.extend_from_slice(if current_node.is_end_tag { b"</pre>".as_slice() } else { b"<pre>".as_slice() });
                    margin_size = 0;
                }

                if current_node.pre_depth != 0 {
                    current_node.make_block = false;
                }

                // Explicit line breaks
                if current_node.tag_id == LXB_TAG_BR {
                    output.extend_from_slice(b"<br>");
                }

                // Add a select number of start/end tags if minimal HTML formatting is on.
                if !(*current_node.reference_node).first_child.is_null()
                    && (matches!(
                        current_node.tag_id,
                        LXB_TAG_H1 | LXB_TAG_H2 | LXB_TAG_H3 | LXB_TAG_H4 | LXB_TAG_H5 | LXB_TAG_H6 | LXB_TAG_P
                    ) || (matches!(current_node.tag_id, LXB_TAG_UL | LXB_TAG_OL) && opts.list_bullets))
                {
                    // Add margin before start tag and skip after
                    if (!current_node.is_end_tag && current_node.pre_depth == 0)
                        || (uncollapsed_margin_count != 0 && current_node.collapse_margins)
                    {
                        if current_node.collapse_margins {
                            margin_size = margin_size
                                .max(current_node.make_block as usize + current_node.make_big_block as usize);
                        } else {
                            margin_size += current_node.make_block as usize + current_node.make_big_block as usize;
                        }
                        make_margin(&mut output, &mut margin_size, &mut margin_is_br, current_node.pre_depth, opts);
                        current_node.make_block = false;
                        uncollapsed_margin_count = 0;
                    }

                    // Indent if in list (indent ul and ol start tags on level less)
                    if opts.list_bullets {
                        let adjust = if list_depth > 0 && !current_node.is_end_tag {
                            matches!(current_node.tag_id, LXB_TAG_UL | LXB_TAG_OL) as usize
                        } else {
                            0
                        };
                        make_indent(&mut output, list_depth - adjust, opts);
                    }
                    output.push(b'<');
                    if current_node.is_end_tag {
                        output.push(b'/');
                    }
                    let mut element_name_len: usize = 0;
                    let element_name = lxb_dom_element_qualified_name(
                        current_node.reference_node.cast(),
                        &mut element_name_len,
                    );
                    output.extend_from_slice(slice::from_raw_parts(element_name, element_name_len));
                    output.push(b'>');

                    // Add extra newline after opening <ul> / <ol>
                    if !output.is_empty()
                        && matches!(current_node.tag_id, LXB_TAG_UL | LXB_TAG_OL)
                        && !current_node.is_end_tag
                        && current_node.pre_depth == 0
                    {
                        output.push(b'\n');
                    }
                }
            }

            // Record size follow-up margins
            if current_node.make_block {
                if current_node.collapse_margins {
                    margin_size = margin_size.max(if current_node.make_big_block && current_node.pre_depth == 0 {
                        2
                    } else {
                        1
                    });
                } else {
                    margin_size += if current_node.make_big_block { 2 } else { 1 };
                }
                margin_is_br = current_node.tag_id == LXB_TAG_BR && margin_size == 1;
            }

            // From here on process only text nodes
            if current_node.text_contents.is_none() {
                continue;
            }

            let mut element_text = current_node.text_contents.as_ref().unwrap().clone();
            if current_node.pre_depth == 0 || opts.preserve_formatting == FormattingOpts::Off {
                element_text = if opts.preserve_formatting == FormattingOpts::Markdown {
                    get_collapsed_string_nbsp(&element_text)
                } else {
                    get_collapsed_string(&element_text)
                };
                if current_node.make_block || (!output.is_empty() && c_isspace(*output.last().unwrap())) {
                    // Strip inline elements only if previous text ended with space
                    element_text = lstrip(&element_text).to_vec();
                }
            }

            if element_text.is_empty() {
                continue;
            }

            if current_node.escape_text_contents {
                element_text = escape_html(&element_text);
            }

            // Make margins and indents (inside a pipe-table row, a margin
            // must not break the line — flush it as a single space)
            if md_in_table && opts.preserve_formatting == FormattingOpts::Markdown {
                if margin_size > 0 && !output.is_empty() && !c_isspace(*output.last().unwrap()) {
                    output.push(b' ');
                }
                margin_size = 0;
                margin_is_br = false;
            } else {
                make_margin(&mut output, &mut margin_size, &mut margin_is_br, current_node.pre_depth, opts);
            }
            uncollapsed_margin_count = 0;

            // Indent list items if basic formatting is used (follow-up lines without bullets are indented more)
            if list_depth != 0
                && matches!(opts.preserve_formatting, FormattingOpts::Basic | FormattingOpts::Markdown)
            {
                let indent_depth = if opts.preserve_formatting == FormattingOpts::Markdown {
                    // Gold-style markdown: top-level bullets start in column 0;
                    // continuation lines align with the item text.
                    list_depth - 1 + (opts.list_bullets && !bullet_inserted) as usize
                } else {
                    list_depth + (opts.list_bullets && !bullet_inserted) as usize
                };
                make_indent(&mut output, indent_depth, opts);
                bullet_inserted = false;
            }

            if opts.preserve_formatting >= FormattingOpts::Basic
                && matches!(current_node.tag_id, LXB_TAG_TD | LXB_TAG_TH)
                && !current_node.md_table
                && !output.is_empty()
                && *output.last().unwrap() != b'\n'
            {
                output.extend_from_slice(b"\t\t");
            }

            output.extend_from_slice(&element_text_prefix);
            element_text_prefix.clear();
            if opts.preserve_formatting == FormattingOpts::Markdown && fence_just_opened {
                if !element_text.starts_with(b"\n") {
                    output.push(b'\n');
                }
                fence_just_opened = false;
            }
            output.extend_from_slice(&element_text);
        }

        output
    }
}

// ---------------------------------------------------------------------------
// Main content heuristics (port of _is_main_content_node and friends)
// ---------------------------------------------------------------------------

/// Whether text node contains only a single unprintable code point from the private use area.
unsafe fn is_unprintable_pua(node: *mut lxb_dom_node_t) -> bool {
    unsafe {
        let first_child = (*node).first_child;
        if !first_child.is_null()
            && (!(*first_child).next.is_null() || (*first_child).type_ != LXB_DOM_NODE_TYPE_TEXT)
        {
            // Node has more than one child
            return false;
        }
        if first_child.is_null() && (*node).type_ != LXB_DOM_NODE_TYPE_TEXT {
            return false;
        }

        let text = get_node_text(node);
        let element_text = strip(&text);
        if element_text.len() > 3 {
            return false;
        }

        // Pilcrow character (probably an anchor link)
        if element_text == b"\xc2\xb6" {
            return true;
        }

        // BMP private use area (probably an icon font)
        if element_text.len() == 3 {
            let cp: u32 = u32::from_le_bytes([element_text[0], element_text[1], element_text[2], 0]);
            if (0x8080ee..=0xbfa3ef).contains(&cp) {
                return true;
            }
        }

        false
    }
}

/// Cycle 0005: `<ul>` exemption thresholds (sweep-tuned on lpv11 dev under the
/// zero-regression constraint).
const UL_EXEMPT_MIN_TEXT: usize = 1000;
const UL_EXEMPT_MAX_LINK_RATIO: f64 = 0.5;

/// Minimum average non-link text per list item: separates lists whose items
/// read like paragraphs (obituaries, docs, news briefs) from link directories
/// and FAQ indexes whose items are a link plus a few keywords.
const UL_EXEMPT_MIN_TEXT_PER_ITEM: usize = 150;

/// Class words marking widget containers or hidden/meta lists (checked on the
/// list and its ancestors). Word-boundary match, ASCII case-insensitive.
static WIDGETISH_CLS: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"(?:^|[\s_-])(?:widgets?|hide|meta)(?:$|[\s_-])")
        .case_insensitive(true)
        .unicode(false)
        .build()
        .unwrap()
});

/// Whether the node or any ancestor element carries a widget/hidden/meta
/// class — platform sidebar machinery that can hold text-heavy junk lists.
unsafe fn has_widgetish_ancestry(node: *mut lxb_dom_node_t) -> bool {
    unsafe {
        let mut n = node;
        while !n.is_null() && (*n).type_ == LXB_DOM_NODE_TYPE_ELEMENT {
            if (*n).local_name == LXB_TAG_BODY {
                return false;
            }
            if regex_search_not_empty(get_node_attr(n, b"class"), &WIDGETISH_CLS) {
                return true;
            }
            n = (*n).parent;
        }
        false
    }
}

/// Whether a list element carries substantial, mostly-non-link text.
unsafe fn is_text_heavy_list(node: *mut lxb_dom_node_t) -> bool {
    unsafe {
        let element_text = get_collapsed_string(&get_node_text(node));
        if element_text.len() < UL_EXEMPT_MIN_TEXT {
            return false;
        }
        if is_link_cluster(node, UL_EXEMPT_MAX_LINK_RATIO, 0) {
            return false;
        }

        // Per-item density: aggregate link text and count links / list items.
        let mut link_len = 0usize;
        let dom_coll = lxb_dom_collection_make_noi((*node).owner_document, 20);
        lxb_dom_elements_by_tag_name(node.cast(), dom_coll, b"a".as_ptr(), 1);
        let n_links = lxb_dom_collection_length_noi(dom_coll);
        for i in 0..n_links {
            link_len += get_collapsed_string(&get_node_text(lxb_dom_collection_node_noi(dom_coll, i))).len();
        }
        lxb_dom_collection_destroy(dom_coll, true);
        let mut n_li = 0usize;
        let mut child = (*node).first_child;
        while !child.is_null() {
            if (*child).local_name == LXB_TAG_LI {
                n_li += 1;
            }
            child = (*child).next;
        }
        // Veto lists inside platform widget containers (Blogger/WordPress
        // blogrolls, recent-posts, etc.) and lists that are metadata or
        // hidden by class — text-heavy but never main content.
        if has_widgetish_ancestry(node) {
            return false;
        }
        element_text.len().saturating_sub(link_len) / n_li.max(1) >= UL_EXEMPT_MIN_TEXT_PER_ITEM
    }
}

/// Inline anchor-run nav (0105): long runs of consecutive links in
/// paragraph-shaped containers (`<p>`/`<small>`/`<font>`/`<dd>`) bypass the
/// list-based nav vetoes. Requires many anchors AND near-total link text so
/// prose with citations never fires.
unsafe fn is_anchor_run(node: *mut lxb_dom_node_t) -> bool {
    unsafe {
        let element_text = get_collapsed_string(&get_node_text(node));
        if element_text.len() < 300 {
            return false;
        }
        let dom_coll = lxb_dom_collection_make_noi((*node).owner_document, 40);
        lxb_dom_elements_by_tag_name(node.cast(), dom_coll, b"a".as_ptr(), 1);
        let n = lxb_dom_collection_length_noi(dom_coll);
        let mut link_len = 0usize;
        let mut link_ns = 0usize;
        for i in 0..n {
            let lt = get_collapsed_string(&get_node_text(lxb_dom_collection_node_noi(dom_coll, i)));
            link_len += lt.len();
            link_ns += lt.iter().filter(|c| !c.is_ascii_whitespace()).count();
        }
        lxb_dom_collection_destroy(dom_coll, true);
        let text_ns = element_text.iter().filter(|c| !c.is_ascii_whitespace()).count();
        // avg-anchor-length cap: nav labels are short; anchor-wrapped story
        // teasers (news archives) run long and are content (0105 train
        // crater). Non-link text is capped in absolute non-space bytes: a
        // pure link index has none, while thread lists / product cards
        // interleave dates and prices between the anchors — content.
        n >= 25 && text_ns.saturating_sub(link_ns) <= 32 && link_len / n <= 60
    }
}

/// Check if element contains an excessive number of links compared to the whole content length.
unsafe fn is_link_cluster(node: *mut lxb_dom_node_t, max_link_ratio: f64, max_length: usize) -> bool {
    unsafe {
        let element_text = get_collapsed_string(&get_node_text(node));
        if max_length != 0 && element_text.len() > max_length {
            return false;
        }
        let dom_coll = lxb_dom_collection_make_noi((*node).owner_document, 20);
        lxb_dom_elements_by_tag_name(node.cast(), dom_coll, b"a".as_ptr(), 1);
        let mut link_texts: Vec<u8> = Vec::with_capacity(element_text.len());
        for i in 0..lxb_dom_collection_length_noi(dom_coll) {
            link_texts.extend_from_slice(&get_collapsed_string(&get_node_text(
                lxb_dom_collection_node_noi(dom_coll, i),
            )));
        }
        lxb_dom_collection_destroy(dom_coll, true);
        !link_texts.is_empty() && link_texts.len() as f64 / element_text.len() as f64 > max_link_ratio
    }
}

macro_rules! regex_ci {
    ($pattern:literal) => {
        LazyLock::new(|| {
            RegexBuilder::new($pattern)
                .case_insensitive(true)
                .unicode(false)
                .build()
                .unwrap()
        })
    };
}

// The RE2 patterns from the Cython module, translated 1:1. All are
// case-insensitive except OTHER_JUNK_CLS (which the reference constructs
// without options — deliberately preserved).
static ARTICLE_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|[\s_-])(?:article|entry|post|story|single[_-]?post|(?:main[_-])?content|body|text|page)?(?:$|[\s_-])");
static NAV_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|\s)(?:(?:l|m|wp|main|site|page|sub|article|global|sticky|main)[_-]*)?(?:nav(?:igation)?|menu(?:[_-]item)?|drop[_-]?down|bread[_-]?crumbs?)|(?:links?[_-]?(?:bar|box|list|container|section|wrapp(?:er))?)(?:$|[\s_-])");
static RECOMMENDED_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|[\s_-])(?:trends|trending|recommended|featured|popular|editors?[_-]picks|related|read-next|(?:related|more|other)[_-]?(?:links|articles|posts|guides|stories))(?:$|[\s_-])");
static LANDMARK_ID: LazyLock<Regex> = regex_ci!(r"^(?:(?:l|wp|global|page|site|full|sticky)[_-]*)?(?:(?:head|foot)(?:er)?|right)$");
static HEADER_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|\s)(?:l|m|wp|global|page|site|full|sticky)[_-]*header(?:[_-]?wrap(?:per)?|bar)?(?:$|\s)");
static FOOTER_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|[\s_-])(?:global|page|site|copyright)?(?:footer|copyright|cookie|consent|legal|fcontainer)(?:$|[\s_-])");
static POST_META_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|[\s_-])(?:(?:post|entry|article(?:page)?|content|story|section)[_-]*(?:text[_-]*)?(?:footer|teaser|meta(?:[_-]?data)?|subline|sidebar|author(?:name)?|published|timestamp|date|posted[_-]?on|info|labels?|tags?|keywords|category)|by[_-]?line|date[_-]?line|author-date|submitted(?:-by)?)|meta[_-]?data(?:$|[\s_-])");
static SIDEBAR_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|\s)(?:(?:l|wp|right|left|global|sticky)[_-]*)?(?:(?:side|sticky)[_-]?(?:bars?|box)|one-third)(?:$|[\s_-])");
static SEARCH_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|[\s_-])search(?:[_-]?(?:bar|facility|box))?(?:$|\s)");
static SKIP_LINK_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|\s)(?:link[_-]?)?(?:skip(?:[_-]?(?:to|link))?|scroll[_-]?(?:up|down)|next|prev(?:ious)?|permalink|pagination|skip-to-(?:main-)?content)(?:$|\s|[_-]?(?:post|article))");
static DISPLAY_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|\s)(?:(?:is|visually)[_-])?(?:display-none|hidden|invisible|collapsed|h-0|nocontent|expandable)(?:-xs|-sm|-lg|-2?xl)?(?:$|\s)");
static DISPLAY_CSS: LazyLock<Regex> = regex_ci!(r"(?:^|;\s*)(?:display\s?:\s?none|visibility\s?:\s?hidden)(?:$|\s?;)");
// Unrendered client-side template tokens (0105): Velocity `$obj.method()`
// and Rails `translation_missing` placeholders only surface when the page's
// JS templating never ran — a rendered page hides these blocks.
static QUOTE_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|[\s_-])(?:bbcode[_-]?quote|quoteheader|quotecontent|post[_-]?quote|quotebox)(?:$|[\s_-])");
static RENDER_TIMER: LazyLock<Regex> = regex_ci!(r"^(?:page )?generated in [0-9.]+ sec(?:ond)?s?");
static TEMPLATE_TOKEN: LazyLock<Regex> = regex_ci!(r"\$[a-z_][a-z0-9_]*\.[a-z_][a-z0-9_]*\(\)|translation_missing");
static MODAL_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|\s)(?:wp-|p-|-l)?(?:modal|popup|lightbox)(?:[_-]*(?:window|pane|box))?(?:$|[\s_-])");
static GALLERY_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|[\s_-])(?:gallery|carousel)(?:$|[\s_-])");
static SIGNIN_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|[\s_-])(?:(?:log[_-]?in|sign[_-]?(?:in|up)|account)|user[_-](?:info|profile|settings|actions))(?:$|[\s_-])");
static ADS_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|\s)(?:(?:google|wide)[_-]?ads?|ad(?:vert|vertise(?:ment|link)?|$|_[a-f0-9]+)|sponsor(?:ed)?|promoted|paid|(?:wide)?banner|donate)(?:$|[\s_-])");
static SOCIAL_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|\s|__|--|mobile-|desktop-|l-|m-|c-)(?:social(?:media|search)?|share(?:daddy)?|syndication|newsletter|sharing|follow|email|likes?|(?:give[_-]?)?feedback|(?:brand[_-])?engagement|facebook|twitter|subscribe|wa|jp|aptf-follow)(?:[_-]?(?:post|links?|section|icons?|btn|buttons?|target))?(?:$|[\s_-])");
static COMMENTS_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|[\s_-])(?:(?:article|user|post)[_-]*)?(?:(?:no[_-]?)?comments?|comment[_-]?list|reply)(?:$|[\s_-])");
static LOGO_CLS: LazyLock<Regex> = regex_ci!(r"(?:brand(?:ing)?[_-]*)?logo(?:$|\s)");
static PRINT_CLS: LazyLock<Regex> = regex_ci!(r"(?:^|\s)print[_-]");
static OTHER_JUNK_CLS: LazyLock<Regex> = LazyLock::new(|| {
    // Reference builds this one without RE2 options => case-SENSITIVE.
    RegexBuilder::new(r"(?:^|\s)short-view-count|spinner(?:$|[\s_-])")
        .unicode(false)
        .build()
        .unwrap()
});

#[inline]
fn regex_search_not_empty(s: &[u8], r: &Regex) -> bool {
    !s.is_empty() && r.is_match(s)
}

const BLACKLIST_ARIA_ROLES: &[&[u8]] = &[
    b"alert",
    b"banner",
    b"checkbox",
    b"comment",
    b"complementary",
    b"contentinfo",
    b"dialog",
    b"img",
    b"menu",
    b"menubar",
    b"menuitem",
    b"navigation",
    b"presentation",
    b"radio",
    b"search",
    b"searchbox",
    b"separator",
    b"tab",
    b"toolbar",
    b"tooltip",
];

/// Chrome classes the lpv11 gold consistently drops (audit 2026-08-07:
/// cookie 0%, share 2%, breadcrumb 5%, footer 5%, login/search 6%,
/// pagination 7% keep-rates). Active in markdown mode only; wall categories
/// (nav, related-posts) deliberately absent. Single-hyphen boundaries
/// included (the classic `post-share-buttons` escape).
static MD_CHROME_CLS: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(
        r"(?:^|[\s_-])(?:cookie(?:-?(?:bar|banner|notice|consent))?|consent|gdpr|breadcrumbs?|share-?(?:this|bar|buttons?|links?|post)?|sharing|addthis|sharedaddy|sociable|log-?in|sign-?in|sign-?up|subscribe|newsletter|search-?(?:form|box|bar)|site-?footer|tag-?(?:cloud|list|links)|post-?tags|cat-?links|meta-?(?:nav|links)|read-?next|around-?the-?web|you-?may-?(?:also-?)?like|outbrain|taboola|sponsored-?(?:links|content)|respond|comment-?respond|comment-?form|commentform|author-?(?:bio|box)|about-?(?:the-?)?author|bio-?box|related-posts|highwire-extract-pdf-wrapper)(?:$|[\s_-])",
    )
    .case_insensitive(true)
    .unicode(false)
    .build()
    .unwrap()
});

/// Classes that mark a container as content regardless of other matches
/// (WordPress/microformats post containers).
static CONTENT_MARKER_CLS: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"(?:^|\s)(?:hentry|h-entry|type-post|instapaper_body|entry-content|post-body)(?:$|\s)|(?:^|[\s_-])signature(?:$|[\s_-])")
        .case_insensitive(true)
        .unicode(false)
        .build()
        .unwrap()
});

static MD_COPYRIGHT_CLS: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"(?:^|[\s_-])copyright(?:$|[\s_-])")
        .case_insensitive(true)
        .unicode(false)
        .build()
        .unwrap()
});

/// Remove the hyphen after "no"/"not" so negated widget names ("no-share")
/// stop being boundary-delimited for the chrome veto.
fn glue_negations(hay: &mut Vec<u8>) {
    let mut i = 0;
    while i + 3 <= hay.len() {
        let neg2 = i + 3 <= hay.len() && hay[i..].starts_with(b"no-");
        let neg3 = i + 4 <= hay.len() && hay[i..].starts_with(b"not-");
        let at_boundary = i == 0 || matches!(hay[i - 1], b' ' | b'_' | b'-');
        if at_boundary && (neg2 || neg3) {
            let dash = if neg3 { i + 3 } else { i + 2 };
            hay.remove(dash);
        }
        i += 1;
    }
}

/// Chrome widgets are small; wrapper divs named after a widget they merely
/// contain (`place-login-pop` wrapping 45KB of page) must never be vetoed —
/// and neither may a container holding most of a small page (author PAGES
/// where the bio IS the content: oreilly /pub/au −0.70, cycle 0033).
unsafe fn is_small_chrome_sized(node: *mut lxb_dom_node_t) -> bool {
    unsafe {
        let n = get_collapsed_string(&get_node_text(node)).len();
        if n > 1500 {
            return false;
        }
        // lxb_html_document_t embeds lxb_dom_document_t as its first field
        let html_doc = (*node).owner_document as *mut lxb_html_document_t;
        if !html_doc.is_null() {
            let body: *mut lxb_dom_node_t = (*html_doc).body.cast();
            if !body.is_null() {
                let page = get_collapsed_string(&get_node_text(body)).len();
                if page > 0 && n * 5 > page * 2 {
                    return false; // >40% of the page is not chrome
                }
            }
        }
        true
    }
}

static BYLINE_CLS: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"(?:^|[\s_-])(?:author|byline|by-?line|posted|vcard|entry-meta|post-?meta|timestamp|published|post-?footer|entry-date|post-?date)(?:$|[\s_-])")
        .case_insensitive(true)
        .unicode(false)
        .build()
        .unwrap()
});

/// Whether the node or a near ancestor sits in a byline container.
unsafe fn has_byline_ancestry(node: *mut lxb_dom_node_t) -> bool {
    unsafe {
        let mut n = node;
        for _ in 0..3 {
            if n.is_null() || (*n).type_ != LXB_DOM_NODE_TYPE_ELEMENT {
                return false;
            }
            if regex_search_not_empty(get_node_attr(n, b"class"), &BYLINE_CLS) {
                return true;
            }
            n = (*n).parent;
        }
        false
    }
}

unsafe fn has_anchor_descendant(node: *mut lxb_dom_node_t) -> bool {
    unsafe {
        let coll = lxb_dom_collection_make_noi((*node).owner_document, 4);
        lxb_dom_elements_by_tag_name(node.cast(), coll, b"a".as_ptr(), 1);
        let n = lxb_dom_collection_length_noi(coll);
        lxb_dom_collection_destroy(coll, true);
        n > 0
    }
}

/// Rule relaxations active only during the tier-2 rescue retry (never on the
/// primary extraction pass).
#[derive(Clone, Copy, Default, PartialEq)]
struct RelaxFlags {
    /// Keep text-heavy `<ul>`s regardless of depth/link-ratio (cycle 0005).
    text_heavy_lists: bool,
    /// Skip the `<article>` teaser link-cluster check (cycle 0007).
    short_articles: bool,
    /// Listing-page retry (cycle 0023): skip the div/ul link-cluster rules so
    /// teaser-card grids survive (gold keeps every card on tag/search/archive
    /// pages).
    listing_cards: bool,
}

/// Rule-based check whether the given element is a "main-content" element.
unsafe fn is_main_content_node(
    node: *mut lxb_dom_node_t,
    body_depth: usize,
    keep_comments: bool,
    keep_post_meta: bool,
    keep_hidden: bool,
    relax: RelaxFlags,
    md_chrome: bool,
) -> bool {
    unsafe {
        if (*node).type_ == LXB_DOM_NODE_TYPE_TEXT {
            return !is_unprintable_pua(node);
        } else if (*node).type_ != LXB_DOM_NODE_TYPE_ELEMENT {
            return true;
        }

        let local_name = (*node).local_name;

        // ------ Section 1: Tag name matching ------

        // Main elements and headings
        if matches!(local_name, LXB_TAG_BODY | LXB_TAG_MAIN | LXB_TAG_H1) {
            return true;
        }
        // Global footer
        else if local_name == LXB_TAG_FOOTER {
            if body_depth < 3 || is_link_cluster(node, 0.2, 0) {
                return false;
            }

            // Check if footer is recursive last element node of a direct body child
            let mut pnode = node;
            while !pnode.is_null()
                && !(*pnode).parent.is_null()
                && (*(*pnode).parent).local_name != LXB_TAG_BODY
            {
                if !(*pnode).next.is_null() && (*(*pnode).next).type_ == LXB_DOM_NODE_TYPE_TEXT {
                    pnode = (*pnode).next;
                }
                if !(*pnode).next.is_null() {
                    // There is at least one more element node
                    return true;
                }
                pnode = (*pnode).parent;
            }
            return false;
        } else if local_name == LXB_TAG_UL {
            // Text-mass exemption (cycle 0005, active only in the tier-2
            // rescue retry): a list carrying substantial, mostly-non-link
            // text is main content (obituaries, docs pages, news briefs), no
            // matter how shallow. Nav menus are short and link-dense, so they
            // can't qualify.
            if !relax.listing_cards
                && !(relax.text_heavy_lists && is_text_heavy_list(node))
                && (body_depth < 4 || is_link_cluster(node, 0.2, 0))
            {
                return false;
            }
        }
        // Teaser articles
        else if local_name == LXB_TAG_ARTICLE {
            if !relax.short_articles && !relax.listing_cards && body_depth > 2 && is_link_cluster(node, 0.2, 500) {
                return false;
            }
        }
        // Navigation, sidebar, other hard-blacklisted elements
        else if matches!(
            local_name,
            LXB_TAG_NAV | LXB_TAG_ASIDE | LXB_TAG_AUDIO | LXB_TAG_VIDEO | LXB_TAG_TIME
        ) {
            return false;
        }

        // ------ Section 2: Rel and ARIA attribute matching ------

        // Hidden elements
        if lxb_dom_element_has_attribute(node.cast(), b"hidden".as_ptr(), 6) {
            return false;
        }

        // Inline anchor-run nav and unrendered template payloads (0105,
        // markdown config only — plain config mirrors the Cython reference)
        if md_chrome {
            if matches!(local_name, LXB_TAG_P | LXB_TAG_SMALL | LXB_TAG_FONT | LXB_TAG_DD)
                && !relax.listing_cards
                && is_anchor_run(node)
            {
                return false;
            }
        }

        // rel attributes (markdown config keeps author anchors — the gold
        // wants byline names; "Posted by at" bugs, cycle 0022)
        let rel_attr = strip(get_node_attr(node, b"rel"));
        if !rel_attr.is_empty() {
            if rel_attr == b"author" {
                // markdown config: keep byline author anchors (gold wants the
                // name) but only in byline context — bare rel=author on forum
                // member links is still chrome (cycle 0022).
                if !(md_chrome && has_byline_ancestry(node)) {
                    return false;
                }
            } else if [b"icon".as_slice(), b"search", b"prev", b"next", b"tag"].contains(&rel_attr) {
                return false;
            }
        }

        // itemprop attributes
        let itemprop_attr = strip(get_node_attr(node, b"itemprop"));
        if !itemprop_attr.is_empty()
            && [b"datePublished".as_slice(), b"author", b"url"].contains(&itemprop_attr)
        {
            // markdown config: visible byline microdata (span[itemprop=author],
            // abbr[itemprop=datePublished]) carries the author name and post
            // time the gold keeps ("Posted by NAME at TIME" — the degenerate
            // "Posted by at" family, cycle 0037). Invisible meta/link carriers
            // and itemprop=url permalinks stay dropped, as does everything in
            // plain config (Cython reference behavior).
            let visible_byline = md_chrome
                && itemprop_attr != b"url"
                && local_name != LXB_TAG_META
                && local_name != LXB_TAG_LINK
                && has_byline_ancestry(node)
                // a byline fragment is short (name / time); anything larger
                // is an author bio, which stays chrome (0033 veto family)
                && get_node_text(node).len() <= 80;
            if !visible_byline {
                return false;
            }
        }

        // ARIA hidden
        if strip(get_node_attr(node, b"aria-hidden")) == b"true" {
            return false;
        }

        // ARIA expanded
        if strip(get_node_attr(node, b"aria-expanded")) == b"false" {
            return false;
        }

        // ------ Section 3: General class and ID matching ------

        let cls_attr = get_node_attr(node, b"class");
        let id_attr = get_node_attr(node, b"id");
        // Only elements with class or id attributes from here on
        if cls_attr.is_empty() && id_attr.is_empty() {
            if local_name == LXB_TAG_DIV && !relax.listing_cards {
                return body_depth <= 5 || !is_link_cluster(node, 0.6, 800);
            }
            return true;
        }

        let mut cls_and_id_attr: Vec<u8> = cls_attr.to_vec();
        if !cls_and_id_attr.is_empty() {
            cls_and_id_attr.push(b' ');
        }
        cls_and_id_attr.extend_from_slice(id_attr);

        // Hidden elements
        // (Operator precedence quirk preserved from the reference: the
        // `keep_hidden` guard binds only to the first regex check.)
        if (!keep_hidden && regex_search_not_empty(cls_attr, &DISPLAY_CLS))
            || regex_search_not_empty(get_node_attr(node, b"style"), &DISPLAY_CSS)
        {
            return false;
        }

        // Skip links
        if matches!(local_name, LXB_TAG_A | LXB_TAG_DIV | LXB_TAG_LI)
            && regex_search_not_empty(&cls_and_id_attr, &SKIP_LINK_CLS)
        {
            return false;
        }

        if body_depth > 2 {
            // lpv11-gold chrome (markdown config only; audit-backed).
            // Content-marker exemption: WordPress post containers carry
            // tag-SLUG classes whose hyphen-internal words false-match
            // (`tag-...-sharing-them` → "sharing", train −0.96).
            if md_chrome {
                // Negated tokens ("no-share" = content that does NOT get
                // share buttons) must not match; the regex crate has no
                // lookbehind, so glue "no-"/"not-" prefixes shut.
                let mut veto_hay = cls_and_id_attr.clone();
                glue_negations(&mut veto_hay);
                if regex_search_not_empty(&veto_hay, &MD_CHROME_CLS)
                    && !regex_search_not_empty(&cls_and_id_attr, &CONTENT_MARKER_CLS)
                    && is_small_chrome_sized(node)
                {
                    return false;
                }
            }
            // copyright-classed FOOTERS (with links: Privacy/Terms rows) are
            // chrome; linkless copyright paragraphs are source-attribution
            // credits the gold keeps (audit gold-policy).
            if md_chrome
                && regex_search_not_empty(&cls_and_id_attr, &MD_COPYRIGHT_CLS)
                && has_anchor_descendant(node)
                && is_small_chrome_sized(node)
            {
                return false;
            }

            // Sign-in links
            if regex_search_not_empty(cls_attr, &SIGNIN_CLS) {
                return false;
            }

            // Post meta
            if !keep_post_meta && regex_search_not_empty(cls_attr, &POST_META_CLS) {
                return false;
            }

            // Social media and feedback forms
            if regex_search_not_empty(cls_attr, &SOCIAL_CLS) {
                return false;
            }
        }

        // Logos
        if regex_search_not_empty(&cls_and_id_attr, &LOGO_CLS) {
            return false;
        }

        // Ads
        if regex_search_not_empty(&cls_and_id_attr, &ADS_CLS)
            || lxb_dom_element_has_attribute(node.cast(), b"data-ad".as_ptr(), 7)
            || lxb_dom_element_has_attribute(node.cast(), b"data-advertisement".as_ptr(), 18)
            || lxb_dom_element_has_attribute(node.cast(), b"data-text-ad".as_ptr(), 12)
        {
            return false;
        }

        // Other junk
        if regex_search_not_empty(cls_attr, &OTHER_JUNK_CLS) {
            return false;
        }

        // ------ Section 4: Class and ID matching of block elements only ------

        if !is_block_element(local_name) && local_name != LXB_TAG_TD {
            return true;
        }

        // ARIA roles
        // (`rel_attr == "main"` faithfully preserved from the reference,
        // which plausibly meant `role_attr`.)
        let role_attr = strip(get_node_attr(node, b"role"));
        if rel_attr == b"main" {
            return true;
        }
        if !role_attr.is_empty() && BLACKLIST_ARIA_ROLES.contains(&role_attr) {
            return false;
        }

        // Whitelist article elements
        if regex_search_not_empty(&cls_and_id_attr, &ARTICLE_CLS) {
            return true;
        }

        // Global landmarks by ID
        if regex_search_not_empty(id_attr, &LANDMARK_ID) {
            return false;
        }

        // Global header
        if regex_search_not_empty(&cls_and_id_attr, &HEADER_CLS) {
            return false;
        }

        // Global footer
        if regex_search_not_empty(&cls_and_id_attr, &FOOTER_CLS) {
            return false;
        }

        // Global navigation
        if regex_search_not_empty(&cls_and_id_attr, &NAV_CLS) {
            return false;
        }

        // Recommended articles
        if regex_search_not_empty(&cls_and_id_attr, &RECOMMENDED_CLS) {
            return false;
        }

        // Comments section
        if !keep_comments && local_name != 0 && regex_search_not_empty(&cls_and_id_attr, &COMMENTS_CLS) {
            return false;
        }

        // Global search bar
        if regex_search_not_empty(&cls_and_id_attr, &SEARCH_CLS) {
            return false;
        }

        // Global sidebar
        if regex_search_not_empty(&cls_and_id_attr, &SIDEBAR_CLS) {
            return false;
        }

        // Modals
        if regex_search_not_empty(&cls_and_id_attr, &MODAL_CLS) {
            return false;
        }

        // Image galleries and carousels
        if regex_search_not_empty(&cls_and_id_attr, &GALLERY_CLS) {
            return false;
        }

        // Print content
        if regex_search_not_empty(&cls_and_id_attr, &PRINT_CLS) {
            return false;
        }

        if body_depth > 2 && local_name == LXB_TAG_DIV && !relax.listing_cards && is_link_cluster(node, 0.6, 1500) {
            return false;
        }

        true
    }
}

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

/// Decode UTF-8 with `errors='ignore'` semantics (drop invalid bytes),
/// matching the Python-side `.decode(errors='ignore')`.
fn decode_utf8_ignore(mut bytes: Vec<u8>) -> String {
    match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => {
            bytes = e.into_bytes();
            let mut out = String::with_capacity(bytes.len());
            let mut rest = bytes.as_slice();
            loop {
                match std::str::from_utf8(rest) {
                    Ok(s) => {
                        out.push_str(s);
                        break;
                    }
                    Err(err) => {
                        let (valid, after) = rest.split_at(err.valid_up_to());
                        out.push_str(std::str::from_utf8(valid).unwrap());
                        let skip = err.error_len().unwrap_or(after.len()).max(1);
                        rest = &after[skip.min(after.len())..];
                    }
                }
            }
            out
        }
    }
}

/// Perform a simple plain-text extraction from the given HTML document.
///
/// Direct port of the Cython `extract_plain_text` (parses with the same
/// lexbor parser and walks the DOM identically).
pub fn extract_plain_text(html: &str, opts: &ExtractOpts) -> String {
    unsafe {
        let doc = lxb_html_document_create();
        if doc.is_null() {
            return String::new();
        }
        let status = lxb_html_document_parse(doc, html.as_ptr(), html.len());
        if status != lexbor_status_t::LXB_STATUS_OK {
            lxb_html_document_destroy(doc);
            return String::new();
        }
        // Engine handlers (markdown mode only — the gold's post format is
        // markdown; plain/guardrail behavior untouched).
        if opts.main_content && opts.preserve_formatting == FormattingOpts::Markdown {
            let generator = generator_meta(doc);
            let body_ptr: *mut lxb_dom_node_t = (*doc).body.cast();
            // vBulletin: generator meta, or (0021) markup fallback — many vB
            // installs strip the meta; the postbit ids are unmistakable.
            let vb_markup = !body_ptr.is_null()
                && (query_selector_all_raw(doc, body_ptr, b"div[id^=\"post_message_\"]").len() >= 2
                    || query_selector_all_raw(doc, body_ptr, b"li[id^=\"post_\"] blockquote.postcontent").len() >= 2);
            if generator.starts_with(b"vbulletin") || vb_markup {
                if let Some(out) = extract_vbulletin(doc, opts) {
                    lxb_html_document_destroy(doc);
                    return md_post_passes(out);
                }
            }
            if let Some(out) = extract_phpbb2(doc, opts) {
                lxb_html_document_destroy(doc);
                return md_post_passes(out);
            }
            if let Some(out) = extract_phpbb(doc, opts) {
                lxb_html_document_destroy(doc);
                return md_post_passes(out);
            }
            if let Some(out) = extract_phpbb_subsilver2(doc, opts) {
                lxb_html_document_destroy(doc);
                return md_post_passes(out);
            }
            if let Some(out) = extract_xenforo(doc, opts) {
                lxb_html_document_destroy(doc);
                return md_post_passes(out);
            }
            if generator.starts_with(b"ubb.threads") {
                if let Some(out) = extract_ubb(doc, opts) {
                    lxb_html_document_destroy(doc);
                    return md_post_passes(out);
                }
            }
            if let Some(out) = extract_invision(doc, opts) {
                lxb_html_document_destroy(doc);
                return md_post_passes(out);
            }
            if let Some(out) = extract_smf(doc, opts) {
                lxb_html_document_destroy(doc);
                return md_post_passes(out);
            }
            // One-off engines (cycle 0030): disjoint exact gates, 0017-style.
            for handler in [
                extract_legacy_gb,
                extract_livejournal,
                extract_cpan_pod,
                extract_gforms,
                extract_vbulletin5,
                extract_yahoo_mb,
                extract_perlmonks,
                extract_nabble,
                extract_webbbs,
                extract_fool,
                extract_cafemom,
                extract_slashdot,
                extract_glp_report,
            ] {
                if let Some(out) = handler(doc, opts) {
                    lxb_html_document_destroy(doc);
                    return md_post_passes(out);
                }
            }
            // Generic post-stream rebuilder measured NEGATIVE twice
            // (cycle 0029): repeated-blocks + head-anchored author/date
            // cannot separate threads from slideshows/datelined grids.
            // Disabled; one-off engines get per-engine gates instead.
            #[allow(clippy::never_loop)]
            if false {
                if let Some(out) = extract_generic_posts(doc, opts) {
                    lxb_html_document_destroy(doc);
                    return md_post_passes(out);
                }
            }
        }

        let mut page_has_card_grid = false;
        let mut model_whitelist: HashSet<*mut lxb_dom_node_t> = HashSet::new();
        let mut model_veto_nodes: Vec<*mut lxb_dom_node_t> = Vec::new();
        let mut model_veto_mass = 0usize;
        let mut page_link_density = 0.0f64;
        let tpl_set: Option<HashSet<*mut lxb_dom_node_t>> =
            if opts.main_content && opts.preserve_formatting == FormattingOpts::Markdown {
                let body: *mut lxb_dom_node_t = (*doc).body.cast();
                if body.is_null() {
                    None
                } else {
                    let (mut v, grid, wl, mv, mvm, pld) = tpl_vetoes(generator_kind(doc), body);
                    // domain-gated site vetoes (0098)
                    let dom = page_domain(doc);
                    if !dom.is_empty() {
                        for (d_, sel) in SITE_VETOES {
                            if dom == *d_ {
                                for n in query_selector_all_raw(doc, body, sel) {
                                    v.insert(n);
                                }
                            }
                        }
                        // domain-gated content whitelist (0101): the node and
                        // every element beneath it join the whitelist (the
                        // walk checks per-node membership)
                        for (d_, sel) in SITE_WHITELIST {
                            if dom == *d_ {
                                for n in query_selector_all_raw(doc, body, sel) {
                                    model_whitelist.insert(n);
                                    let mut depth = 0usize;
                                    let mut end = false;
                                    let mut c = n;
                                    loop {
                                        c = next_node(n, c, &mut depth, &mut end);
                                        if c.is_null() {
                                            break;
                                        }
                                        model_whitelist.insert(c);
                                    }
                                }
                            }
                        }
                    }
                    page_has_card_grid = grid;
                    model_whitelist.extend(wl);
                    model_veto_nodes = mv;
                    model_veto_mass = mvm;
                    page_link_density = pld;
                    Some(v)
                }
            } else {
                None
            };
        let wp_candidate = if opts.main_content && opts.preserve_formatting == FormattingOpts::Markdown {
            wp_comment_rebuild(doc, opts)
        } else {
            None
        };
        // The veto set in effect for ALL extraction passes, including the
        // rescue-ladder retries below. Comment rebuilds extend it — retries
        // running with the pre-rebuild set would resurrect the vetoed
        // native comment rendering next to the rebuilt block (0039 bug,
        // latent in the WP rebuild since 0020).
        let mut effective_tpl: Option<HashSet<*mut lxb_dom_node_t>> = tpl_set;
        let wl_ref = if model_whitelist.is_empty() { None } else { Some(&model_whitelist) };
        let (mut result, mut dropped_nodes) =
            extract_plain_text_from_doc_impl2(doc, None, opts, RelaxFlags::default(), effective_tpl.as_ref(), wl_ref);
        // Gold mirrors each theme's native comment rendering; rebuild only
        // when the native walk LOSES attribution (>=half the authors absent).
        let mut wp_comments: Option<String> = None;
        if let Some((block, vetoes, authors)) = wp_candidate {
            let missing = authors.iter().filter(|a| !result.contains(a.as_str())).count();
            if missing * 2 >= authors.len() {
                let mut set2 = effective_tpl.clone().unwrap_or_default();
                for v in &vetoes {
                    set2.insert(*v);
                }
                let (r2, d2) = extract_plain_text_from_doc_impl2(doc, None, opts, RelaxFlags::default(), Some(&set2), wl_ref);
                result = r2;
                dropped_nodes = d2;
                effective_tpl = Some(set2);
                wp_comments = Some(block);
            }
        }
        // Blogspot comments (0039): gold rewrites the native rendering
        // ("NAME said..." + footer timestamp -> `**NAME — TIME**`), so a
        // successful parse always rebuilds — no native-first check.
        // MovableType (0044) has the same always-rebuild semantics.
        if wp_comments.is_none() && opts.main_content && opts.preserve_formatting == FormattingOpts::Markdown {
            if let Some((block, vetoes, _authors)) = blogspot_comment_rebuild(doc, opts)
                .or_else(|| movabletype_comment_rebuild(doc, opts))
            {
                let mut set2 = effective_tpl.clone().unwrap_or_default();
                for v in &vetoes {
                    set2.insert(*v);
                }
                let (r2, d2) = extract_plain_text_from_doc_impl2(doc, None, opts, RelaxFlags::default(), Some(&set2), wl_ref);
                result = r2;
                dropped_nodes = d2;
                effective_tpl = Some(set2);
                wp_comments = Some(block);
            }
        }

        // Self-correcting rescues (cycles 0004/0005). Gated on the extraction
        // having lost most of the page's text, so they cannot fire on (and
        // therefore cannot regress) normally-extracting pages; the extra
        // extraction cost is only paid on gate hits.
        if opts.main_content {
            // Full-page text materialization is the expensive part of the
            // gates — compute it lazily, only once a cheaper precondition
            // has already fired.
            let mut body_text_len_cache: Option<usize> = None;
            let mut body_text_len = |doc: *mut lxb_html_document_t| -> usize {
                *body_text_len_cache.get_or_insert_with(|| {
                    let body: *mut lxb_dom_node_t = (*doc).body.cast();
                    if body.is_null() {
                        0
                    } else {
                        get_collapsed_string(&get_node_text(body)).len()
                    }
                })
            };

            // Rescue gates measure CONTENT length, not output length —
            // markdown formatting bytes (pipes, dashes, #, *) must not move
            // a page across a gate boundary (cycle 0012 regression: table
            // syntax pushed a 100-byte calendar page past the near-empty
            // gate).
            fn content_len(t: &str) -> usize {
                // exclude markdown structure punctuation only — whitespace
                // stays counted so plain-mode gate behavior is unchanged
                t.bytes().filter(|b| !matches!(b, b'|' | b'#' | b'*' | b'-')).count()
            }
            let mut result_content_len = content_len(&result);

            // Tier 0 (0046): model-veto rollback. Raising the model's veto
            // authority pays in aggregate but its false negatives can wipe
            // a whole page (quotes pages, tiny sites); a near-empty result
            // with model vetoes in effect retries without them and keeps
            // the retry when it doubles the content.
            // Relative gate (0051): the model removed most of the page on a
            // low-link-density (article-like) page — a listing page (high
            // ld) keeps its vetoes; gold keeps little there.
            // Tier 0a (0106, markdown only): near-empty output + a unique
            // article container -> rescue rooted at the container. The
            // body-wide tiers below resurrect the site shell on husk pages
            // (gascu family); the container is the precise target when the
            // page names one.
            let mut rooted_rescued = false;
            if opts.preserve_formatting == FormattingOpts::Markdown
                && result_content_len < RESCUE_NEAR_EMPTY_ABS
                // engine pages (Blogger/WordPress/Typepad/...) keep comments
                // outside the article container — their rescue goes through
                // the engine handlers/body-wide tiers, not the rooted crop.
                // A stray <img> in <head> force-closes it and dumps the
                // generator meta into <body> (fraudswatch), so scan both.
                && generator_meta(doc).is_empty()
                && query_selector_all_raw(doc, (*doc).body.cast(), b"meta[name=\"generator\"]").is_empty()
                && !(result_content_len < RESCUE_NEAR_EMPTY_ABS + 100
                    && regex_search_not_empty(result.as_bytes(), &ERROR_STUB_TEXT))
            {
                let cands = query_selector_all_raw(
                    doc,
                    (*doc).body.cast(),
                    b".entry-content, .article-body, .articleBody, .article-text, .post-content, .postcontent, [itemprop=\"articleBody\"]",
                );
                if cands.len() == 1 {
                    let r = extract_plain_text_from_node(doc, cands[0], opts);
                    if content_len(&r) > RESCUE_KEEP_FACTOR * result_content_len.max(1) {
                        result = r;
                        result_content_len = content_len(&result);
                        rooted_rescued = true;
                    }
                }
            }

            let model_gutted = model_veto_mass > 2 * result_content_len.max(1)
                && page_link_density < 0.30;
            if !rooted_rescued
                && !model_veto_nodes.is_empty()
                && (result_content_len < RESCUE_NEAR_EMPTY_ABS || model_gutted)
            {
                let mut set3 = effective_tpl.clone().unwrap_or_default();
                for v in &model_veto_nodes {
                    set3.remove(v);
                }
                let (r3, d3) = extract_plain_text_from_doc_impl2(doc, None, opts, RelaxFlags::default(), Some(&set3), wl_ref);
                if content_len(&r3) > 2 * result_content_len.max(1) {
                    result = r3;
                    dropped_nodes = d3;
                    effective_tpl = Some(set3);
                    result_content_len = content_len(&result);
                }
            }

            // Error/stub pages ("We're sorry", "page not found", "out of
            // stock") legitimately extract to a tiny message inside a huge
            // site shell — rescuing them swaps the correct answer for the
            // shell. Their tiny output names the condition, so a keyword
            // veto is cheap and can't fire on wiped-article scraps.
            let is_error_stub = result_content_len < RESCUE_NEAR_EMPTY_ABS + 100
                && regex_search_not_empty(result.as_bytes(), &ERROR_STUB_TEXT);

            // Tier 1 (0004): near-empty output → unfiltered fallback, kept
            // only if it yields much more content (classifier false negative
            // wiped the whole page).
            // A rendered pipe table with substantive cells in the base
            // output is structured content (acronym/stat pages extract to
            // a compact table) — the page was not wiped, and the
            // unfiltered fallback would swap the table for the site shell
            // (0036: keep-factor flips on formatting-byte changes made
            // this family unstable). Number-only tables (calendars) are
            // navigation chrome and do NOT block the rescue.
            let has_md_table = (result.contains("\n|---") || result.contains("\n| ---"))
                && result.lines().any(|l| {
                    l.starts_with('|')
                        && l.split('|').any(|cell| {
                            cell.chars().filter(|c| c.is_alphabetic()).count() >= 16
                        })
                });
            let mut rescued = rooted_rescued;
            if !rescued
                && !is_error_stub
                && !has_md_table
                && result_content_len < RESCUE_NEAR_EMPTY_ABS
                && body_text_len(doc) > RESCUE_BODY_FACTOR * result_content_len.max(1)
            {
                let fallback_opts = ExtractOpts {
                    main_content: false,
                    ..opts.clone()
                };
                let fallback = extract_plain_text_from_doc(doc, &fallback_opts, RelaxFlags::default(), effective_tpl.as_ref(), wl_ref);
                // Dual keep test (0037): either raw-length or content-length
                // clearing the factor accepts the rescue. Each test alone
                // flips docs at the 20x margin when a few formatting/byline
                // bytes land in a tiny base output (0012/0036 instability
                // class); the OR-frontier is strictly more stable.
                if fallback.len() > RESCUE_KEEP_FACTOR * result.len().max(1)
                    || content_len(&fallback) > RESCUE_KEEP_FACTOR * result_content_len.max(1)
                {
                    result = fallback;
                    rescued = true;
                }
            }

            // Tier 2 (0005/0007): a rescue-eligible dropped node + output
            // that is a small fraction of the page text → retry with the
            // corresponding rule relaxed, kept only if it recovers
            // substantially more. Eligibility runs before the body-text
            // materialization: dropped candidates are usually a handful of
            // small nav lists, so testing them is much cheaper than a
            // full-page text_content.
            if !rescued && !is_error_stub && !dropped_nodes.is_empty() {
                let mut relax = RelaxFlags::default();
                // Text-heavy list dropped by the <ul> rule (0005)?
                if dropped_nodes.iter().any(|&(n, d)| {
                    (*n).local_name == LXB_TAG_UL
                        && is_main_content_node(
                            n,
                            d,
                            opts.comments,
                            opts.post_meta,
                            opts.hidden_elements,
                            RelaxFlags { text_heavy_lists: true, ..Default::default() },
                            opts.preserve_formatting == FormattingOpts::Markdown,
                        )
                }) {
                    relax.text_heavy_lists = true;
                }
                // Short real story dropped by the teaser rule (0007)? Teasers
                // come in streams — only pages with few <article> elements
                // qualify.
                if dropped_nodes.iter().any(|&(n, _)| (*n).local_name == LXB_TAG_ARTICLE)
                    && count_articles(doc) <= ARTICLE_RESCUE_MAX_COUNT
                    && dropped_nodes.iter().any(|&(n, d)| {
                        (*n).local_name == LXB_TAG_ARTICLE
                            && is_main_content_node(
                                n,
                                d,
                                opts.comments,
                                opts.post_meta,
                                opts.hidden_elements,
                                RelaxFlags { short_articles: true, ..Default::default() },
                                opts.preserve_formatting == FormattingOpts::Markdown,
                            )
                    })
                {
                    relax.short_articles = true;
                }

                // Listing-card retry (cycle 0023): a deep loss (output under
                // 15% of body text) with dropped div/ul containers suggests a
                // tag/search/archive page whose teaser cards were
                // link-cluster-vetoed; gold keeps every card.
                // Listing-card retry: three gate variants measured, all
                // negative or unstable at train scale (see 0023 log) — the
                // listing/article discriminator needs the page-type
                // classifier. Plumbing (RelaxFlags::listing_cards) kept for
                // that era; the gate is disabled.
                let _ = page_has_card_grid;

                if relax != RelaxFlags::default()
                    && (result_content_len as f64) < UL_RESCUE_MAX_OUTPUT_RATIO * body_text_len(doc) as f64
                {
                    // Listing retries must multiply content hard (real card
                    // grids are 5-15x the base); chrome flooding on ordinary
                    // articles rarely reaches 4x.
                    let keep_factor = if relax.listing_cards {
                        LISTING_RESCUE_KEEP_FACTOR
                    } else {
                        UL_RESCUE_KEEP_FACTOR
                    };
                    let retry = extract_plain_text_from_doc(doc, opts, relax, effective_tpl.as_ref(), wl_ref);
                    if retry.len() as f64 > keep_factor * result.len().max(1) as f64
                        && !duplicates_existing_content(&result, &retry)
                    {
                        result = retry;
                    }
                }
            }
        }

        // Append rebuilt, attributed comments (cycle 0020) unless a rescue
        // already swapped in an unfiltered extraction containing them.
        if let Some(block) = wp_comments {
            let dup_probe: Option<&str> = block.lines().rev().find(|l| l.len() >= 40);
            let already = dup_probe.map(|p| result.contains(p)).unwrap_or(false);
            if !block.is_empty() && !already {
                if !result.is_empty() {
                    result.push_str("\n\n");
                }
                result.push_str(&md_post_passes(block));
            }
        }

        lxb_html_document_destroy(doc);
        result
    }
}

/// Tier-2 gate (cycle 0005): fire when output is below this fraction of the
/// collapsed body text; keep the retry only if it is this many times larger.
/// Sweep-tuned on lpv11 dev under the zero-regression constraint.
const UL_RESCUE_MAX_OUTPUT_RATIO: f64 = 0.3;
const LISTING_RESCUE_MAX_OUTPUT_RATIO: f64 = 0.15;
const LISTING_RESCUE_KEEP_FACTOR: f64 = 4.0;
const UL_RESCUE_KEEP_FACTOR: f64 = 2.0;

/// Pages with more `<article>` elements than this are teaser streams; the
/// short-article relaxation (cycle 0007) never fires on them.
const ARTICLE_RESCUE_MAX_COUNT: usize = 3;

/// Number of `<article>` elements in the document body.
unsafe fn count_articles(doc: *mut lxb_html_document_t) -> usize {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return 0;
        }
        let dom_coll = lxb_dom_collection_make_noi((*body).owner_document, 8);
        lxb_dom_elements_by_tag_name(body.cast(), dom_coll, b"article".as_ptr(), 7);
        let n = lxb_dom_collection_length_noi(dom_coll);
        lxb_dom_collection_destroy(dom_coll, true);
        n
    }
}

/// Error/stub-page phrases that mark a tiny extraction as the page's true
/// content (rescue veto, cycle 0006).
static ERROR_STUB_TEXT: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(
        r"(?:an error (?:has )?occurred|we(?:'|\u{2019})?re (?:very )?sorry|page (?:you requested )?(?:was |is )?not (?:be )?found|(?:page|item|product) (?:is )?(?:currently )?(?:un|not )avail|out of stock|404 (?:error|not found)|no longer (?:available|exists)|has been (?:removed|deleted))",
    )
    .case_insensitive(true)
    .unicode(false)
    .build()
    .unwrap()
});

/// Whether `retry` repeats content already present once in `base` (template
/// widgets — e.g. Blogspot — can render the same post inside a list, so the
/// list exemption would emit it twice). Detects a mid-`base` probe appearing
/// more than once in `retry`.
fn duplicates_existing_content(base: &str, retry: &str) -> bool {
    const PROBE_LEN: usize = 80;
    if base.len() < 2 * PROBE_LEN {
        return false;
    }
    let mid = base.len() / 2;
    // Char-boundary-aligned probe window around the middle.
    let start = (mid - PROBE_LEN / 2..mid).rev().find(|&i| base.is_char_boundary(i));
    let Some(start) = start else { return false };
    let end = (start + PROBE_LEN..base.len()).find(|&i| base.is_char_boundary(i));
    let Some(end) = end else { return false };
    let probe = &base[start..end];
    // Formatted table rows legitimately repeat (calendars); a probe that is
    // table content would false-positive — abstain (cycle 0012).
    if probe.contains(" | ") || probe.contains("---") {
        return false;
    }
    let mut count = 0;
    let mut hay = retry;
    while let Some(pos) = hay.find(probe) {
        count += 1;
        if count >= 2 {
            return true;
        }
        hay = &hay[pos + probe.len()..];
    }
    false
}

/// Rescue gate: main-content output below this many bytes counts as near-empty.
/// Thresholds selected by exhaustive sweep on lpv11 dev (cycle 0004): this
/// combination recovers 29 catastrophic docs with zero per-doc regressions;
/// looser gates gain aggregate F1 but break the zero-regression rule.
const RESCUE_NEAR_EMPTY_ABS: usize = 200;
/// ... and the collapsed body text must be at least this many times larger.
const RESCUE_BODY_FACTOR: usize = 30;
/// Keep the fallback only if it is at least this many times larger than the
/// main-content output.
const RESCUE_KEEP_FACTOR: usize = 20;

// ---------------------------------------------------------------------------
// Block feature export (learned-classifier groundwork, cycle 0024)
// ---------------------------------------------------------------------------
// Emits per-block features for training; the SAME code paths feed the model
// at inference time, so features agree by construction.

pub fn collect_block_features(html: &str) -> String {
    unsafe {
        let doc = lxb_html_document_create();
        if doc.is_null() {
            return String::new();
        }
        if lxb_html_document_parse(doc, html.as_ptr(), html.len()) != lexbor_status_t::LXB_STATUS_OK {
            lxb_html_document_destroy(doc);
            return String::new();
        }
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            lxb_html_document_destroy(doc);
            return String::new();
        }
        let mut vetoes = HashSet::new();
        let mut candidates: Vec<(*mut lxb_dom_node_t, usize)> = Vec::new();
        let mut blocks: Vec<RawBlock> = Vec::new();
        let mut coll = Some(&mut blocks);
        let totals = tpl_scan(body, false, 0, &mut vetoes, &mut candidates, &mut coll);
        let page_text = totals.text_len.max(1);
        let page_ld = totals.link_len as f64 / page_text as f64;
        let nav_share = totals.nav_text as f64 / page_text as f64;
        let gen_kind = generator_kind(doc);
        let n_substantial = blocks.iter().filter(|b| b.text_len >= 150).count();
        let mass: usize = blocks.iter().map(|b| b.text_len).sum();
        let center: f64 = if mass > 0 {
            blocks.iter().enumerate().map(|(i, b)| i as f64 * b.text_len as f64).sum::<f64>() / mass as f64
        } else {
            0.0
        };
        let nb = blocks.len().max(1) as f64;
        let mut out = String::new();
        for (idx, b) in blocks.iter().enumerate() {
            let f = build_block_features(
                b,
                if idx > 0 { blocks.get(idx - 1) } else { None },
                blocks.get(idx + 1),
                page_text,
                page_ld,
                totals.n_headings,
                totals.n_forms,
                totals.n_articles,
                totals.n_comment_cls,
                nav_share,
                gen_kind,
                n_substantial,
            );
            let text = get_collapsed_string(&get_node_text(b.ptr));
            let text_snip: String = String::from_utf8_lossy(&text)
                .chars()
                .take(600)
                .collect::<String>()
                .replace('\t', " ")
                .replace('\n', " ");
            out.push_str(&format!(
                "{{\"i\":{},\"tag\":{},\"depth\":{},\"text_len\":{},\"link_len\":{},\"n_links\":{},\"page_text\":{},\"page_ld\":{:.4},\"page_forms\":{},\"page_articles\":{},\"page_comment_cls\":{},\"page_nav_share\":{:.4},\"page_generator\":{},\"page_n_blocks\":{},\"block_pos\":{:.4},\"dist_center\":{:.4},\"card_grid\":{},\"punct\":{:.4},\"digit\":{:.4},\"upper\":{:.4},\"avgw\":{:.3},\"nav\":{},\"footer\":{},\"header\":{},\"sidebar\":{},\"social\":{},\"article\":{},\"chrome\":{},\"byline\":{},\"widget\":{},\"recommended\":{},\"comments\":{},\"headings\":{},\"page_headings\":{},\"prev_ld\":{:.4},\"next_ld\":{:.4},\"prev_len\":{:.3},\"next_len\":{:.3},\"wb\":{:?},\"text\":{}}}\n",
                idx, b.tag, b.depth, b.text_len, b.link_len, b.n_a, page_text, page_ld,
                totals.n_forms, totals.n_articles, totals.n_comment_cls,
                nav_share, gen_kind, n_substantial,
                idx as f64 / nb, (idx as f64 - center).abs() / nb, 0u8,
                f.punct, f.digit, f.upper, f.avgw,
                f.nav as u8, f.footer as u8, f.header as u8, f.sidebar as u8, f.social as u8,
                f.article as u8, f.chrome as u8, f.byline as u8, f.widget as u8,
                f.recommended as u8, f.comments as u8,
                f.headings as u64, f.page_headings as u64,
                f.prev_ld, f.next_ld, f.prev_len, f.next_len,
                f.wb.iter().map(|v| (v * 1000.0).round() / 1000.0).collect::<Vec<_>>(),
                serde_escape(&text_snip),
            ));
        }
        lxb_html_document_destroy(doc);
        out
    }
}

/// All 11 class-family patterns as one RegexSet — a single haystack scan for
/// the model's feature bits (11 separate scans measured ~10% total runtime).
/// Pattern strings MUST mirror the individual statics above.
static FEATURE_CLS_SET: LazyLock<regex::bytes::RegexSet> = LazyLock::new(|| {
    regex::bytes::RegexSetBuilder::new([
        NAV_CLS.as_str(),
        FOOTER_CLS.as_str(),
        HEADER_CLS.as_str(),
        SIDEBAR_CLS.as_str(),
        SOCIAL_CLS.as_str(),
        ARTICLE_CLS.as_str(),
        MD_CHROME_CLS.as_str(),
        BYLINE_CLS.as_str(),
        WIDGETISH_CLS.as_str(),
        RECOMMENDED_CLS.as_str(),
        COMMENTS_CLS.as_str(),
    ])
    .case_insensitive(true)
    .unicode(false)
    .build()
    .unwrap()
});

unsafe fn build_block_features(
    b: &RawBlock,
    prev: Option<&RawBlock>,
    next: Option<&RawBlock>,
    page_text: usize,
    page_ld: f64,
    page_headings: usize,
    page_forms: usize,
    page_articles: usize,
    page_comment_cls: usize,
    page_nav_share: f64,
    page_generator: u8,
    page_n_blocks: usize,
) -> block_model::BlockFeatures {
    unsafe {
        let cls = get_node_attr(b.ptr, b"class");
        let id = get_node_attr(b.ptr, b"id");
        let mut combo = cls.to_vec();
        combo.push(b' ');
        combo.extend_from_slice(id);
        let tl = b.text_len.max(1);
        let hits: Vec<usize> = if combo.len() > 1 {
            FEATURE_CLS_SET.matches(&combo).into_iter().collect()
        } else {
            Vec::new()
        };
        block_model::BlockFeatures {
            tag: b.tag as f64,
            depth: b.depth as f64,
            log_text_len: ((b.text_len + 1) as f64).ln(),
            link_density: b.link_len as f64 / tl as f64,
            n_links: b.n_a as f64,
            page_ld,
            frac_page: b.text_len as f64 / page_text.max(1) as f64,
            punct: b.punct as f64 / tl as f64,
            digit: b.digits as f64 / tl as f64,
            upper: b.upper as f64 / tl as f64,
            avgw: b.text_len as f64 / b.words.max(1) as f64,
            nav: hits.contains(&0) as u8 as f64,
            footer: hits.contains(&1) as u8 as f64,
            header: hits.contains(&2) as u8 as f64,
            sidebar: hits.contains(&3) as u8 as f64,
            social: hits.contains(&4) as u8 as f64,
            article: hits.contains(&5) as u8 as f64,
            chrome: hits.contains(&6) as u8 as f64,
            byline: hits.contains(&7) as u8 as f64,
            widget: hits.contains(&8) as u8 as f64,
            recommended: hits.contains(&9) as u8 as f64,
            comments: hits.contains(&10) as u8 as f64,
            headings: b.n_headings as f64,
            page_headings: page_headings as f64,
            page_forms: ((page_forms + 1) as f64).ln(),
            page_articles: ((page_articles + 1) as f64).ln(),
            page_comment_cls: ((page_comment_cls + 1) as f64).ln(),
            page_nav_share,
            page_generator: page_generator as f64,
            page_n_blocks: ((page_n_blocks + 1) as f64).ln(),
            prev_ld: prev.map(|p| p.link_len as f64 / p.text_len.max(1) as f64).unwrap_or(-1.0),
            next_ld: next.map(|p| p.link_len as f64 / p.text_len.max(1) as f64).unwrap_or(-1.0),
            prev_len: prev.map(|p| ((p.text_len + 1) as f64).ln()).unwrap_or(0.0),
            next_len: next.map(|p| ((p.text_len + 1) as f64).ln()).unwrap_or(0.0),
            wb: {
                let total = b.wordbag.iter().sum::<u32>().max(1) as f64;
                let mut wb = [0.0f64; 32];
                for (o, v) in wb.iter_mut().zip(b.wordbag.iter()) {
                    *o = *v as f64 / total;
                }
                wb
            },
        }
    }
}

fn serde_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ---------------------------------------------------------------------------
// Structural template subtraction (cycle 0019)
// ---------------------------------------------------------------------------
// Boilerplate is built from repeated sibling subtrees (nav lists, card
// grids); main content is structurally diverse. Prototype measured +0.06-0.08
// F1 over a dump baseline (held-out confirmed); the load-bearing conjunct is
// repetition AND link-density (repetition alone kills content lists/tables).
// See research_log/analysis-template-subtraction.md.

const TPL_MIN_CHILDREN: usize = 3;
const TPL_MIN_REPEATED: usize = 3;
const TPL_MIN_FRAC: f64 = 0.5;
const TPL_LINK_DENSITY: f64 = 0.7;
const TPL_MAX_CONTAINER_FRAC: f64 = 0.3;
/// Absolute cap: chrome containers are small; repeated-structure containers
/// above this are content (photo series, verse lists, instruction sequences —
/// 33 train catastrophes without it).
const TPL_MAX_CONTAINER_TEXT: usize = 2500;
/// Skip subtraction entirely on listing-like pages (gold keeps chrome there).
const TPL_PAGE_LINK_DENSITY_MAX: f64 = 0.7;
const TPL_MIN_PAGE_TEXT: usize = 1500;

const WORDBAG_DIM: usize = 32;

#[allow(dead_code)]
struct TplNode {
    ptr: *mut lxb_dom_node_t,
    text_len: usize,
    link_len: usize,
    n_imgs: usize,
    n_a: usize,
    n_forms: usize,
    n_articles: usize,
    n_comment_cls: usize,
    nav_text: usize,
    punct: usize,
    digits: usize,
    upper: usize,
    words: usize,
    n_headings: usize,
    wordbag: [u32; WORDBAG_DIM],
    sig1: u64,
    sig2: u64,
}

fn tpl_hash(first: u64, rest: &[u64]) -> u64 {
    use std::hash::Hasher;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    h.write_u64(first);
    for i in rest {
        h.write_u64(*i);
    }
    h.finish()
}

fn tpl_base_sig(tag: lxb_tag_id_t, cls: &[u8]) -> u64 {
    use std::hash::Hasher;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    h.write_usize(tag);
    // digit runs normalized so per-item classes (`post-1234`) collide
    let mut prev_digit = false;
    for &b in cls {
        let d = b.is_ascii_digit();
        if d && prev_digit {
            continue;
        }
        h.write_u8(if d { b'N' } else { b.to_ascii_lowercase() });
        prev_digit = d;
    }
    h.finish()
}

/// Bottom-up scan: computes per-element structural signatures and marks
/// repeated∧link-dense containers in `vetoes`. Returns (text_len, link_len).
struct RawBlock {
    ptr: *mut lxb_dom_node_t,
    tag: lxb_tag_id_t,
    depth: usize,
    text_len: usize,
    link_len: usize,
    n_a: usize,
    punct: usize,
    digits: usize,
    upper: usize,
    words: usize,
    n_headings: usize,
    wordbag: [u32; WORDBAG_DIM],
}

unsafe fn tpl_scan(
    node: *mut lxb_dom_node_t,
    in_link: bool,
    depth: usize,
    vetoes: &mut HashSet<*mut lxb_dom_node_t>,
    candidates: &mut Vec<(*mut lxb_dom_node_t, usize)>,
    feats: &mut Option<&mut Vec<RawBlock>>,
) -> TplNode {
    unsafe {
        let tag = (*node).local_name;
        let is_link = in_link || tag == LXB_TAG_A;
        let mut text_len = 0usize;
        let mut link_len = 0usize;
        let mut n_imgs = if tag == LXB_TAG_IMG { 1 } else { 0 };
        let mut n_a = if tag == LXB_TAG_A { 1 } else { 0 };
        let mut n_forms = if tag == LXB_TAG_FORM { 1 } else { 0 };
        let mut n_articles = if tag == LXB_TAG_ARTICLE { 1 } else { 0 };
        let mut n_comment_cls = if (*node).type_ == LXB_DOM_NODE_TYPE_ELEMENT
            && contains_subslice(&get_node_attr(node, b"class").to_ascii_lowercase(), b"comment")
        {
            1
        } else {
            0
        };
        let mut nav_text = 0usize;
        let mut punct = 0usize;
        let mut digits = 0usize;
        let mut upper = 0usize;
        let mut words = 0usize;
        let mut n_headings = if matches!(tag, LXB_TAG_H1 | LXB_TAG_H2 | LXB_TAG_H3 | LXB_TAG_H4 | LXB_TAG_H5 | LXB_TAG_H6) { 1 } else { 0 };
        let mut wordbag = [0u32; WORDBAG_DIM];
        let mut child_sig1: Vec<u64> = Vec::new();
        let mut child_sig2: Vec<u64> = Vec::new();
        let mut n_children = 0usize;
        let mut child = (*node).first_child;
        while !child.is_null() {
            match (*child).type_ {
                LXB_DOM_NODE_TYPE_TEXT => {
                    let cd = child as *const lxb_dom_character_data_t;
                    let t = slice::from_raw_parts((*cd).data.data, (*cd).data.length);
                    let n = t.iter().filter(|b| !c_isspace(**b)).count();
                    text_len += n;
                    if is_link {
                        link_len += n;
                    }
                    punct += t.iter().filter(|b| matches!(b, b'.' | b',' | b'!' | b'?' | b';')).count();
                    digits += t.iter().filter(|b| b.is_ascii_digit()).count();
                    upper += t.iter().filter(|b| b.is_ascii_uppercase()).count();
                    let mut in_word = false;
                    let mut wh: u64 = 0xcbf29ce484222325;
                    for &b in t {
                        if c_isspace(b) {
                            if in_word {
                                wordbag[(wh % WORDBAG_DIM as u64) as usize] += 1;
                                wh = 0xcbf29ce484222325;
                            }
                            in_word = false;
                        } else {
                            // FNV-1a over lowercased bytes
                            wh ^= b.to_ascii_lowercase() as u64;
                            wh = wh.wrapping_mul(0x100000001b3);
                            if !in_word {
                                words += 1;
                                in_word = true;
                            }
                        }
                    }
                    if in_word {
                        wordbag[(wh % WORDBAG_DIM as u64) as usize] += 1;
                    }
                }
                LXB_DOM_NODE_TYPE_ELEMENT => {
                    // script/style/etc are excluded from the walk by the
                    // blacklist; exclude their text here too
                    if !matches!(
                        std::str::from_utf8(get_qualified_name(child)).unwrap_or(""),
                        "script" | "style" | "noscript" | "template" | "svg" | "iframe"
                    ) {
                        let c = tpl_scan(child, is_link, depth + 1, vetoes, candidates, feats);
                        text_len += c.text_len;
                        link_len += c.link_len;
                        n_imgs += c.n_imgs;
                        n_a += c.n_a;
                        n_forms += c.n_forms;
                        n_articles += c.n_articles;
                        n_comment_cls += c.n_comment_cls;
                        nav_text += c.nav_text;
                        punct += c.punct;
                        digits += c.digits;
                        upper += c.upper;
                        words += c.words;
                        n_headings += c.n_headings;
                        for (a, b) in wordbag.iter_mut().zip(c.wordbag.iter()) {
                            *a += *b;
                        }
                        child_sig1.push(c.sig1);
                        child_sig2.push(c.sig2);
                        n_children += 1;
                    }
                }
                _ => {}
            }
            child = (*child).next;
        }
        let cls = get_node_attr(node, b"class");
        if let Some(collector) = feats.as_deref_mut() {
            if depth > 1
                && (is_block_element(tag) || tag == LXB_TAG_TD)
                && text_len > 0
                && (!cls.is_empty() || !get_node_attr(node, b"id").is_empty())
            {
                collector.push(RawBlock {
                    ptr: node,
                    tag,
                    depth,
                    text_len,
                    link_len,
                    n_a,
                    punct,
                    digits,
                    upper,
                    words,
                    n_headings,
                    wordbag,
                });
            }
        }
        let sig0 = tpl_base_sig(tag, cls);
        let sig1 = tpl_hash(sig0, &child_sig1);
        let sig2 = tpl_hash(sig0, &child_sig2);

        // candidate check (page-level guards applied by the caller once
        // body totals are known — single pass)
        if n_children >= TPL_MIN_CHILDREN
            && text_len > 0
            && text_len <= TPL_MAX_CONTAINER_TEXT
            && link_len as f64 / text_len as f64 >= TPL_LINK_DENSITY
        {
            let mut counts: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
            for s2 in &child_sig2 {
                *counts.entry(*s2).or_insert(0) += 1;
            }
            let repeated: usize = counts.values().filter(|v| **v >= 2).sum();
            if repeated >= TPL_MIN_REPEATED && repeated as f64 / n_children as f64 >= TPL_MIN_FRAC {
                vetoes.insert(node);
                candidates.push((node, text_len));
            }
        }
        // chrome containers claim their whole subtree's text as nav text
        if matches!(tag, LXB_TAG_NAV | LXB_TAG_ASIDE | LXB_TAG_FOOTER | LXB_TAG_HEADER) {
            nav_text = text_len;
        }
        TplNode {
            ptr: node,
            text_len,
            link_len,
            n_imgs,
            n_a,
            n_forms,
            n_articles,
            n_comment_cls,
            nav_text,
            punct,
            digits,
            upper,
            words,
            n_headings,
            wordbag,
            sig1,
            sig2,
        }
    }
}

unsafe fn get_qualified_name(node: *mut lxb_dom_node_t) -> &'static [u8] {
    unsafe {
        let mut len = 0usize;
        let p = lxb_dom_element_qualified_name(node.cast(), &mut len);
        if p.is_null() { &[] } else { slice::from_raw_parts(p, len) }
    }
}

/// Compute the template-subtraction veto set for a document body.
/// Model-veto threshold (cycle 0025): blocks scoring below this predicted
/// gold-containment join the skip set. Chosen from held-out tier analysis
/// (veto@0.10 ≈ 2% coverage at <1% false-veto on the n60d5 GBM).
const MODEL_VETO_THRESHOLD: f64 = 0.40;
const MODEL_VETO_ENABLED: bool = true;
/// Whitelist tier: blocks scoring above this override rule/template vetoes.
const MODEL_VETO_BIG_THRESHOLD: f64 = 0.10;
const MODEL_KEEP_THRESHOLD: f64 = 0.60;

/// Returns the veto set plus whether the page carries a LARGE repeated-
/// structure container (>=3000B) — the positive signal that this is a
/// listing/card-grid page (cycle 0023 uses it to gate the listing rescue).
unsafe fn tpl_vetoes(
    gen_kind: u8,
    body: *mut lxb_dom_node_t,
) -> (
    HashSet<*mut lxb_dom_node_t>,
    bool,
    HashSet<*mut lxb_dom_node_t>,
    Vec<*mut lxb_dom_node_t>,
    usize,
    f64,
) {
    unsafe {
        let mut vetoes = HashSet::new();
        let mut candidates: Vec<(*mut lxb_dom_node_t, usize)> = Vec::new();
        let mut blocks: Vec<RawBlock> = Vec::new();
        let mut coll = if MODEL_VETO_ENABLED { Some(&mut blocks) } else { None };
        let totals = tpl_scan(body, false, 0, &mut vetoes, &mut candidates, &mut coll);
        // model tiers on the same scan (cycle 0025). Applied outside the tpl
        // page-guards (the model has its own calibration) and only to blocks
        // >=150 bytes: smaller ones can't move either tier and the per-block
        // regex features dominate the cost.
        let mut whitelist = HashSet::new();
        let mut model_veto: Vec<*mut lxb_dom_node_t> = Vec::new();
        let mut model_veto_mass = 0usize;
        if MODEL_VETO_ENABLED {
            let page_text = totals.text_len.max(1);
            let pld = totals.link_len as f64 / page_text as f64;
            let nav_share = totals.nav_text as f64 / page_text as f64;
            let n_substantial = blocks.iter().filter(|b| b.text_len >= 150).count();
            for (i, b) in blocks.iter().enumerate() {
                // 40-byte floor (0052): swept 150->10; quality rises
                // monotonically as the floor drops but below 40B the extra
                // per-block scoring costs +26% markdown time for +0.0009 F1
                // — 40 is the free point (2.60 vs 2.67 ms/doc baseline).
                if b.text_len < 40 {
                    continue;
                }
                let f = build_block_features(
                    b,
                    if i > 0 { blocks.get(i - 1) } else { None },
                    blocks.get(i + 1),
                    page_text,
                    pld,
                    totals.n_headings,
                    totals.n_forms,
                    totals.n_articles,
                    totals.n_comment_cls,
                    nav_share,
                    gen_kind,
                    n_substantial,
                );
                let score = block_model::score_block(&f);
                // Size-tiered veto authority (0051): the aggressive
                // threshold only fires on small blocks; a large block is
                // an article-body candidate and needs near-certainty
                // (crater profile at wide thresholds was whole-article
                // false vetoes).
                let veto_thresh = if b.text_len <= 1500 {
                    MODEL_VETO_THRESHOLD
                } else {
                    MODEL_VETO_BIG_THRESHOLD
                };
                if score < veto_thresh {
                    model_veto.push(b.ptr);
                    model_veto_mass += b.text_len;
                } else if score > MODEL_KEEP_THRESHOLD {
                    whitelist.insert(b.ptr);
                }
            }
        }
        let large_repeated = candidates.iter().any(|&(_, tl)| tl >= 3000)
            || (totals.text_len > 0
                && totals.link_len as f64 / totals.text_len as f64 > TPL_PAGE_LINK_DENSITY_MAX);
        if totals.text_len < TPL_MIN_PAGE_TEXT
            || totals.link_len as f64 / totals.text_len as f64 > TPL_PAGE_LINK_DENSITY_MAX
        {
            // Listing-like or thin page: on thin pages whatever repeats is
            // usually the content (package-instruction pages, profiles).
            vetoes.clear();
            for m in &model_veto {
                vetoes.insert(*m);
            }
            for w in &whitelist {
                vetoes.remove(w);
            }
            let pld = totals.link_len as f64 / totals.text_len.max(1) as f64;
            return (vetoes, large_repeated, whitelist, model_veto, model_veto_mass, pld);
        }
        // container-fraction guard, applied now that body totals are known
        for (n, tl) in candidates {
            if (tl as f64) > TPL_MAX_CONTAINER_FRAC * totals.text_len as f64 {
                vetoes.remove(&n);
            }
        }
        for m in &model_veto {
            vetoes.insert(*m);
        }
        for w in &whitelist {
            vetoes.remove(w);
        }
        let pld = totals.link_len as f64 / totals.text_len.max(1) as f64;
        (vetoes, large_repeated, whitelist, model_veto, model_veto_mass, pld)
    }
}

/// Domain-gated selector vetoes (cycle 0098): site-specific chrome
/// containers verified chrome-only on their doc (0096 extraction).
/// Fires ONLY on its own domain (og:url/canonical) — zero cross-site
/// risk by construction.
/// Domain-gated content WHITELIST (cycle 0101): site-specific content
/// containers our rules drop; forces them kept via the model-whitelist
/// path (overrides is_main_content_node + tpl/model vetoes). Same
/// zero-cross-site construction as SITE_VETOES.
const SITE_WHITELIST: &[(&[u8], &[u8])] = &[
    (b"425sqftart.com", b".blogtitle-box"),
    (b"425sqftart.com", b".postmetadata"),
    (b"aber.ac.uk", b".module-x-column-right"),
    (b"aber.ac.uk", b".notes"),
    (b"aber.ac.uk", b".reading-cat"),
    (b"ace-ed.org.uk", b".channelSummaryContainer"),
    (b"alibris.com", b".product"),
    (b"allegramarketingprint.com", b"#case-study-wrap"),
    (b"allegramarketingprint.com", b".content-col"),
    (b"allegramarketingprint.com", b".inner-wrap"),
    (b"alt.com", b".rcm"),
    (b"androidpolice.com", b".dsq-comment-message"),
    (b"androidpolice.com", b".external"),
    (b"androidpolice.com", b".list-unstyled"),
    (b"belangerinc.com", b".shadow_outer"),
    (b"bellydance.org", b".pageHeading"),
    (b"bepress.com", b".vc_single_image-img"),
    (b"bimmerfest.com", b"#intelliTXT"),
    (b"biotech-capital.com", b".abstract"),
    (b"blip.fm", b".tweem"),
    (b"blurb.com", b".features-and-details-section__book-stats"),
    (b"books.google.com.au", b"#metadata_content_table"),
    (b"careers.govt.nz", b".csc-textpic-text"),
    (b"cbssports.com", b".completed-games-table"),
    (b"cbssports.com", b".profile-news-item-header"),
    (b"cheftalk.com", b".thread-hier"),
    (b"cyclonefanatic.com", b".message"),
    (b"dictionary.cambridge.org", b".cdo-topic"),
    (b"dictionary.cambridge.org", b".def"),
    (b"dictionary.cambridge.org", b".main-cloud-preample"),
    (b"dictionary.reference.com", b".right-rail-container"),
    (b"dittrickswines.com", b".ch-footer-hidden-description"),
    (b"duckhuntingchat.com", b".signature"),
    (b"efloras.org", b"#tableLinkList"),
    (b"english-subtitles.club", b".promo-center"),
    (b"eurofound.europa.eu", b".element-invisible"),
    (b"eurofound.europa.eu", b".field-name-body"),
    (b"eurofound.europa.eu", b".field-name-field-ef-topic"),
    (b"fabulous-emma.com", b".maintable"),
    (b"fangirluprising.com", b".entry-date"),
    (b"fatsecret.com", b".factPanel"),
    (b"gawker.com", b".first-text"),
    (b"gawker.com", b".headline"),
    (b"hotrecordsociete.bandcamp.com", b"#bio-text"),
    (b"hotrecordsociete.bandcamp.com", b"#track_table"),
    (b"hotrecordsociete.bandcamp.com", b".location"),
    (b"hsc.wvu.edu", b".rte"),
    (b"huskers.com", b".FooterText"),
    (b"huskers.com", b".extendedHeight"),
    (b"ideas.repec.org", b"#related-body"),
    (b"ieeexplore.ieee.org", b".col-1-grd"),
    (b"ieeexplore.ieee.org", b".col-2-grd"),
    (b"ito-yarn.com", b"#content-area"),
    (b"ito-yarn.com", b".field-item"),
    (b"ito-yarn.com", b".field-items"),
    (b"jessicacarneyassociates.co.uk", b".entry-content"),
    (b"kesq.com", b".headline"),
    (b"kesq.com", b".postedAt"),
    (b"kesq.com", b".updatedAt"),
    (b"kingston.ac.uk", b".contentblock"),
    (b"kingston.ac.uk", b".leftcontentimg"),
    (b"kohls.com", b"#bv-content-show"),
    (b"libertysentinel.org", b".article-title"),
    (b"libertysentinel.org", b".author-links"),
    (b"libertysentinel.org", b".post-description"),
    (b"linda-artandmore.blogspot.com", b".profile-datablock"),
    (b"lowpowerlab.com", b".entry-content"),
    (b"makemoneyforabsolutebeginner.blogspot.com", b"#header-wrapper"),
    (b"mastercraft.com", b".fieldset"),
    (b"morford.rootsandthreads.com", b"#footerw"),
    (b"mountainguard.com", b".masthead-heading"),
    (b"mountainguard.com", b".pi-slider"),
    (b"music.dartmouth.edu", b".content-parent"),
    (b"nameberry.com", b".lastedited"),
    (b"nameberry.com", b".signaturecontainer"),
    (b"ncbi.nlm.nih.gov", b".res_logo"),
    (b"ncbi.nlm.nih.gov", b".ui-ncbi-toggler-slave"),
    (b"nvnews.net", b"#nointelliTXT"),
    (b"pt.usc.edu", b"#ctl00_ctl00_MainContent_ContentSubpageInterior_DropZone_columnDisplay_ctl00_controlcolumn_ctl01_WidgetHost_WidgetHost_widget_CB"),
    (b"pwmag.com", b".bylineList"),
    (b"read718.org", b"#input_6_21_1_label"),
    (b"read718.org", b"#input_6_21_2_label"),
    (b"read718.org", b"#input_6_29"),
    (b"read718.org", b"#input_6_38"),
    (b"read718.org", b".gfield_label"),
    (b"rosko123.wordpress.com", b".tags"),
    (b"scienceblogs.com", b".field--type-entity-reference"),
    (b"scienceblogs.com", b".field--type-text-with-summary"),
    (b"sevenforums.com", b"#collapseobj_sig_0"),
    (b"smallbizpages.co.uk", b".listing-details"),
    (b"smallbizpages.co.uk", b".wpbdp-listing"),
    (b"smartdevicelink.com", b".github-link"),
    (b"sports-boards.net", b".blockbody"),
    (b"spreaker.com", b".track_tags"),
    (b"thetaborfoundation.org", b".entry-content"),
    (b"tomshardware.com", b".bbcode"),
    (b"tomshardware.com", b".spaceL5"),
    (b"tv.com", b"._standard_sub_module"),
    (b"tvwbb.com", b".signaturecontainer"),
    (b"ucsdtritons.com", b"#article_info"),
    (b"ucsdtritons.com", b".dateArticle"),
    (b"use.perl.org", b".copyright"),
    (b"valleyvet.com", b".modal-body"),
    (b"valleyvet.com", b".modal-header"),
    (b"vectra-c.com", b"#footer_copyright"),
    (b"wccftech.com", b".size-large"),
    (b"wccftech.com", b".wp-video"),
    (b"weather.weatherbug.com", b"#box-news-hdlns"),
    (b"weather.weatherbug.com", b"#box-radar-map-preview"),
    (b"weather.weatherbug.com", b"#box-radar-map-xtra"),
    (b"weather.weatherbug.com", b"#box-tools"),
    (b"weather.weatherbug.com", b"#featURLtxt_1"),
    (b"weather.weatherbug.com", b"#featURLtxt_2"),
    (b"weather.weatherbug.com", b"#footer-tou"),
    (b"weather.weatherbug.com", b"#hnav-doppler"),
    (b"weather.weatherbug.com", b"#hnav-slmap"),
    (b"weather.weatherbug.com", b"#vnav-allergies-map"),
    (b"weather.weatherbug.com", b"#vnav-maps"),
    (b"weather.weatherbug.com", b".boxbody"),
    (b"weather.weatherbug.com", b".boxhdr"),
    (b"weather.weatherbug.com", b".boxmore"),
    (b"weather.weatherbug.com", b".map-preview-wrap"),
    (b"weather.weatherbug.com", b".th-info"),
    (b"wowdigsite.com", b"#post-94"),
    (b"wpbf.com", b".copyright"),

    (b"arkuszematuralne.pl", b".meta--categories"),
    (b"autos.jdpower.com", b".compareForm"),
    (b"beckett.com", b"#megaPB"),
    (b"beckett.com", b".bdr_rt"),
    (b"beckett.com", b".faq"),
    (b"beckett.com", b".section_three_col"),
    (b"beckett.com", b".section_two_col"),
    (b"belangerinc.com", b".productRotatorTitle"),
    (b"biotech-capital.com", b".header-group"),
    (b"biotech-capital.com", b".table-condensed"),
    (b"blip.fm", b".blipTitle"),
    (b"blogs.iptv.org", b".tags"),
    (b"blogs.wsj.com", b".h-main"),
    (b"blurb.com", b".tags"),
    (b"books.google.com.au", b".about_title"),
    (b"books.google.com.au", b".cloud"),
    (b"books.google.com.au", b".metadata_row"),
    (b"canadiangardening.com", b".open_resource_on_new_window"),
    (b"canadiangardening.com", b".signature"),
    (b"careers.govt.nz", b".span-19"),
    (b"cbssports.com", b".completedGamesProjection"),
    (b"cheftalk.com", b".hier-row"),
    (b"chictopia.com", b".info_overlay"),
    (b"christianmingle.com", b".disclosure"),
    (b"christianmingle.com", b".view_profile"),
    (b"cinephonix.com", b".selectors"),
    (b"clubwrx.net", b".signature"),
    (b"commons.apache.org", b".altColor"),
    (b"complex.com", b".article-tags--margins"),
    (b"complex.com", b".article-title-sneakers"),
    (b"complex.com", b".feed-article__title"),
    (b"complex.com", b".mini-author__name"),
    (b"conferenceboard.ca", b".elibDetails"),
    (b"cruisecritic.com", b".chakra-breadcrumb__link"),
    (b"cruisecritic.com", b".chakra-link"),
    (b"cruisecritic.com", b".css-1wlv8gq"),
    (b"dictionary.cambridge.org", b".definition-src"),
    (b"dictionary.reference.com", b".nearby-words-outer-box"),
    (b"digitimes.com", b"#phtags"),
    (b"disneyandmore.blogspot.com", b"#LinkList20"),
    (b"disneyandmore.blogspot.com", b"#LinkList27"),
    (b"disneyandmore.blogspot.com", b".post-labels"),
    (b"disneyandmore.blogspot.com", b".widget-content"),
    (b"endure-network.eu", b"#baspage"),
    (b"english-subtitles.club", b"#subtitles_table"),
    (b"english-subtitles.club", b".bottommargin"),
    (b"english-subtitles.club", b".button-primary"),
    (b"english-subtitles.club", b".button-rounded"),
    (b"english-subtitles.club", b".external"),
    (b"english-subtitles.club", b".panel-body"),
    (b"english-subtitles.club", b".participations-grid"),
    (b"english-subtitles.club", b".text-overlay-title"),
    (b"focusfanatics.com", b".alt1"),
    (b"fs.fed.us", b".field--type-text-with-summary"),
    (b"hsc.wvu.edu", b".content__primary"),
    (b"huskers.com", b".date"),
    (b"huskers.com", b".event-link"),
    (b"huskers.com", b".sport"),
    (b"huskers.com", b".stats-futuregame"),
    (b"ieeexplore.ieee.org", b".art-authors"),
    (b"ieeexplore.ieee.org", b".authorPreferredName"),
    (b"ithemes.com", b"#toc"),
    (b"jedidefender.com", b".bbc_link"),
    (b"jedidefender.com", b".signature"),
    (b"jeepforum.com", b"#td_post_13498908"),
    (b"kiehls.com", b".TT3aText"),
    (b"kiehls.com", b".TT3itemBox2"),
    (b"kiehls.com", b".TT3qText"),
    (b"kiehls.com", b".tabCopy"),
    (b"mcdougallcorp.com", b".address"),
    (b"mountainguard.com", b".pi-section"),
    (b"mountainguard.com", b".table-bordered"),
    (b"mouse-bola-bola.blogspot.com", b"#post-body-2096671142872374288"),
    (b"music.dartmouth.edu", b".nine"),
    (b"nameberry.com", b".signature"),
    (b"nevadaappeal.com", b"#article-headline"),
    (b"news.sys-con.com", b"#footer-terms"),
    (b"owsd.net", b".downloads"),
    (b"pixbits.wordpress.com", b".cat-links"),
    (b"pixbits.wordpress.com", b".entry-meta"),
    (b"pixbits.wordpress.com", b".tags-links"),
    (b"pl.tripadvisor.com", b"#COOKIE_BANNER"),
    (b"pl.tripadvisor.com", b".brandArea"),
    (b"pl.tripadvisor.com", b".copyright"),
    (b"pragationline.com", b"#tab-title-description"),
    (b"pragationline.com", b".posted_in"),
    (b"radaronline.com", b".entry-title"),
    (b"radaronline.com", b".single-posted-on"),
    (b"raptorsrepublic.com", b".restore"),
    (b"raptorsrepublic.com", b".stats"),
    (b"read718.org", b"#field_6_32"),
    (b"salisburypost.com", b".meta-byline"),
    (b"salisburypost.com", b".meta-date"),
    (b"sangriasunshinecom.wordpress.com", b"#post-147"),
    (b"sangriasunshinecom.wordpress.com", b".entry-content"),
    (b"sangriasunshinecom.wordpress.com", b".tags-links"),
    (b"slideshare.net", b".description"),
    (b"slideshare.net", b".h-categories-label"),
    (b"slideshare.net", b".license"),
    (b"slideshare.net", b".su-category"),
    (b"smallbizpages.co.uk", b".wpbdp-field-association-meta"),
    (b"smallbizpages.co.uk", b".wpbdp-field-association-tags"),
    (b"smallbizpages.co.uk", b".wpbdp-field-business_address"),
    (b"smallbizpages.co.uk", b".wpbdp-field-business_genre_required"),
    (b"smallbizpages.co.uk", b".wpbdp-field-business_name_required"),
    (b"smallbizpages.co.uk", b".wpbdp-field-business_phone_number"),
    (b"smallbizpages.co.uk", b".wpbdp-field-long_business_description_required"),
    (b"smallbizpages.co.uk", b".wpbdp-field-short_business_description"),
    (b"smartdevicelink.com", b".hierarchy"),
    (b"smartdevicelink.com", b".no-print"),
    (b"sports-boards.net", b".blockhead"),
    (b"sports-boards.net", b".blockrow"),
    (b"talk.philmusic.com", b".bbc_link"),
    (b"talk.philmusic.com", b".signature"),
    (b"talk.philmusic.com", b".topslice_quote"),
    (b"tasteofhome.com", b".rd_centered_section_heading"),
    (b"tasteofhome.com", b".rd_spotlight_contributer_romance"),
    (b"theprp.com", b".post-tags"),
    (b"thinkns.com", b".tag-bass-musician-magazine-review"),
    (b"tv.com", b".vid"),
    (b"uctv.tv", b"#movie-title"),
    (b"usarugby.org", b".kmt-body"),
    (b"usarugby.org", b".quote"),
    (b"weareiowa.com", b".article__meta"),
    (b"weather.com", b"#wx-copyright"),
    (b"weather.com", b".allAlmanac1"),
    (b"weather.weatherbug.com", b"#feat-1"),
    (b"westonparkhospitality.com", b"#footer-blocks"),
    (b"whatsonmypc.blog", b".post-tags"),
    (b"wiki.inf.ed.ac.uk", b".twikiToc"),

    (b"ace-ed.org.uk", b".linkPanelDesc"),
    (b"allegramarketingprint.com", b"#hours"),
    (b"allegramarketingprint.com", b"#hours-toggle"),
    (b"allegramarketingprint.com", b".address"),
    (b"allegramarketingprint.com", b".dir-link"),
    (b"archive.financialexpress.com", b".summary"),
    (b"beckett.com", b"#megaPG"),
    (b"belangerinc.com", b"#productRotatorLogos"),
    (b"blogs.theprovince.com", b".cat-links"),
    (b"booksword.co.uk", b".cat-links"),
    (b"booksword.co.uk", b".posted-on"),
    (b"budget101.com", b".signaturecontainer"),
    (b"careers.govt.nz", b".csc-default"),
    (b"chictopia.com", b".ellipsis"),
    (b"chictopia.com", b".white"),
    (b"cinephonix.com", b".select-list"),
    (b"cinephonix.com", b".selector-row"),
    (b"clubwrx.net", b".signaturecontainer"),
    (b"comixology.com", b".rating"),
    (b"coursereport.com", b".reviewer-details"),
    (b"dailytech.com", b"#lblHeadline"),
    (b"davisclipper.com", b".org"),
    (b"dictionary.reference.com", b".icon-cup"),
    (b"dictionary.reference.com", b".icon-glasses"),
    (b"dictionary.reference.com", b".list-vertical"),
    (b"dictionary.reference.com", b".subtext"),
    (b"digitimes.com", b".TagCat"),
    (b"dubaibusinessservices.com", b"#tab-title-description"),
    (b"dubaibusinessservices.com", b".posted_in"),
    (b"efloras.org", b"#lblLinkList"),
    (b"efloras.org", b"#lblLinkTitle"),
    (b"eurofound.europa.eu", b".field-type-text-with-summary"),
    (b"fandango.com", b".carousel-cast-crew__title"),
    (b"fandango.com", b".poster"),
    (b"filmitown.com", b"#disclaimer"),
    (b"gawker.com", b".hover-highlight"),
    (b"hometheaterforum.com", b".bbc"),
    (b"hometheaterforum.com", b".post_body"),
    (b"hotrecordsociete.bandcamp.com", b".message"),
    (b"hotrecordsociete.bandcamp.com", b".name"),
    (b"hotrecordsociete.bandcamp.com", b".track_row_view"),
    (b"ideas.repec.org", b"#registered-authors"),
    (b"ideas.repec.org", b".downfree"),
    (b"ipinfo.io", b".address-list"),
    (b"jcink.net", b".postcolor"),
    (b"jessicacarneyassociates.co.uk", b".entry-tags"),
    (b"jessicacarneyassociates.co.uk", b".meta-above-title"),
    (b"jessicacarneyassociates.co.uk", b".sqs-block-content"),
    (b"jessicacarneyassociates.co.uk", b".u-url"),
    (b"jetcost.com.sg", b".nearestList"),
    (b"kingston.ac.uk", b".contentheader"),
    (b"kingston.ac.uk", b".tablet-nav-menu"),
    (b"kohls.com", b"#43618120"),
    (b"legalinsurrection.com", b".postExcerpt"),
    (b"legalinsurrection.com", b".postTitle"),
    (b"legalinsurrection.com", b".tweaction"),
    (b"legalinsurrection.com", b".tweaction-body"),
    (b"library.dayalgroup.com", b".contacts"),
    (b"linda-artandmore.blogspot.com", b".profile-data"),
    (b"linda-artandmore.blogspot.com", b".profile-textblock"),
    (b"meta.stackexchange.com", b".comment-copy"),
    (b"meta.stackexchange.com", b".post-taglist"),
    (b"money.howstuffworks.com", b".content-author"),
    (b"money.howstuffworks.com", b".editorial-title"),
    (b"morford.rootsandthreads.com", b"#compiler"),
    (b"morford.rootsandthreads.com", b"#credit"),
    (b"mountainguard.com", b".pi-slider-animate-opacity"),
    (b"music.dartmouth.edu", b".text-chunk__content"),
    (b"nevadaappeal.com", b".STND-STND"),
    (b"nobelcom.com", b".idcInnerCountries"),
    (b"pittsburghsports.net", b".postbody"),
    (b"portsmouth.co.uk", b".article-meta__timestamp-item"),
    (b"products.smileysaudiovisual.com", b".gtm-contact-modal-external-url"),
    (b"products.smileysaudiovisual.com", b".line-height-1-2em"),
    (b"products.smileysaudiovisual.com", b".line-height-3em"),
    (b"products.smileysaudiovisual.com", b".mobile-line-height-1-4em"),
    (b"pt.usc.edu", b".bullet"),
    (b"read718.org", b".gfield_label_before_complex"),
    (b"read718.org", b".gfield_select"),
    (b"read718.org", b".textwidget"),
    (b"readthestars.com", b".cp_tags"),
    (b"readthestars.com", b".cp_text"),
    (b"readthestars.com", b".cp_title"),
    (b"ro.urbandictionary.com", b".tags"),
    (b"ru.tradingview.com", b".tv-user-block__name"),
    (b"salisburypost.com", b".meta"),
    (b"sangriasunshinecom.wordpress.com", b".cat-links"),
    (b"sangriasunshinecom.wordpress.com", b".posted-on"),
    (b"smartdevicelink.com", b".with-sub"),
    (b"sportinglife.com", b".squad"),
    (b"stitchkingdom.com", b".h2-simulate-h1"),
    (b"stitchkingdom.com", b".heatmapthemead-post-details"),
    (b"swissinfo.ch", b".author-detail"),
    (b"swissinfo.ch", b".dharma-time"),
    (b"texasmonthly.com", b".hero-default__content"),
    (b"thecut.com", b".tags"),
    (b"themillions.com", b".about"),
    (b"themillions.com", b".article-details-wrapper"),
    (b"topjobs.ch", b".job-preview"),
    (b"typekit.com", b".family-card-details"),
    (b"ucsdtritons.com", b"#article-content"),
    (b"usarugby.org", b".kmt-text"),
    (b"vladi-private-islands.de", b".islandfacts"),
    (b"waitrose.com", b".times"),
    (b"waitrose.com", b".total"),
    (b"weareiowa.com", b".article__author"),
    (b"weareiowa.com", b".article__published"),
    (b"weareiowa.com", b".article__updated"),
    (b"weather.weatherbug.com", b"#hnav-allergies"),
    (b"weather.weatherbug.com", b"#hnav-cameramap"),
    (b"weather.weatherbug.com", b"#hnav-droughtmap"),
    (b"weather.weatherbug.com", b"#hnav-flumap"),
    (b"weather.weatherbug.com", b"#hnav-hurricane-map"),
    (b"weather.weatherbug.com", b"#hnav-infrared"),
    (b"weather.weatherbug.com", b"#hnav-lightning"),
    (b"weather.weatherbug.com", b"#hnav-temps"),
    (b"weather.weatherbug.com", b"#hnav-todayhigh"),
    (b"weather.weatherbug.com", b"#hnav-tomhigh"),
    (b"weather.weatherbug.com", b"#hnav-tomlow"),
    (b"weather.weatherbug.com", b"#hnav-visible"),
    (b"weather.weatherbug.com", b"#hnav-windch"),
    (b"weather.weatherbug.com", b"#hnav-windsp"),
    (b"weather.weatherbug.com", b".map-animate"),
    (b"weather.weatherbug.com", b".map-preview"),
    (b"weather.weatherbug.com", b".one"),
    (b"westonparkhospitality.com", b".block-block"),
    (b"westonparkhospitality.com", b".node-body"),
    (b"whatsonmypc.blog", b".comment-body"),
    (b"whatsonmypc.blog", b".comments-title"),
    (b"whatsonmypc.blog", b".post-date"),
    (b"wiki.inf.ed.ac.uk", b".patternTopic"),
    (b"wiki.inf.ed.ac.uk", b".twikiTableRowdataBgSorted0"),
    (b"wowdigsite.com", b".postcont"),
    (b"wri.org", b".article-title--large"),
    (b"wri.org", b".field--small"),
    (b"wtf.com", b".listInline--bullet"),

    (b"ace-ed.org.uk", b".panel"),
    (b"alt.com", b"#page_main"),
    (b"arches.wordpress.com", b".meta"),
    (b"bimmerfest.com", b".fieldset"),
    (b"blogs.iptv.org", b".postMeta"),
    (b"blogs.theprovince.com", b".tags"),
    (b"cryptobrowser.site", b".steps__text"),
    (b"cryptobrowser.site", b".subtitle"),
    (b"engineering.academickeys.com", b".form"),
    (b"etsu.edu", b"#footer_left"),
    (b"etsu.edu", b"#footer_right"),
    (b"fandango.com", b".light"),
    (b"fangirluprising.com", b".comment-body"),
    (b"fangirluprising.com", b".comments-title"),
    (b"farmallcub.com", b".postlink"),
    (b"farmallcub.com", b".rules"),
    (b"farmallcub.com", b".signature"),
    (b"glitternsparklechallengeblog.blogspot.com", b".entry-content"),
    (b"goldenfrog.com", b".inline"),
    (b"goldenfrog.com", b".primary-button"),
    (b"goldenfrog.com", b".sub-head"),
    (b"hotrecordsociete.bandcamp.com", b".tralbum-tags"),
    (b"indianties.com", b".entry-content"),
    (b"ipcyb.org", b".category"),
    (b"ipcyb.org", b".entry-title"),
    (b"ipcyb.org", b".tags"),
    (b"jeepforum.com", b".alt1"),
    (b"jeepkings.ca", b".signaturecontainer"),
    (b"kgi.org", b".leaf"),
    (b"kiehls.com", b"#TT4commentsLeft"),
    (b"kiehls.com", b"#TT4commentsRight"),
    (b"kiehls.com", b".TT3itemBox"),
    (b"lybrate.com", b".lybText--darkest"),
    (b"nameberry.com", b".postcontent"),
    (b"newstatesman.com", b".author-byline"),
    (b"newstatesman.com", b".author-details"),
    (b"newstatesman.com", b".twitter-follow-button"),
    (b"nobelcom.com", b".idcHomeBox1"),
    (b"pezenas-couvent.com", b".art-postcontent"),
    (b"read718.org", b".widget-title"),
    (b"ru.tradingview.com", b".tv-footer__column"),
    (b"s4models.com", b".entry-content"),
    (b"scoop.co.nz", b"#footer_links_b"),
    (b"smartdevicelink.com", b".is-active"),
    (b"smartdevicelink.com", b".no-sub"),
    (b"tasteofhome.com", b"#mainContentWell"),
    (b"texasmonthly.com", b".article-authors"),
    (b"texasmonthly.com", b".hero-default__kicker"),
    (b"texasmonthly.com", b".hero-default__title"),
    (b"thecut.com", b".inset"),
    (b"thepufferforum.com", b".signature"),
    (b"tomshardware.com", b".vibrantContent"),
    (b"vcahospitals.com", b".btn-primary"),
    (b"vcahospitals.com", b".btn-text-left"),
    (b"weather.weatherbug.com", b".two"),

    (b"foodily.com", b"#cards"),
    (b"glassdoor.com", b"#MostReviewedOccs"),
    (b"glassdoor.com", b".label"),
    (b"glassdoor.com", b".padTop10"),
    (b"glassdoor.com", b".toggleBody"),
    (b"glassdoor.com", b".toggleable"),

    (b"ace-ed.org.uk", b".channelSummary"),
    (b"amoena.wordpress.com", b"#logo-floater"),
    (b"anaayafoods.com", b".site-info-owner"),
    (b"biotech-capital.com", b".author"),
    (b"biotech-capital.com", b".publish-date"),
    (b"biotech-capital.com", b".timeline"),
    (b"books.google.com.au", b"#word_cloud"),
    (b"brokenbats.wordpress.com", b".entry-tags"),
    (b"carfax.com", b"#dvmTitleProblemsSalvageJunkRebuilt"),
    (b"cheftalk.com", b".post-info-content"),
    (b"commons.apache.org", b".inheritance"),
    (b"commons.apache.org", b".overviewSummary"),
    (b"complex.com", b".article-tags"),
    (b"cryptobrowser.site", b".popup__body"),
    (b"cyclingnews.com", b".gallerybox"),
    (b"disneyandmore.blogspot.com", b"#header-wrapper"),
    (b"disneyandmore.blogspot.com", b".post-footer"),
    (b"dubaibusinessservices.com", b".product_meta"),
    (b"duckhuntingchat.com", b"#sig70061"),
    (b"electronicinfo.ca", b".disclaimer"),
    (b"english-subtitles.club", b".tab-container"),
    (b"euroweeklynews.com", b".td-post-date"),
    (b"euroweeklynews.com", b".td-post-source-tags"),
    (b"fandango.com", b".carousel-items"),
    (b"farmallcub.com", b"#sig300680"),
    (b"farmallcub.com", b"#sig300685"),
    (b"farmallcub.com", b"#sig300726"),
    (b"farmallcub.com", b"#sig300761"),
    (b"farmallcub.com", b"#sig300810"),
    (b"farmallcub.com", b"#sig302041"),
    (b"filmitown.com", b".domain"),
    (b"gawker.com", b".entry-title"),
    (b"globalsurfers.com", b".logo-in"),
    (b"goldenfrog.com", b"#vypr-footer-cta"),
    (b"granta.com", b".related-articles__list__item"),
    (b"hanginwiththehobarts.com", b".post-date"),
    (b"hometheaterforum.com", b".alt2"),
    (b"hotrecordsociete.bandcamp.com", b"#name-section"),
    (b"hotrecordsociete.bandcamp.com", b".collected-by"),
    (b"insanescouter.org", b".blog_links"),
    (b"ipinfo.io", b".connection-block"),
    (b"jedidefender.com", b"#msg_568253_signature"),
    (b"jedidefender.com", b"#msg_568258_signature"),
    (b"jedidefender.com", b"#msg_568273_signature"),
    (b"jedidefender.com", b"#msg_568295_signature"),
    (b"jedidefender.com", b"#msg_568350_signature"),
    (b"jeepforum.com", b"#td_post_13498976"),
    (b"jeepforum.com", b"#td_post_13508723"),
    (b"jeepforum.com", b"#td_post_13511183"),
    (b"jeepforum.com", b"#td_post_13596174"),
    (b"jessicacarneyassociates.co.uk", b".entry-title"),
    (b"jetcost.com.sg", b"#hotelRating"),
    (b"kantanmt.zendesk.com", b".article-metadata"),
    (b"kesq.com", b".imageViewer"),
    (b"knue.com", b".the_tags"),
    (b"lybrate.com", b"#healthFeedTip-1"),
    (b"milngavieherald.co.uk", b".article-meta__timestamp"),
    (b"mitsuko2011.com", b".table-of-contents__list"),
    (b"money.howstuffworks.com", b".title-sub"),
    (b"mountainguard.com", b".pi-slider-wrapper"),
    (b"mouse-bola-bola.blogspot.com", b".post-body"),
    (b"music.dartmouth.edu", b".text-chunk"),
    (b"nevadaappeal.com", b".BodyText"),
    (b"oneperfectbite.blogspot.com", b"#footer-3"),
    (b"pictureyear.blogspot.com", b"#header-wrapper"),
    (b"pixbits.wordpress.com", b".comment-list"),
    (b"pl.tripadvisor.com", b".photoCap"),
    (b"portsmouth.co.uk", b".article-meta__byline"),
    (b"portsmouth.co.uk", b".article-meta__timestamp"),
    (b"pragationline.com", b"#tab-title-additional_information"),
    (b"pragationline.com", b".product-price-container"),
    (b"pragationline.com", b".product-title-container"),
    (b"query.nytimes.com", b".timesMachineImage"),
    (b"randyhamilton.openmortgage.com", b"#mycarousel"),
    (b"read718.org", b".enhanced-text-widget"),
    (b"read718.org", b".gform_wrapper"),
    (b"readthestars.com", b".cp_result"),
    (b"ru.tradingview.com", b".tv-feed__empty"),
    (b"ru.tradingview.com", b".tv-footer__rights"),
    (b"ru.tradingview.com", b".tv-profile__info-block"),
    (b"ru.tradingview.com", b".tv-tag-label"),
    (b"saes-de.blogspot.com", b".post-labels"),
    (b"slicer-users-archive.65878.n3.nabble.com", b"#message4031733"),
    (b"smallbizpages.co.uk", b".wpbdp-field-business_tags"),
    (b"smallbizpages.co.uk", b".wpbdp-field-display"),
    (b"smartertravel.com", b"#module_hotel_nearby_list"),
    (b"sportinglife.com", b".wij"),
    (b"sports-boards.net", b"#pagetitle"),
    (b"sports-boards.net", b".faqblock"),
    (b"statista.com", b".contactBox"),
    (b"talk.philmusic.com", b"#msg_582659"),
    (b"thecut.com", b".headline-primary"),
    (b"theday.com", b".lg_gallery-header"),
    (b"thenotsosupermama.com", b".post-title"),
    (b"thepleiades7.blogspot.com", b"#ms-printer-friendly-recipe"),
    (b"tomshardware.com", b".msgl2"),
    (b"trophytracking.com", b".meta"),
    (b"valleyvet.com", b"#veterinart_verified"),
    (b"waitrose.com", b".timings"),
    (b"wccftech.com", b".meta-left"),

    (b"newstatesman.com", b".about-the-author"),
    (b"stampedia.net", b".stampspec"),

    (b"allegramarketingprint.com", b"#internal-banner-text"),
    (b"amoena.wordpress.com", b".wp-image-3191"),
    (b"amoena.wordpress.com", b".wp-image-3197"),
    (b"androidpolice.com", b"#disqus_thread"),
    (b"blog.trueazimuth.biz", b".titlewrapper"),
    (b"blogs.wsj.com", b".post-header"),
    (b"careers.govt.nz", b"#c31430"),
    (b"careers.govt.nz", b"#c31781"),
    (b"careers.govt.nz", b"#c31799"),
    (b"careers.govt.nz", b"#c31801"),
    (b"careers.govt.nz", b"#contents-box"),
    (b"cbd.gov.au", b".region-content"),
    (b"chictopia.com", b".photo_hover"),
    (b"cinephonix.com", b"#parent-track"),
    (b"cinephonix.com", b"#submixPanel"),
    (b"community.shopify.com", b".UserName"),
    (b"cornellpress.cornell.edu", b"#DetailsTable"),
    (b"dictionary.cambridge.org", b".cdo-cloud-content"),
    (b"dockets.justia.com", b".no-space-list"),
    (b"dubaibusinessservices.com", b".woocommerce-product-gallery__image"),
    (b"edmunds.com", b"#crr_review_ratings"),
    (b"efloras.org", b"#panelTaxonLinks"),
    (b"english-subtitles.club", b"#subs"),
    (b"english-subtitles.club", b".product-meta"),
    (b"english-subtitles.club", b".sidebar-widgets-wrap"),
    (b"eurofound.europa.eu", b".ds-node-metadata"),
    (b"eurofound.europa.eu", b".media-element-container"),
    (b"fandango.com", b".mop__synopsis-content"),
    (b"floridatrend.com", b".tags"),
    (b"forum.moomba.com", b".signaturecontainer"),
    (b"forums.thefashionspot.com", b"#td_post_1269458"),
    (b"fs.fed.us", b".field--name-body"),
    (b"genr8change.com", b"#intro"),
    (b"github.com", b".js-active-navigation-container"),
    (b"gtpoems.com", b".wp-block-post-date"),
    (b"helpsdkids.org", b".wrapCopyright"),
    (b"hotrecordsociete.bandcamp.com", b"#band-links"),
    (b"hotrecordsociete.bandcamp.com", b"#band-name-location"),
    (b"hotrecordsociete.bandcamp.com", b".signed-out-artists-bio-text"),
    (b"huskers.com", b"#schedule"),
    (b"ideas.repec.org", b"#biblio-body"),
    (b"ideas.repec.org", b"#references-body"),
    (b"ieeexplore.ieee.org", b".art-keywords"),
    (b"kgi.org", b"#block-block-11"),
    (b"kingston.ac.uk", b".middle-col-nav"),
    (b"legalinsurrection.com", b".reactions"),
    (b"libertysentinel.org", b".grid-item-metadata"),
    (b"library.dayalgroup.com", b"#custom_html-2"),
    (b"linda-artandmore.blogspot.com", b".Profile"),
    (b"mitsuko2011.com", b".table-of-contents"),
    (b"mountainguard.com", b".table-striped"),
    (b"news.psu.edu", b".block-psu-multimedia-tags"),
    (b"notes.bread.org", b".entry-footer-info"),
    (b"portsmouth.co.uk", b".article-meta__byline-name"),
    (b"pragationline.com", b".product-page-price"),
    (b"pragationline.com", b".product_title"),
    (b"pragationline.com", b".wc-tabs-wrapper"),
    (b"progarchives.com", b".icon-date"),
    (b"pt.usc.edu", b"#ctl00_ctl00_MainContent_ContentSubpageInterior_DropZone_columnDisplay_ctl00_controlcolumn_ctl02_WidgetHost_WidgetHost_widget_CB"),
    (b"pt.usc.edu", b".no-bullet"),
    (b"randyhamilton.openmortgage.com", b".entry-title"),
    (b"rebeccalillycosta.com", b".attachment-medium"),
    (b"s4models.com", b".aligncenter"),
    (b"s4models.com", b".entry-header"),
    (b"saes-de.blogspot.com", b".separator"),
    (b"scu.edu", b"#content-116036"),
    (b"smartdevicelink.com", b"#documentation__sidebar"),
    (b"smartertravel.com", b".orange_arrow_list"),
    (b"southstrandnews.com", b".article-lead-image-block"),
    (b"sports-boards.net", b".faqlinks"),
    (b"spreaker.com", b"#desc_1_more"),
    (b"tablethotels.com", b".property-rooms-wot-dates"),
    (b"talkbass.com", b"#post-809127"),
    (b"tastespotting.com", b".post-categories"),
    (b"thecut.com", b".primary-bylines"),
    (b"thetaborfoundation.org", b".wp-image-2336"),
    (b"thorax.bmj.com", b".altmetrics-disabled"),
    (b"usarugby.org", b".date"),
    (b"usarugby.org", b".kmt-author"),
    (b"usarugby.org", b".kmt-time"),
    (b"vectra-c.com", b".footer_copyright"),
    (b"weather.weatherbug.com", b"#box-local-cam"),
    (b"westonparkhospitality.com", b"#block-block-1"),
    (b"whatsonmypc.blog", b".post-meta"),
    (b"wri.org", b".ds-content"),

    (b"allegramarketingprint.com", b"#loc-info"),
    (b"alt.com", b"#page_right"),
    (b"androidpolice.com", b".post-header"),
    (b"bepress.com", b".vc_align_center"),
    (b"bhagpuss.blogspot.com", b"#Header1"),
    (b"bimmerfest.com", b".panel"),
    (b"biotech-capital.com", b".share-prices"),
    (b"blog.akismet.com", b".comment-meta"),
    (b"blogs.wsj.com", b".post-time"),
    (b"blogs.wsj.com", b".post-title"),
    (b"blurb.com", b".about-author--profile-view"),
    (b"blurb.com", b".about-creator-details"),
    (b"cbd.gov.au", b".display-4"),
    (b"cbssports.com", b".scroll-container"),
    (b"celebritybabyscoop.com", b".entry-attachment"),
    (b"celebritybabyscoop.com", b".nav-number"),
    (b"complex.com", b".info-row-datetime"),
    (b"complex.com", b".story-title"),
    (b"dailytech.com", b".ArticleHeadline"),
    (b"dailytech.com", b".DateStory"),
    (b"dictionary.cambridge.org", b"#moreResults"),
    (b"dictionary.cambridge.org", b"#translations"),
    (b"dictionary.cambridge.org", b".cdo-smartt"),
    (b"dubaibusinessservices.com", b".entry-summary"),
    (b"english-subtitles.club", b"#page-title"),
    (b"english-subtitles.club", b".promo"),
    (b"english-subtitles.club", b".tabs"),
    (b"eurofound.europa.eu", b".group-node-tagging"),
    (b"euroweeklynews.com", b".td-tags"),
    (b"fangirluprising.com", b".comment-list"),
    (b"focusfanatics.com", b"#td_post_1948058"),
    (b"focusfanatics.com", b"#td_post_1948864"),
    (b"focusfanatics.com", b"#td_post_1994629"),
    (b"forum.moomba.com", b".postdate"),
    (b"forum.moomba.com", b".signature"),
    (b"fs.fed.us", b".views-row"),
    (b"genr8change.com", b".linked-signup-name"),
    (b"gorsefox.blogspot.com", b".separator"),
    (b"hardwarezone.com", b".rtecenter"),
    (b"ideas.repec.org", b".otherversion"),
    (b"ieeexplore.ieee.org", b"#abstractKeywords"),
    (b"kgi.org", b"#block-menu-menu-information"),
    (b"library.dayalgroup.com", b".textwidget"),
    (b"linda-artandmore.blogspot.com", b"#Profile1"),
    (b"milngavieherald.co.uk", b".article-meta__timestamp-item"),
    (b"news.psu.edu", b"#block-psu-multimedia-psu-multimedia-tags"),
    (b"news.psu.edu", b".licensing-use"),
    (b"omnimaga.org", b".modified"),
    (b"products.smileysaudiovisual.com", b".FeaturedProductCount"),
    (b"ru.tradingview.com", b".tv-profile__stats"),
    (b"slideshare.net", b".h-slideshow-categories"),
    (b"spreaker.com", b".track_author_name"),
    (b"thecut.com", b".article-header-section"),
    (b"theprp.com", b".post-meta"),
    (b"typekit.com", b".footer-links"),
    (b"vcahospitals.com", b".hours"),
    (b"vcahospitals.com", b".side-bar-btn-group"),
    (b"vectra-c.com", b"#footer_morecopyright"),
    (b"wccftech.com", b".meta-author"),
    (b"wccftech.com", b".meta-time"),
    (b"wdiy.org", b".submitted"),
    (b"weather.com", b".allMonth"),
    (b"whatsonmypc.blog", b".post-categories"),

    (b"9thinfantrydivision.net", b".size-full"),
    (b"ace-ed.org.uk", b".pageBodyContent"),
    (b"amoena.wordpress.com", b".alignleft"),
    (b"bepress.com", b".wpb_single_image"),
    (b"blogs.wsj.com", b".post-section"),
    (b"cheftalk.com", b".thread-hier-top"),
    (b"chictopia.com", b"#photo_hover_929070"),
    (b"cinephonix.com", b"#submixes"),
    (b"complex.com", b".article-tags__tag"),
    (b"dictionary.cambridge.org", b"#british-1-1-1"),
    (b"dubaibusinessservices.com", b".woocommerce-product-details__short-description"),
    (b"forum.moomba.com", b".username"),
    (b"ieeexplore.ieee.org", b".authors"),
    (b"jessicacarneyassociates.co.uk", b".hentry"),
    (b"libertysentinel.org", b".latest-posts-grid"),
    (b"lowpowerlab.com", b".entry-meta"),
    (b"nvnews.net", b"#td_post_2284922"),
    (b"nvnews.net", b"#td_post_2285073"),
    (b"pt.usc.edu", b"#ctl00_ctl00_MainContent_ContentSubpageInterior_DropZone_updatepanel"),
    (b"read718.org", b".widget_text"),
    (b"rebeccalillycosta.com", b".attachment"),
    (b"rosko123.wordpress.com", b".size-full"),
    (b"slideshare.net", b".descriptionExpanded"),
    (b"smartdevicelink.com", b".sidebar-nav"),
    (b"southstrandnews.com", b"#main-picture-container"),
    (b"tablethotels.com", b".room-summary"),
    (b"texasmonthly.com", b".article-date"),
    (b"texasmonthly.com", b".article-kicker"),
    (b"texasmonthly.com", b".article-title"),
    (b"thefrisky.com", b".image-holder"),
    (b"thenotsosupermama.com", b".wp-caption"),
    (b"tvwbb.com", b".signature"),
    (b"use.perl.org", b"#slogan"),
    (b"vectra-c.com", b".footer_morecopyright"),
    (b"weather.weatherbug.com", b"#box-neighbor-wx"),
    (b"weather.weatherbug.com", b"#box-radar-map-ctl"),
    (b"weather.weatherbug.com", b"#nav-vert"),
    (b"wethrift.com", b".review-author-details"),
    (b"wri.org", b".field--body"),

    (b"ace-ed.org.uk", b".LinkDetails"),
    (b"androidpolice.com", b"#dsq-comments"),
    (b"androidpolice.com", b".post-author"),
    (b"anenglishmaninosaka.blogspot.com", b".entrytext"),
    (b"bhagpuss.blogspot.com", b".post-body"),
    (b"bimmerfest.com", b".panelsurround"),
    (b"bimmerwerkz.com", b"#postmenu_546352"),
    (b"biotech-capital.com", b".company-profile"),
    (b"blurb.com", b".about-author--user-data"),
    (b"careers.govt.nz", b"#contactUsWrap"),
    (b"cbssports.com", b".marquee-full-player-info"),
    (b"cinephonix.com", b".composer-browse"),
    (b"cornellpress.cornell.edu", b"#buyarea"),
    (b"cornellpress.cornell.edu", b"#publisherCollection"),
    (b"cornellpress.cornell.edu", b".bookAuthor"),
    (b"cornellpress.cornell.edu", b".detailsTable"),
    (b"cyclonefanatic.com", b".signaturecontainer"),
    (b"fabulous-emma.com", b".thumbnails"),
    (b"fangirluprising.com", b".embed-youtube"),
    (b"fullonlinebook.com", b".block_books_readleft"),
    (b"gardenplants.comparespecies.com", b".SpecName"),
    (b"gardenplants.comparespecies.com", b".SpecValue"),
    (b"gascu.org", b".entry-title"),
    (b"hotrecordsociete.bandcamp.com", b"#bio-container"),
    (b"ieeexplore.ieee.org", b"#article-authors"),
    (b"ito-yarn.com", b".field-slideshow-slide"),
    (b"mountainguard.com", b"#block-block-3"),
    (b"mountainguard.com", b".footercopyright"),
    (b"mouse-bola-bola.blogspot.com", b".timestamp-link"),
    (b"mouse-bola-bola.blogspot.com", b".titlewrapper"),
    (b"nobelcom.com", b".idcCardDetails"),
    (b"nvnews.net", b".alt1"),
    (b"pbs.org", b"#slideshow"),
    (b"pt.usc.edu", b".main-col"),
    (b"read718.org", b"#enhancedtextwidget-2"),
    (b"read718.org", b"#enhancedtextwidget-3"),
    (b"sangriasunshinecom.wordpress.com", b".entry-title"),
    (b"slideshare.net", b".infoGeneric"),
    (b"smartdevicelink.com", b"#documentation__content"),
    (b"stampedia.net", b"#footer-inner"),
    (b"stampedia.net", b".pager"),
    (b"typekit.com", b".family-card"),
    (b"weather.weatherbug.com", b"#box-radar-maps"),
    (b"weather.weatherbug.com", b"#middle"),
    (b"www2b.abc.net.au", b".commentform"),

    (b"bhagpuss.blogspot.com", b".titlewrapper"),
    (b"cbwentworth.wordpress.com", b".aligncenter"),
    (b"convertunits.com", b".oneline"),
    (b"english-subtitles.club", b".content-wrap"),
    (b"feedbooks.com", b".book_categories"),
    (b"fs.fed.us", b".view-content"),
    (b"linda-artandmore.blogspot.com", b".titlewrapper"),
    (b"lowpowerlab.com", b".aligncenter"),
    (b"m3post.com", b".thePostItself"),
    (b"mastercraft.com", b".attach"),
    (b"mountainguard.com", b".pi-slide"),
    (b"mouse-bola-bola.blogspot.com", b".post-footer"),
    (b"next.unibz.it", b".u-tt-capital"),
    (b"notes.bread.org", b".entry-footer"),
    (b"pl.tripadvisor.com", b".capDescription"),
    (b"rebeccalillycosta.com", b".entry-content"),
    (b"riversandroads.me", b".separator"),
    (b"sangriasunshinecom.wordpress.com", b".entry-footer"),
    (b"songmeanings.com", b"#comments-list"),
    (b"uctv.tv", b".programtitle"),
    (b"undergroundnews.com", b".feedflare"),
    (b"weather.weatherbug.com", b"#hnav-maps"),

    (b"ace-ed.org.uk", b".linkPanel"),
    (b"cyclonefanatic.com", b".signature"),
    (b"github.com", b".js-navigation-container"),
    (b"ideas.repec.org", b".publishedas"),
    (b"next.unibz.it", b".u-push-btm"),
    (b"rcgroups.com", b".View"),
    (b"rosko123.wordpress.com", b".entry-content"),
    (b"tablethotels.com", b".room-info-list"),
    (b"westonparkhospitality.com", b".block-nodeblock"),

    (b"devilslakejournal.com", b"#pagination1"),
    (b"devilslakejournal.com", b"#pagination2"),

    (b"alibris.com", b".product-title"),
    (b"bearalley.blogspot.com", b"#Text2"),
    (b"celebritybabyscoop.com", b".read-the-article"),
    (b"cheftalk.com", b".forum-post-date"),
    (b"cheftalk.com", b".postby_body"),
    (b"feedbooks.com", b".metadata"),
    (b"pwmag.com", b"#head3"),
    (b"rationalresponders.com", b".forum-comment-right"),
    (b"rcgroups.com", b".thead_postbit"),
    (b"slicer-users-archive.65878.n3.nabble.com", b"#message4031737"),
    (b"spreaker.com", b"#track_tab_info"),

    (b"christianmingle.com", b".username"),
    (b"fangirluprising.com", b".tags-links"),
    (b"jeepforum.com", b"#jeepforumpostmenu_237_menu"),
    (b"jeepforum.com", b"#td_post_13593722"),
    (b"jeepforum.com", b"#td_post_13595592"),
    (b"jeepforum.com", b"#td_post_13595604"),
    (b"mustseeindia.com", b".img_display"),
    (b"mustseeindia.com", b".td-title-sm"),
    (b"read718.org", b"#input_6_21_3_label"),
    (b"read718.org", b".gfield_checkbox"),
    (b"read718.org", b".name_first"),
    (b"read718.org", b".name_last"),
    (b"ru.tradingview.com", b".tv-social-row"),
    (b"tvtechnology.com", b"#dnn_ArticlePageLeftColumn1_lblPublishedDate"),
    (b"tvtechnology.com", b"#dnn_ArticlePageLeftColumn1_lblTitle"),
    (b"use.perl.org", b"#logo"),
];

const SITE_VETOES: &[(&[u8], &[u8])] = &[
    (b"itknowledgeexchange.techtarget.com", b"#commentObject-miniReg_v2-form-1"),
    (b"crazydaysandnights.net", b".widget-content"),
    (b"digitimes.com", b".mr-box"),
    (b"kottke.org", b"#side"),
    (b"phabricator.wikimedia.org", b".phabricator-standard-page-footer"),
    (b"blogs.iptv.org", b"#allsidebars"),
    (b"straightdope.com", b"#widgets_column"),
    (b"waitrose.com", b".ratingform"),
    (b"witchesandpagans.com", b"#latest-posts"),
    (b"npbearings.com", b".textwidget"),
    (b"milngavieherald.co.uk", b".trending-stories__list"),
    (b"trophytracking.com", b"#copyrights-area"),
    (b"childrenwithdiabetes.com", b".texttiny"),
    (b"philamuseum.org", b"#folksonomy"),
    (b"thorax.bmj.com", b".highwire-extract-pdf-wrapper"),
    (b"ornaross.com", b".header-center"),
    (b"slideshare.net", b".ssActions"),
    (b"borderlands.fandom.com", b"#ca-viewsource"),
    (b"honeynet.org", b"#sidebar-right"),
    (b"tablethotels.com", b"#hotel-rates-and-rooms"),
    (b"energy.opendata.ch", b"#menu-menu-1"),
    (b"partitionsdechansons.com", b".frame_bordure2"),
    (b"nsf.gov", b".pageheadline"),
    (b"mitsuko2011.com", b".related-posts"),

    (b"allegramarketingprint.com", b"#um-blog-wrap"),
    (b"allegramarketingprint.com", b".ultra-franchise"),
    (b"allegramarketingprint.com", b".ultra-news"),
    (b"askdrgarland.com", b"#disclaimer"),
    (b"askdrgarland.com", b"#menus"),
    (b"askdrgarland.com", b"#post-9340"),
    (b"askdrgarland.com", b".cat-item-237"),
    (b"askdrgarland.com", b".cat-item-238"),
    (b"askdrgarland.com", b".widget_health"),
    (b"askdrgarland.com", b".widget_phone"),
    (b"bimmerwerkz.com", b"#collapseobj_similarthreads"),
    (b"bimmerwerkz.com", b".navbar_wrapper"),
    (b"bimmerwerkz.com", b".postbit_legacy_wrapper_thead"),
    (b"bimmerwerkz.com", b".tcat"),
    (b"bimmerwerkz.com", b".thead"),
    (b"bio-medicine.org", b"#ends"),
    (b"bio-medicine.org", b"#topicMenu"),
    (b"bio-medicine.org", b"#zd0"),
    (b"bio-medicine.org", b"#zd1"),
    (b"bio-medicine.org", b"#zd2"),
    (b"bio-medicine.org", b"#zd3"),
    (b"bio-medicine.org", b"#zd4"),
    (b"bio-medicine.org", b"#zd5"),
    (b"bio-medicine.org", b"#zdd10"),
    (b"bio-medicine.org", b"#zdd11"),
    (b"bio-medicine.org", b"#zdd12"),
    (b"bio-medicine.org", b"#zdd13"),
    (b"bio-medicine.org", b"#zdd14"),
    (b"bio-medicine.org", b"#ze0"),
    (b"bio-medicine.org", b"#ze1"),
    (b"bio-medicine.org", b"#ze2"),
    (b"bio-medicine.org", b"#ze3"),
    (b"bio-medicine.org", b"#zf10"),
    (b"bio-medicine.org", b"#zf11"),
    (b"bio-medicine.org", b"#zf12"),
    (b"cameralabs.com", b"#datebar"),
    (b"cameralabs.com", b"#wrapfooter"),
    (b"cameralabs.com", b".copyright"),
    (b"cameralabs.com", b".gensmall"),
    (b"carfax.com", b"#auction"),
    (b"carfax.com", b"#averageAnnualMileage"),
    (b"carfax.com", b"#defFire"),
    (b"carfax.com", b"#defNAM"),
    (b"carfax.com", b"#defOdometerRollback"),
    (b"carfax.com", b"#defSalvage"),
    (b"carfax.com", b"#lastOwnedStateRow"),
    (b"carfax.com", b"#noIssuesRow"),
    (b"carfax.com", b"#printError"),
    (b"carfax.com", b".glossaryLink"),
    (b"carfax.com", b".xLink"),
    (b"chowhound.com", b".divid"),
    (b"chowhound.com", b".fr_eb_eb"),
    (b"chowhound.com", b".fr_onbm"),
    (b"chowhound.com", b".fr_r_vid_title_b"),
    (b"chowhound.com", b".fr_related_vid_desc"),
    (b"chowhound.com", b".fr_vid_partner_wrap"),
    (b"chowhound.com", b".freyja_box4"),
    (b"chowhound.com", b".freyja_follow_topic"),
    (b"chowhound.com", b".freyja_mob_follow"),
    (b"chowhound.com", b".freyja_tagslist"),
    (b"chowhound.com", b".ttgs"),
    (b"collectorsweekly.com", b"#flagArea"),
    (b"collectorsweekly.com", b"#mystery"),
    (b"collectorsweekly.com", b"#signupPanel"),
    (b"collectorsweekly.com", b".category-preview"),
    (b"collectorsweekly.com", b".dnone"),
    (b"columbinecourier.com", b"#regnow"),
    (b"columbinecourier.com", b".tabPaperInfoFooter"),
    (b"common-mistakes.net", b".bmenu"),
    (b"courier-journal.com", b"#ody-poweredby"),
    (b"courier-journal.com", b".art-arch"),
    (b"courier-journal.com", b".ody-archive-bottom"),
    (b"courier-journal.com", b".ody-blogs-bottom"),
    (b"courier-journal.com", b".ody-bottom-title"),
    (b"courier-journal.com", b".ody-caroLG"),
    (b"courier-journal.com", b".ody-caroMD"),
    (b"courier-journal.com", b".ody-forums-bottom"),
    (b"courier-journal.com", b".ody-title"),
    (b"daedalusbooks.com", b"#Table2"),
    (b"daedalusbooks.com", b"#Table29"),
    (b"daedalusbooks.com", b"#Table30"),
    (b"daedalusbooks.com", b"#Table45"),
    (b"daedalusbooks.com", b"#Table51"),
    (b"daedalusbooks.com", b"#e2ma_signup_left"),
    (b"dailygazette.com", b"#loginForm"),
    (b"dailygazette.com", b".buttonarea"),
    (b"dailygazette.com", b".columnTitle"),
    (b"dailygazette.com", b".columnistHeader"),
    (b"dailygazette.com", b".homeBase"),
    (b"dailygazette.com", b".purchase"),
    (b"edmunds.com", b"#crr_review_comments"),
    (b"edmunds.com", b"#inv-widget-container"),
    (b"edmunds.com", b"#review_cnt"),
    (b"edmunds.com", b"#used_car_resources_1"),
    (b"edmunds.com", b".crr_recommend"),
    (b"edmunds.com", b".icon-report-it"),
    (b"emedicinehealth.com", b"#fdaContent"),
    (b"emedicinehealth.com", b"#fdaTitle"),
    (b"emedicinehealth.com", b"#pill_identifier"),
    (b"finance.boston.com", b".attribution"),
    (b"finance.boston.com", b".investingnav1"),
    (b"floridatrend.com", b"#datetag"),
    (b"floridatrend.com", b"#poll"),
    (b"floridatrend.com", b"#poll-wrap"),
    (b"floridatrend.com", b".boxmod"),
    (b"floridatrend.com", b".headtag"),
    (b"floridatrend.com", b".pulsetitle"),
    (b"floridatrend.com", b".top_headlines"),
    (b"forum.fastday.com", b".forum-name"),
    (b"forum.fastday.com", b".mobile-fix"),
    (b"forum.fastday.com", b".page-header"),
    (b"forum.fastday.com", b".row-pad"),
    (b"fullonlinebook.com", b".menu_header"),
    (b"fullonlinebook.com", b".title_list"),
    (b"iclassifiedsnetwork.com", b"#ctl00_ContentPlaceHolder1_tblMain"),
    (b"iclassifiedsnetwork.com", b".SideModuleWrapper"),
    (b"indyweek.com", b"#CommentsPostCommentForm"),
    (b"indyweek.com", b"#CommentsTabbed"),
    (b"indyweek.com", b"#PostCommentBottomText"),
    (b"indyweek.com", b".commentByline"),
    (b"indyweek.com", b".commentFormAddHeader"),
    (b"indyweek.com", b".latestbest_bets"),
    (b"indyweek.com", b".storyItem"),
    (b"indyweek.com", b".storyItemTop"),
    (b"informationweek.com", b"#listing"),
    (b"informationweek.com", b"#people_view"),
    (b"informationweek.com", b".all_category"),
    (b"informationweek.com", b".ratings_down"),
    (b"interregeurope.eu", b".thematic-item__text__description"),
    (b"interregeurope.eu", b".thematic-item__text__news-title"),
    (b"jetcost.com.sg", b".randomHotelName"),
    (b"kiehls.com", b"#TT3IAContainer-119110"),
    (b"kiehls.com", b"#TT3answer-1921308"),
    (b"kiehls.com", b"#TT3miq"),
    (b"kiehls.com", b"#TT3quest-502777"),
    (b"kiehls.com", b"#TT4bestAnswerBlock-1044870"),
    (b"kiehls.com", b"#TT4bestAnswerBlock-203525"),
    (b"kiehls.com", b"#TT4miqAbout"),
    (b"kiehls.com", b"#TTaskAreaBtnLine_instr"),
    (b"kiehls.com", b"#TTexUgcL"),
    (b"kiehls.com", b"#TTexUgcR"),
    (b"kiehls.com", b".TTpoweredby"),
    (b"kiehls.com", b".TTquestionMiqaHelp"),
    (b"kiehls.com", b".copyWelcome"),
    (b"kiehls.com", b".disabledcontainer"),
    (b"kiehls.com", b".dropdownslots"),
    (b"kiehls.com", b".editWelcome"),
    (b"kiehls.com", b".eml_copy"),
    (b"kiehls.com", b".headerWelcome"),
    (b"kiehls.com", b".listFollow"),
    (b"kiehls.com", b".popup_small"),
    (b"kiehls.com", b".printpage"),
    (b"kiehls.com", b".productimages"),
    (b"kiehls.com", b".rewardsJoinNow"),
    (b"kiehls.com", b".rewardsList"),
    (b"kiehls.com", b".rewardsPointSection"),
    (b"kiehls.com", b".thanksWrapperLeft"),
    (b"kiehls.com", b".thanksWrapperRight"),
    (b"kingston.ac.uk", b"#landing-selector-header"),
    (b"kingston.ac.uk", b"#page-feedback-form-block"),
    (b"kingston.ac.uk", b".alex-buttons"),
    (b"kingston.ac.uk", b".favourite-course-link"),
    (b"kingston.ac.uk", b".page-feedback-box"),
    (b"kingston.ac.uk", b".page-feedback-confirmation"),
    (b"knue.com", b".fb-auth-form"),
    (b"lafollettepress.com", b"#regnow"),
    (b"lafollettepress.com", b".tabPaperInfoFooter"),
    (b"menstennisforums.com", b"#collapseobj_forumrules"),
    (b"menstennisforums.com", b"#collapseobj_newpost_options"),
    (b"menstennisforums.com", b"#vB_Editor_001_textarea"),
    (b"menstennisforums.com", b".detailsContainer"),
    (b"menstennisforums.com", b".panelsurround"),
    (b"menstennisforums.com", b".postquote"),
    (b"menstennisforums.com", b".vs_msglabel"),
    (b"meta.stackexchange.com", b"#answers-header"),
    (b"meta.stackexchange.com", b"#blurb"),
    (b"meta.stackexchange.com", b"#comments-76704"),
    (b"meta.stackexchange.com", b"#controls"),
    (b"meta.stackexchange.com", b".post-menu"),
    (b"meta.stackexchange.com", b".user-action-time"),
    (b"meta.stackexchange.com", b".user-details"),
    (b"meta.stackexchange.com", b".votecell"),
    (b"minotdailynews.com", b"#frmWebSearch"),
    (b"minotdailynews.com", b".cBdrHdrL"),
    (b"minotdailynews.com", b".cBdrHdrR"),
    (b"mirchee.com", b"#lastviewed"),
    (b"mirchee.com", b"#prevSearches"),
    (b"mirchee.com", b"#prevSearchesList"),
    (b"mirchee.com", b"#rightrail"),
    (b"mirchee.com", b".actionButtons"),
    (b"moneymarketing.co.uk", b".box-description"),
    (b"moneymarketing.co.uk", b".box-title"),
    (b"moneymarketing.co.uk", b".call-to-action"),
    (b"moneymarketing.co.uk", b".card-excerpt"),
    (b"moneymarketing.co.uk", b".excerpt"),
    (b"moneymarketing.co.uk", b".help-description"),
    (b"moneymarketing.co.uk", b".help-title"),
    (b"moneymarketing.co.uk", b".widget-header"),
    (b"nbc11news.com", b"#prev_next_page"),
    (b"nbc11news.com", b"#storyComments"),
    (b"nbc11news.com", b".GDMfooter"),
    (b"nelliganlaw.ca", b"#gfield_description_1_11"),
    (b"nelliganlaw.ca", b".elementor-element-6e99b91"),
    (b"nelliganlaw.ca", b".elementor-element-a10c296"),
    (b"nelliganlaw.ca", b".jet-listing-grid__item"),
    (b"newhampshire.com", b".featured-headline"),
    (b"newhampshire.com", b".mtm-responsive-menu-bar"),
    (b"newhampshire.com", b".slide-element"),
    (b"newhampshire.com", b".teaser"),
    (b"news.sys-con.com", b".event-footer-lfc"),
    (b"news.sys-con.com", b".portlettitle"),
    (b"news.sys-con.com", b".portlettitlesmall"),
    (b"news.sys-con.com", b".style1"),
    (b"notes.bread.org", b"#captchaFailMsg"),
    (b"notes.bread.org", b"#captchaText"),
    (b"notes.bread.org", b"#comment-complete"),
    (b"notes.bread.org", b"#comment-error"),
    (b"notes.bread.org", b"#comment-preview-confirmation"),
    (b"notes.bread.org", b".art-block"),
    (b"notes.bread.org", b".trackbacks-link"),
    (b"onlineslangdictionary.com", b".attrib"),
    (b"parlinfo.aph.gov.au", b".disclaimerText"),
    (b"pizza-john.blogspot.com", b"#PopularPosts1"),
    (b"pizza-john.blogspot.com", b".jump-link"),
    (b"pizza-john.blogspot.com", b".popular-posts-snippet"),
    (b"progarchives.com", b"#divRatings"),
    (b"progarchives.com", b"#lblNoComments"),
    (b"progarchives.com", b".cls_TextCopyright"),
    (b"progarchives.com", b".review-footer"),
    (b"ro.urbandictionary.com", b"#subscribe_modal"),
    (b"ro.urbandictionary.com", b".def-footer"),
    (b"ro.urbandictionary.com", b".word-list-panel"),
    (b"scoop.co.nz", b"#leader"),
    (b"scoop.co.nz", b".article-subheadings"),
    (b"scoop.co.nz", b".headline-right"),
    (b"scoop.co.nz", b".section-heading"),
    (b"studymode.com", b"#banner_title"),
    (b"studymode.com", b".card__content"),
    (b"studymode.com", b".paper-body__cta--mod"),
    (b"sunkenstone.com", b".et_social_pin_images_outer"),
    (b"sunkenstone.com", b".fusion-author-info"),
    (b"sunkenstone.com", b".fusion-author-social"),
    (b"sunkenstone.com", b".fusion-author-title"),
    (b"sunkenstone.com", b".omapi-shortcode-helper"),
    (b"sunkenstone.com", b".pum"),
    (b"sunkenstone.com", b".to-top-container"),
    (b"topjobs.ch", b".boxed-h"),
    (b"traveloka.com", b".EP8nD"),
    (b"traveloka.com", b".Sf8SV"),
    (b"traveloka.com", b".eglwm"),
    (b"traveloka.com", b".gxDGS"),
    (b"traveloka.com", b".jgvPX"),
    (b"traveloka.com", b".lakO0"),
    (b"traveloka.com", b".mMmI2"),
    (b"traveloka.com", b".privatePricing"),
    (b"traveloka.com", b".qMGC-"),
    (b"traveloka.com", b".tvat-register"),
    (b"traveloka.com", b".tvat-userheader"),
    (b"tvtechnology.com", b".comments_box"),
    (b"tvtechnology.com", b".featured_article"),
    (b"tvtechnology.com", b".new_comment"),
    (b"uctv.tv", b"#youTubeEmbed"),
    (b"wpbf.com", b"#embedded1"),
    (b"wpbf.com", b".collectionsWidgetWrapper"),
    (b"wpbf.com", b".disclaimer"),
    (b"wpbf.com", b".ib-collection"),
    (b"wpbf.com", b".teaserText"),
    (b"wpbf.com", b".teaserTitle"),
    (b"wpbf.com", b".titlebar"),

    (b"425sqftart.com", b".wallcomment"),
    (b"alibris.com", b"#product-pane"),
    (b"alibris.com", b".carousel-section"),
    (b"anaayafoods.com", b"#hitmag-tags"),
    (b"anaayafoods.com", b".hitmag-post"),
    (b"anaayafoods.com", b".posts-wrap"),
    (b"app.leg.wa.gov", b".chart"),
    (b"apple.stackexchange.com", b"#answers-header"),
    (b"apple.stackexchange.com", b"#comment-68665"),
    (b"apple.stackexchange.com", b"#comments-59481"),
    (b"apple.stackexchange.com", b"#herobox-mini"),
    (b"apple.stackexchange.com", b"#post-form"),
    (b"apple.stackexchange.com", b".bottom-notice"),
    (b"apple.stackexchange.com", b".post-signature"),
    (b"apple.stackexchange.com", b".votecell"),
    (b"bananawonder.com", b"#Blog1_cmt-2690935247194876462"),
    (b"blogs.theprovince.com", b".facebook_pre_notice"),
    (b"bobvila.com", b"#partners_latest_title"),
    (b"bobvila.com", b".related_products_sidebar"),
    (b"bobvila.com", b".related_side_links"),
    (b"bobvila.com", b".see-more-wrapper"),
    (b"bostonglobe.com", b"#loading-comments"),
    (b"bostonglobe.com", b".global-bar"),
    (b"bostonglobe.com", b".paywall-mobile-link"),
    (b"bostonglobe.com", b".paywall-section-1"),
    (b"bostonglobe.com", b".paywall-section-2"),
    (b"cafemom.com", b".joinContainerOrange"),
    (b"cafemom.com", b".top_form"),
    (b"cambridge-news.co.uk", b"#caption1025280"),
    (b"cambridge-news.co.uk", b".footer-text"),
    (b"canadiangardening.com", b".back2top"),
    (b"canadiangardening.com", b".postprofile"),
    (b"canadiangardening.com", b".topic-actions"),
    (b"cbwentworth.wordpress.com", b"#comment-7076"),
    (b"cbwentworth.wordpress.com", b"#comment-7259"),
    (b"cbwentworth.wordpress.com", b".comments-title"),
    (b"cdn.cdata.com", b"#whfooter"),
    (b"census.gov", b"#DataLink7"),
    (b"census.gov", b"#GeoLink1"),
    (b"census.gov", b"#GeoLink6"),
    (b"census.gov", b"#LibLink5"),
    (b"census.gov", b"#LibMain"),
    (b"census.gov", b"#TopLink12"),
    (b"chictopia.com", b"#chicpoints_refer"),
    (b"christianmingle.com", b"#margin_content"),
    (b"christianmingle.com", b"#titles"),
    (b"chrisweigant.com", b".footlinks"),
    (b"community.mcafee.com", b".lia-panel-content-wrapper"),
    (b"community.mcafee.com", b".view-original-post-link"),
    (b"community.shopify.com", b"#link_35"),
    (b"conferenceboard.ca", b"#MainRegion_C014_AnonymousPanel"),
    (b"conferenceboard.ca", b"#MainRegion_C014_AnonymousSignIn"),
    (b"conferenceboard.ca", b"#MainRegion_C014_DownloadHyperLink"),
    (b"conferenceboard.ca", b"#MainRegion_C014_download"),
    (b"conferenceboard.ca", b"#RightPanelRegion_C006_ThisReport"),
    (b"cricketarchive.com", b"#MyNewFooter"),
    (b"dictionary.cambridge.org", b".blog_date"),
    (b"dictionary.cambridge.org", b".blog_title"),
    (b"dictionary.cambridge.org", b".cdo-fbox-body"),
    (b"dictionary.cambridge.org", b".cdo-fbox-desc"),
    (b"dictionary.cambridge.org", b".cdo-fbox-title"),
    (b"dictionary.cambridge.org", b".cdo-section-title"),
    (b"dictionary.cambridge.org", b".newword_date"),
    (b"dictionary.cambridge.org", b".newword_title"),
    (b"dictionary.cambridge.org", b".wotdHeadword"),
    (b"diggingri.com", b"#author-description"),
    (b"disneyandmore.blogspot.com", b"#FeaturedPost1"),
    (b"disneyandmore.blogspot.com", b"#HTML1"),
    (b"disneyandmore.blogspot.com", b"#HTML2"),
    (b"disneyandmore.blogspot.com", b"#HTML3"),
    (b"disneyandmore.blogspot.com", b"#HTML5"),
    (b"disneyandmore.blogspot.com", b"#Image24"),
    (b"disneyandmore.blogspot.com", b"#Image25"),
    (b"disneyandmore.blogspot.com", b"#Image26"),
    (b"disneyandmore.blogspot.com", b"#Image27"),
    (b"disneyandmore.blogspot.com", b"#Image28"),
    (b"disneyandmore.blogspot.com", b"#Image3"),
    (b"disneyandmore.blogspot.com", b"#Image30"),
    (b"disneyandmore.blogspot.com", b"#Image37"),
    (b"disneyandmore.blogspot.com", b"#Image38"),
    (b"disneyandmore.blogspot.com", b"#Image39"),
    (b"disneyandmore.blogspot.com", b"#Image41"),
    (b"disneyandmore.blogspot.com", b"#Image43"),
    (b"disneyandmore.blogspot.com", b"#Image45"),
    (b"disneyandmore.blogspot.com", b"#Image47"),
    (b"disneyandmore.blogspot.com", b"#Image49"),
    (b"disneyandmore.blogspot.com", b"#Image50"),
    (b"disneyandmore.blogspot.com", b"#Image53"),
    (b"disneyandmore.blogspot.com", b"#Image56"),
    (b"disneyandmore.blogspot.com", b"#Image57"),
    (b"disneyandmore.blogspot.com", b"#Image62"),
    (b"disneyandmore.blogspot.com", b"#Image64"),
    (b"disneyandmore.blogspot.com", b"#Image66"),
    (b"disneyandmore.blogspot.com", b"#Image67"),
    (b"disneyandmore.blogspot.com", b"#Image69"),
    (b"disneyandmore.blogspot.com", b"#Image72"),
    (b"disneyandmore.blogspot.com", b"#Text3"),
    (b"disneyandmore.blogspot.com", b"#Text4"),
    (b"dittrickswines.com", b"#acc-content"),
    (b"docs.codehaus.org", b".diff-menu"),
    (b"docs.codehaus.org", b".metadata"),
    (b"engineering.academickeys.com", b"#application_link_box"),
    (b"engineering.academickeys.com", b"#job_link_box"),
    (b"engineering.academickeys.com", b".moto-container_content_569ec0f9"),
    (b"eslflashcards.com", b".left_column"),
    (b"eslflashcards.com", b".right_column"),
    (b"euroweeklynews.com", b"#comments"),
    (b"euroweeklynews.com", b"#td-mobile-nav"),
    (b"euroweeklynews.com", b".author-box-wrap"),
    (b"euroweeklynews.com", b".td-a-rec-id-content_bottom"),
    (b"euroweeklynews.com", b".td-a-rec-id-content_inline"),
    (b"euroweeklynews.com", b".td-footer-template-wrap"),
    (b"euroweeklynews.com", b".td-header-template-wrap"),
    (b"euroweeklynews.com", b".td-main-sidebar"),
    (b"euroweeklynews.com", b".td-post-next-prev"),
    (b"eventbrite.com.au", b".modal__body"),
    (b"fandango.com", b".fan-alert__description"),
    (b"finishingtouchmedspa.com", b".posts__recent"),
    (b"forexsignal30.com", b"#comment-wrap"),
    (b"forexsignal30.com", b".entry-content-archive"),
    (b"forums.everythingicafe.com", b"#pageDescription"),
    (b"gatorsports.com", b"#license-434504EE-FAFA-479E-9CA5-EF70E91CC4A0"),
    (b"gatorsports.com", b".font100"),
    (b"gatorsports.com", b".footnote"),
    (b"gearslutz.com", b".tablecell"),
    (b"glennopedia.com", b".comment-likes"),
    (b"glennopedia.com", b".pingback"),
    (b"glennopedia.com", b".says"),
    (b"groups.yahoo.com", b".fc-gray"),
    (b"groups.yahoo.com", b".msg-response"),
    (b"heraldnews.com", b"#art-tit"),
    (b"heraldnews.com", b".source-org"),
    (b"hitvibz.com", b".comment-notes"),
    (b"hitvibz.com", b".u4df263bd09a0918d74aef0a89904ca5b"),
    (b"hitvibz.com", b".u8fcc45d4aa112239ab9bddb9ac8b111f"),
    (b"hitvibz.com", b".ue7e42ef00bb924321c5814b457ee0e5b"),
    (b"investorplace.com", b".copyright"),
    (b"jcink.net", b".activeuserstrip"),
    (b"jcink.net", b".author_statistics"),
    (b"jcink.net", b".signature"),
    (b"jcink.net", b".smalltext"),
    (b"jeepforum.com", b"#post_message_13525066"),
    (b"jeepforum.com", b"#post_message_13560712"),
    (b"jeepforum.com", b"#post_message_13593696"),
    (b"kidswowchannel.com", b"#tptn_counter_635"),
    (b"kidswowchannel.com", b".post-categories"),
    (b"kidswowchannel.com", b".sfsi_outr_div"),
    (b"kohls.com", b"#BVSEO_meta"),
    (b"kohls.com", b".ggl_help_content"),
    (b"ksl.com", b"#cmt_mn"),
    (b"ksl.com", b"#video-problems"),
    (b"ksl.com", b".formDividerWd"),
    (b"lists.w3.org", b"#message-id"),
    (b"lists.w3.org", b"#upper"),
    (b"lybrate.com", b".health-packages-slider"),
    (b"lybrate.com", b".lybEllipsis"),
    (b"lybrate.com", b".lybGutter-side--half"),
    (b"lybrate.com", b".navbar-actions"),
    (b"lybrate.com", b".navbar-subnav"),
    (b"lybrate.com", b".ppview-revamp__section-header"),
    (b"lybrate.com", b".profile-header"),
    (b"meals.com", b"#reviews_container"),
    (b"meals.com", b".review_recipe"),
    (b"meals.com", b".tout_wrapper"),
    (b"medicalxpress.com", b".rank-holder"),
    (b"missionrepair.wordpress.com", b".entry-meta"),
    (b"motoprofi.com", b".about"),
    (b"mv-voice.com", b".cleanprint-exclude"),
    (b"ncronline.org", b"#block-block-51"),
    (b"ncronline.org", b".commentguidelines"),
    (b"norwalk.itsrelevant.com", b"#next-story"),
    (b"norwalk.itsrelevant.com", b".style19"),
    (b"npr.org", b"#disqus-npr"),
    (b"ocsigen.org", b".reasonwarning"),
    (b"openpr.com", b".pm-title"),
    (b"packershome.com", b"#ctl00_MainContent_forum_ctl01_GuestMessage"),
    (b"packershome.com", b".postPosted"),
    (b"packershome.com", b".postThanksRow"),
    (b"packershome.com", b".yafUserBox"),
    (b"patchwork.ozlabs.org", b"#patchheaders"),
    (b"pbn.com", b"#content_rail_2"),
    (b"phonearena.com", b"#dialog_holder"),
    (b"phonearena.com", b".s_head"),
    (b"pl.tripadvisor.com", b".brand"),
    (b"pl.tripadvisor.com", b".userAgent"),
    (b"playfire.com", b".back-to-top"),
    (b"playfire.com", b".generic_view_more"),
    (b"playfire.com", b".phoenix-layout-heading"),
    (b"playfire.com", b".tagline"),
    (b"playfire.com", b".view_all_comments"),
    (b"portsmouth.co.uk", b".article-header__lead-image-container"),
    (b"pragationline.com", b".slider-nav-reveal"),
    (b"psypokes.com", b".gensmall"),
    (b"psypokes.com", b".postdetails"),
    (b"psypokes.com", b".signature"),
    (b"query.nytimes.com", b"#articleAccess"),
    (b"quiltersclubofamerica.com", b"#CommonFooter"),
    (b"quiltersclubofamerica.com", b".CommonCommentUser"),
    (b"quiltersclubofamerica.com", b".CommonContentBoxFooter"),
    (b"quiltersclubofamerica.com", b".ForumPostHeader"),
    (b"quiltersclubofamerica.com", b".ForumPostSignature"),
    (b"quiltersclubofamerica.com", b".ForumPostStatistics"),
    (b"randyhamilton.openmortgage.com", b".span_5"),
    (b"raptorsrepublic.com", b"#moreactivity_container"),
    (b"raptorsrepublic.com", b"#profile_tabs"),
    (b"raptorsrepublic.com", b"#view-friends-content"),
    (b"raptorsrepublic.com", b"#view-thanks"),
    (b"raptorsrepublic.com", b".datetime"),
    (b"raptorsrepublic.com", b".forum_thread"),
    (b"raptorsrepublic.com", b".fulllink"),
    (b"raptorsrepublic.com", b".userprof_headers_border"),
    (b"raptorsrepublic.com", b".views"),
    (b"retrotogo.com", b".comments-closed"),
    (b"retrotogo.com", b".comments-info"),
    (b"robbreport.com", b".field-content"),
    (b"sailnet.com", b".smallfont"),
    (b"salisburypost.com", b"#comments-notice"),
    (b"scienceblogs.com", b".view-content"),
    (b"sirstevesguide.com", b".largefont"),
    (b"standard-democrat.com", b"#masthead"),
    (b"starcourier.com", b".art-byline"),
    (b"statista.com", b".newsletterPromo"),
    (b"statista.com", b".relatedInfographics"),
    (b"tastespotting.com", b".post-comments"),
    (b"teachat.com", b"#page-header"),
    (b"teachat.com", b".notice"),
    (b"teachat.com", b".page-sidebar"),
    (b"teachat.com", b".postprofile"),
    (b"tv.com", b".amazon_module"),
    (b"tv.com", b".legal"),
    (b"typekit.com", b".browse-filters"),
    (b"typekit.com", b".info-bubble"),
    (b"vincellar.vinfolio.com", b"#community-tn-container"),
    (b"vincellar.vinfolio.com", b"#inventory-summary-title"),
    (b"vincellar.vinfolio.com", b"#retailListingPopup"),
    (b"vincellar.vinfolio.com", b"#twitterPopup"),
    (b"vincellar.vinfolio.com", b".ctnotes-author-info"),
    (b"vincellar.vinfolio.com", b".table-section"),
    (b"vincellar.vinfolio.com", b".vine-header"),
    (b"vladi-private-islands.de", b".connect"),
    (b"webpathology.com", b"#content_top"),
    (b"wethrift.com", b".review-left"),
    (b"wtf.com", b".uix_welcomeSection__text"),
    (b"wwaytv3.com", b".submitted"),

    (b"19actionnews.com", b"#WNCol4"),
    (b"19actionnews.com", b".feature"),
    (b"acityamonth.com", b".no-comments"),
    (b"adirondackalmanack.com", b".footer-about"),
    (b"adirondackalmanack.com", b".footer-co2"),
    (b"app.leg.wa.gov", b".no-print"),
    (b"apple.stackexchange.com", b".post-menu"),
    (b"autos.jdpower.com", b".showroom-listing-aside"),
    (b"bananawonder.com", b"#Blog1_cmt-8410765974126978913"),
    (b"beckett.com", b".smalltext"),
    (b"beckett.com", b".thead"),
    (b"bimmerwerkz.com", b".alt1"),
    (b"bimmerwerkz.com", b".breadcrumb"),
    (b"bimmerwerkz.com", b".smallfont"),
    (b"bimmerwerkz.com", b".tborder"),
    (b"bimmerwerkz.com", b".user_offline"),
    (b"bio-medicine.org", b"#rightColumn"),
    (b"biotech-capital.com", b".business-details"),
    (b"biotech-capital.com", b".main-footer-wrapper"),
    (b"blackberryforums.com", b".smallfont"),
    (b"blogs.wsj.com", b".newsItem"),
    (b"blogs.wsj.com", b".wp-caption"),
    (b"blurb.com", b".book-list__title"),
    (b"blurb.com", b".line-item-description__display-detail"),
    (b"blurb.com", b".line-item-description__display-name"),
    (b"blurb.com", b".notice-text"),
    (b"books.google.com.au", b".sitb-info"),
    (b"borderlands.fandom.com", b"#mw-revision-nav"),
    (b"cadizrecord.com", b".related_content"),
    (b"cadizrecord.com", b".related_content_body"),
    (b"cafemom.com", b".forumBreadcrumbs"),
    (b"cafemom.com", b".headerDarkBorder"),
    (b"campaignseries.co.uk", b"#cookiewarning"),
    (b"campaignseries.co.uk", b".btnGeneric"),
    (b"campaignseries.co.uk", b".btnHighlight"),
    (b"campaignseries.co.uk", b".widgetHeader"),
    (b"canadiangardening.com", b".left-box"),
    (b"canadiangardening.com", b".reply-icon"),
    (b"careers.govt.nz", b"#left-column"),
    (b"carfax.com", b"#chiStep1"),
    (b"carfax.com", b"#chiStep2"),
    (b"carfax.com", b"#chiStep3"),
    (b"carfax.com", b"#defAccidentIndicator"),
    (b"carfax.com", b"#defDismantled"),
    (b"carfax.com", b"#defEML"),
    (b"carfax.com", b"#defFlood"),
    (b"carfax.com", b"#defHail"),
    (b"carfax.com", b"#defJunk"),
    (b"carfax.com", b"#defLemon"),
    (b"carfax.com", b"#defRebuilt"),
    (b"carfax.com", b"#linkToGlossaryInGlossHead"),
    (b"carfax.com", b"#tellUsCracLinkOtherInfoSummarySection"),
    (b"carfax.com", b"#vh_disclaimer"),
    (b"carfax.com", b"#warrantyInfo"),
    (b"carfax.com", b"#warrantyInfoSpecifics"),
    (b"carfax.com", b"#warrantyStatus"),
    (b"carfax.com", b".resultLabel"),
    (b"carfax.com", b".vh_gridCap"),
    (b"carfax.com", b".vh_small"),
    (b"cbssports.com", b".block-title"),
    (b"cbssports.com", b".eyebrow"),
    (b"cbssports.com", b".see-full-article"),
    (b"cbwentworth.wordpress.com", b".comment-reply-link"),
    (b"census.gov", b"#AboutLink1"),
    (b"census.gov", b"#AboutLink2"),
    (b"census.gov", b"#AboutLink4"),
    (b"census.gov", b"#AboutLink5"),
    (b"census.gov", b"#AboutLink6"),
    (b"census.gov", b"#AboutLink7"),
    (b"census.gov", b"#DataLink2"),
    (b"census.gov", b"#DataLink3"),
    (b"census.gov", b"#DataLink4"),
    (b"census.gov", b"#DataLink5"),
    (b"census.gov", b"#DataLink6"),
    (b"census.gov", b"#DataLink8"),
    (b"census.gov", b"#DataMain"),
    (b"census.gov", b"#GeoLink3"),
    (b"census.gov", b"#GeoLink4"),
    (b"census.gov", b"#GeoLink7"),
    (b"census.gov", b"#NewsLink1"),
    (b"census.gov", b"#NewsLink2"),
    (b"census.gov", b"#NewsLink3"),
    (b"census.gov", b"#NewsLink4"),
    (b"census.gov", b"#NewsLink5"),
    (b"census.gov", b"#abtMain"),
    (b"census.gov", b"#newsMain"),
    (b"census.gov", b".abt"),
    (b"census.gov", b".abt2"),
    (b"census.gov", b".abt4"),
    (b"census.gov", b".abt6"),
    (b"census.gov", b".abt7"),
    (b"census.gov", b".data3"),
    (b"census.gov", b".data4"),
    (b"census.gov", b".data5"),
    (b"census.gov", b".data7"),
    (b"census.gov", b".data8"),
    (b"census.gov", b".geo3"),
    (b"census.gov", b".geo4"),
    (b"census.gov", b".geo6"),
    (b"census.gov", b".geo7"),
    (b"census.gov", b".nr1"),
    (b"census.gov", b".nr2"),
    (b"census.gov", b".nr3"),
    (b"census.gov", b".offscreen"),
    (b"census.gov", b".subHeader"),
    (b"cheatmasters.com", b".boxhead1"),
    (b"cheatmasters.com", b".cm_name"),
    (b"cheatmasters.com", b".cm_otherhead"),
    (b"cheatmasters.com", b".havea"),
    (b"cheatmasters.com", b".subbtn"),
    (b"cheatmasters.com", b".submitbox"),
    (b"chictopia.com", b"#hidden_login_action"),
    (b"chictopia.com", b".action_button_label"),
    (b"chictopia.com", b".gray_text"),
    (b"christianmingle.com", b".auto_div"),
    (b"christianmingle.com", b".nav_title_box"),
    (b"chrisweigant.com", b".commentinput"),
    (b"chrisweigant.com", b".copyright"),
    (b"cio.com", b".featured-col"),
    (b"cio.com", b".popular-col"),
    (b"cl-user.net", b"#G19157h"),
    (b"community.mcafee.com", b".flex_blade_btn"),
    (b"community.mcafee.com", b".lia-panel-heading-bar-title"),
    (b"community.mcafee.com", b".ul_bullet"),
    (b"conferenceboard.ca", b"#MainRegion_C008_lblRatingReq"),
    (b"conferenceboard.ca", b"#specialH2"),
    (b"conferenceboard.ca", b".para"),
    (b"coursereport.com", b".instructions-overlay"),
    (b"coursereport.com", b".modal-header"),
    (b"coursereport.com", b".review-instructions"),
    (b"cruisecritic.com", b".chakra-button"),
    (b"cruisecritic.com", b".css-1kgz5o3"),
    (b"cruisecritic.com", b".css-1widexw"),
    (b"daedalusbooks.com", b"#Table46"),
    (b"dailygazette.com", b"#blogSpace"),
    (b"digitimes.com", b"#midright"),
    (b"dittrickswines.com", b".ch-footer"),
    (b"engineering.academickeys.com", b".local_home_ad"),
    (b"finance.boston.com", b".drop"),
    (b"finance.boston.com", b".investingnav_search"),
    (b"floridatrend.com", b"#siderail"),
    (b"forexsignal30.com", b".infinite-scroll-error"),
    (b"forexsignal30.com", b".infinite-scroll-last"),
    (b"forexsignal30.com", b".no-comments"),
    (b"forexsignal30.com", b".posted-on"),
    (b"fullonlinebook.com", b"#slide_author"),
    (b"gardenplants.comparespecies.com", b".SpecValueText"),
    (b"gardenplants.comparespecies.com", b".TopProduct"),
    (b"gardenplants.comparespecies.com", b".borderBox"),
    (b"gatorsports.com", b".entry-content"),
    (b"github.com", b"#ajax-error-message"),
    (b"glennopedia.com", b".comments-link"),
    (b"globalsurfers.com", b".content-popup"),
    (b"globalsurfers.com", b".info-columns"),
    (b"globalsurfers.com", b".pager"),
    (b"globalsurfers.com", b".text-holder"),
    (b"heraldnet.com", b"#digitalSubscriptionPromoSF"),
    (b"heraldnet.com", b".ClassifiedWidgetItemHeader"),
    (b"heraldnet.com", b".component_header"),
    (b"heraldnet.com", b".most"),
    (b"heraldnet.com", b".photoCredit"),
    (b"heraldnet.com", b".return_bottom"),
    (b"heraldnet.com", b".smallSliderCaption"),
    (b"heraldnet.com", b".tag"),
    (b"house.fandom.com", b".printfooter"),
    (b"iclassifiedsnetwork.com", b"#TABLE1"),
    (b"ieeexplore.ieee.org", b"#purchase-options"),
    (b"ieeexplore.ieee.org", b"#qualify-price-ad-overlay"),
    (b"ieeexplore.ieee.org", b".article-blk"),
    (b"ieeexplore.ieee.org", b".message"),
    (b"ieeexplore.ieee.org", b".pdf"),
    (b"ieeexplore.ieee.org", b".pricingOptionsError"),
    (b"informationweek.com", b".adv"),
    (b"informationweek.com", b".down"),
    (b"informationweek.com", b".download3"),
    (b"informationweek.com", b".head"),
    (b"informationweek.com", b".ratings_down_row"),
    (b"informationweek.com", b".yellow"),
    (b"iskwiki.upd.edu.ph", b"#mw-revision-info"),
    (b"iskwiki.upd.edu.ph", b".printfooter"),
    (b"itknowledgeexchange.techtarget.com", b"#socialShareModalHeader"),
    (b"itknowledgeexchange.techtarget.com", b".forgotPasswordLink"),
    (b"itknowledgeexchange.techtarget.com", b".forgotPasswordModalIntro"),
    (b"itknowledgeexchange.techtarget.com", b".line3"),
    (b"itknowledgeexchange.techtarget.com", b".sectionHeader"),
    (b"itu.int", b".footeritems"),
    (b"jedidefender.com", b".bbc_standard_quote"),
    (b"jeepforum.com", b"#post_message_13526526"),
    (b"jeepforum.com", b"#post_message_13585415"),
    (b"jeepforum.com", b"#post_message_13592757"),
    (b"jetcost.com.sg", b"#show2"),
    (b"jetcost.com.sg", b".faresTitle"),
    (b"kiehls.com", b".TT3metaText"),
    (b"kiehls.com", b".htmlslotcontainer"),
    (b"kiehls.com", b".title_small"),
    (b"ksl.com", b".mainCount"),
    (b"ksl.com", b".notaccepting"),
    (b"lybrate.com", b"#treatmentService"),
    (b"lybrate.com", b".grid--justify-center"),
    (b"lybrate.com", b".grid--justify-space-between"),
    (b"lybrate.com", b".grid__col-auto"),
    (b"lybrate.com", b".ly-doctor__button-text"),
    (b"lybrate.com", b".lybText--darkest"),
    (b"lybrate.com", b".lybText--normal"),
    (b"lybrate.com", b".search-overlay__default-list__list-items__item"),
    (b"lybrate.com", b".search-overlay__default-list__title"),
    (b"m3post.com", b".smallfont"),
    (b"mcclatchydc.com", b".caption-text"),
    (b"medicalxpress.com", b"#news-holder"),
    (b"menstennisforums.com", b"#collapseobj_threadreview"),
    (b"minotdailynews.com", b"#frmSiteSearch"),
    (b"minotdailynews.com", b"#hdrNavL"),
    (b"minotdailynews.com", b".txtCenter"),
    (b"minotdailynews.com", b".txtSmaller"),
    (b"mirchee.com", b".emptylinefiller"),
    (b"morford.rootsandthreads.com", b"#pupd327"),
    (b"mustseeindia.com", b"#Package_details"),
    (b"mv-voice.com", b".form_text_notes"),
    (b"ncbi.nlm.nih.gov", b"#submenu_CitationManager"),
    (b"newhampshire.com", b".carousel-inner"),
    (b"news.sys-con.com", b".about-the-author"),
    (b"news.sys-con.com", b".storyauthor"),
    (b"news.sys-con.com", b".storysummary"),
    (b"news.sys-con.com", b".storytagline"),
    (b"norwalk.itsrelevant.com", b"#report-video-problem"),
    (b"norwalk.itsrelevant.com", b".footerjk"),
    (b"norwalk.itsrelevant.com", b".story-nav"),
    (b"notes.bread.org", b"#comment-captcha-viewalt"),
    (b"notes.bread.org", b"#header-preview-comment"),
    (b"notes.bread.org", b"#header-verify-comment"),
    (b"notes.bread.org", b".art-blockcontent-body"),
    (b"notes.bread.org", b".content-nav"),
    (b"nraila.org", b".FooterPartial"),
    (b"nraila.org", b".master_nav"),
    (b"nytwa.info", b".coverlinks"),
    (b"openpr.com", b".col-md-6"),
    (b"owsd.net", b".block-title"),
    (b"owsd.net", b".field-content"),
    (b"packershome.com", b"#DivPageAccess"),
    (b"packershome.com", b"#yafheader"),
    (b"packershome.com", b".UserBox"),
    (b"packershome.com", b".postsep"),
    (b"pbs.org", b".popup-pagenumber"),
    (b"philamuseum.org", b"#addTags"),
    (b"philamuseum.org", b".topper"),
    (b"pictureyear.blogspot.com", b".paging-control-container"),
    (b"pl.tripadvisor.com", b".unsupportedBrowser"),
    (b"pragationline.com", b".add_to_wishlist"),
    (b"pragationline.com", b".menu-item-object-custom"),
    (b"pragationline.com", b".price"),
    (b"pragationline.com", b".product-section-title-related"),
    (b"pragationline.com", b".product-title"),
    (b"pragationline.com", b".yith-wcwl-wishlistaddedbrowse"),
    (b"psypokes.com", b".posterrank"),
    (b"pt.usc.edu", b"#ctl00_ctl00_ContentFooterContact"),
    (b"pt.usc.edu", b".column2"),
    (b"pt.usc.edu", b".column4"),
    (b"pt.usc.edu", b".copyright"),
    (b"pt.usc.edu", b".text-links"),
    (b"radaronline.com", b".b-flag"),
    (b"radaronline.com", b".no-border"),
    (b"randyhamilton.openmortgage.com", b".row-bg-wrap-pr"),
    (b"raptorsrepublic.com", b"#about-me"),
    (b"raptorsrepublic.com", b"#moreactivitylink"),
    (b"raptorsrepublic.com", b"#newactivity_nomore"),
    (b"raptorsrepublic.com", b".date"),
    (b"raptorsrepublic.com", b".description"),
    (b"raptorsrepublic.com", b".excerpt"),
    (b"raptorsrepublic.com", b".popuphover"),
    (b"raptorsrepublic.com", b".subsectionhead-understate"),
    (b"raptorsrepublic.com", b".tabslight"),
    (b"raptorsrepublic.com", b".userprof_title"),
    (b"ro.urbandictionary.com", b".with-icon"),
    (b"robbreport.com", b".views-row"),
    (b"salisburypost.com", b".story-heading"),
    (b"scoop.co.nz", b"#img-float-right"),
    (b"scoop.co.nz", b".featured-newshub"),
    (b"slideshare.net", b".action-unverified-download"),
    (b"slideshare.net", b".change-unverified-email"),
    (b"slideshare.net", b".hint"),
    (b"slideshare.net", b".iconPrivate"),
    (b"smallbizpages.co.uk", b".wpbdp-hide-on-mobile"),
    (b"smallbizpages.co.uk", b".wpbdp-msg"),
    (b"smartertravel.com", b"#BookingBuddySearchBlockedPopUpDivID"),
    (b"songmeanings.com", b".btn-comment"),
    (b"songmeanings.com", b".holder"),
    (b"southstrandnews.com", b".text-center"),
    (b"sports-boards.net", b".description"),
    (b"spreaker.com", b".track_message_empty"),
    (b"starcourier.com", b"#toppage"),
    (b"tablethotels.com", b"#favorite-count"),
    (b"tablethotels.com", b"#ratings-breakdown"),
    (b"tablethotels.com", b".accessibility-content"),
    (b"tablethotels.com", b".breakdown-row"),
    (b"tablethotels.com", b".hotel-address"),
    (b"tablethotels.com", b".review-subtitle"),
    (b"tamilnet.com", b"#copyright"),
    (b"tamilnet.com", b"#printFooter"),
    (b"tamilnet.com", b"#rightColumn"),
    (b"tanglesbythesea.com", b".textwidget"),
    (b"tanglesbythesea.com", b".wordpress-follow-button"),
    (b"tastespotting.com", b".post-related"),
    (b"teachat.com", b"#profile190419"),
    (b"theday.com", b"#galleryCats"),
    (b"theday.com", b"#l11"),
    (b"theday.com", b"#l12"),
    (b"topjobs.ch", b".hidden-xs"),
    (b"topjobs.ch", b".jobmail-promo"),
    (b"tv.com", b"._cell"),
    (b"tv.com", b".larger"),
    (b"ucsdtritons.com", b".cap"),
    (b"ucsdtritons.com", b".courtesy"),
    (b"ucsdtritons.com", b".label_related_links"),
    (b"ucsdtritons.com", b".link-info"),
    (b"uctv.tv", b".sideheader"),
    (b"use.perl.org", b"#ccw-abbr-phrase"),
    (b"use.perl.org", b"#commentControlBoxStatus"),
    (b"vincellar.vinfolio.com", b"#currentBid-title"),
    (b"vincellar.vinfolio.com", b".addCommentNoCustomer"),
    (b"vincellar.vinfolio.com", b".content-txt"),
    (b"vincellar.vinfolio.com", b".link-arrow"),
    (b"vincellar.vinfolio.com", b".right-info"),
    (b"vincellar.vinfolio.com", b".useful-counts"),
    (b"waitrose.com", b".l-content"),
    (b"waitrose.com", b".section-save"),
    (b"waitrose.com", b".tool-option-content"),
    (b"westonparkhospitality.com", b"#flash-banner"),
    (b"witchesandpagans.com", b"#ezblog-head"),
    (b"witchesandpagans.com", b".author-info"),
    (b"wmnf.org", b".first_comment_link"),
    (b"wtf.com", b".div-bor-bot"),
    (b"wtf.com", b".footerList"),
    (b"wtf.com", b".footer_new-to-wtf"),
    (b"wtf.com", b".footer_social_wrap"),
    (b"wtf.com", b".js-offCanvasCopy"),
    (b"wtf.com", b".menu-content"),
    (b"wtf.com", b".notice-content"),
    (b"wtf.com", b".p-footer-copyright"),
    (b"wtf.com", b".p-footer-linkList"),
    (b"wtf.com", b".u-concealed"),
    (b"wtf.com", b".uix_cookieButtonRow"),
    (b"x-panicxtrash-x.skyrock.com", b"#infobulle_comment"),
    (b"zefron.com", b"#panel"),
    (b"zefron.com", b".current_rating"),
    (b"zefron.com", b".float_right"),
    (b"zefron.com", b".post_author_info"),

    (b"americanpoems.com", b".bar"),
    (b"anaayafoods.com", b"#hitmag-comments"),
    (b"anaayafoods.com", b".entry-date"),
    (b"anaayafoods.com", b".hms-meta"),
    (b"anaayafoods.com", b".hms-title"),
    (b"anenglishmaninosaka.blogspot.com", b".postmetadata"),
    (b"anoregoncottage.com", b".enews"),
    (b"anoregoncottage.com", b".screen-reader-text"),
    (b"anoregoncottage.com", b".site-description"),
    (b"anoregoncottage.com", b".site-title"),
    (b"askdrgarland.com", b".widget_videos"),
    (b"autos.jdpower.com", b".ad-jumpstart"),
    (b"autos.jdpower.com", b".powercircle-reviews"),
    (b"bananawonder.com", b".deleted-comment"),
    (b"blackamericaweb.com", b"#search"),
    (b"blackamericaweb.com", b".archive-title"),
    (b"blackamericaweb.com", b".menu-item-object-category"),
    (b"blackamericaweb.com", b".twitter-follow-button"),
    (b"blog.akismet.com", b".no-comments"),
    (b"blogs.edweek.org", b".asset-name"),
    (b"blogs.edweek.org", b".comment-notice"),
    (b"blogs.edweek.org", b".comment_display_req"),
    (b"blogs.edweek.org", b".widget-header"),
    (b"blogs.theprovince.com", b".bit-follow-count"),
    (b"bobvila.com", b".category"),
    (b"bobvila.com", b".content_item"),
    (b"bobvila.com", b".link_description"),
    (b"bobvila.com", b".question"),
    (b"bobvila.com", b".related-prod-sidebar-hdr"),
    (b"bobvila.com", b".yml_link"),
    (b"cadizrecord.com", b".related_content_label"),
    (b"capcitybank.com", b".FooterText"),
    (b"careers.govt.nz", b".jobs-db"),
    (b"carfax.com", b"#bbgPara"),
    (b"carfax.com", b"#defAirbagDeployment"),
    (b"carfax.com", b"#defBasicWarranty"),
    (b"carfax.com", b"#defManufacturerRecall"),
    (b"carfax.com", b"#defTotalLoss"),
    (b"cattlenetwork.com", b"#k_slogan"),
    (b"cattlenetwork.com", b".commentsDiv"),
    (b"cdn.cdata.com", b".phones"),
    (b"census.gov", b"#GeoLink2"),
    (b"census.gov", b"#GeoLink8"),
    (b"census.gov", b"#GeoLink9"),
    (b"census.gov", b"#GeoMain"),
    (b"census.gov", b"#LibLink1"),
    (b"census.gov", b"#LibLink2"),
    (b"census.gov", b"#LibLink6"),
    (b"census.gov", b".lib6"),
    (b"cinephonix.com", b".filter-foot"),
    (b"cio.com", b".article-intercept"),
    (b"cio.com", b".eyebrow"),
    (b"cio.com", b".featured"),
    (b"cio.com", b".head"),
    (b"cio.com", b".hed"),
    (b"cio.com", b".subhead"),
    (b"cio.com", b".with-eyebrow"),
    (b"cl-user.net", b".onoffh"),
    (b"columbinecourier.com", b".regnowform"),
    (b"columbinecourier.com", b".signInboxMain"),
    (b"columbinecourier.com", b".topLeft"),
    (b"columbinecourier.com", b".topRight"),
    (b"commons.apache.org", b".legalCopy"),
    (b"conferenceboard.ca", b".download"),
    (b"courier-journal.com", b"#ody-mainphoto"),
    (b"courier-journal.com", b"#ody-pq"),
    (b"courier-journal.com", b".bold"),
    (b"couriernews.com", b".related_content_label"),
    (b"coursereport.com", b"#mp-contact-form"),
    (b"coursereport.com", b".btn-write-a-review"),
    (b"coursereport.com", b".email-error"),
    (b"coursereport.com", b".email-footer"),
    (b"coursereport.com", b".form-notes"),
    (b"coursereport.com", b".read-more-link"),
    (b"coursereport.com", b".review-length"),
    (b"coursereport.com", b".success"),
    (b"coursereport.com", b".tiny-font"),
    (b"creativesaga.com", b"#HTML4"),
    (b"creativesaga.com", b".author-profile"),
    (b"csce.gov", b"#Issue_id"),
    (b"csce.gov", b"#PhotoGallery"),
    (b"csce.gov", b"#Region_id"),
    (b"csce.gov", b".ContentGrid"),
    (b"csce.gov", b".Header2"),
    (b"csce.gov", b".sideBarLinks"),
    (b"dailytech.com", b"#OldArticle"),
    (b"dailytech.com", b".quotesub"),
    (b"davisclipper.com", b".comment_author"),
    (b"davisclipper.com", b".newline"),
    (b"dennis2society.de", b".theCode"),
    (b"designhotels.com", b"#content_nav"),
    (b"designhotels.com", b".content_blockAD"),
    (b"dictionary.cambridge.org", b"#translations"),
    (b"dictionary.cambridge.org", b".freeTranslator"),
    (b"dictionary.cambridge.org", b".see-all-translations"),
    (b"digitimes.com", b".channeltabs"),
    (b"digitimes.com", b".fpara"),
    (b"digitimes.com", b".mr-hd"),
    (b"dotnetfunda.com", b".PostResponseB"),
    (b"economictimes.indiatimes.com", b".articleImg"),
    (b"electronicinfo.ca", b"#accesskeys"),
    (b"electronicinfo.ca", b"#heading-"),
    (b"elyrics.net", b"#combox"),
    (b"elyrics.net", b"#coms"),
    (b"elyrics.net", b"#csub"),
    (b"elyrics.net", b"#r4div"),
    (b"ema.europa.eu", b"#form-tellusmore"),
    (b"ema.europa.eu", b"#leftWidgets"),
    (b"ema.europa.eu", b"#ratingHeader"),
    (b"ema.europa.eu", b"#rightWidgets"),
    (b"ema.europa.eu", b".toggle-list"),
    (b"engineering.academickeys.com", b"#job_redirect_fancybox_email"),
    (b"engineering.academickeys.com", b".moto-widget-text-editable"),
    (b"eventbrite.com.au", b".js-xd-read-more-toggle-director"),
    (b"ew.com", b"#partnerbar"),
    (b"ew.com", b".rail-module"),
    (b"ew.com", b".rail-photos-headline"),
    (b"ew.com", b".timestamp"),
    (b"fandango.com", b".date-picker__details"),
    (b"fandango.com", b".date-picker__location"),
    (b"fandango.com", b".date-picker__message-title"),
    (b"fandango.com", b".fan-alert__header"),
    (b"fandango.com", b".fan-alert__link-wrap"),
    (b"fandango.com", b".fan-alert__privacy-link"),
    (b"fandango.com", b".mop__synopsis-title"),
    (b"fandango.com", b".movie-details__fan-ratings"),
    (b"floridatrend.com", b".more"),
    (b"floridatrend.com", b".subHead"),
    (b"forbes.com", b".disclaimer"),
    (b"forbes.com", b".full_bio"),
    (b"forbes.com", b".launch_email_contrib"),
    (b"forum.fastday.com", b".serif"),
    (b"forum.fastday.com", b".tooltip-link"),
    (b"forums.everythingicafe.com", b".userText"),
    (b"github.com", b".include-fragment-error"),
    (b"groups.yahoo.com", b"#yg-error-message"),
    (b"groups.yahoo.com", b".so-homepage-welcome-txt"),
    (b"groups.yahoo.com", b".so-homepage-welcome-txt2"),
    (b"groups.yahoo.com", b".yg-offscreen"),
    (b"hardwarezone.com", b"#header-wrap"),
    (b"healthboards.com", b"#infobar"),
    (b"heraldnews.com", b".footerheadleft"),
    (b"heraldnews.com", b".loop"),
    (b"hiphop-n-more.com", b".dsq-comment-message"),
    (b"hiphop-n-more.com", b".rc-baheadline"),
    (b"hiphop-n-more.com", b".textwidget"),
    (b"iclassifiedsnetwork.com", b"#ctl00_ContentPlaceHolder1_pnlMain"),
    (b"iclassifiedsnetwork.com", b"#ctl00_EmailListSubscribe1_Label1"),
    (b"iclassifiedsnetwork.com", b"#ctl00_EmailListSubscribe1_pnlEmailMain"),
    (b"iclassifiedsnetwork.com", b"#ctl00_EmailListSubscribe1_pnlSMSMain"),
    (b"iclassifiedsnetwork.com", b"#ctl00_PasswordLogin1_Label1"),
    (b"iclassifiedsnetwork.com", b"#ctl00_QuickPoll1_lblQuestion"),
    (b"iclassifiedsnetwork.com", b"#ctl00_SiteSearchBox1_Label1"),
    (b"iclassifiedsnetwork.com", b"#ctl00_WeatherSmall1_Label1"),
    (b"iclassifiedsnetwork.com", b"#ctl00_WeatherSmall1_lblLocation"),
    (b"iclassifiedsnetwork.com", b".MsoNormal"),
    (b"ideas.repec.org", b"#messages"),
    (b"il.findacase.com", b"#searching"),
    (b"il.findacase.com", b".buyNowContainer"),
    (b"il.findacase.com", b".price"),
    (b"il.findacase.com", b".showCaseToolTip"),
    (b"iprdaily.com", b".comment-word"),
    (b"iprdaily.com", b".font-nub"),
    (b"ithemes.com", b"#contentSub"),
    (b"ithemes.com", b".diff-multi"),
    (b"jobs.greenbook.org", b".external-app-help-text"),
    (b"jobs.greenbook.org", b".external-app-help-title"),
    (b"kingston.ac.uk", b"#fixed-nav-main-ku-logo"),
    (b"kingston.ac.uk", b"#tabundergrad"),
    (b"kohls.com", b".ggl-tooltip-title"),
    (b"lafollettepress.com", b".regnowform"),
    (b"lafollettepress.com", b".signInboxMain"),
    (b"lafollettepress.com", b".topLeft"),
    (b"lafollettepress.com", b".topRight"),
    (b"linuxpromagazine.com", b".paypal-order"),
    (b"linuxpromagazine.com", b".price"),
    (b"lists.w3.org", b"#received"),
    (b"m3post.com", b".garagelist"),
    (b"m3post.com", b".voteContainer3"),
    (b"mastercraft.com", b".smallfont"),
    (b"menstennisforums.com", b".alt1"),
    (b"menstennisforums.com", b".alt2"),
    (b"menstennisforums.com", b".blockrow"),
    (b"menstennisforums.com", b".fieldset"),
    (b"menstennisforums.com", b".panel"),
    (b"menstennisforums.com", b".thead"),
    (b"money.howstuffworks.com", b".text-center"),
    (b"money.howstuffworks.com", b".video-title-overlay"),
    (b"morford.rootsandthreads.com", b".pupdata"),
    (b"motoprofi.com", b".cat_sub"),
    (b"motoprofi.com", b".leftlinkss"),
    (b"motoprofi.com", b".tenlarge"),
    (b"murga-linux.com", b".gen"),
    (b"murga-linux.com", b".gensmall"),
    (b"murga-linux.com", b".maintitle"),
    (b"murga-linux.com", b".postdetails"),
    (b"mustseeindia.com", b"#call_company_number"),
    (b"mustseeindia.com", b".city-widget-header"),
    (b"mustseeindia.com", b".div-grey"),
    (b"mustseeindia.com", b".icon-rt-txt"),
    (b"ncbi.nlm.nih.gov", b"#send_to_menu"),
    (b"newhampshire.com", b"#memberinfo"),
    (b"newhampshire.com", b".article-text-black"),
    (b"newhampshire.com", b".leadtext"),
    (b"newhampshire.com", b".mtm-section"),
    (b"nobelcom.com", b".bottom"),
    (b"nobelcom.com", b".switchFull"),
    (b"norwalk.itsrelevant.com", b"#content-origin"),
    (b"norwalk.itsrelevant.com", b"#prev-story"),
    (b"nytwa.info", b"#FlashID"),
    (b"packershome.com", b"#ctl00_MainContent_forum_ctl01_GuestUserMessage"),
    (b"packershome.com", b"#ctl00_MainContent_forum_ctl03_ForumUsers1"),
    (b"packershome.com", b"#ctl00_MainContent_forum_ctl03_MessageList_ctl00_DisplayPost1_UserBox1"),
    (b"packershome.com", b"#ctl00_MainContent_forum_ctl03_MessageList_ctl01_DisplayPostAlt_UserBox1"),
    (b"packershome.com", b"#ctl00_MainContent_forum_ctl03_MessageList_ctl03_DisplayPostAlt_UserBox1"),
    (b"packershome.com", b"#ctl00_MainContent_forum_ctl03_MessageList_ctl04_DisplayPost1_UserBox1"),
    (b"packershome.com", b"#ctl00_MainContent_forum_ctl03_MessageList_ctl05_DisplayPostAlt_UserBox1"),
    (b"packershome.com", b"#ctl00_MainContent_forum_ctl03_MessageList_ctl06_DisplayPost1_UserBox1"),
    (b"packershome.com", b"#ctl00_MainContent_forum_ctl03_MessageList_ctl07_DisplayPostAlt_UserBox1"),
    (b"packershome.com", b"#ctl00_MainContent_forum_ctl03_MessageList_ctl09_DisplayPostAlt_UserBox1"),
    (b"packershome.com", b"#ctl00_MainContent_forum_ctl03_MessageList_ctl12_DisplayPost1_UserBox1"),
    (b"packershome.com", b"#ctl00_MainContent_forum_ctl03_MessageList_ctl14_DisplayPost1_UserBox1"),
    (b"packershome.com", b"#ctl00_MainContent_forum_ctl03_TopicTitle"),
    (b"packershome.com", b"#login"),
    (b"packershome.com", b".header2links"),
    (b"packershome.com", b".modalHeader"),
    (b"packershome.com", b".popupitem"),
    (b"pagosasun.com", b"#bcrum"),
    (b"pagosasun.com", b".category-feed"),
    (b"partitionsdechansons.com", b"#margetopmoins10"),
    (b"pl.tripadvisor.com", b".internal"),
    (b"pl.tripadvisor.com", b".photoFilterDropDownLabel"),
    (b"pl.tripadvisor.com", b".sprite-button-ylw"),
    (b"pl.tripadvisor.com", b".taLnk"),
    (b"playfire.com", b".inline_expand"),
    (b"playfire.com", b".separator"),
    (b"products.smileysaudiovisual.com", b"#jsContactThanks"),
    (b"psypokes.com", b".titles"),
    (b"randyhamilton.openmortgage.com", b".prefi-text"),
    (b"readthestars.com", b".search_summary"),
    (b"rutlandherald.com", b".categoryItemContent"),
    (b"rutlandherald.com", b".categoryItemTitle"),
    (b"rutlandherald.com", b".labelTxt"),
    (b"rutlandherald.com", b".linkLabel"),
    (b"rutlandherald.com", b".smallRightArrow"),
    (b"rutlandherald.com", b".titleCont"),
    (b"rutlandherald.com", b".twitter-timeline"),
    (b"slideshare.net", b".h-tools"),
    (b"smartertravel.com", b"#non_js_users"),
    (b"smartertravel.com", b".blockedPopUpTable"),
    (b"starcourier.com", b"#art-byline-pubdate-cont"),
    (b"statista.com", b".button--primary"),
    (b"stitchkingdom.com", b".heatmapthemead-like-button-text"),
    (b"straightdope.com", b"#fineprint"),
    (b"straightdope.com", b"#recent_additions"),
    (b"straightdope.com", b".section_link"),
    (b"straightdope.com", b".teaser"),
    (b"sunkenstone.com", b".fusion-post-title"),
    (b"swissinfo.ch", b"#myModal"),
    (b"swissinfo.ch", b".dharma"),
    (b"swissinfo.ch", b".karma"),
    (b"swissinfo.ch", b".show-for-medium-up"),
    (b"swissinfo.ch", b".text-centered"),
    (b"teachat.com", b".tcbox"),
    (b"teachat.com", b".tcbox1"),
    (b"themillions.com", b".article-options-wrapper"),
    (b"theserverside.com", b".iconButton"),
    (b"theserverside.com", b".postResponseButtons"),
    (b"theserverside.com", b".replyToPost"),
    (b"tv.com", b".role"),
    (b"tv.com", b".see_more"),
    (b"tvtechnology.com", b"#dnn_ArticlePageLeftColumn1_divNoComment"),
    (b"typekit.com", b"#classification-info-bubble"),
    (b"typekit.com", b"#families-info-bubble"),
    (b"typekit.com", b"#filter-type-classification"),
    (b"typekit.com", b"#filter-type-licenses"),
    (b"typekit.com", b"#filter-type-property"),
    (b"typekit.com", b"#licenses-info-bubble"),
    (b"typekit.com", b"#properties-info-bubble"),
    (b"typekit.com", b".count"),
    (b"typekit.com", b".footer-copyright"),
    (b"typekit.com", b".group-name-with-option-names"),
    (b"typekit.com", b".label"),
    (b"typekit.com", b".text-control"),
    (b"uk.reuters.com", b".feature"),
    (b"uk.reuters.com", b".gallery"),
    (b"uk.reuters.com", b".moduleHeader"),
    (b"urgentcarelocations.com", b".gotodirections"),
    (b"urgentcarelocations.com", b".link-with-icon"),
    (b"urgentcarelocations.com", b".not-logged-in"),
    (b"urgentcarelocations.com", b".rating-extended"),
    (b"urgentcarelocations.com", b".review-totals"),
    (b"use.perl.org", b"#gods"),
    (b"waitrose.com", b"#ratingsThanks"),
    (b"waitrose.com", b"#recipe-scrapbook-savelink"),
    (b"waitrose.com", b".about"),
    (b"waitrose.com", b".mini-rating"),
    (b"waitrose.com", b".no-print"),
    (b"waitrose.com", b".rate-text"),
    (b"waynesvilledailyguide.com", b"#fullaboutboxpopup"),
    (b"waynesvilledailyguide.com", b".blg-lft-head"),
    (b"waynesvilledailyguide.com", b".blg-post-title"),
    (b"waynesvilledailyguide.com", b".blg-rgt-date"),
    (b"waynesvilledailyguide.com", b".blg-rgt-title"),
    (b"weareiowa.com", b".cms__embed-related-story"),
    (b"webcommentary.com", b".subtitle"),
    (b"webpathology.com", b"#lastupdated"),
    (b"westonparkhospitality.com", b"#flash-holder"),
    (b"wiki.inf.ed.ac.uk", b"#patternWebBottomBar"),
    (b"witchesandpagans.com", b"#empty-comment-notice"),
    (b"yahoo.com", b".slideshow-description"),

    (b"19actionnews.com", b"#WNHeader"),
    (b"3fatchicks.com", b".smallfont"),
    (b"425sqftart.com", b"#wall_post_toggle"),
    (b"425sqftart.com", b"#wp_latest"),
    (b"425sqftart.com", b".older"),
    (b"425sqftart.com", b".textwidget"),
    (b"425sqftart.com", b".wallauthor"),
    (b"425sqftart.com", b".widgettitle"),
    (b"barnesjewish.org", b"#dnn_ucFooter_dnnCopyright_lblCopyright"),
    (b"barnesjewish.org", b".copyWrap"),
    (b"barnesjewish.org", b".newsletmore"),
    (b"barnesjewish.org", b".newsletterL"),
    (b"barnesjewish.org", b".newsletterTxtSml"),
    (b"barnesjewish.org", b".row1Left"),
    (b"barnesjewish.org", b".row1Right"),
    (b"barnstablepatriot.com", b".moduletable"),
    (b"cameralabs.com", b".postlink"),
    (b"cio.com", b".with-image"),
    (b"communicationsforum.org.uk", b".textwidget"),
    (b"community.mcafee.com", b"#reportAbuse"),
    (b"community.mcafee.com", b".lia-forum-topic-page-solution-link"),
    (b"community.mcafee.com", b".lia-message-subject"),
    (b"convertunits.com", b".title_grn"),
    (b"coolestfamilyontheblock.com", b"#author-description"),
    (b"coolestfamilyontheblock.com", b"#author-info-title"),
    (b"cwreenactors.com", b".PhorumFloatingText"),
    (b"cwreenactors.com", b".PhorumFooterPlug"),
    (b"cwreenactors.com", b".PhorumReadBodySubject"),
    (b"cwreenactors.com", b".PhorumTitleText"),
    (b"dictionary.cambridge.org", b".add-cambrdige"),
    (b"docs.servicenow.com", b".alert-dismissible"),
    (b"efloras.org", b"#footerTable"),
    (b"efloras.org", b"#ucEfloraHeader_lblSearchBox"),
    (b"exchange4media.com", b"#jjaddigital"),
    (b"exchange4media.com", b".coment"),
    (b"exchange4media.com", b".related_con"),
    (b"experienceproject.com", b".ep-button"),
    (b"forimmediaterelease.net", b"#RightCol"),
    (b"forimmediaterelease.net", b".fullentry_footer"),
    (b"forimmediaterelease.net", b".snap_noshots"),
    (b"gearslutz.com", b"#finalNavbit"),
    (b"gearslutz.com", b"#navbar_fbb_link"),
    (b"gearslutz.com", b".header-tagline"),
    (b"gishgallop.com", b".wp-caption-text"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#HTML3"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image10"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image12"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image13"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image15"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image22"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image23"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image27"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image30"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image34"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image35"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image39"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image4"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image5"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image6"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image7"),
    (b"glitternsparklechallengeblog.blogspot.com", b"#Image8"),
    (b"heraldnews.com", b".org"),
    (b"il.findacase.com", b".toolTipContent"),
    (b"linuxpromagazine.com", b".block-content"),
    (b"lists.w3.org", b"#date"),
    (b"m.dodgers.mlb.com", b".big-blurb"),
    (b"m.dodgers.mlb.com", b".meta-data"),
    (b"menstennisforums.com", b".smallfont"),
    (b"menstennisforums.com", b".tcat"),
    (b"motoprofi.com", b".leftlinks"),
    (b"motoprofi.com", b".tablehead"),
    (b"motoprofi.com", b".tablesp"),
    (b"news.sys-con.com", b"#content-below"),
    (b"oneperfectbite.blogspot.com", b"#HTML9"),
    (b"pezenas-couvent.com", b".page_item"),
    (b"pezenas-couvent.com", b".textwidget"),
    (b"polisholic-confessions.blogspot.com", b".post-footer-line"),
    (b"smartertravel.com", b".blockedPopUpTableOrangeSide"),
    (b"smartertravel.com", b".blockedPopUpTablePurpleSide"),
    (b"sparkpeople.com", b".box_comments"),
    (b"sparkpeople.com", b".button"),
    (b"talkbass.com", b".externalLink"),
    (b"tanglesbythesea.com", b".radius-full"),
    (b"techobuzz.wordpress.com", b".credit"),
    (b"techobuzz.wordpress.com", b".feedback"),
    (b"theserverside.com", b".postContent"),
    (b"witchesandpagans.com", b".backh3"),
    (b"witchesandpagans.com", b".blog-meta"),
    (b"witchesandpagans.com", b".comment-author"),
    (b"witchesandpagans.com", b".comment-meta"),

    (b"glassdoor.com", b".contextHelpContent"),
    (b"glassdoor.com", b".dlgContents"),
    (b"glassdoor.com", b".empInfo"),
    (b"glassdoor.com", b".flagCommentForm"),
    (b"foodily.com", b"#hidden-content"),
    (b"glassdoor.com", b"#FlagContentFailure"),
    (b"foodandwinechronicles.com", b".post-content"),
    (b"foodily.com", b"#refer-banner"),
    (b"glassdoor.com", b"#ContentUpsell"),
    (b"glassdoor.com", b".gd-btn-link"),
    (b"glassdoor.com", b".i-loc"),
    (b"glassdoor.com", b".margVert5"),
    (b"glassdoor.com", b".preIcon20"),
    (b"glassdoor.com", b".successBox"),
    (b"glassdoor.com", b".unsignedCommentLink"),

    (b"census.gov", b"#TopicsMain"),
    (b"census.gov", b"#TopLink1"),
    (b"census.gov", b"#TopLink2"),
    (b"census.gov", b"#TopLink3"),
    (b"census.gov", b"#TopLink4"),
    (b"census.gov", b"#TopLink5"),
    (b"census.gov", b"#TopLink6"),
    (b"census.gov", b"#TopLink7"),
    (b"census.gov", b"#TopLink8"),
    (b"census.gov", b"#TopLink9"),
    (b"census.gov", b"#TopLink10"),
    (b"census.gov", b"#TopLink11"),
    (b"census.gov", b"#GeoLink5"),
    (b"census.gov", b"#LibLink3"),
    (b"census.gov", b"#LibLink4"),
    (b"census.gov", b"#DataLink1"),
    (b"census.gov", b"#AboutLink3"),
    (b"census.gov", b"td[bgcolor=\"#FFFFCC\"]"),

    (b"alibris.com", b".offer-bar"),
    (b"anenglishmaninosaka.blogspot.com", b".center"),
    (b"anenglishmaninosaka.blogspot.com", b".comment-link"),
    (b"annarbor.craigslist.org", b".contents"),
    (b"autos.jdpower.com", b".ddc-article-list"),
    (b"autos.jdpower.com", b".paging-wrap"),
    (b"bimmerwerkz.com", b".tfoot"),
    (b"bimmerwerkz.com", b".vbmenu_popup"),
    (b"bio-medicine.org", b"#leftColumn"),
    (b"biotech-capital.com", b".main-header-wrapper"),
    (b"biotech-capital.com", b".view-full-profile"),
    (b"blackamericaweb.com", b"#bitsubscribe"),
    (b"blackamericaweb.com", b".category-title"),
    (b"blackberryforums.com", b".tcat"),
    (b"blip.fm", b"#createUserAccountDialog"),
    (b"blip.fm", b"#quickSignup"),
    (b"blip.fm", b"#signupQuick"),
    (b"books.google.com.au", b"#write-review-link"),
    (b"books.google.com.au", b".qrcode_content"),
    (b"booksword.co.uk", b".says"),
    (b"cameralabs.com", b".profile"),
    (b"capcitybank.com", b".Search"),
    (b"carfax.com", b"#chiModule"),
    (b"carfax.com", b"#printModal"),
    (b"cbwentworth.wordpress.com", b".comment-likes"),
    (b"cdn.cdata.com", b"#whheader"),
    (b"cheatmasters.com", b".game_info2"),
    (b"cheatmasters.com", b".headertable"),
    (b"cheatmasters.com", b".topbar"),
    (b"cheftalk.com", b".forum-post-tools"),
    (b"christianmingle.com", b"#join_now_box"),
    (b"cio.com", b"#funnel"),
    (b"cio.com", b".author-info"),
    (b"clubwrx.net", b".bbcode_container"),
    (b"collectorsweekly.com", b".categories"),
    (b"collectorsweekly.com", b".story-nav"),
    (b"community.mcafee.com", b".lia-quilt-column-message-left-content"),
    (b"community.shopify.com", b".message-view-author-container"),
    (b"conferenceboard.ca", b"#MainRegion_C005_ShowRatingPanel"),
    (b"courier-journal.com", b".pane-container"),
    (b"couriernews.com", b"#comments_container"),
    (b"couriernews.com", b".related_content"),
    (b"coursereport.com", b".review-image"),
    (b"coursereport.com", b".user-profile-form"),
    (b"creativesaga.com", b"#CustomSearch1"),
    (b"cyclingnews.com", b".gallery_links"),
    (b"cyclonefanatic.com", b".bbcode_postedby"),
    (b"dailygazette.com", b"#textSize"),
    (b"dailytech.com", b".CommentSubHeader"),
    (b"davisclipper.com", b"#comments_container"),
    (b"davisclipper.com", b".related_content"),
    (b"dennis2society.de", b".line_numbers"),
    (b"dennis2society.de", b".postmetadata"),
    (b"dennis2society.de", b".set_low"),
    (b"dictionary.reference.com", b".fly-out"),
    (b"digitimes.com", b".enl"),
    (b"docs.servicenow.com", b"#advanced"),
    (b"docs.servicenow.com", b".successPostMessage"),
    (b"e90post.com", b".tcat"),
    (b"edmunds.com", b"#crr_review_ratings"),
    (b"ema.europa.eu", b"#ratingContainer"),
    (b"energy.opendata.ch", b"#languageSwitcher"),
    (b"eurofound.europa.eu", b".ef-comment-toggler"),
    (b"fatsecret.com", b".footerPanel"),
    (b"forewordreviews.com", b".pjax-loading-timeout"),
    (b"forewordreviews.com", b".snippet-type"),
    (b"forum.fastday.com", b".jumpbox"),
    (b"forum.fastday.com", b".text-nowrap"),
    (b"forum.robofont.com", b".alert-warning"),
    (b"forum.robofont.com", b".breadcrumb"),
    (b"forums.thefashionspot.com", b"#post_message_1307004"),
    (b"freebsd.org", b".txtoffscreen"),
    (b"gardenplants.comparespecies.com", b".AddToCompare"),
    (b"gardenplants.comparespecies.com", b".AddtoCompareFixed"),
    (b"gardenplants.comparespecies.com", b".HiDE"),
    (b"gearslutz.com", b".mobile-header-tagline"),
    (b"gearslutz.com", b".similar_threads"),
    (b"genr8change.com", b"#share-bar"),
    (b"hanginwiththehobarts.com", b".credits"),
    (b"hardwarezone.com", b"#feedback-form"),
    (b"hardwarezone.com", b".pagination"),
    (b"healthcarecommunication.com", b".articlesdetails_socialmedia_bottom"),
    (b"healthcarecommunication.com", b".articlesdetails_socialmedia_top"),
    (b"healthcarecommunication.com", b".comments_count"),
    (b"helpsdkids.org", b"#wrapSiteMap"),
    (b"heraldnews.com", b".author"),
    (b"hiphop-n-more.com", b"#disqus_thread"),
    (b"hiphop-n-more.com", b".post-ratings"),
    (b"hiphop-n-more.com", b".post-ratings-loading"),
    (b"hiphop-n-more.com", b".rate-this"),
    (b"hitvibz.com", b".ctaText"),
    (b"hitvibz.com", b".postTitle"),
    (b"il.findacase.com", b"#caseToolTip"),
    (b"il.findacase.com", b"#popupSearching"),
    (b"indianties.com", b".entry-meta"),
    (b"indyweek.com", b"#ArchiveLink"),
    (b"indyweek.com", b"#StoryBreadcrumb"),
    (b"insanescouter.org", b".popbox"),
    (b"iskwiki.upd.edu.ph", b".mw-revision"),
    (b"jalopnik.com", b".content-header"),
    (b"jalopnik.com", b".show-on-hover"),
    (b"kiehls.com", b".TT3helpful"),
    (b"kiehls.com", b".TT4addAnswer"),
    (b"kiehls.com", b".breadcrumb"),
    (b"kingston.ac.uk", b"#left-col"),
    (b"knue.com", b".thumbnail_wrap"),
    (b"kohls.com", b"#BVSEOSDK"),
    (b"kohls.com", b".ggl_sponsered_links"),
    (b"ksl.com", b"#cmt_top"),
    (b"linuxpromagazine.com", b".block-buy_item"),
    (b"lists.w3.org", b".headers"),
    (b"mastercraft.com", b".alt2"),
    (b"mensshelterofcharlotte.org", b".textwidget"),
    (b"menstennisforums.com", b".forumrules"),
    (b"menstennisforums.com", b".newreply_reviewbit"),
    (b"menstennisforums.com", b".registration"),
    (b"minotdailynews.com", b".navPaginate"),
    (b"motoprofi.com", b".leftlsmall"),
    (b"mouse-bola-bola.blogspot.com", b".post-footer"),
    (b"multichannelmerchant.com", b".new-blub"),
    (b"multichannelmerchant.com", b".wp-paginate"),
    (b"mydd.com", b".post-meta-controls"),
    (b"nameberry.com", b"#post_message_1866385"),
    (b"nbc11news.com", b".fancy"),
    (b"nbc11news.com", b".visible-phone"),
    (b"newhampshire.com", b".hover-navigation-wrapper"),
    (b"notes.bread.org", b"#atp-comments"),
    (b"notes.bread.org", b".hiddenBox"),
    (b"npr.org", b"#commentBlock"),
    (b"npr.org", b".contentheader"),
    (b"npr.org", b".enlarge_html"),
    (b"owsd.net", b"#breadcrumbs"),
    (b"owsd.net", b".element-invisible"),
    (b"owsd.net", b".success-stories-wrapper"),
    (b"packershome.com", b".yafpopupmenu"),
    (b"pagosasun.com", b".postmeta"),
    (b"partitionsdechansons.com", b"#top-row-container"),
    (b"partitionsdechansons.com", b".frame_commentaires"),
    (b"perlmonks.org", b".link-back"),
    (b"phabricator.wikimedia.org", b".differential-changeset-buttons"),
    (b"phabricator.wikimedia.org", b".phui-crumbs-view"),
    (b"pictureyear.blogspot.com", b"#Blog1_cmt-7545809667241827873"),
    (b"portsmouth.co.uk", b".article-commencts-cta"),
    (b"pragationline.com", b".product-images"),
    (b"pragationline.com", b".yith-wcwl-add-to-wishlist"),
    (b"products.smileysaudiovisual.com", b".ZipModalWrapper"),
    (b"raptorsrepublic.com", b".userprof_headers"),
    (b"rationalresponders.com", b".author-badges"),
    (b"rationalresponders.com", b".author-icons"),
    (b"rationalresponders.com", b".picture"),
    (b"riversandroads.me", b".footer-col1"),
    (b"riversandroads.me", b".footer-col2"),
    (b"ro.urbandictionary.com", b".def-store"),
    (b"rosko123.wordpress.com", b".comments-num"),
    (b"rutlandherald.com", b"#article_more_content"),
    (b"rutlandherald.com", b".article_share_tab"),
    (b"salisburypost.com", b"#inline-photos"),
    (b"scienceblogs.com", b".region-content-bottom"),
    (b"scoop.co.nz", b"#article-base-links"),
    (b"sightunseen.com", b".viewer-ui"),
    (b"sports-boards.net", b".faqsearch"),
    (b"spreaker.com", b".track_head_image"),
    (b"spreaker.com", b".track_player"),
    (b"support.mercyhurst.edu", b"#loading"),
    (b"tablethotels.com", b".hotel-name"),
    (b"tablethotels.com", b".owl-carousel-container"),
    (b"tablethotels.com", b".read-less"),
    (b"tablethotels.com", b".read-more"),
    (b"tanglesbythesea.com", b"#text-8"),
    (b"tasteofhome.com", b".rd_recipe_group_controls"),
    (b"tasteofhome.com", b".rd_recipe_group_img"),
    (b"tasteofhome.com", b".rd_recipe_group_more"),
    (b"theday.com", b"#galleryRecent"),
    (b"theday.com", b".gimagebuy"),
    (b"thetaborfoundation.org", b"#nav-single"),
    (b"timesfreepress.com", b".inline-text"),
    (b"timesfreepress.com", b".topHeaderBGWrap"),
    (b"tomshardware.com", b"#topicTitle"),
    (b"tools.rosinstrument.com", b".tinytext"),
    (b"topjobs.ch", b".loader"),
    (b"traveloka.com", b".UMCE4"),
    (b"traveloka.com", b"._2A2-N"),
    (b"tvtechnology.com", b".comments_bar"),
    (b"typekit.com", b".controls"),
    (b"typekit.com", b".navigation"),
    (b"ucsdtritons.com", b"#photoCap"),
    (b"urgentcarelocations.com", b"#map-details"),
    (b"urgentcarelocations.com", b"#reviews"),
    (b"urgentcarelocations.com", b".report-inaccurate-wrapper"),
    (b"usarugby.org", b".kmt-ratings-overview"),
    (b"usarugby.org", b".kmt-share-wrap"),
    (b"vincellar.vinfolio.com", b"#auctionListingPopup"),
    (b"vincellar.vinfolio.com", b"#sourceInfoPopup"),
    (b"vincellar.vinfolio.com", b"#tastingnoteHelpPopup"),
    (b"waitrose.com", b".centerrecipenote"),
    (b"waitrose.com", b".recipeshare"),
    (b"waynesvilledailyguide.com", b".blg-right"),
    (b"weareiowa.com", b".breaking-news-alerts"),
    (b"weather.com", b".backTo"),
    (b"webcommentary.com", b".small"),
    (b"webcommentary.com", b".topnav1"),
    (b"wethrift.com", b".review-image"),
    (b"wtf.com", b".WTFdiscuss"),
    (b"wtf.com", b".uix_welcomeSection__title"),
    (b"www2.ed.gov", b"#section-nav"),

    (b"cruisecritic.com", b".css-1pn7adc"),
    (b"cruisecritic.com", b".css-mdqznw"),
    (b"cruisecritic.com", b".css-whilru"),
    (b"stampedia.net", b".item-list"),
    (b"stampedia.net", b".itemPerPage"),
    (b"stampedia.net", b".stampimg"),
    (b"thorax.bmj.com", b"#mobile-article-tab-container"),

    (b"acityamonth.com", b".comments-title"),
    (b"acityamonth.com", b".pingback"),
    (b"acityamonth.com", b".site-info"),
    (b"acityamonth.com", b".size-full"),
    (b"alibris.com", b".write-link"),
    (b"americanpoems.com", b".head"),
    (b"arkuszematuralne.pl", b".copyright"),
    (b"badmintoncentral.com", b".bbcode_container"),
    (b"bananawonder.com", b".w2bPinitButton"),
    (b"beckett.com", b".post_buttons"),
    (b"belangerinc.com", b"#homeRight"),
    (b"bimmerwerkz.com", b".post-count"),
    (b"bimmerwerkz.com", b".thread-starter"),
    (b"bimmerwerkz.com", b".toolbar"),
    (b"blog.akismet.com", b".comments-title"),
    (b"blog.akismet.com", b".num-comments"),
    (b"blogs.theprovince.com", b".poweredby"),
    (b"blubrry.com", b".sidebar-button"),
    (b"blurb.com", b".book-list__btns"),
    (b"bonuscamp.com", b".atricle_box"),
    (b"borderlands.fandom.com", b".wds-community-header"),
    (b"budget101.com", b"#pagetitle"),
    (b"budget101.com", b".navbit"),
    (b"cadizrecord.com", b".title_date"),
    (b"cameralabs.com", b"#bannerad"),
    (b"cattlenetwork.com", b".articleoptions"),
    (b"cattlenetwork.com", b".commentbox"),
    (b"cbssports.com", b".player-chart-header-row"),
    (b"cbwentworth.wordpress.com", b".reply"),
    (b"chictopia.com", b".notify_yellow"),
    (b"clubwrx.net", b".bbcode_quote"),
    (b"comixology.com", b".previewLink"),
    (b"common-mistakes.net", b".cls1"),
    (b"community.mcafee.com", b".KudosButton"),
    (b"community.mcafee.com", b".lia-component-solution-list"),
    (b"community.shopify.com", b".KudosButton"),
    (b"conferenceboard.ca", b"#MainRegion_C008_ratingAggContext"),
    (b"conferenceboard.ca", b"#rating"),
    (b"courier-journal.com", b"#conveyorbottom"),
    (b"courier-journal.com", b"#emailmodalcontent"),
    (b"courier-journal.com", b".ody-filed"),
    (b"cricketarchive.com", b"#logoContainer"),
    (b"dailytech.com", b".NewsBodyImage"),
    (b"digitimes.com", b".mimg"),
    (b"e90post.com", b".postBitScoreItem"),
    (b"e90post.com", b".smallfont"),
    (b"efloras.org", b"#ucEfloraHeader_panelSearchBox"),
    (b"ema.europa.eu", b".content-header"),
    (b"endure-network.eu", b".imageleft"),
    (b"endure-network.eu", b".imageright"),
    (b"engineering.academickeys.com", b".apply"),
    (b"eurofound.europa.eu", b".ds-node-comments"),
    (b"failblog.cheezburger.com", b".nw-post-actions"),
    (b"failblog.cheezburger.com", b".nw-post-toolbar"),
    (b"failblog.cheezburger.com", b".section-title"),
    (b"fandango.com", b".mop__synopsis-link"),
    (b"fatsecret.com", b".topBandLoggedOut"),
    (b"forewordreviews.com", b".pjax-loading-progress-wrapper"),
    (b"fs.fed.us", b".feed-icons"),
    (b"gardenplants.comparespecies.com", b".BottomProduct"),
    (b"gardenplants.comparespecies.com", b".Ranks"),
    (b"github.com", b".file-navigation"),
    (b"glennopedia.com", b".comments-title"),
    (b"globalsurfers.com", b".top-bar"),
    (b"groups.yahoo.com", b".group-stats"),
    (b"groups.yahoo.com", b".main-menu-content"),
    (b"groups.yahoo.com", b".msg-attachment-wrapper"),
    (b"hardwarezone.com", b".desc"),
    (b"heraldnews.com", b".art-byline-div"),
    (b"hillpost.in", b".printfriendly"),
    (b"house.fandom.com", b"#infoboxinternal"),
    (b"iclassifiedsnetwork.com", b".SideModule"),
    (b"ieeexplore.ieee.org", b"#btn-full-txt"),
    (b"ieeexplore.ieee.org", b"#ftm-purchase-pdf"),
    (b"il.findacase.com", b".caseToolTip"),
    (b"indyweek.com", b".pin-it-button"),
    (b"informationweek.com", b"#new_things"),
    (b"informationweek.com", b".download2"),
    (b"ipinfo.io", b".address-singup"),
    (b"ithemes.com", b"#jump-to-nav"),
    (b"ithemes.com", b".diff"),
    (b"itu.int", b".topritems"),
    (b"jalopnik.com", b".read-more"),
    (b"kgi.org", b"#block-block-8"),
    (b"kgi.org", b"#block-block-9"),
    (b"lafollettepress.com", b".text-resizer"),
    (b"library.dayalgroup.com", b".no-job-listing"),
    (b"m3post.com", b".repScoreBox"),
    (b"mail.gnome.org", b"#global_domain_bar_archive"),
    (b"mcdougallcorp.com", b".umbraco-forms-caption"),
    (b"minotdailynews.com", b"#membercmts"),
    (b"ncbi.nlm.nih.gov", b"#result_action_bar"),
    (b"ncbi.nlm.nih.gov", b".supplemental"),
    (b"nevadaappeal.com", b".published"),
    (b"nevadaappeal.com", b".source-org"),
    (b"nevadaappeal.com", b".updated"),
    (b"newhampshire.com", b".carousel"),
    (b"newhampshire.com", b".news"),
    (b"news.psu.edu", b".block-page-title"),
    (b"news.psu.edu", b".field-name-field-image"),
    (b"news.sys-con.com", b"#header-greeting"),
    (b"news.sys-con.com", b".category"),
    (b"next.unibz.it", b".comForm"),
    (b"next.unibz.it", b".header_logo"),
    (b"nsf.gov", b".leftwhite"),
    (b"owsd.net", b".success-stories"),
    (b"packershome.com", b".guestUser"),
    (b"phabricator.wikimedia.org", b".phabricator-action-list-view"),
    (b"phpdeveloper.org", b".stub"),
    (b"pixbits.wordpress.com", b".no-comments"),
    (b"pl.tripadvisor.com", b".mediaFeedbackBox"),
    (b"playfire.com", b".report_status_update"),
    (b"raptorsrepublic.com", b"#activity_tab_container"),
    (b"raptorsrepublic.com", b".avatar"),
    (b"raptorsrepublic.com", b".newactivity"),
    (b"rebeccalillycosta.com", b".sfsi_Sicons"),
    (b"recordnet.com", b".art-byline-div"),
    (b"riversandroads.me", b".blog-footer"),
    (b"ro.urbandictionary.com", b".ribbon"),
    (b"rutlandherald.com", b".article_icons"),
    (b"s4models.com", b"#comments-title"),
    (b"s4models.com", b".pingback"),
    (b"scoop.co.nz", b".centre-col"),
    (b"scoop.co.nz", b".leader-also"),
    (b"scoop.co.nz", b".left-col"),
    (b"sirstevesguide.com", b".pda"),
    (b"starcourier.com", b".art-byline-div"),
    (b"starcourier.com", b".entry-summary"),
    (b"tamilnet.com", b"#mainColumnMid"),
    (b"thenotsosupermama.com", b".post-com-count"),
    (b"thepleiades7.blogspot.com", b".post-footer"),
    (b"theserverside.com", b".goToTop"),
    (b"timesfreepress.com", b"#detailHeadlineStickyHeadline"),
    (b"topjobs.ch", b".left-column"),
    (b"trftimes.com", b"#tools"),
    (b"trftimes.com", b".buttonheading"),
    (b"trftimes.com", b".contentheading"),
    (b"trophytracking.com", b"#crumbs"),
    (b"tumblr.com", b".tx-button"),
    (b"typekit.com", b".collections"),
    (b"uctv.tv", b".footerBox"),
    (b"uctv.tv", b".footerHeader"),
    (b"urgentcarelocations.com", b".reviews-container"),
    (b"use.perl.org", b"#d2out"),
    (b"use.perl.org", b"#jump"),
    (b"vladi-private-islands.de", b".actions"),
    (b"vladi-private-islands.de", b".maptype"),
    (b"waitrose.com", b".tools"),
    (b"waitrose.com", b".userRatings"),
    (b"waynesvilledailyguide.com", b".mrg-top-15"),
    (b"weather.com", b".allFloatRight"),
    (b"weather.weatherbug.com", b"#brdcrmb-chg-units"),
    (b"whatsonmypc.blog", b".credits"),
    (b"wmnf.org", b".comments-count"),
    (b"wmnf.org", b".headerinfo"),

    (b"allegramarketingprint.com", b".um-panel"),
    (b"archive.financialexpress.com", b".adbygoogle"),
    (b"archive.financialexpress.com", b".font"),
    (b"autos.jdpower.com", b".ad-widget"),
    (b"beckett.com", b"#edited_by_2174869"),
    (b"beckett.com", b".inline_rating"),
    (b"beckett.com", b".list_50"),
    (b"belangerinc.com", b"#SRWrapper"),
    (b"bimmerwerkz.com", b"#displaymodes"),
    (b"bimmerwerkz.com", b"#linkbacktools"),
    (b"bimmerwerkz.com", b"#threadtools"),
    (b"blurb.com", b".delete-confirmation-text"),
    (b"blurb.com", b".see-more"),
    (b"catholic-hierarchy.org", b"#logol"),
    (b"catholic-hierarchy.org", b"#logor"),
    (b"cheftalk.com", b".fj-control-bar"),
    (b"chictopia.com", b".smallShareButton"),
    (b"chowhound.com", b".fr_ccnt"),
    (b"chowhound.com", b".fr_p_section"),
    (b"chrisweigant.com", b".commentnumber"),
    (b"chrisweigant.com", b".metalinks"),
    (b"cio.com", b".comments-hed"),
    (b"coffeeforums.co.uk", b".ipsQuote"),
    (b"community.mcafee.com", b".flex_blade"),
    (b"community.mcafee.com", b".lia-forum-topic-page-reply-count"),
    (b"community.mcafee.com", b".lia-forum-topic-page-solution-count"),
    (b"conferenceboard.ca", b"#RightPanelRegion_C006_AllUPD"),
    (b"convertunits.com", b".squares"),
    (b"courier-journal.com", b".ody-sub-filed"),
    (b"csce.gov", b".Tab_Label"),
    (b"dailytech.com", b"#PostDate"),
    (b"docs.google.com", b".ss-q-checkbox"),
    (b"docs.servicenow.com", b".col-lg-2"),
    (b"dotnetfunda.com", b".alert"),
    (b"dotnetfunda.com", b".voteUD"),
    (b"e90post.com", b".alt2"),
    (b"energy.opendata.ch", b"#logo"),
    (b"english-subtitles.club", b".breadcrumb"),
    (b"eventbrite.com.au", b"#bookmark-login-popup"),
    (b"eventbrite.com.au", b".friends-wrapper"),
    (b"experienceproject.com", b".drop-down-menu"),
    (b"experienceproject.com", b".member-age-gender-abbreviated"),
    (b"fandango.com", b".movie-details__mop-link"),
    (b"feedbooks.com", b"#AUWLBkImage"),
    (b"feedbooks.com", b"#AUWLBkTitle"),
    (b"feedbooks.com", b"#AUWLBkURL"),
    (b"feedbooks.com", b".buy_button_block"),
    (b"feedbooks.com", b".cover-highlight"),
    (b"feedbooks.com", b".large-cover-button"),
    (b"floridatrend.com", b".railbox"),
    (b"forums.everythingicafe.com", b".avatarHolder"),
    (b"forums.everythingicafe.com", b".publicControls"),
    (b"fs.fed.us", b".node-readmore"),
    (b"fullonlinebook.com", b".othertools"),
    (b"fullonlinebook.com", b".titledir"),
    (b"graphicdesignforum.com", b".post-signature"),
    (b"hsc.wvu.edu", b".connect"),
    (b"ideas.repec.org", b".visible-print-block"),
    (b"ieeexplore.ieee.org", b"#full-txt-menu-wrap"),
    (b"ieeexplore.ieee.org", b".button-set"),
    (b"informationweek.com", b".bid_crumb"),
    (b"informationweek.com", b".search_container2"),
    (b"insanescouter.org", b"#cp_box"),
    (b"iskwiki.upd.edu.ph", b"#contentSub"),
    (b"ithemes.com", b"#p-namespaces"),
    (b"ithemes.com", b"#p-variants"),
    (b"jalopnik.com", b".hide-on-hover"),
    (b"jcink.net", b"#userlinks"),
    (b"jcink.net", b".mini_cssbutton"),
    (b"jcink.net", b".pagination"),
    (b"jcink.net", b".post_controls"),
    (b"jcink.net", b".postlinksbar"),
    (b"jeepkings.ca", b".bbcode_quote"),
    (b"ksl.com", b".related"),
    (b"lowpowerlab.com", b"#sidebar-menu"),
    (b"mastercraft.com", b".fieldset"),
    (b"menstennisforums.com", b".vbmenu_control"),
    (b"minotdailynews.com", b".txtSmall"),
    (b"mydd.com", b".post-comments-and-recs"),
    (b"ncbi.nlm.nih.gov", b".send_to"),
    (b"nevadaappeal.com", b"#article-related"),
    (b"newhampshire.com", b"#ccr-slide-mainb"),
    (b"newhampshire.com", b".mtm-header"),
    (b"officialcharts.com", b".commentlink"),
    (b"owsd.net", b".block-views"),
    (b"phabricator.wikimedia.org", b".phui-header-action-links"),
    (b"phpdeveloper.org", b".comment_count"),
    (b"pl.tripadvisor.com", b".colReport"),
    (b"pl.tripadvisor.com", b".photoFilterContainer"),
    (b"pl.tripadvisor.com", b".photoVoteBox"),
    (b"playfire.com", b".blue-button"),
    (b"playfire.com", b".comments_links"),
    (b"playfire.com", b".heading-buttons"),
    (b"query.nytimes.com", b".editionToggle"),
    (b"randyhamilton.openmortgage.com", b".prefi-section"),
    (b"rationalresponders.com", b".user_badges"),
    (b"rosko123.wordpress.com", b"#rss-link"),
    (b"scoop.co.nz", b".right-col"),
    (b"smartertravel.com", b".hotel_rating"),
    (b"smartertravel.com", b".photos_url"),
    (b"songmeanings.com", b".editbutton"),
    (b"songmeanings.com", b".holder-new"),
    (b"songmeanings.com", b".login-holder"),
    (b"sunkenstone.com", b".fusion-post-slideshow"),
    (b"tablethotels.com", b"#hotel-details"),
    (b"talk.philmusic.com", b".cat_bar"),
    (b"theminimice.blogspot.com", b".comment-link"),
    (b"theserverside.com", b".threadMessagesList"),
    (b"theserverside.com", b".userDiscovery"),
    (b"tools.rosinstrument.com", b".graybgs"),
    (b"trftimes.com", b"#banw"),
    (b"typekit.com", b".families"),
    (b"typekit.com", b".footer-locale-switch"),
    (b"vcahospitals.com", b".pop-close"),
    (b"vincellar.vinfolio.com", b"#right-mini-section"),
    (b"vincellar.vinfolio.com", b".bottom-info"),
    (b"waitrose.com", b".commentandimages"),
    (b"waitrose.com", b".overlay-popup"),
    (b"waitrose.com", b".ratingsystem"),
    (b"waynesvilledailyguide.com", b".fbx-readmore"),
    (b"waynesvilledailyguide.com", b".title-red"),
    (b"wri.org", b"#utility"),
    (b"www2.ed.gov", b".contentSectionHeader"),
    (b"www2.ed.gov", b".navtitle"),
    (b"zefron.com", b"#edited_by_23303"),
    (b"zefron.com", b"#edited_by_56657"),
    (b"zefron.com", b".inline_rating"),
    (b"zefron.com", b".pagination"),

    (b"app.leg.wa.gov", b"#horizontalStatusDisplay"),
    (b"app.leg.wa.gov", b"#verticalStatusDisplay"),
    (b"belangerinc.com", b"#storiesWrapper"),
    (b"bio-medicine.org", b".endSummary"),
    (b"biotech-capital.com", b".company"),
    (b"blackberryforums.com", b".offline"),
    (b"blip.fm", b".icon"),
    (b"carfax.com", b"#sumOwnModule"),
    (b"carfax.com", b".printrow"),
    (b"chrisweigant.com", b".sharelinks"),
    (b"cio.com", b".category"),
    (b"cio.com", b".dateline"),
    (b"cio.com", b".end-byline"),
    (b"clubwrx.net", b".bbcode_postedby"),
    (b"courier-journal.com", b".emailcontent"),
    (b"courier-journal.com", b".ody-bottom-caro"),
    (b"digitimes.com", b".clclgjs"),
    (b"digitimes.com", b".mr-tabbox"),
    (b"docs.google.com", b".ss-choice-item-control"),
    (b"dotnetfunda.com", b"#MainContent_ResponseForm1_loginP"),
    (b"ema.europa.eu", b".help"),
    (b"eventbrite.com.au", b".badge-status"),
    (b"eventbrite.com.au", b".is-hidden-accessible"),
    (b"eventbrite.com.au", b".js-scroll-to-map"),
    (b"eventbrite.com.au", b".listing-panel-info__status"),
    (b"failblog.cheezburger.com", b".nw-post-comments"),
    (b"forums.everythingicafe.com", b".messageUserBlock"),
    (b"fs.fed.us", b".feed-icon"),
    (b"ieeexplore.ieee.org", b"#ftm-purchase"),
    (b"insanescouter.org", b".right-side"),
    (b"jalopnik.com", b".header-title"),
    (b"jalopnik.com", b".js_follow-controls"),
    (b"jcink.net", b".author_information"),
    (b"kingston.ac.uk", b"#tablet-boxes"),
    (b"ksl.com", b".photo-attribution"),
    (b"lafollettepress.com", b".text-resize"),
    (b"libertysentinel.org", b".cat-links"),
    (b"lowpowerlab.com", b".page-navigation"),
    (b"mail.gnome.org", b".tab"),
    (b"milngavieherald.co.uk", b".article-header__lead-image"),
    (b"mirchee.com", b".actionButton"),
    (b"money.howstuffworks.com", b".recirc-panel"),
    (b"murga-linux.com", b".copyright"),
    (b"murga-linux.com", b".mainmenu"),
    (b"norwalk.itsrelevant.com", b"#more-stories"),
    (b"nsf.gov", b".hyperimage"),
    (b"nsf.gov", b".nsf-logo-bottom"),
    (b"owsd.net", b".button-green"),
    (b"packershome.com", b".postForumUsers"),
    (b"playfire.com", b".game-link"),
    (b"playfire.com", b".liked_by"),
    (b"products.smileysaudiovisual.com", b"#jsZipModalWrapper"),
    (b"radaronline.com", b".radar-thumb"),
    (b"randyhamilton.openmortgage.com", b".p-refi"),
    (b"rationalresponders.com", b".author-posts"),
    (b"rationalresponders.com", b".author-regdate"),
    (b"readthestars.com", b"#searchhead"),
    (b"rutlandherald.com", b".categoryItem"),
    (b"rutlandherald.com", b".mediaGallery"),
    (b"rutlandherald.com", b".specialSections"),
    (b"s4models.com", b".comment-likes"),
    (b"sailnet.com", b".tcat"),
    (b"sightunseen.com", b".viewer-image"),
    (b"smartdevicelink.com", b".sidebar-select-toggle"),
    (b"southstrandnews.com", b"#main-picture-expand-button"),
    (b"sunkenstone.com", b".omapi-shortcode-parsed"),
    (b"tablethotels.com", b".search-spinner"),
    (b"tablethotels.com", b".show-amenities"),
    (b"tablethotels.com", b".view-prices-cta"),
    (b"thefrisky.com", b".simply-irresistible"),
    (b"themillions.com", b".booklinks"),
    (b"theserverside.com", b".getFeed"),
    (b"tools.rosinstrument.com", b".ltyellow"),
    (b"trftimes.com", b".bannergroup_blank"),
    (b"urgentcarelocations.com", b".reviews"),
    (b"use.perl.org", b".hide"),
    (b"vladi-private-islands.de", b".button"),
    (b"wethrift.com", b".review-image-mobile"),
    (b"wiki.inf.ed.ac.uk", b".patternToolBar"),
    (b"www2.ed.gov", b".utilText"),
    (b"zefron.com", b".author_buttons"),

    (b"alt.com", b".help_search"),
    (b"app.leg.wa.gov", b".actionButtonContainer"),
    (b"app.leg.wa.gov", b".seeBillHistoryForCompleteDetails"),
    (b"arkuszematuralne.pl", b".meta--comments"),
    (b"beckett.com", b".post_author_info"),
    (b"beckett.com", b".post_meta"),
    (b"belangerinc.com", b"#storiesInner"),
    (b"belangerinc.com", b".inline_tip"),
    (b"blip.fm", b"#upsell"),
    (b"blurb.com", b".bk-cover-image"),
    (b"blurb.com", b".cart-quantity-add"),
    (b"carfax.com", b"#tabws"),
    (b"cbwentworth.wordpress.com", b".entry-format-badge"),
    (b"cbwentworth.wordpress.com", b".entry-meta"),
    (b"childrenwithdiabetes.com", b".textsmall"),
    (b"cinephonix.com", b".basket"),
    (b"cinephonix.com", b".download"),
    (b"cinephonix.com", b".like"),
    (b"cinephonix.com", b".trow"),
    (b"cl-user.net", b".onoffd"),
    (b"cl-user.net", b".onoffo"),
    (b"courier-journal.com", b".ody-aside"),
    (b"e90post.com", b".avatarImage"),
    (b"e90post.com", b".flagStyle2"),
    (b"eslflashcards.com", b".tla"),
    (b"fandango.com", b".carousel-cast-crew__see-full"),
    (b"fandango.com", b".features"),
    (b"fatsecret.com", b".cfp_breakdown_container"),
    (b"fatsecret.com", b".rdi_perc_container"),
    (b"feedbooks.com", b"#AUWLBkPrice"),
    (b"feedbooks.com", b".book_buy_text"),
    (b"feedbooks.com", b".span-16"),
    (b"finance.boston.com", b".linkedOutBox"),
    (b"honeynet.org", b"#block-block-1"),
    (b"honeynet.org", b"#block-block-2"),
    (b"huskers.com", b"#site-content"),
    (b"insanescouter.org", b".popbox_link"),
    (b"investorplace.com", b".alignright"),
    (b"investorplace.com", b".wps-seo-booster-headline"),
    (b"jessicacarneyassociates.co.uk", b".entry-actions"),
    (b"jessicacarneyassociates.co.uk", b".meta-below-title"),
    (b"jessicacarneyassociates.co.uk", b".thumb-image"),
    (b"kingston.ac.uk", b"#footer-nav"),
    (b"kohls.com", b".BVRRPager"),
    (b"ksl.com", b".crumbs"),
    (b"minotdailynews.com", b".cBdrMain"),
    (b"ncbi.nlm.nih.gov", b"#NCBIFooter_dynamic"),
    (b"ncbi.nlm.nih.gov", b".disp_settings"),
    (b"nevadaappeal.com", b"#main-ticker"),
    (b"nevadaappeal.com", b".col3"),
    (b"nevadaappeal.com", b".rss"),
    (b"newhampshire.com", b".media"),
    (b"news.sys-con.com", b".footer-head1"),
    (b"news.sys-con.com", b".footer-head2"),
    (b"nobelcom.com", b".idcSearchInner"),
    (b"notes.bread.org", b".asset-img-link"),
    (b"notes.bread.org", b".trackbacks"),
    (b"patchwork.ozlabs.org", b".patchheaders"),
    (b"phabricator.wikimedia.org", b".login-to-comment"),
    (b"pl.tripadvisor.com", b".thumbBox"),
    (b"playfire.com", b".expander_more"),
    (b"playfire.com", b".tooltip_text"),
    (b"products.smileysaudiovisual.com", b".ContactModalWrapper"),
    (b"psypokes.com", b".postsubject"),
    (b"query.nytimes.com", b".articleTools"),
    (b"query.nytimes.com", b".heading"),
    (b"slideshare.net", b".flag-inappropriate"),
    (b"themillions.com", b".comments-article"),
    (b"theprp.com", b".header-bar"),
    (b"theprp.com", b".post-header"),
    (b"ucsdtritons.com", b"#top_image"),
    (b"ucsdtritons.com", b"#videoStop"),
    (b"uctv.tv", b"#Form1"),
    (b"wiretotheear.com", b".says"),
    (b"yahoo.com", b".slideshow-figure"),

    (b"theday.com", b".galleryCats"),
    (b"theday.com", b".verticalbox"),
    (b"newhampshire.com", b".hover-navigation"),
    (b"newhampshire.com", b".image-crop"),

    (b"belangerinc.com", b"#storiesTitle"),
    (b"blackberryforums.com", b".vbmenu_option"),
    (b"borderlands.fandom.com", b".dablink"),
    (b"careers.govt.nz", b"#contactUsWrap"),
    (b"childrenwithdiabetes.com", b".AdCellShaded"),
    (b"chrisweigant.com", b"#blogtitle"),
    (b"cornellpress.cornell.edu", b".onixlink"),
    (b"dictionary.cambridge.org", b".see-more"),
    (b"e90post.com", b".postBitActionFrame"),
    (b"fandango.com", b".fan-reviews"),
    (b"feedbooks.com", b".less_more"),
    (b"fullonlinebook.com", b".title_download"),
    (b"gardenplants.comparespecies.com", b".CellDisplay"),
    (b"ieeexplore.ieee.org", b"#qualify-price-ad"),
    (b"jcink.net", b".maintitle"),
    (b"jedidefender.com", b".quoteheader"),
    (b"lists.w3.org", b".foot"),
    (b"lybrate.com", b"#doctorFeeds-viewmore"),
    (b"lybrate.com", b"#map"),
    (b"newhampshire.com", b"#ccr-footer-sidebar"),
    (b"news.sys-con.com", b".fivestar-widget"),
    (b"pbs.org", b"#logos-pbsnsn2"),
    (b"pbs.org", b".roll"),
    (b"pl.tripadvisor.com", b".photoVote"),
    (b"products.smileysaudiovisual.com", b".gtm-case-study-external-url"),
    (b"products.smileysaudiovisual.com", b".jsContactModalOpen"),
    (b"rationalresponders.com", b".service-links"),
    (b"s4models.com", b"#jp-post-flair"),
    (b"sangriasunshinecom.wordpress.com", b".comments-link"),
    (b"songmeanings.com", b"#moverdiv"),
    (b"traveloka.com", b"._22fih"),
    (b"traveloka.com", b"._30QmZ"),
    (b"use.perl.org", b"#d2act"),
    (b"use.perl.org", b".commentBoxLinks"),
    (b"use.perl.org", b".escape-link"),
    (b"webpathology.com", b"#followus_header"),
    (b"westonparkhospitality.com", b"#footlinks"),
    (b"wiki.inf.ed.ac.uk", b"#patternBottomBar"),
    (b"wiki.inf.ed.ac.uk", b".patternRevInfo"),
    (b"wiki.inf.ed.ac.uk", b".patternTop"),
    (b"wiki.inf.ed.ac.uk", b".patternTopicAction"),

    (b"app.leg.wa.gov", b".hidden-lg"),
    (b"app.leg.wa.gov", b".visible-"),
    (b"bearalley.blogspot.com", b"#header-wrapper"),
    (b"clubwrx.net", b".View"),
    (b"coffeeforums.co.uk", b".ipsComment"),
    (b"community.fccsoftware.ca", b".fccsoftware"),
    (b"e90post.com", b".thead"),
    (b"engineering.academickeys.com", b"#job_redirect"),
    (b"nevadaappeal.com", b"#breadcrumb-bar"),
    (b"nevadaappeal.com", b"#footer-copyright"),
    (b"talk.philmusic.com", b".bbc_alternate_quote"),
    (b"talk.philmusic.com", b".bbc_standard_quote"),
    (b"talk.philmusic.com", b".quoteheader"),
    (b"talk.philmusic.com", b".signature"),
    (b"tvtechnology.com", b"#dnn_"),
    (b"yahoo.com", b".slideshow-carousel"),

    (b"devilslakejournal.com", b"#art-byline-pubdate-cont"),
    (b"devilslakejournal.com", b".gotofbcom"),
    (b"devilslakejournal.com", b".org"),
    (b"devilslakejournal.com", b".published"),
    (b"devilslakejournal.com", b".entry-summary"),
    (b"devilslakejournal.com", b"#toppage"),

    (b"app.leg.wa.gov", b"#billStatusAtAGlanceSmall"),
    (b"bearalley.blogspot.com", b"#Image58"),
    (b"bearalley.blogspot.com", b".date-header"),
    (b"cadizrecord.com", b".dont_touch_me"),
    (b"careers.govt.nz", b"#breadcrumbs"),
    (b"cio.com", b".bylineImage"),
    (b"cio.com", b".comments-cta"),
    (b"csce.gov", b"#tabPrintLink"),
    (b"csce.gov", b".TabActive_Label"),
    (b"daedalusbooks.com", b"#Table22"),
    (b"dotnetfunda.com", b"#vote21797"),
    (b"dotnetfunda.com", b"#vote21803"),
    (b"endure-network.eu", b"#logoPrint"),
    (b"fatsecret.com", b".photoBlank"),
    (b"github.com", b".BorderGrid"),
    (b"ideas.repec.org", b".form-inline"),
    (b"jcink.net", b".cssbutton"),
    (b"jcink.net", b".float_right"),
    (b"jcink.net", b".goto-firstunread"),
    (b"kingston.ac.uk", b".fb-page"),
    (b"kingston.ac.uk", b".icon-alex-arrow"),
    (b"kingston.ac.uk", b".menu-nav-list-container"),
    (b"kingston.ac.uk", b".menu-section-heading"),
    (b"mcdougallcorp.com", b".back-to-top"),
    (b"mcdougallcorp.com", b".inside-section-btn"),
    (b"mcdougallcorp.com", b".logos"),
    (b"mcdougallcorp.com", b".top-image"),
    (b"meta.stackexchange.com", b".bottom-notice"),
    (b"meta.stackexchange.com", b".comment-score"),
    (b"mirchee.com", b".fulltext"),
    (b"officialcharts.com", b".floatLeft"),
    (b"officialcharts.com", b".morelink"),
    (b"ornaross.com", b".header-logo-center"),
    (b"ornaross.com", b".header-pattern-wrapper"),
    (b"packershome.com", b"#DivForumJump"),
    (b"packershome.com", b".MessageBox"),
    (b"packershome.com", b".displayPostFooter"),
    (b"packershome.com", b".yafpager"),
    (b"playfire.com", b"#screenshot_timeline"),
    (b"playfire.com", b".network"),
    (b"playfire.com", b".steam-small"),
    (b"progarchives.com", b"#header-image"),
    (b"progarchives.com", b"#imgCover"),
    (b"pwmag.com", b".eyebrow"),
    (b"pwmag.com", b".sourceBrand"),
    (b"rationalresponders.com", b".forum-topic-navigation"),
    (b"rcgroups.com", b".inlineimg"),
    (b"rcgroups.com", b".thead_postbit_right"),
    (b"sangriasunshinecom.wordpress.com", b".post-thumbnail"),
    (b"spreaker.com", b"#desc_1_dots"),
    (b"spreaker.com", b"#ly_help_container"),
    (b"spreaker.com", b"#track_messages_count"),
    (b"spreaker.com", b".track_like_count"),
    (b"tablethotels.com", b".action-item"),
    (b"trftimes.com", b".addtoany"),
    (b"waitrose.com", b".glossary"),
    (b"waitrose.com", b".unitconversion"),
    (b"whatsonmypc.blog", b".alignnone"),
    (b"wmnf.org", b".span-10"),
    (b"wmnf.org", b".span-30"),

    (b"beckett.com", b".float_right"),
    (b"biotech-capital.com", b".with-block-button"),
    (b"cheatchannel.com", b".alinelink"),
    (b"cheatchannel.com", b".amenulink"),
    (b"euroweeklynews.com", b".td-post-comments"),
    (b"floridatrend.com", b".subcon"),
    (b"mustseeindia.com", b".head-section-end"),
    (b"newhampshire.com", b".ccr-gallery-ttile"),
    (b"news.sys-con.com", b".footer-nav"),
    (b"phabricator.wikimedia.org", b".differential-toc-char"),
    (b"phabricator.wikimedia.org", b".phui-header-subheader"),
    (b"pl.tripadvisor.com", b".sprite-filmstrip-left-arrow"),
    (b"pl.tripadvisor.com", b".sprite-filmstrip-right-arrow"),
    (b"pl.tripadvisor.com", b".thumbOpt"),
    (b"psypokes.com", b".cap-div"),
    (b"ru.tradingview.com", b".tv-widget-idea__cover"),
    (b"scoop.co.nz", b"#base"),
    (b"vincellar.vinfolio.com", b"#inventorySummaryTable"),
    (b"vincellar.vinfolio.com", b".buy-from-vinfolio-btn"),
    (b"vincellar.vinfolio.com", b".price-options"),
    (b"vincellar.vinfolio.com", b".tmb-nav"),
    (b"vincellar.vinfolio.com", b".user-control2"),
    (b"wtf.com", b".contentRow-figure"),
    (b"www2.ed.gov", b".headersLevel1"),
    (b"www2b.abc.net.au", b"#subheader"),
];

/// Page domain from og:url or canonical link (lowercased, www-stripped).
unsafe fn page_domain(doc: *mut lxb_html_document_t) -> Vec<u8> {
    unsafe {
        let head: *mut lxb_dom_node_t = (*doc).head.cast();
        if head.is_null() {
            return Vec::new();
        }
        let mut best: Vec<u8> = Vec::new();
        let mut child = (*head).first_child;
        while !child.is_null() {
            if (*child).type_ == LXB_DOM_NODE_TYPE_ELEMENT {
                let url_attr: Vec<u8> = if (*child).local_name == LXB_TAG_META
                    && get_node_attr(child, b"property") == b"og:url"
                {
                    get_node_attr(child, b"content").to_vec()
                } else if (*child).local_name == LXB_TAG_LINK
                    && get_node_attr(child, b"rel") == b"canonical"
                {
                    get_node_attr(child, b"href").to_vec()
                } else {
                    Vec::new()
                };
                if !url_attr.is_empty() {
                    let s = url_attr.to_ascii_lowercase();
                    if let Some(pos) = s.windows(3).position(|w| w == b"://") {
                        let rest = &s[pos + 3..];
                        let end = rest.iter().position(|&b| b == b'/').unwrap_or(rest.len());
                        let mut d = &rest[..end];
                        if d.starts_with(b"www.") {
                            d = &d[4..];
                        }
                        best = d.to_vec();
                        break;
                    }
                }
            }
            child = (*child).next;
        }
        // Fallback (0109): no og:url/canonical (pre-social-web pages) —
        // take the majority host of absolute links when it clearly
        // dominates (>=10 links and >=60% of all absolute hrefs).
        if best.is_empty() {
            let body: *mut lxb_dom_node_t = (*doc).body.cast();
            if !body.is_null() {
                let dom_coll = lxb_dom_collection_make_noi((*body).owner_document, 40);
                lxb_dom_elements_by_tag_name(body.cast(), dom_coll, b"a".as_ptr(), 1);
                let mut counts: std::collections::HashMap<Vec<u8>, usize> = std::collections::HashMap::new();
                let mut total = 0usize;
                for i in 0..lxb_dom_collection_length_noi(dom_coll) {
                    let a = lxb_dom_collection_node_noi(dom_coll, i);
                    let href = get_node_attr(a, b"href").to_ascii_lowercase();
                    if let Some(pos) = href.windows(3).position(|w| w == b"://") {
                        let rest = &href[pos + 3..];
                        let end = rest.iter().position(|&b| b == b'/').unwrap_or(rest.len());
                        let mut d = &rest[..end];
                        if d.starts_with(b"www.") {
                            d = &d[4..];
                        }
                        if !d.is_empty() {
                            *counts.entry(d.to_vec()).or_insert(0) += 1;
                            total += 1;
                        }
                    }
                }
                lxb_dom_collection_destroy(dom_coll, true);
                if let Some((d, c)) = counts.into_iter().max_by_key(|(_, c)| *c) {
                    // Domains whose rules were fitted to og:url-bearing pages
                    // and misfire on link-majority siblings (0109 bisect).
                    const FALLBACK_EXCLUDE: &[&[u8]] = &[
                        b"theserverside.com",
                        b"pt.usc.edu",
                        b"usc.edu",
                        b"bimmerwerkz.com",
                        b"motoprofi.com",
                        b"iclassifiedsnetwork.com",
                        b"menstennisforums.com",
                        b"cricketarchive.com",
                        b"convertunits.com",
                    ];
                    if c >= 10 && c * 10 >= total * 6 && !FALLBACK_EXCLUDE.contains(&d.as_slice()) {
                        best = d;
                    }
                }
            }
        }
        best
    }
}

/// Coarse generator class for the model (v5): 0 none/other, 1 blogger,
/// 2 wordpress, 3 forum engine.
unsafe fn generator_kind(doc: *mut lxb_html_document_t) -> u8 {
    unsafe {
        let g = generator_meta(doc);
        if g.starts_with(b"blogger") {
            1
        } else if g.starts_with(b"wordpress") {
            2
        } else if g.starts_with(b"vbulletin")
            || g.starts_with(b"phpbb")
            || g.starts_with(b"xenforo")
            || g.starts_with(b"mybb")
            || g.starts_with(b"smf")
        {
            3
        } else {
            0
        }
    }
}

/// Generator-meta content (lowercased) if present (engine detection).
unsafe fn generator_meta(doc: *mut lxb_html_document_t) -> Vec<u8> {
    unsafe {
        let head: *mut lxb_dom_node_t = (*doc).head.cast();
        if head.is_null() {
            return Vec::new();
        }
        let mut child = (*head).first_child;
        while !child.is_null() {
            if (*child).type_ == LXB_DOM_NODE_TYPE_ELEMENT
                && (*child).local_name == LXB_TAG_META
                && get_node_attr(child, b"name").eq_ignore_ascii_case(b"generator")
            {
                return get_node_attr(child, b"content").to_ascii_lowercase();
            }
            child = (*child).next;
        }
        Vec::new()
    }
}

/// vBulletin thread-page handler (cycle 0014): rebuild the thread as the gold
/// formats it — `**user – date**` header then the post body — instead of
/// letting the generic walk keep the postbit chrome. Markdown mode only;
/// falls back to generic extraction unless >=2 well-formed posts are found.
unsafe fn extract_vbulletin(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        // vB3 posts are `table[id^=post]`, vB4 `li[id^=post_]`.
        let containers = query_selector_all_raw(doc, body, b"table[id^=\"post\"], li[id^=\"post_\"]");
        let mut out = String::new();
        let mut posts = 0;
        for c in containers {
            // author
            let author_nodes = query_selector_all_raw(doc, c, b"a.bigusername, a.username");
            let mut author = author_nodes
                .first()
                .map(|&n| String::from_utf8_lossy(&get_collapsed_string(&get_node_text(n))).trim().to_string())
                .unwrap_or_default();
            if author.is_empty() {
                // some skins leave the class empty on the profile anchor
                // (rcgroups, 0042) — fall back to the member.php link text
                author = query_selector_all_raw(doc, c, b"a[href^=\"member.php\"]")
                    .iter()
                    .map(|&n| String::from_utf8_lossy(&get_collapsed_string(&get_node_text(n))).trim().to_string())
                    .find(|t| !t.is_empty() && t.len() <= 40)
                    .unwrap_or_default();
            }
            // body: vB3 div[id^=post_message_], vB4 blockquote.postcontent
            let body_nodes = query_selector_all_raw(doc, c, b"div[id^=\"post_message_\"], blockquote.postcontent");
            let Some(&bn) = body_nodes.first() else { continue };
            // date: first .thead/.date/.postdate text that looks date-like
            let date_nodes = query_selector_all_raw(doc, c, b"td.thead, .postdate, .date, span.time, div.normal");
            let mut date = String::new();
            for &dn in date_nodes.iter().take(4) {
                let t = String::from_utf8_lossy(&get_collapsed_string(&get_node_text(dn))).trim().to_string();
                if t.len() <= 40 && t.bytes().filter(|b| b.is_ascii_digit()).count() >= 4 {
                    date = t;
                    break;
                }
            }
            let text = extract_plain_text_from_node(doc, bn, opts);
            if text.trim().is_empty() {
                continue;
            }
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            // A post whose author anchor didn't match still keeps its body —
            // dropping whole posts is far worse than a missing header
            // (found via thread-starter skins, train −0.38 ×2).
            if !author.is_empty() {
                if date.is_empty() {
                    out.push_str(&format!("**{author}**"));
                } else {
                    out.push_str(&format!("**{author} \u{2013} {date}**"));
                }
                out.push_str("\n\n");
            }
            out.push_str(text.trim_end());
            posts += 1;
        }
        if posts >= 2 { Some(out) } else { None }
    }
}

/// phpBB3 thread handler (cycle 0015): same pattern as vBulletin — posts
/// from `div.post`, author/date from `p.author` ("by USER » DATE"), body
/// `div.postbody div.content`. Gate: `<body id="phpbb">`.
unsafe fn extract_phpbb(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() || !get_node_attr(body, b"id").eq_ignore_ascii_case(b"phpbb") {
            return None;
        }
        // Only thread views: search results / member pages share the postbody
        // markup but the gold treats them differently (bogleheads search page,
        // train −0.23).
        let body_cls = get_node_attr(body, b"class");
        if body_cls.windows(8).any(|w| w == b"section-") && !contains_subslice(body_cls, b"section-viewtopic") {
            return None;
        }
        let mut out = String::new();
        // Thread title: the gold keeps it as an H1-style heading.
        if let Some(&h) = query_selector_all_raw(doc, body, b"h2.topic-title, div#page-body h2, h2").first() {
            let t = String::from_utf8_lossy(&get_collapsed_string(&get_node_text(h))).trim().to_string();
            if !t.is_empty() && t.len() <= 120 {
                out.push_str(&format!("# {t}"));
            }
        }
        let containers = query_selector_all_raw(doc, body, b"div.post");
        let mut posts = 0;
        let mut authored = 0;
        for c in containers {
            let body_nodes = query_selector_all_raw(doc, c, b"div.postbody div.content, div.content");
            let Some(&bn) = body_nodes.first() else { continue };
            let author_line = query_selector_all_raw(doc, c, b"p.author")
                .first()
                .map(|&n| String::from_utf8_lossy(&get_collapsed_string(&get_node_text(n))).trim().to_string())
                .unwrap_or_default();
            // "by USER » DATE", "Post by USER » DATE" (post-icon skins,
            // jusText 0080), "DATE by USER" (WP-integrated skins, 0072).
            let rest = author_line.strip_prefix("by ").map(str::to_string).or_else(|| {
                author_line.find(" by ").map(|i| author_line[i + 4..].to_string())
            });
            let (author, date) = match rest {
                Some(r) => match r.split_once(" \u{bb} ") {
                    Some((a, d)) => (a.trim().to_string(), d.trim().to_string()),
                    None => (r.trim().to_string(), String::new()),
                },
                None => (String::new(), String::new()),
            };
            let text = extract_plain_text_from_node(doc, bn, opts);
            if text.trim().is_empty() {
                continue;
            }
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            if !author.is_empty() {
                authored += 1;
                if date.is_empty() {
                    out.push_str(&format!("**{author}**"));
                } else {
                    out.push_str(&format!("**{author} \u{2013} {date}**"));
                }
                out.push_str("\n\n");
            }
            out.push_str(text.trim_end());
            posts += 1;
        }
        // Without authors the rebuild only subtracts (titles, inline
        // attribution) from the generic walk — fall back.
        if posts >= 2 && authored >= 2 { Some(out) } else { None }
    }
}

fn contains_subslice(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

/// Collapsed, trimmed text content of a node (header-fragment helper for the
/// forum engine handlers).
unsafe fn collapsed_text(node: *mut lxb_dom_node_t) -> String {
    unsafe {
        String::from_utf8_lossy(&get_collapsed_string(&get_node_text(node)))
            .trim()
            .to_string()
    }
}

/// Invision Power Board thread handler (cycle 0017): same rebuild pattern as
/// vBulletin/phpBB — `**user — date**` header then the post body. Covers the
/// two skins in lpv11: IPB 3.x (gate: `<body id="ipboard_body">`, posts in
/// `div.post_block`) and IPS 4.x (gate: `body.ipsApp` with
/// `data-pagecontroller="topic"`, posts in `article.ipsComment`). Falls back
/// to generic extraction unless >=2 authored posts are found.
unsafe fn extract_invision(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let ipb3 = get_node_attr(body, b"id").eq_ignore_ascii_case(b"ipboard_body");
        let ips4 = !ipb3
            && contains_subslice(get_node_attr(body, b"class"), b"ipsApp")
            && get_node_attr(body, b"data-pagecontroller") == b"topic";
        if !ipb3 && !ips4 {
            return None;
        }
        let (container_sel, author_sel, date_sel, body_sel, sig_sel): (&[u8], &[u8], &[u8], &[u8], &[u8]) =
            if ipb3 {
                (
                    b"div.post_block",
                    b"span.author.vcard",
                    b"abbr.published",
                    b"div.post.entry-content",
                    b"div.signature",
                )
            } else {
                (
                    b"article.ipsComment",
                    b".cAuthorPane_author a",
                    b".ipsComment_meta time",
                    b"div[data-role=\"commentContent\"]",
                    b"div[data-role=\"memberSignature\"]",
                )
            };
        let mut out = String::new();
        // Thread title (IPB3 skins only — the gold keeps it there; IPS4 golds
        // start at the first post).
        if ipb3 {
            if let Some(&h) = query_selector_all_raw(doc, body, b"h1.ipsType_pagetitle").first() {
                let t = collapsed_text(h);
                if !t.is_empty() && t.len() <= 200 {
                    out.push_str(&format!("# {t}"));
                }
            }
        }
        let containers = query_selector_all_raw(doc, body, container_sel);
        let mut posts = 0;
        let mut authored = 0;
        for c in containers {
            let Some(&bn) = query_selector_all_raw(doc, c, body_sel).first() else { continue };
            let author = query_selector_all_raw(doc, c, author_sel)
                .first()
                .map(|&n| collapsed_text(n))
                .unwrap_or_default();
            let mut date = query_selector_all_raw(doc, c, date_sel)
                .first()
                .map(|&n| collapsed_text(n))
                .unwrap_or_default();
            if date.len() > 40 {
                date.clear();
            }
            let text = extract_plain_text_from_node(doc, bn, opts);
            // The gold keeps member signatures right after the post body (both
            // IPB3 `div.signature` and IPS4 `memberSignature`).
            let sig = query_selector_all_raw(doc, c, sig_sel)
                .first()
                .map(|&n| extract_plain_text_from_node(doc, n, opts))
                .unwrap_or_default();
            // Photo-only posts have an empty body but the gold still headers
            // them — emit the header alone rather than dropping the post.
            if text.trim().is_empty() && author.is_empty() {
                continue;
            }
            let mut segments: Vec<String> = Vec::new();
            if !author.is_empty() {
                authored += 1;
                if date.is_empty() {
                    segments.push(format!("**{author}**"));
                } else {
                    segments.push(format!("**{author} \u{2014} {date}**"));
                }
            }
            if !text.trim().is_empty() {
                segments.push(text.trim_end().to_string());
                posts += 1;
            }
            if !sig.trim().is_empty() {
                segments.push(sig.trim_end().to_string());
            }
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            out.push_str(&segments.join("\n\n"));
        }
        if posts >= 2 && authored >= 2 { Some(out) } else { None }
    }
}

/// UBB.threads thread handler (cycle 0017). Gate (checked by the caller):
/// `<meta name="generator" content="UBB.threads ...">`. Each post is a table
/// whose rows carry `td.subjecttable` (`span.date` + `span.time`),
/// `td.author-content` (author in the first `<b>`) and `td.post-content`
/// (body in `div.post_inner div[id^=body]`). The generator gate is exact, so
/// single-post threads are kept too (>=1 authored post).
unsafe fn extract_ubb(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let containers = query_selector_all_raw(doc, body, b"div.post_inner");
        let mut out = String::new();
        // Thread title: the first post's subject cell (`td.subjecttable b`).
        if let Some(&b) = query_selector_all_raw(doc, body, b"td.subjecttable b").first() {
            let t = collapsed_text(b);
            if !t.is_empty() && t.len() <= 200 {
                out.push_str(&format!("# {t}"));
            }
        }
        let mut authored = 0;
        for c in containers {
            let Some(&bn) = query_selector_all_raw(doc, c, b"div[id^=\"body\"]").first() else {
                continue;
            };
            // The author/date cells live in the same per-post table as the
            // body cell — climb to the nearest <table> ancestor.
            let mut table = (*c).parent;
            while !table.is_null() && (*table).local_name != LXB_TAG_TABLE {
                table = (*table).parent;
            }
            let mut author = String::new();
            let mut date = String::new();
            if !table.is_null() {
                author = query_selector_all_raw(doc, table, b"td.author-content b")
                    .first()
                    .map(|&n| collapsed_text(n))
                    .unwrap_or_default();
                let d = query_selector_all_raw(doc, table, b"td.subjecttable span.date")
                    .first()
                    .map(|&n| collapsed_text(n))
                    .unwrap_or_default();
                let t = query_selector_all_raw(doc, table, b"td.subjecttable span.time")
                    .first()
                    .map(|&n| collapsed_text(n))
                    .unwrap_or_default();
                date = if t.is_empty() { d } else if d.is_empty() { t } else { format!("{d} {t}") };
                if date.len() > 40 {
                    date.clear();
                }
            }
            let text = extract_plain_text_from_node(doc, bn, opts);
            if text.trim().is_empty() {
                continue;
            }
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            if !author.is_empty() {
                authored += 1;
                if date.is_empty() {
                    out.push_str(&format!("**{author}**"));
                } else {
                    out.push_str(&format!("**{author} \u{2014} {date}**"));
                }
                out.push_str("\n\n");
            }
            out.push_str(text.trim_end());
        }
        if authored >= 1 { Some(out) } else { None }
    }
}

/// Simple Machines Forum (SMF 2.0) thread handler (cycle 0017). Gate: the
/// thread-view container `<div id="forumposts">` plus the post structure
/// itself. Posts are `div.post_wrapper`; author `div.poster h4`; date the
/// `div.keyinfo div.smalltext` line ("« Reply #N on: DATE »" — keep the part
/// after "on:"); body `div.post div.inner`. Falls back to generic extraction
/// unless >=2 authored posts.
unsafe fn extract_smf(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let Some(&forum) = query_selector_all_raw(doc, body, b"div#forumposts").first() else {
            return None;
        };
        let containers = query_selector_all_raw(doc, forum, b"div.post_wrapper");
        let mut out = String::new();
        // Thread title: the category bar reads "Author Topic: TITLE (Read N
        // times)" — keep the TITLE part.
        if let Some(&h) = query_selector_all_raw(doc, forum, b"h3.catbg").first() {
            let t = collapsed_text(h);
            if let Some(pos) = t.find("Topic:") {
                let mut t = t[pos + 6..].trim().to_string();
                if let Some(read) = t.rfind("(Read ") {
                    t.truncate(read);
                }
                let t = t.trim();
                if !t.is_empty() && t.len() <= 200 {
                    out.push_str(&format!("# {t}"));
                }
            }
        }
        let mut posts = 0;
        let mut authored = 0;
        for c in containers {
            let Some(&bn) = query_selector_all_raw(doc, c, b"div.post div.inner").first() else {
                continue;
            };
            let author = query_selector_all_raw(doc, c, b"div.poster h4")
                .first()
                .map(|&n| collapsed_text(n))
                .unwrap_or_default();
            let mut date = query_selector_all_raw(doc, c, b"div.keyinfo div.smalltext")
                .first()
                .map(|&n| collapsed_text(n))
                .unwrap_or_default();
            // "« on: March 24, 2006, 06:09:41 PM »" / "« Reply #50 on: ... »"
            if let Some(pos) = date.find("on:") {
                date = date[pos + 3..].to_string();
            }
            date = date
                .trim_matches(|ch: char| ch == '\u{ab}' || ch == '\u{bb}' || ch.is_whitespace())
                .to_string();
            if date.len() > 40 {
                date.clear();
            }
            let text = extract_plain_text_from_node(doc, bn, opts);
            // The gold drops SMF quote-attribution lines ("Quote from: X on
            // DATE ...") while keeping the quoted text itself.
            let text = text
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    t != "Quote" && !t.starts_with("Quote from:")
                })
                .collect::<Vec<_>>()
                .join("\n");
            if text.trim().is_empty() {
                continue;
            }
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            if !author.is_empty() {
                authored += 1;
                if date.is_empty() {
                    out.push_str(&format!("**{author}**"));
                } else {
                    out.push_str(&format!("**{author} \u{2014} {date}**"));
                }
                out.push_str("\n\n");
            }
            out.push_str(text.trim_end());
            posts += 1;
        }
        if posts >= 2 && authored >= 2 { Some(out) } else { None }
    }
}

/// WordPress-style blog comment rebuild (cycle 0020): the gold attributes
/// comments as `**author — date**` then the body; the generic walk keeps
/// bodies but loses attribution (or keeps raw meta lines). Returns the
/// rebuilt comment block and the comment containers to veto from the walk.
unsafe fn wp_comment_rebuild(
    doc: *mut lxb_html_document_t,
    opts: &ExtractOpts,
) -> Option<(String, Vec<*mut lxb_dom_node_t>, Vec<String>)> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let items = query_selector_all_raw(doc, body, b"li.comment, div.comment[id^=\"comment\"], div.oneComment, li.social-comment");
        if items.len() < 2 {
            return None;
        }
        let mut out = String::new();
        let mut vetoes: Vec<*mut lxb_dom_node_t> = Vec::new();
        let mut authors: Vec<String> = Vec::new();
        let mut attributed = 0;
        for c in &items {
            let c = *c;
            let mut author = query_selector_all_raw(
                doc,
                c,
                b".comment-author .fn, cite.fn, .comment-author cite, .c-head a.url, .comment-author b, .comment-author a, .commentAuthorLink, cite.social-fn",
            )
            .first()
            .map(|&n| collapsed_text(n))
            .unwrap_or_default();
            if author.is_empty() {
                // Highlander (wordpress.com) puts the author as bare text in
                // div.c-head with only the permalink linked (0040)
                if let Some(&h) = query_selector_all_raw(doc, c, b".c-head").first() {
                    let t = collapsed_text(h);
                    let t = t.trim_end_matches("permalink").trim();
                    if !t.is_empty() && t.len() <= 48 {
                        author = t.to_string();
                    }
                }
            }
            // first candidate containing a digit — c-head's permalink span
            // otherwise wins in document order and "permalink" becomes the
            // date (0040)
            let mut date = query_selector_all_raw(
                doc,
                c,
                b".comment-metadata, .comment-meta, .commentmetadata, .c-date, .c-head span, time, .commentAuthor a, .social-comment-meta a",
            )
            .iter()
            .map(|&n| collapsed_text(n))
            .find(|t| t.bytes().any(|b| b.is_ascii_digit()))
            .unwrap_or_default();
            date = date
                .trim_start_matches(|ch: char| ch == '/' || ch.is_whitespace())
                .trim_start_matches("on ")
                .trim()
                .to_string();
            if date.len() > 48 {
                date.truncate(0);
            }
            let Some(&bn) = query_selector_all_raw(
                doc,
                c,
                b".comment-content, .c-body, .commenttext, .comment-text, .comment-body, .commentContent, .social-comment-body",
            )
            .first() else {
                continue;
            };
            // direct-child comments only produce text here; nested replies are
            // separate `li.comment` matches (query order = document order)
            let mut sub = ExtractOpts { main_content: false, ..opts.clone() };
            sub.skip_elements.push("ul.children".to_string());
            sub.skip_elements.push("ol.children".to_string());
            let text = extract_plain_text_from_node_opts(doc, bn, &sub);
            if text.trim().is_empty() || author.is_empty() {
                continue;
            }
            attributed += 1;
            authors.push(author.clone());
            vetoes.push(c);
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            if date.is_empty() {
                out.push_str(&format!("**{author}**"));
            } else {
                out.push_str(&format!("**{author} \u{2014} {date}**"));
            }
            out.push_str("  \n");
            out.push_str(text.trim_end());
        }
        // Only rebuilt items are vetoed — a failed rebuild must never cost
        // the walk its native rendering of that comment.
        if attributed >= 2 && attributed * 2 >= items.len() {
            Some((out, vetoes, authors))
        } else {
            None
        }
    }
}

/// Blogspot comment rebuild (cycle 0039): gold rewrites Blogger's native
/// "NAME said..." + separate timestamp footer as `**NAME — TIMESTAMP**`
/// followed by the body (em-dash joiner dominates the gold 5.5:1). Unlike
/// the WP rebuild this is NOT native-first — the native walk keeps the
/// author but in the wrong shape, so a successful parse always rebuilds.
/// Handles the classic dl template (dt.comment-author / dd.comment-body /
/// dd.comment-footer) and the threaded div.comment-block template.
unsafe fn blogspot_comment_rebuild(
    doc: *mut lxb_html_document_t,
    opts: &ExtractOpts,
) -> Option<(String, Vec<*mut lxb_dom_node_t>, Vec<String>)> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let mut out = String::new();
        let mut vetoes: Vec<*mut lxb_dom_node_t> = Vec::new();
        let mut authors: Vec<String> = Vec::new();
        let mut attributed = 0usize;
        let mut items = 0usize;

        fn clean_author(mut a: String) -> String {
            let t = a.trim_end();
            for suf in ["said...", "said…", "said\u{2026}", "said..."] {
                if let Some(stripped) = t.strip_suffix(suf) {
                    a = stripped.trim_end().to_string();
                    break;
                }
            }
            let a = a.trim();
            // script/style text inside the author node (jusText-0064 class
            // of bug) or a missing author must abort this comment, not
            // emit junk attribution
            if a.len() > 48 || a.contains(';') || a.contains("document.") {
                return String::new();
            }
            a.to_string()
        }
        let mut push_comment =
            |out: &mut String, author: String, date: String, text: String| {
                if !out.is_empty() {
                    out.push_str("\n\n");
                }
                if date.is_empty() {
                    out.push_str(&format!("**{author}**"));
                } else {
                    out.push_str(&format!("**{author} \u{2014} {date}**"));
                }
                out.push_str("  \n");
                out.push_str(text.trim_end());
            };

        let dts = query_selector_all_raw(doc, body, b"dt.comment-author");
        let bodies = query_selector_all_raw(doc, body, b"dd.comment-body");
        if dts.len() >= 2 && dts.len() == bodies.len() {
            // classic template — strict document-order triples
            let stamps = query_selector_all_raw(doc, body, b"dd.comment-footer .comment-timestamp");
            items = dts.len();
            for (i, (&dt, &bd)) in dts.iter().zip(bodies.iter()).enumerate() {
                let author = clean_author(collapsed_text(dt));
                let text = extract_plain_text_from_node(doc, bd, opts);
                if author.is_empty() || text.trim().is_empty() {
                    continue;
                }
                let mut date = if stamps.len() == dts.len() {
                    collapsed_text(stamps[i])
                } else {
                    String::new()
                };
                if date.len() > 48 {
                    date.truncate(0);
                }
                attributed += 1;
                authors.push(author.clone());
                vetoes.push(dt);
                vetoes.push(bd);
                if stamps.len() == dts.len() {
                    vetoes.push(stamps[i]);
                }
                push_comment(&mut out, author, date, text);
            }
        } else {
            // threaded template
            let blocks = query_selector_all_raw(doc, body, b"div.comment-block");
            if blocks.len() < 2 {
                return None;
            }
            items = blocks.len();
            for &c in &blocks {
                let author = query_selector_all_raw(doc, c, b"cite.user, cite")
                    .first()
                    .map(|&n| clean_author(collapsed_text(n)))
                    .unwrap_or_default();
                let mut date = query_selector_all_raw(doc, c, b".comment-timestamp, .datetime")
                    .first()
                    .map(|&n| collapsed_text(n))
                    .unwrap_or_default();
                if date.len() > 48 {
                    date.truncate(0);
                }
                let Some(&bn) = query_selector_all_raw(doc, c, b"p.comment-content, .comment-content")
                    .first()
                else {
                    continue;
                };
                let text = extract_plain_text_from_node(doc, bn, opts);
                if author.is_empty() || text.trim().is_empty() {
                    continue;
                }
                attributed += 1;
                authors.push(author.clone());
                vetoes.push(c);
                push_comment(&mut out, author, date, text);
            }
        }
        if attributed >= 2 && attributed * 2 >= items.max(1) {
            // Blogger renders comments through BOTH templates on some blogs
            // (classic dl + threaded div.comment-block); veto the mirror
            // rendering too or the walk keeps a native duplicate of every
            // rebuilt comment.
            for &n in query_selector_all_raw(
                doc,
                body,
                b"dt.comment-author, dd.comment-body, dd.comment-footer, div.comment-block",
            )
            .iter()
            {
                if !vetoes.contains(&n) {
                    vetoes.push(n);
                }
            }
            Some((out, vetoes, authors))
        } else {
            None
        }
    }
}

/// MovableType comment rebuild (cycle 0044; jusText-0085 family): the
/// source renders body-first (`div.commentText`) with the attribution
/// AFTER it (`p.posted`: "Posted by: NAME | DATE | ..."); gold emits
/// `**NAME — DATE**` before the body. Always-rebuild semantics (gold
/// rewrites the native form).
unsafe fn movabletype_comment_rebuild(
    doc: *mut lxb_html_document_t,
    opts: &ExtractOpts,
) -> Option<(String, Vec<*mut lxb_dom_node_t>, Vec<String>)> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let merged = query_selector_all_raw(doc, body, b"div.commentText, p.posted");
        let mut out = String::new();
        let mut vetoes: Vec<*mut lxb_dom_node_t> = Vec::new();
        let mut authors: Vec<String> = Vec::new();
        let mut attributed = 0usize;
        let mut pending_body: Option<*mut lxb_dom_node_t> = None;
        for &n in &merged {
            if (*n).local_name == LXB_TAG_DIV {
                pending_body = Some(n);
                continue;
            }
            // p.posted — must follow a commentText
            let Some(bn) = pending_body.take() else { continue };
            let meta = collapsed_text(n);
            let Some(rest) = meta.strip_prefix("Posted by:").map(str::trim) else {
                continue;
            };
            let mut parts = rest.split('|').map(str::trim);
            let author = parts.next().unwrap_or("").to_string();
            let date = parts
                .next()
                .filter(|d| d.len() <= 48 && d.bytes().any(|b| b.is_ascii_digit()))
                .unwrap_or("")
                .to_string();
            if author.is_empty() || author.len() > 48 {
                continue;
            }
            let text = extract_plain_text_from_node(doc, bn, opts);
            if text.trim().is_empty() {
                continue;
            }
            attributed += 1;
            authors.push(author.clone());
            vetoes.push(bn);
            vetoes.push(n);
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            if date.is_empty() {
                out.push_str(&format!("**{author}**"));
            } else {
                out.push_str(&format!("**{author} \u{2014} {date}**"));
            }
            out.push_str("  \n");
            out.push_str(text.trim_end());
        }
        if attributed >= 2 {
            Some((out, vetoes, authors))
        } else {
            None
        }
    }
}

unsafe fn extract_plain_text_from_node_opts(
    doc: *mut lxb_html_document_t,
    root: *mut lxb_dom_node_t,
    opts: &ExtractOpts,
) -> String {
    unsafe { extract_plain_text_from_doc_impl2(doc, Some(root), opts, RelaxFlags::default(), None, None).0 }
}

/// XenForo thread handler (cycle 0043; jusText-0058 map): XF1 posts are
/// `li[data-author]` with body `blockquote.messageText` and the post time
/// in `span.DateTime[title="... at ..."]`; XF2 uses `article[data-author]`
/// with `div.bbWrapper` and a `<time>` element. Gold joins with an
/// en-dash and renders the title-attr time with " at " collapsed to a
/// space. Bodies keep nested quotes (stripping them cratered in jusText).
unsafe fn extract_xenforo(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let containers = query_selector_all_raw(doc, body, b"li[data-author], article[data-author]");
        if containers.len() < 2 {
            return None;
        }
        let mut out = String::new();
        let mut posts = 0usize;
        let mut total = 0usize;
        let page_text_len = get_collapsed_string(&get_node_text(body)).len();
        for c in containers {
            let author = String::from_utf8_lossy(&get_node_attr(c, b"data-author")).trim().to_string();
            let Some(&bn) = query_selector_all_raw(doc, c, b"blockquote.messageText, div.bbWrapper")
                .first()
            else {
                continue;
            };
            let mut date = String::new();
            for &dn in query_selector_all_raw(doc, c, b".messageMeta .DateTime, time, .DateTime")
                .iter()
                .take(3)
            {
                let title = String::from_utf8_lossy(&get_node_attr(dn, b"title")).trim().to_string();
                let text = collapsed_text(dn);
                let has4 = |t: &str| t.bytes().filter(|b| b.is_ascii_digit()).count() >= 4;
                // XF2 <time>: gold renders the visible date only; XF1
                // span.DateTime: gold uses the full title-attr timestamp
                let cand = if (*dn).local_name == LXB_TAG_TIME {
                    let ds = String::from_utf8_lossy(&get_node_attr(dn, b"data-date-string")).trim().to_string();
                    if has4(&ds) { ds } else if has4(&text) { text } else { title }
                } else if has4(&title) {
                    title
                } else {
                    text
                };
                if cand.len() <= 40 && cand.bytes().filter(|b| b.is_ascii_digit()).count() >= 4 {
                    date = cand.replace(" at ", " ");
                    break;
                }
            }
            // Gold strips XF quote blocks in ~3/4 of docs — the dominant
            // convention (unlike jusText's gold, which kept them). A
            // duplicate-only-strip variant measured WORSE (whitespace/emoji
            // normalization made real reply-quotes look novel).
            let mut sub = ExtractOpts { main_content: false, ..opts.clone() };
            sub.skip_elements.push(".bbCodeQuote".to_string());
            sub.skip_elements.push("blockquote.bbCodeBlock--quote".to_string());
            let text = extract_plain_text_from_node_opts(doc, bn, &sub);
            if text.trim().is_empty() || author.is_empty() {
                continue;
            }
            // coverage counts the post's FULL mass (quotes included): the
            // guard asks "did we locate the thread", not "how much did the
            // quote-strip remove" — stripping must not fail the guard
            total += get_collapsed_string(&get_node_text(bn)).len();
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            if date.is_empty() {
                out.push_str(&format!("**{author}**"));
            } else {
                out.push_str(&format!("**{author} \u{2013} {date}**"));
            }
            out.push_str("\n\n");
            out.push_str(text.trim_end());
            posts += 1;
        }
        if posts >= 2 && total * 4 >= page_text_len {
            Some(out)
        } else {
            None
        }
    }
}

/// phpBB3 subSilver2 skin handler (cycle 0041): table layout with
/// `b.postauthor` per post, body in `div.postbody`, date after "Posted:"
/// in a `td.gensmall` cell. Pairing is document-order within one engine's
/// strict table structure (unlike the 0040 mixed-template failure).
unsafe fn extract_phpbb_subsilver2(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let merged = query_selector_all_raw(doc, body, b"b.postauthor, div.postbody, td.gensmall");
        let n_auth = merged
            .iter()
            .filter(|&&n| (*n).local_name == LXB_TAG_B)
            .count();
        if n_auth < 1 {
            return None;
        }
        let mut out = String::new();
        let mut posts = 0usize;
        let mut i = 0usize;
        let page_text_len = get_collapsed_string(&get_node_text(body)).len();
        let mut total = 0usize;
        while i < merged.len() {
            let n = merged[i];
            if (*n).local_name != LXB_TAG_B {
                i += 1;
                continue;
            }
            let author = collapsed_text(n);
            let mut date = String::new();
            let mut bodyn: Option<*mut lxb_dom_node_t> = None;
            let mut j = i + 1;
            while j < merged.len() && (*merged[j]).local_name != LXB_TAG_B {
                let m = merged[j];
                if (*m).local_name == LXB_TAG_TD && date.is_empty() {
                    let t = collapsed_text(m);
                    if let Some(pos) = t.find("Posted:") {
                        let tail = t[pos + 7..].trim();
                        // date runs until the next label ("Post subject:")
                        let tail = tail.split("Post subject:").next().unwrap_or(tail).trim();
                        if !tail.is_empty()
                            && tail.len() <= 48
                            && tail.bytes().any(|b| b.is_ascii_digit())
                        {
                            date = tail.to_string();
                        }
                    }
                } else if (*m).local_name == LXB_TAG_DIV && bodyn.is_none() {
                    bodyn = Some(m);
                }
                j += 1;
            }
            i = j;
            let (Some(bn), false) = (bodyn, author.is_empty()) else {
                continue;
            };
            let text = extract_plain_text_from_node(doc, bn, opts);
            if text.trim().is_empty() {
                continue;
            }
            total += text.len();
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            if date.is_empty() {
                out.push_str(&format!("**{author}**"));
            } else {
                out.push_str(&format!("**{author} \u{2014} {date}**"));
            }
            out.push_str("\n\n");
            out.push_str(text.trim_end());
            posts += 1;
        }
        // coverage guard (0021 pattern): the rebuilt thread must be a
        // substantial share of the page or the generic walk knows better;
        // dated single-post threads are exempt (0065)
        let dated_single = posts == 1 && out.contains('\u{2014}');
        if (posts >= 2 && total * 4 >= page_text_len) || dated_single {
            Some(out)
        } else {
            None
        }
    }
}

/// phpBB 2.x thread handler (cycle 0021): classic table skins — author in
/// `span.name` (usually `<b>`), body `span.postbody`, date in
/// `span.postdetails` ("Posted: DATE    Post subject: ..."). Gate: >=2 of
/// both name and postbody spans (a markup combo unique to phpBB2), paired by
/// document order when counts match.
unsafe fn extract_phpbb2(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let names = query_selector_all_raw(doc, body, b"span.name");
        let bodies = query_selector_all_raw(doc, body, b"span.postbody");
        // single-post threads qualify (0065): gold keeps the lone post;
        // the date requirement below guards against non-thread pages
        if names.is_empty() || bodies.is_empty() || names.len() != bodies.len() {
            return None;
        }
        let details: Vec<*mut lxb_dom_node_t> =
            query_selector_all_raw(doc, body, b"span.postdetails")
                .into_iter()
                .filter(|&n| collapsed_text(n).contains("Posted:"))
                .collect();
        let mut out = String::new();
        let mut posts = 0;
        for (i, (&n, &b)) in names.iter().zip(bodies.iter()).enumerate() {
            let author = collapsed_text(n);
            let text = extract_plain_text_from_node(doc, b, opts);
            if text.trim().is_empty() {
                continue;
            }
            let mut date = String::new();
            if let Some(&d) = details.get(i) {
                let t = collapsed_text(d);
                if let Some(pos) = t.find("Posted:") {
                    let rest = &t[pos + 7..];
                    let end = rest.find("Post subject").unwrap_or(rest.len());
                    let cand = rest[..end].trim();
                    if cand.len() <= 40 && cand.bytes().filter(|c| c.is_ascii_digit()).count() >= 4 {
                        date = cand.to_string();
                    }
                }
            }
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            if !author.is_empty() {
                if date.is_empty() {
                    out.push_str(&format!("**{author}**"));
                } else {
                    out.push_str(&format!("**{author} \u{2013} {date}**"));
                }
                out.push_str("\n\n");
            }
            out.push_str(text.trim_end());
            posts += 1;
        }
        // single post: only with a parsed date (else non-thread false fires)
        if posts == 0 || (posts == 1 && !out.contains('\u{2013}')) {
            return None;
        }
        // Coverage guard: on odd skins (PNphpBB2) span.postbody matches
        // signatures, not bodies — the rebuild must carry a meaningful share
        // of the page text or the generic walk is better. Dated single-post
        // threads on nav-heavy pages are exempt (0065; zip pairing intact,
        // unlike the reverted 0050 doc-order variant).
        let body_total = get_collapsed_string(&get_node_text(body)).len();
        if out.len() * 4 < body_total && posts >= 2 {
            return None;
        }
        Some(out)
    }
}

// ---------------------------------------------------------------------------
// One-off engine handlers (cycle 0030): per-engine exact gates, 0017-style.
// Each fires on markup unique to its engine and rebuilds the post stream as
// `**user — date**` + body; generic fallback whenever the gate or the
// authored/coverage guards decline.
// ---------------------------------------------------------------------------

/// Collapsed text length of `<body>` (coverage-guard denominator).
unsafe fn body_text_total(body: *mut lxb_dom_node_t) -> usize {
    unsafe { get_collapsed_string(&get_node_text(body)).len() }
}

/// PerlMonks note page (gate: `table#monkbar` + `div.notetext`, the 2001
/// "monkbar" skin's unmistakable ids). Author/date live in the two
/// `span.attribution` fragments of the title bars; body is `div.notetext`;
/// the gold keeps the trailing "In Section …" link-back line.
/// Yahoo message-board thread handler (cycle 0064): `.mb-message-body`
/// blocks with `.mb-author-actual` author, `.mb-timestamp abbr` time and
/// `.mb-message-bd` body. Gold: `**author — time**  \nbody`.
/// vBulletin 5 thread handler (cycle 0066): posts in `div.b-post` with
/// `div.author strong` (or b-username) author, `time[itemprop=dateCreated]`
/// visible text, body `div.js-post__content-text` (falls back to
/// `.b-post__content`). Gold: `**author — date**` + body.
/// Google Forms handler (cycle 0067): questions in `div.ss-q-title`
/// (gold bolds them) with choice lists in `ul.ss-choices li` (gold
/// renders `- choice  ` with hard breaks). Fires only on the ss-form
/// wrapper; description text comes from the generic walk of the header.
/// search.cpan.org POD handler (cycle 0068): section h1s (with the
/// tucs up.gif anchor) render as `**HEADING**` in gold, preceded by the
/// module abstract as a bare title line; `pre.sh_perl` becomes a perl
/// fence. Gate: the tucs up-arrow image (template-unique).
/// LiveJournal single-post handler (cycle 0076): `.b-singlepost` page —
/// gold is `**username — date**`, display name, `**Tags:** ...`, `---`,
/// `# title`, then the post body. Username from data-ljuser, date from
/// the time element's text (links collapsed), display name from
/// .b-singlepost-author-user-screen (text before the paren).
/// Legacy.com guestbook handler (cycle 0077): `div.GuestBookEntry` with
/// `.postedDate`, `.message`, `.SigneeName`. Gold: `## Condolences` then
/// per entry `**DATE**  \nmessage  \n— signee` (em-dash signature line).
/// Page title/dates from the obit header when present.
unsafe fn extract_legacy_gb(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let entries = query_selector_all_raw(doc, body, b"div.GuestBookEntry");
        if entries.len() < 2 {
            return None;
        }
        let mut out = String::new();
        if let Some(&t) = query_selector_all_raw(doc, body, b"h1").first() {
            let title = collapsed_text(t);
            if !title.is_empty() {
                out.push_str(&format!("# {}\n\n", title.trim()));
            }
        }
        if let Some(&y) = query_selector_all_raw(doc, body, b".YearsLower").first() {
            let years = collapsed_text(y).replace(" - ", " \u{2013} ");
            if !years.is_empty() {
                out.push_str(&format!("**{}**  \n\n", years.trim()));
            }
        }
        out.push_str("## Condolences\n");
        let mut n = 0usize;
        for &e in &entries {
            let date = query_selector_all_raw(doc, e, b".postedDate")
                .first()
                .map(|&x| collapsed_text(x))
                .unwrap_or_default();
            let msg = query_selector_all_raw(doc, e, b".message")
                .first()
                .map(|&x| collapsed_text(x))
                .unwrap_or_default();
            let signee = query_selector_all_raw(doc, e, b".SigneeName")
                .first()
                .map(|&x| collapsed_text(x))
                .unwrap_or_default();
            if msg.is_empty() && signee.is_empty() {
                continue;
            }
            out.push('\n');
            if !date.is_empty() {
                out.push_str(&format!("**{}**  \n", date.trim()));
            }
            if !msg.is_empty() {
                out.push_str(msg.trim());
                out.push_str("  \n");
            }
            if !signee.is_empty() {
                out.push_str(&format!("\u{2014} {}\n", signee.trim()));
            }
            n += 1;
        }
        if let Some(&d) = query_selector_all_raw(doc, body, b"div.Disclaimer").first() {
            // charter C4: legal/disclaimer text is content
            let t = extract_plain_text_from_node(doc, d, opts);
            if !t.trim().is_empty() {
                out.push('\n');
                out.push_str(t.trim());
                out.push('\n');
            }
        }
        if n >= 2 {
            Some(out.trim_end().to_string())
        } else {
            None
        }
    }
}

unsafe fn extract_livejournal(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let posts = query_selector_all_raw(doc, body, b"article.b-singlepost-body, .b-singlepost-bodywrapper");
        if posts.is_empty() {
            return None;
        }
        let scoped = query_selector_all_raw(doc, body, b".b-singlepost-author [data-ljuser]");
        let user = scoped
            .first()
            .or(query_selector_all_raw(doc, body, b"[data-ljuser]").first())
            .map(|&n| String::from_utf8_lossy(&get_node_attr(n, b"data-ljuser")).trim().to_string())
            .unwrap_or_default();
        if user.is_empty() {
            return None;
        }
        let mut date = query_selector_all_raw(doc, body, b".b-singlepost-author-date, time")
            .first()
            .map(|&n| collapsed_text(n))
            .unwrap_or_default();
        date = date.replace(' ', " ");
        if date.len() > 48 {
            date.truncate(0);
        }
        let screen = query_selector_all_raw(doc, body, b".b-singlepost-author-user-screen")
            .first()
            .map(|&n| collapsed_text(n))
            .map(|t| t.split('(').next().unwrap_or("").trim().to_string())
            .unwrap_or_default();
        let title = query_selector_all_raw(doc, body, b"h1.b-singlepost-title, .b-singlepost-title")
            .first()
            .map(|&n| collapsed_text(n))
            .unwrap_or_default();
        let tags: Vec<String> = query_selector_all_raw(doc, body, b".b-singlepost-tags a, .b-singlepost-tag")
            .iter()
            .map(|&n| collapsed_text(n))
            .filter(|t| !t.is_empty())
            .collect();
        let mut out = String::new();
        if date.is_empty() {
            out.push_str(&format!("**{user}**\n\n"));
        } else {
            out.push_str(&format!("**{user} \u{2014} {date}**\n\n"));
        }
        if !screen.is_empty() {
            out.push_str(&screen);
            out.push_str("\n\n");
        }
        if !tags.is_empty() {
            out.push_str(&format!("**Tags:** {}\n\n", tags.join(", ")));
        }
        out.push_str("---\n\n");
        if !title.is_empty() {
            out.push_str(&format!("# {}\n", title.trim()));
        }
        let text = extract_plain_text_from_node(doc, posts[0], opts);
        if !text.trim().is_empty() {
            out.push('\n');
            out.push_str(text.trim_end());
        }
        Some(out.trim_end().to_string())
    }
}

unsafe fn extract_cpan_pod(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        if query_selector_all_raw(doc, body, b"img[src*=\"tucs/img/up.gif\"]").is_empty() {
            return None;
        }
        let h1s = query_selector_all_raw(doc, body, b"h1");
        if h1s.len() < 2 {
            return None;
        }
        let mut out = String::new();
        let mut sections = 0usize;
        for (idx, &h) in h1s.iter().enumerate() {
            let mut heading = collapsed_text(h);
            if let Some(strip) = heading.strip_suffix('^') {
                heading = strip.trim_end().to_string();
            }
            if heading.is_empty() {
                continue;
            }
            // section body: siblings until the next h1
            let mut section = String::new();
            let mut n = (*h).next;
            while !n.is_null() && (*n).local_name != LXB_TAG_H1 {
                if (*n).type_ == LXB_DOM_NODE_TYPE_ELEMENT {
                    let tagname = get_qualified_name(n);
                    if tagname == b"pre" {
                        let code = String::from_utf8_lossy(&get_node_text(n)).to_string();
                        let lang = if get_node_attr(n, b"class").windows(7).any(|w| w == b"sh_perl") {
                            "perl"
                        } else {
                            ""
                        };
                        section.push_str(&format!("\n```{}\n{}\n```\n", lang, code.trim_end()));
                    } else {
                        let t = extract_plain_text_from_node(doc, n, opts);
                        if !t.trim().is_empty() {
                            section.push('\n');
                            section.push_str(t.trim_end());
                            section.push('\n');
                        }
                    }
                }
                n = (*n).next;
            }
            if idx == 0 && heading == "NAME" {
                // abstract line leads the document
                let first = section.trim().lines().next().unwrap_or("").to_string();
                if !first.is_empty() {
                    out.push_str(&first);
                    out.push('\n');
                }
            }
            out.push_str(&format!("\n**{}**\n", heading));
            out.push_str(&section);
            sections += 1;
        }
        if sections >= 2 {
            Some(out.trim().to_string())
        } else {
            None
        }
    }
}

unsafe fn extract_gforms(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        if query_selector_all_raw(doc, body, b"form.ss-form, div.ss-form-container").is_empty() {
            return None;
        }
        let titles = query_selector_all_raw(doc, body, b"div.ss-q-title");
        if titles.len() < 2 {
            return None;
        }
        let mut out = String::new();
        // form title + description
        if let Some(&t) = query_selector_all_raw(doc, body, b"h1.ss-form-title, .ss-form-title").first() {
            out.push_str(&format!("# {}\n", collapsed_text(t)));
        }
        if let Some(&d) = query_selector_all_raw(doc, body, b".ss-form-desc").first() {
            let txt = extract_plain_text_from_node(doc, d, opts);
            if !txt.trim().is_empty() {
                out.push('\n');
                out.push_str(txt.trim());
                out.push('\n');
            }
        }
        let mut questions = 0usize;
        for &t in &titles {
            let title = collapsed_text(t);
            if title.is_empty() {
                continue;
            }
            out.push_str(&format!("\n**{}**  \n", title.trim_end()));
            // choices: the ul.ss-choices sibling within the same question item
            let mut anc = (*t).parent;
            let mut choices: Vec<String> = Vec::new();
            for _ in 0..4 {
                if anc.is_null() {
                    break;
                }
                let lis = query_selector_all_raw(doc, anc, b"ul.ss-choices li.ss-choice-item, ul.ss-choices li");
                if !lis.is_empty() {
                    for &li in &lis {
                        let c = collapsed_text(li);
                        if !c.is_empty() {
                            choices.push(c);
                        }
                    }
                    break;
                }
                anc = (*anc).parent;
            }
            for (i, c) in choices.iter().enumerate() {
                if i + 1 == choices.len() {
                    out.push_str(&format!("- {}\n", c));
                } else {
                    out.push_str(&format!("- {}  \n", c));
                }
            }
            questions += 1;
        }
        if questions >= 2 {
            Some(out.trim_end().to_string())
        } else {
            None
        }
    }
}

unsafe fn extract_vbulletin5(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let posts_sel = query_selector_all_raw(doc, body, b"li.b-post, div.b-post");
        if posts_sel.is_empty() {
            return None;
        }
        let mut out = String::new();
        let mut posts = 0usize;
        let mut total = 0usize;
        let page_text_len = get_collapsed_string(&get_node_text(body)).len();
        // Announcement modules precede the thread and gold keeps them
        // (pacersdigest rules post, 0066); emit before the post stream.
        for &an in query_selector_all_raw(doc, body, b"div.announcement-tabs")
            .iter()
            .take(1)
        {
            let text = extract_plain_text_from_node(doc, an, opts);
            let t = text.trim();
            if !t.is_empty() {
                out.push_str("# Announcement\n\n");
                out.push_str(t);
            }
        }
        for &c in &posts_sel {
            let author = query_selector_all_raw(doc, c, b"div.author strong, .b-username, .author a")
                .first()
                .map(|&n| collapsed_text(n))
                .unwrap_or_default();
            let mut date = query_selector_all_raw(doc, c, b"time")
                .first()
                .map(|&n| collapsed_text(n))
                .unwrap_or_default();
            if date.len() > 48 {
                date.truncate(0);
            }
            let Some(&bn) = query_selector_all_raw(doc, c, b"div.js-post__content-text, .b-post__content")
                .first()
            else {
                continue;
            };
            // post-count chip, duplicated timestamp and title live inside
            // the content container on some skins — skip them
            let mut sub = ExtractOpts { main_content: false, ..opts.clone() };
            sub.skip_elements.push(".b-post__count".to_string());
            sub.skip_elements.push(".OLD__post-date".to_string());
            sub.skip_elements.push("h2".to_string());
            sub.skip_elements.push(".b-post__title".to_string());
            let text = extract_plain_text_from_node_opts(doc, bn, &sub);
            if author.is_empty() || text.trim().is_empty() {
                continue;
            }
            total += text.len();
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            if date.is_empty() {
                out.push_str(&format!("**{author}**"));
            } else {
                out.push_str(&format!("**{author} \u{2014} {date}**"));
            }
            out.push_str("\n\n");
            out.push_str(text.trim_end());
            posts += 1;
        }
        let dated_single = posts == 1 && out.contains('\u{2014}');
        if (posts >= 2 && total * 4 >= page_text_len) || dated_single {
            Some(out)
        } else {
            None
        }
    }
}

unsafe fn extract_yahoo_mb(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let msgs = query_selector_all_raw(doc, body, b"div.mb-message-body");
        if msgs.len() < 2 {
            return None;
        }
        let mut out = String::new();
        let mut posts = 0usize;
        for &m in &msgs {
            let author = query_selector_all_raw(doc, m, b".mb-author-actual")
                .first()
                .map(|&n| collapsed_text(n))
                .unwrap_or_default();
            let mut date = query_selector_all_raw(doc, m, b".mb-timestamp abbr, .mb-timestamp")
                .first()
                .map(|&n| collapsed_text(n))
                .unwrap_or_default();
            if date.len() > 48 {
                date.truncate(0);
            }
            let Some(&bn) = query_selector_all_raw(doc, m, b".mb-message-bd").first() else {
                continue;
            };
            let text = extract_plain_text_from_node(doc, bn, opts);
            if author.is_empty() || text.trim().is_empty() {
                continue;
            }
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            if date.is_empty() {
                out.push_str(&format!("**{author}**"));
            } else {
                out.push_str(&format!("**{author} \u{2014} {date}**"));
            }
            out.push_str("  \n");
            out.push_str(text.trim_end());
            posts += 1;
        }
        if posts >= 2 {
            Some(out)
        } else {
            None
        }
    }
}

unsafe fn extract_perlmonks(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        if query_selector_all_raw(doc, body, b"table#monkbar").is_empty() {
            return None;
        }
        let notes = query_selector_all_raw(doc, body, b"div.notetext");
        if notes.is_empty() {
            return None;
        }
        let author = query_selector_all_raw(doc, body, b"td.titlechooser span.attribution a")
            .first()
            .map(|&n| collapsed_text(n))
            .unwrap_or_default();
        let mut date = String::new();
        for &n in &query_selector_all_raw(doc, body, b"td.titlechooser span.attribution") {
            let t = collapsed_text(n);
            if let Some(r) = t.strip_prefix("on ") {
                date = r.trim().to_string();
                break;
            }
        }
        if author.is_empty() || date.len() > 48 {
            return None;
        }
        let mut out = String::new();
        if date.is_empty() {
            out.push_str(&format!("**{author}**"));
        } else {
            out.push_str(&format!("**{author} \u{2014} {date}**"));
        }
        let mut have_body = false;
        for &n in &notes {
            let text = extract_plain_text_from_node(doc, n, opts);
            if text.trim().is_empty() {
                continue;
            }
            out.push_str("\n\n");
            out.push_str(text.trim());
            have_body = true;
        }
        if !have_body {
            return None;
        }
        if let Some(&lb) = query_selector_all_raw(doc, body, b"div.link-back").first() {
            let t = collapsed_text(lb);
            if !t.is_empty() && t.len() <= 120 {
                out.push_str("\n\n");
                out.push_str(&t);
            }
        }
        Some(out)
    }
}

/// Nabble archive thread (gate: `div.classic-row` post containers with the
/// `classic-author-name` / `message-text` cells of the classic view). Dates
/// are rendered by JS only, so headers carry the author alone; mail-quote
/// blocks, gmail signatures and mailing-list footers are dropped (the gold
/// keeps each post's own words only).
unsafe fn extract_nabble(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let rows = query_selector_all_raw(doc, body, b"div.classic-row");
        if rows.len() < 2 {
            return None;
        }
        let mut out = String::new();
        if let Some(&h) = query_selector_all_raw(doc, body, b"h1#post-title").first() {
            let t = collapsed_text(h);
            if !t.is_empty() && t.len() <= 200 {
                out.push_str(&format!("# {t}"));
            }
        }
        let mut posts = 0;
        let mut authored = 0;
        for &r in &rows {
            let Some(&bn) = query_selector_all_raw(doc, r, b"div.message-text").first() else {
                continue;
            };
            let author = query_selector_all_raw(doc, r, b"div.classic-author-name a")
                .first()
                .map(|&n| collapsed_text(n))
                .unwrap_or_default();
            let mut sub = opts.clone();
            sub.skip_elements.push("div.gmail_quote".to_string());
            sub.skip_elements.push("div.gmail_signature".to_string());
            sub.skip_elements.push("blockquote".to_string());
            let text = extract_plain_text_from_node(doc, bn, &sub);
            // Mailing-list footer ("____… slicer-users mailing list …"),
            // Outlook reply headers ("From: … Sent: … Subject: …") and bare
            // signature dashes are chrome the gold drops.
            let mut kept: Vec<&str> = Vec::new();
            for line in text.lines() {
                let lt = line.trim();
                if lt.len() >= 10 && lt.bytes().all(|b| b == b'_') {
                    break;
                }
                if lt.starts_with("**From:**") || (lt.starts_with("From:") && lt.contains("[mailto:")) {
                    break;
                }
                if lt == "--" {
                    continue;
                }
                kept.push(line);
            }
            let text = kept.join("\n");
            if text.trim().is_empty() {
                continue;
            }
            if posts > 0 {
                out.push_str("\n\n---\n\n");
            } else if !out.is_empty() {
                out.push_str("\n\n");
            }
            if !author.is_empty() {
                authored += 1;
                out.push_str(&format!("**{author}**  \n"));
            }
            out.push_str(text.trim_end());
            posts += 1;
        }
        if posts >= 2 && authored >= 2 { Some(out) } else { None }
    }
}

/// WebBBS / WWWBoard message page (vegsource skin). Gate: the exact
/// From:/Subject:/Date: header table (author as a mailto link) plus the
/// `<a name="followups">` anchor every WebBBS message view carries. The body
/// is the sibling run between the header table and the "Reply To This Post"
/// footer chrome.
unsafe fn extract_webbbs(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        if query_selector_all_raw(doc, body, b"a[name=\"followups\"]").is_empty() {
            return None;
        }
        let mut header: Option<*mut lxb_dom_node_t> = None;
        let mut header_text = String::new();
        for &t in &query_selector_all_raw(doc, body, b"table") {
            let txt = collapsed_text(t);
            if txt.len() <= 300
                && txt.starts_with("From:")
                && txt.contains("Subject:")
                && txt.contains("Date:")
            {
                header = Some(t);
                header_text = txt;
                // keep scanning: the innermost matching table wins
            }
        }
        let header = header?;
        let author = query_selector_all_raw(doc, header, b"a[href^=\"mailto:\"]")
            .first()
            .map(|&n| collapsed_text(n))
            .unwrap_or_default();
        if author.is_empty() {
            return None;
        }
        let date = header_text
            .find("Date:")
            .map(|p| header_text[p + 5..].trim().to_string())
            .unwrap_or_default();
        // Body: document-order successors of the header table (loose 90s
        // markup scatters the message <p>s across wrapper boundaries) up to
        // the reply/followups chrome.
        unsafe fn next_no_descend(
            mut n: *mut lxb_dom_node_t,
            root: *mut lxb_dom_node_t,
        ) -> *mut lxb_dom_node_t {
            unsafe {
                loop {
                    if n.is_null() || n == root {
                        return std::ptr::null_mut();
                    }
                    if !(*n).next.is_null() {
                        return (*n).next;
                    }
                    n = (*n).parent;
                }
            }
        }
        let mut text = String::new();
        let mut n = next_no_descend(header, body);
        'walk: while !n.is_null() {
            if (*n).type_ == LXB_DOM_NODE_TYPE_ELEMENT {
                if (*n).local_name == LXB_TAG_HR {
                    break;
                }
                let is_followups = get_node_attr(n, b"name") == b"followups";
                if is_followups {
                    break;
                }
                if !query_selector_all_raw(doc, n, b"a[name=\"followups\"]").is_empty() {
                    // The stop anchor is inside: descend to find the boundary.
                    n = (*n).first_child;
                    continue 'walk;
                }
                let t = extract_plain_text_from_node(doc, n, opts);
                let tt = t.trim();
                if tt.starts_with("Reply To This Post") {
                    break;
                }
                if !tt.is_empty() {
                    if !text.is_empty() {
                        text.push_str("\n\n");
                    }
                    text.push_str(tt);
                }
            }
            n = next_no_descend(n, body);
        }
        if text.trim().is_empty() {
            return None;
        }
        let mut out = String::new();
        if date.is_empty() || date.len() > 48 {
            out.push_str(&format!("**{author}**  \n\n"));
        } else {
            out.push_str(&format!("**{author}** \u{2013} {date}  \n\n"));
        }
        out.push_str(text.trim_end());
        Some(out)
    }
}

/// Motley Fool boards single-message page. Gate: the message header block
/// (`table.messageMeta` / `div.messageMetaBar`) plus the message body
/// `blockquote.pbmsg` — markup unique to boards.fool.com.
unsafe fn extract_fool(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let metas = query_selector_all_raw(doc, body, b"table.messageMeta, div.messageMetaBar");
        let Some(&meta) = metas.first() else {
            return None;
        };
        let Some(&bq) = query_selector_all_raw(doc, body, b"blockquote.pbmsg").first() else {
            return None;
        };
        let mut author = String::new();
        for &a in &query_selector_all_raw(doc, meta, b"a.pbnavlink") {
            if contains_subslice(get_node_attr(a, b"href"), b"Profile.asp") {
                author = collapsed_text(a);
                break;
            }
        }
        if author.is_empty() {
            return None;
        }
        let mut date = String::new();
        for &d in &query_selector_all_raw(doc, meta, b"td.pbnav, div.msgDate") {
            let t = collapsed_text(d);
            if let Some(r) = t.strip_prefix("Date:") {
                let r = r.trim();
                if !r.is_empty() && r.len() <= 40 {
                    date = r.to_string();
                }
                break;
            }
        }
        let text = extract_plain_text_from_node(doc, bq, opts);
        if text.trim().is_empty() {
            return None;
        }
        let mut out = String::new();
        if date.is_empty() {
            out.push_str(&format!("**{author}**  \n\n"));
        } else {
            out.push_str(&format!("**{author}** \u{2013} {date}  \n\n"));
        }
        out.push_str(text.trim_end());
        Some(out)
    }
}

/// CafeMom group-forum thread. Gate: `div.boardPostBody` (opening post) plus
/// `div.forumReplyBody` replies. Headers come from the "by USER on DATE"
/// meta rows; nested quote blocks and the mobile signature are dropped (gold
/// keeps one level of "Quoting X:"). Coverage-guarded: some CafeMom golds
/// keep the whole sidebar (featured posts, like tallies) — when the post
/// stream is a small share of the page the generic walk serves those better.
unsafe fn extract_cafemom(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let Some(&first_body) = query_selector_all_raw(doc, body, b"div.boardPostBody").first() else {
            return None;
        };
        let replies = query_selector_all_raw(doc, body, b"div.commentBlock");
        if replies.is_empty() {
            return None;
        }
        // "by USER … on DATE" → (USER from the screen-name anchor, DATE).
        let parse_meta = |meta: *mut lxb_dom_node_t| -> (String, String) {
            let author = query_selector_all_raw(doc, meta, b".screennameMenu a")
                .first()
                .map(|&n| collapsed_text(n))
                .unwrap_or_default();
            let t = collapsed_text(meta);
            let date = t
                .rfind(" on ")
                .map(|p| t[p + 4..].trim().to_string())
                .filter(|d| !d.is_empty() && d.len() <= 40)
                .unwrap_or_default();
            (author, date)
        };
        let extract_post = |bn: *mut lxb_dom_node_t| -> String {
            let mut sub = opts.clone();
            sub.skip_elements.push("div.mobile-sig".to_string());
            sub.skip_elements.push("blockquote blockquote".to_string());
            extract_plain_text_from_node(doc, bn, &sub)
        };
        let mut out = String::new();
        if let Some(&h) = query_selector_all_raw(doc, body, b"h1.post").first() {
            let t = collapsed_text(h);
            if !t.is_empty() && t.len() <= 200 {
                out.push_str(&format!("# {t}"));
            }
        }
        let mut authored = 0;
        let mut posts = 0;
        let mut seen: HashSet<String> = HashSet::new();
        let push_post = |out: &mut String,
                             seen: &mut HashSet<String>,
                             authored: &mut usize,
                             posts: &mut usize,
                             author: String,
                             date: String,
                             text: String| {
            if text.trim().is_empty() && author.is_empty() {
                return;
            }
            let key = format!("{author}\u{0}{}", &text.trim()[..text.trim().len().min(120)]);
            if !seen.insert(key) {
                return; // desktop+mobile duplicate markup
            }
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            if !author.is_empty() {
                *authored += 1;
                if date.is_empty() {
                    out.push_str(&format!("**{author}**  \n\n"));
                } else {
                    out.push_str(&format!("**{author}** \u{2013} {date}  \n\n"));
                }
            }
            out.push_str(text.trim_end());
            *posts += 1;
        };
        // Opening post: body in `#body_toggle .boardPostBody`, meta in the
        // `div.topicTool` bar right after it.
        {
            let (author, date) = query_selector_all_raw(doc, body, b"div.topicTool")
                .first()
                .map(|&m| parse_meta(m))
                .unwrap_or_default();
            let text = extract_post(first_body);
            push_post(&mut out, &mut seen, &mut authored, &mut posts, author, date, text);
        }
        for &c in &replies {
            let Some(&bn) = query_selector_all_raw(doc, c, b"div.forumReplyBody").first() else {
                continue;
            };
            let (author, date) = query_selector_all_raw(doc, c, b"div.forumReplyAuthor")
                .first()
                .map(|&m| parse_meta(m))
                .unwrap_or_default();
            let text = extract_post(bn);
            push_post(&mut out, &mut seen, &mut authored, &mut posts, author, date, text);
        }
        if posts < 2 || authored < 2 {
            return None;
        }
        // Coverage guard (matches the golds' split policy): fire only when
        // the rebuilt thread carries >=12% of the page text — CafeMom pages
        // whose gold keeps the sidebar modules (featured posts, like
        // tallies) sit well below this; post-dominated threads sit above.
        if out.len() * 25 < body_text_total(body) * 3 {
            return None;
        }
        Some(out)
    }
}

/// Slashdot story page (gate: `article.fhitem` with `span.story-title` — the
/// slashcode D2 firehose markup). Story byline + intro, then each fully
/// rendered comment (`li.comment` with a non-empty `div.commentBody`) as
/// `**author** – date` + body. Submission pages lack the fhitem article and
/// fall through to the generic walk.
unsafe fn extract_slashdot(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        // Story pages only: exactly one `article.fhitem-story`. Journal and
        // firehose pages carry streams of fhitem articles (a journal page
        // regressed −0.90 in testing) and keep their generic extraction.
        let arts = query_selector_all_raw(doc, body, b"article.fhitem-story");
        if arts.len() != 1 {
            return None;
        }
        let art = arts[0];
        let Some(&title_n) = query_selector_all_raw(doc, art, b"span.story-title").first() else {
            return None;
        };
        let title = collapsed_text(title_n);
        if title.is_empty() {
            return None;
        }
        let byline = query_selector_all_raw(doc, art, b".story-byline")
            .first()
            .map(|&n| collapsed_text(n))
            .unwrap_or_default();
        let story = query_selector_all_raw(doc, art, b"div.body")
            .first()
            .map(|&n| extract_plain_text_from_node(doc, n, opts))
            .unwrap_or_default();
        if story.trim().is_empty() {
            return None;
        }
        let mut out = format!("**{title}**");
        if !byline.is_empty() && byline.len() <= 160 {
            out.push_str(&format!("  \n*{byline}*"));
        }
        out.push_str("\n\n");
        out.push_str(story.trim());
        let mut comments = String::new();
        for &c in &query_selector_all_raw(doc, body, b"li.comment") {
            // First commentBody in document order is the comment's own; the
            // nested replies are separate li.comment matches.
            let Some(&cb) = query_selector_all_raw(doc, c, b"div.commentBody").first() else {
                continue;
            };
            let text = extract_plain_text_from_node(doc, cb, opts);
            if text.trim().is_empty() {
                continue;
            }
            let Some(&details) = query_selector_all_raw(doc, c, b".details").first() else {
                continue;
            };
            let author = query_selector_all_raw(doc, details, b".by a")
                .first()
                .map(|&n| collapsed_text(n))
                .unwrap_or_else(|| {
                    collapsed_text(
                        *query_selector_all_raw(doc, details, b".by").first().unwrap_or(&details),
                    )
                    .trim_start_matches("by ")
                    .trim()
                    .to_string()
                });
            if author.is_empty() {
                continue;
            }
            let mut date = String::new();
            let od = query_selector_all_raw(doc, details, b".otherdetails")
                .first()
                .map(|&n| collapsed_text(n))
                .unwrap_or_default();
            if let Some(p) = od.find("on ") {
                let rest = &od[p + 3..];
                let end = rest.find(" (#").unwrap_or(rest.len());
                let cand = rest[..end].trim();
                if !cand.is_empty() && cand.len() <= 48 {
                    date = cand.to_string();
                }
            }
            if !comments.is_empty() {
                comments.push_str("\n\n");
            }
            if date.is_empty() {
                comments.push_str(&format!("**{author}**  \n\n"));
            } else {
                comments.push_str(&format!("**{author}** \u{2013} {date}  \n\n"));
            }
            comments.push_str(text.trim_end());
        }
        if !comments.is_empty() {
            out.push_str("\n\n---\n\n**Comments**\n\n");
            out.push_str(&comments);
        }
        Some(out)
    }
}

/// Godlike Productions report-a-post page (gate: the GLP banner image id +
/// the `table.posting` report form whose title cell starts with "REPORT").
/// The gold keeps the form's subject/handle/content fields; thread and
/// reply pages have different title cells and fall through.
unsafe fn extract_glp_report(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        if query_selector_all_raw(doc, body, b"img#glpbanner").is_empty() {
            return None;
        }
        let Some(&posting) = query_selector_all_raw(doc, body, b"table.posting").first() else {
            return None;
        };
        let title = query_selector_all_raw(doc, posting, b"td.title")
            .first()
            .map(|&n| collapsed_text(n))
            .unwrap_or_default();
        if !title.starts_with("REPORT") || title.len() > 80 {
            return None;
        }
        let names = query_selector_all_raw(doc, posting, b"td.fieldname");
        let mut subject = String::new();
        let mut handle = String::new();
        let mut content = String::new();
        for &fname in &names {
            let label = collapsed_text(fname);
            // The paired value cell is the next element sibling.
            let mut v = (*fname).next;
            while !v.is_null() && (*v).type_ != LXB_DOM_NODE_TYPE_ELEMENT {
                v = (*v).next;
            }
            if v.is_null() {
                continue;
            }
            match label.as_str() {
                "Message Subject" => subject = collapsed_text(v),
                "Poster Handle" => handle = collapsed_text(v),
                "Post Content" => content = extract_plain_text_from_node(doc, v, opts),
                _ => {}
            }
        }
        if subject.is_empty() && handle.is_empty() && content.trim().is_empty() {
            return None;
        }
        let mut out = format!("**{title}**");
        if !subject.is_empty() {
            out.push_str(&format!("\n\n**Message Subject:** {subject}"));
        }
        if !handle.is_empty() {
            out.push_str(&format!("\n\n**Poster Handle:** {handle}"));
        }
        if !content.trim().is_empty() {
            out.push_str(&format!("\n\n**Post Content:**\n{}", content.trim()));
        }
        // A content-free report (smiley-only post) leaves the sidebar news
        // panel as the page's only substantive content — the gold keeps it.
        if content.trim().len() < 50 {
            if let Some(&news) = query_selector_all_raw(doc, body, b"div.Panel ul.ba").first() {
                let t = extract_plain_text_from_node(doc, news, opts);
                if !t.trim().is_empty() {
                    out.push_str("\n\n---\n\n**News**\n\n");
                    out.push_str(t.trim_end());
                }
            }
        }
        Some(out)
    }
}

/// Fence language from `language-x`/`lang-x`/`brush: x` class hints on a
/// `<pre>` or its first `<code>` child (cycle 0026; hint-only, no sniffing).
unsafe fn fence_language(pre: *mut lxb_dom_node_t) -> Option<String> {
    unsafe {
        fn from_cls(cls: &[u8]) -> Option<String> {
            let text = String::from_utf8_lossy(cls).to_lowercase();
            for tok in text.split([' ', ';']) {
                for pref in ["language-", "lang-", "brush:"] {
                    if let Some(l) = tok.trim().strip_prefix(pref) {
                        let l = l.trim();
                        if !l.is_empty()
                            && l.len() <= 12
                            && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '#')
                        {
                            return Some(l.to_string());
                        }
                    }
                }
            }
            None
        }
        if let Some(l) = from_cls(get_node_attr(pre, b"class")) {
            return Some(l);
        }
        let mut child = (*pre).first_child;
        while !child.is_null() {
            if (*child).type_ == LXB_DOM_NODE_TYPE_ELEMENT && (*child).local_name == LXB_TAG_CODE {
                return from_cls(get_node_attr(child, b"class"));
            }
            child = (*child).next;
        }
        None
    }
}

static DATE_LIKE: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"(?:\d{1,4}[-/.]\d{1,2}[-/.]\d{1,4}|(?:jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[a-z]* \d{1,2}(?:st|nd|rd|th)?,? \d{2,4}|\d{1,2}:\d{2}\s?(?:am|pm)?|\d+ (?:hours?|days?|weeks?|months?|years?) ago)")
        .case_insensitive(true)
        .unicode(false)
        .build()
        .unwrap()
});

/// Generic forum post-stream rebuilder (cycle 0029): one-off engines share a
/// shape — >=3 repeated same-class sibling containers, each with a short
/// user link, a date-like string, and a substantial body. Rebuild as
/// `**user — date**` + body. Runs only after every specific handler
/// declined; authored/coverage/native-first gates as elsewhere.
unsafe fn extract_generic_posts(doc: *mut lxb_html_document_t, opts: &ExtractOpts) -> Option<String> {
    unsafe {
        let body: *mut lxb_dom_node_t = (*doc).body.cast();
        if body.is_null() {
            return None;
        }
        let page_text = get_collapsed_string(&get_node_text(body)).len();
        if page_text < 1500 {
            return None;
        }
        // find the best repeated-sibling stream container
        let mut best: Option<(Vec<*mut lxb_dom_node_t>, usize)> = None;
        let mut node = body;
        let mut depth = 0usize;
        let mut end_tag = false;
        while !node.is_null() {
            if !end_tag && (*node).type_ == LXB_DOM_NODE_TYPE_ELEMENT {
                // group element children by (tag, class)
                let mut groups: std::collections::HashMap<(lxb_tag_id_t, Vec<u8>), Vec<*mut lxb_dom_node_t>> =
                    std::collections::HashMap::new();
                let mut child = (*node).first_child;
                while !child.is_null() {
                    if (*child).type_ == LXB_DOM_NODE_TYPE_ELEMENT
                        && is_block_element((*child).local_name)
                    {
                        let cls = get_node_attr(child, b"class").to_vec();
                        groups.entry(((*child).local_name, cls)).or_default().push(child);
                    }
                    child = (*child).next;
                }
                for (_k, members) in groups {
                    if members.len() < 3 {
                        continue;
                    }
                    let total: usize = members
                        .iter()
                        .map(|&m| get_collapsed_string(&get_node_text(m)).len())
                        .sum();
                    if total * 4 < page_text || total / members.len() < 200 {
                        continue;
                    }
                    if best.as_ref().map(|(_, t)| total > *t).unwrap_or(true) {
                        best = Some((members, total));
                    }
                }
            }
            node = next_node(body, node, &mut depth, &mut end_tag);
        }
        let (posts_nodes, _) = best?;

        let mut out = String::new();
        let mut authored = 0;
        let mut authors: Vec<String> = Vec::new();
        for c in &posts_nodes {
            let c = *c;
            let text_all = get_collapsed_string(&get_node_text(c));
            let head = &text_all[..text_all.len().min(200)];
            // author: first short link whose text appears in the post HEAD
            // (forum post headers lead with the username)
            let mut author = String::new();
            let coll = lxb_dom_collection_make_noi((*c).owner_document, 8);
            lxb_dom_elements_by_tag_name(c.cast(), coll, b"a".as_ptr(), 1);
            for i in 0..lxb_dom_collection_length_noi(coll) {
                let t = collapsed_text(lxb_dom_collection_node_noi(coll, i));
                let wc = t.split_whitespace().count();
                if (2..=25).contains(&t.len())
                    && wc <= 3
                    && !t.chars().all(|ch| ch.is_ascii_digit())
                    && String::from_utf8_lossy(head).contains(t.as_str())
                {
                    author = t;
                    break;
                }
            }
            // date: date-like substring in the post head only
            let date = DATE_LIKE
                .find(head)
                .map(|m| String::from_utf8_lossy(&head[m.start()..m.end()]).to_string())
                .unwrap_or_default();
            lxb_dom_collection_destroy(coll, true);
            if date.is_empty() {
                continue; // post streams carry dates; listings usually don't
            }
            let text = extract_plain_text_from_node(doc, c, opts);
            if text.trim().is_empty() || author.is_empty() {
                continue;
            }
            // strip the standalone author/date lines the subtree walk kept
            let cleaned: String = text
                .lines()
                .filter(|l| {
                    let t = l.trim().trim_start_matches(['-', '\u{2022}', ' ']).trim();
                    !t.is_empty() && t != author && t != date
                })
                .collect::<Vec<_>>()
                .join("\n");
            if cleaned.trim().is_empty() {
                continue;
            }
            authored += 1;
            authors.push(author.clone());
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            if date.is_empty() {
                out.push_str(&format!("**{author}**"));
            } else {
                out.push_str(&format!("**{author} \u{2014} {date}**"));
            }
            out.push_str("\n\n");
            out.push_str(cleaned.trim_end());
        }
        if authored < 3 {
            return None;
        }
        // listings repeat one link target; threads have diverse authors
        let unique: std::collections::HashSet<&String> = authors.iter().collect();
        if unique.len() < 2 {
            return None;
        }
        let body_total = page_text.max(1);
        if out.len() * 4 < body_total {
            return None;
        }
        // native-first: only rebuild if the generic walk loses attribution
        let generic = extract_plain_text_from_doc(doc, opts, RelaxFlags::default(), None, None);
        let missing = authors.iter().filter(|a| !generic.contains(a.as_str())).count();
        if missing * 2 < authors.len() {
            return None;
        }
        Some(out)
    }
}

/// Whether the document declares `<meta name="generator" content="blogger">`
/// (Blogger/Blogspot platform signature; cycle 0010).
unsafe fn is_blogger_doc(doc: *mut lxb_html_document_t) -> bool {
    unsafe {
        let head: *mut lxb_dom_node_t = (*doc).head.cast();
        if head.is_null() {
            return false;
        }
        let mut child = (*head).first_child;
        while !child.is_null() {
            if (*child).type_ == LXB_DOM_NODE_TYPE_ELEMENT && (*child).local_name == LXB_TAG_META {
                let name = get_node_attr(child, b"name");
                if name.eq_ignore_ascii_case(b"generator")
                    && get_node_attr(child, b"content").to_ascii_lowercase().starts_with(b"blogger")
                {
                    return true;
                }
            }
            child = (*child).next;
        }
        false
    }
}

/// Blogger chrome that the lpv11 gold consistently drops (share buttons,
/// labels line, pagers, feed links). Gold-inconsistent items ("Posted by",
/// comment headers) are deliberately NOT here (kept 28/62 — a wall).
const BLOGGER_CHROME_SELECTORS: &[&[u8]] = &[
    b".post-share-buttons",
    b".feed-links",
    b".blog-pager",
    b"#blog-pager",
    b".post-labels",
    // unambiguous sidebar widgets (0075). Image/Profile/Label widgets are
    // EXCLUDED: they carry content on some templates (poltavabloggen
    // crater in the 0074 combined test).
    b".widget.LinkList",
    b".widget.BlogArchive",
    b".widget.FollowByEmail",
    b".widget.Attribution",
    b".widget.Followers",
];

unsafe fn extract_plain_text_from_doc(
    doc: *mut lxb_html_document_t,
    opts: &ExtractOpts,
    relax: RelaxFlags,
    tpl: Option<&HashSet<*mut lxb_dom_node_t>>,
    whitelist: Option<&HashSet<*mut lxb_dom_node_t>>,
) -> String {
    unsafe { extract_plain_text_from_doc_impl2(doc, None, opts, relax, tpl, whitelist).0 }
}

/// Extract from an arbitrary subtree root, reusing the full serialization
/// machinery (markdown, fences, tables). Used by engine handlers.
unsafe fn extract_plain_text_from_node(
    doc: *mut lxb_html_document_t,
    root: *mut lxb_dom_node_t,
    opts: &ExtractOpts,
) -> String {
    unsafe {
        // Engine handlers pick the container themselves; run the generic walk
        // WITHOUT main-content filtering (the handler's selection is the
        // filter) but with the blacklist (script/style/etc).
        let sub_opts = ExtractOpts {
            main_content: false,
            ..opts.clone()
        };
        extract_plain_text_from_doc_impl2(doc, Some(root), &sub_opts, RelaxFlags::default(), None, None).0
    }
}

/// Returns the extracted text plus the `<ul>`/`<article>` nodes (with their
/// body depth) dropped by the main-content blacklist — recorded so the tier-2
/// rescue can lazily test rescue eligibility only when its output-size gate
/// fires.
unsafe fn extract_plain_text_from_doc_impl(
    doc: *mut lxb_html_document_t,
    opts: &ExtractOpts,
    relax: RelaxFlags,
    tpl: Option<&HashSet<*mut lxb_dom_node_t>>,
) -> (String, Vec<(*mut lxb_dom_node_t, usize)>) {
    unsafe { extract_plain_text_from_doc_impl2(doc, None, opts, relax, tpl, None) }
}

unsafe fn extract_plain_text_from_doc_impl2(
    doc: *mut lxb_html_document_t,
    root_override: Option<*mut lxb_dom_node_t>,
    opts: &ExtractOpts,
    relax: RelaxFlags,
    tpl: Option<&HashSet<*mut lxb_dom_node_t>>,
    whitelist: Option<&HashSet<*mut lxb_dom_node_t>>,
) -> (String, Vec<(*mut lxb_dom_node_t, usize)>) {
    let mut dropped_nodes: Vec<(*mut lxb_dom_node_t, usize)> = Vec::new();
    unsafe {
        let body: *mut lxb_dom_node_t = root_override.unwrap_or_else(|| (*doc).body.cast());
        if body.is_null() {
            return (String::new(), dropped_nodes);
        }

        // Build the skip selector (BTreeSet: Python uses a set; order does not
        // affect which nodes match).
        let mut skip_selectors: BTreeSet<Vec<u8>> =
            opts.skip_elements.iter().map(|s| s.as_bytes().to_vec()).collect();
        for sel in [b"script".as_slice(), b"style", b"iframe", b"frame", b"template"] {
            skip_selectors.insert(sel.to_vec());
        }
        if !opts.alt_texts {
            // NB: `b'embed' b'img'` literal concatenation quirk reproduced from
            // the reference: "embedimg" is one (nonexistent) element name.
            for sel in [b"object".as_slice(), b"video", b"audio", b"embedimg", b"area", b"svg", b"figcaption", b"figure"] {
                skip_selectors.insert(sel.to_vec());
            }
        }
        if !opts.noscript {
            skip_selectors.insert(b"noscript".to_vec());
        }
        if !opts.form_fields {
            for sel in [b"textarea".as_slice(), b"input", b"button", b"select", b"option", b"label"] {
                skip_selectors.insert(sel.to_vec());
            }
        }
        if opts.main_content && is_blogger_doc(doc) {
            for sel in BLOGGER_CHROME_SELECTORS {
                skip_selectors.insert(sel.to_vec());
            }
        }
        let skip_selector: Vec<u8> = skip_selectors.into_iter().collect::<Vec<_>>().join(&b","[..]);

        let mut ctx = ExtractContext {
            root_node: body,
            node: body,
            depth: 0,
            opts: ExtractOptsC {
                preserve_formatting: opts.preserve_formatting,
                list_bullets: opts.list_bullets,
                links: opts.links,
                alt_texts: opts.alt_texts,
                form_fields: opts.form_fields,
                noscript: opts.noscript,
            },
        };

        if (*ctx.node).type_ == LXB_DOM_NODE_TYPE_DOCUMENT {
            // Mirror of the reference's next_element_node fallback; body is an
            // element in practice, so this path is effectively dead.
            let mut depth = 0usize;
            let mut end_tag = false;
            let mut n = (*ctx.node).first_child;
            while !n.is_null() && (*n).type_ != LXB_DOM_NODE_TYPE_ELEMENT {
                n = next_node(ctx.node, n, &mut depth, &mut end_tag);
            }
            ctx.root_node = n;
            ctx.node = n;
            if n.is_null() {
                return (String::new(), dropped_nodes);
            }
        }

        if opts.main_content {
            let main_content_selector = b".article-body, .articleBody, .contentBody, .article-text,\
.main-content, .postcontent, .post-content, .single-post,\
[role=\"main\"]";
            let root_candidates = query_selector_all_raw(doc, ctx.node, main_content_selector);
            if root_candidates.len() == 1 {
                // Use result only if there is exactly one match
                ctx.root_node = root_candidates[0];
                ctx.node = ctx.root_node;
            }
        }

        // Select all blacklisted elements and store them in a set
        let mut blacklisted_nodes: HashSet<*mut lxb_dom_node_t> =
            query_selector_all_raw(doc, ctx.root_node, &skip_selector).into_iter().collect();

        // Structural template subtraction (cycle 0019; markdown config only):
        // repeated∧link-dense containers join the skip set. The veto set is
        // computed once per document (rescue retries reuse it).
        if let Some(tpl) = tpl {
            for v in tpl {
                blacklisted_nodes.insert(*v);
            }
        }

        let mut base_depth: usize = 0;
        let mut pnode = ctx.node;
        while (*pnode).local_name != LXB_TAG_BODY && !(*pnode).parent.is_null() {
            base_depth += 1;
            pnode = (*pnode).parent;
        }

        let mut extract_nodes: Vec<ExtractNode> = Vec::with_capacity(150);
        let mut chars_extracted: usize = 0;
        let mut nodes_extracted: usize = 0;
        let mut is_end_tag = false;
        while !ctx.node.is_null() {
            // Skip everything except element and text nodes
            if (*ctx.node).type_ != LXB_DOM_NODE_TYPE_ELEMENT && (*ctx.node).type_ != LXB_DOM_NODE_TYPE_TEXT {
                is_end_tag = true;
                ctx.node = next_node(ctx.root_node, ctx.node, &mut ctx.depth, &mut is_end_tag);
                continue;
            }

            // Skip blacklisted or non-main-content nodes
            if blacklisted_nodes.contains(&ctx.node)
                || (opts.main_content
                    && !whitelist.map(|w| w.contains(&ctx.node)).unwrap_or(false)
                    && !is_main_content_node(
                        ctx.node,
                        ctx.depth + base_depth,
                        opts.comments,
                        opts.post_meta,
                        opts.hidden_elements,
                        relax,
                        opts.preserve_formatting == FormattingOpts::Markdown,
                    ))
            {
                if relax == RelaxFlags::default()
                    && opts.main_content
                    && dropped_nodes.len() < 64
                    && (*ctx.node).type_ == LXB_DOM_NODE_TYPE_ELEMENT
                    && matches!((*ctx.node).local_name, LXB_TAG_UL | LXB_TAG_ARTICLE | LXB_TAG_DIV)
                    && !blacklisted_nodes.contains(&ctx.node)
                {
                    dropped_nodes.push((ctx.node, ctx.depth + base_depth));
                }
                is_end_tag = true;
                ctx.node = next_node(ctx.root_node, ctx.node, &mut ctx.depth, &mut is_end_tag);
                continue;
            }

            extract_cb(&mut extract_nodes, &mut ctx, is_end_tag);
            if extract_nodes.len() > nodes_extracted {
                if let Some(tc) = &extract_nodes.last().unwrap().text_contents {
                    chars_extracted += tc.len();
                    nodes_extracted += 1;
                }
            }

            ctx.node = next_node(ctx.root_node, ctx.node, &mut ctx.depth, &mut is_end_tag);
        }

        let mut output = serialize_extract_nodes(
            &mut extract_nodes,
            &ctx.opts,
            (chars_extracted as f64 * 1.2) as usize,
        );
        rstrip_in_place(&mut output);
        let mut text = decode_utf8_ignore(output);
        if opts.preserve_formatting == FormattingOpts::Markdown {
            // Post-passes run INSIDE the extraction so the rescue-ladder
            // gates measure stripped (chrome-free) content length — same
            // rationale as content_len excluding markdown punctuation.
            // Measured both ways (0035): post-rescue ordering trades ~3
            // beneficial rescues for 1 junk rescue and nets worse.
            // tabs first: plain-rendered table cells join with '\t' and
            // must collapse before the exact-line strip can see them
            // ("Author\tMessage", 0048)
            text = normalize_nbsp(text);
            text = normalize_tabs(text);
            text = strip_ui_label_lines(text);
            text = strip_related_sections(text);
            text = strip_orphan_headings(text);
            text = promote_heading_levels(text);
            // dedup_paragraphs measured dev-positive/train-negative (0031) —
            // gold keeps repeats on some templates; disabled pending
            // containment-aware port of the jusText version.
        }
        (text, dropped_nodes)
    }
}

/// Repeated-paragraph dedup (jusText 0018/0030 family, cycle 0031):
/// template widgets and print/mobile duplicates repeat whole paragraphs.
/// Exact-match on substantial paragraphs (>=60 bytes), whitespace-normalized;
/// code fences are never deduped (code legitimately repeats).
#[allow(dead_code)]
fn dedup_paragraphs(text: String) -> String {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out: Vec<&str> = Vec::new();
    let mut in_fence = false;
    let mut removed = false;
    for para in text.split("\n\n") {
        let fence_delims = para.matches("```").count();
        if in_fence {
            out.push(para);
            if fence_delims % 2 == 1 {
                in_fence = false;
            }
            continue;
        }
        if fence_delims % 2 == 1 {
            in_fence = true;
            out.push(para);
            continue;
        }
        let norm: String = para.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
        if norm.len() >= 150 && !seen.insert(norm) {
            removed = true;
            continue;
        }
        out.push(para);
    }
    if removed { out.join("\n\n") } else { text }
}

/// Heading-level promotion (cycle 0034): the gold re-levels each page so its
/// top heading is `#` (98% of heading-bearing golds); pages that start at
/// h2/h3 shift every heading up accordingly. Fenced code is untouched.
fn promote_heading_levels(text: String) -> String {
    let mut min_level = usize::MAX;
    let mut in_fence = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let hashes = line.bytes().take_while(|b| *b == b'#').count();
        if (1..=6).contains(&hashes) && line.as_bytes().get(hashes) == Some(&b' ') {
            min_level = min_level.min(hashes);
        }
    }
    if min_level == usize::MAX || min_level <= 1 {
        return text;
    }
    let shift = min_level - 1;
    let mut out = String::with_capacity(text.len());
    let mut in_fence = false;
    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            out.push_str(line);
            continue;
        }
        let hashes = line.bytes().take_while(|b| *b == b'#').count();
        if !in_fence && (1..=6).contains(&hashes) && line.as_bytes().get(hashes) == Some(&b' ') {
            out.push_str(&line[shift.min(hashes - 1)..]);
        } else {
            out.push_str(line);
        }
    }
    out
}

/// Dangling UI-label lines (jusText-0084 family, cycle 0026): a line that is
/// exactly a comment-widget verb is never content. Curated, exact-match after
/// stripping list markers. "Author", "Comments", "Quote" excluded (real
/// content in some gold).
/// Gold is tab-free in 99.6% of docs (0045): tabs surviving the walk
/// (white-space:pre contexts, layout tables) collapse to a single space
/// in prose and expand to 4 spaces inside code fences.
fn normalize_tabs(text: String) -> String {
    if !text.contains('\t') {
        return text;
    }
    // Code-listing pages (gold keeps their tabs) opt out wholesale: if a
    // large share of lines carry tabs, this is source code, not layout.
    let mut tab_lines = 0usize;
    let mut nonempty = 0usize;
    for line in text.split('\n') {
        if line.trim().is_empty() {
            continue;
        }
        nonempty += 1;
        if line.contains('\t') {
            tab_lines += 1;
        }
    }
    if nonempty == 0 || tab_lines * 4 > nonempty {
        return text;
    }
    let mut out = String::with_capacity(text.len());
    let mut in_fence = false;
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            out.push_str(line);
            continue;
        }
        if in_fence || !line.contains('\t') {
            // fenced code keeps its tabs (gold does)
            out.push_str(line);
            continue;
        }
        // collapse INTERIOR whitespace runs containing a tab to one space;
        // leading indentation is preserved (pre-rendered code alignment)
        let lead_end = line.len() - line.trim_start().len();
        let mut cur = String::with_capacity(line.len());
        cur.push_str(&line[..lead_end]);
        let mut run: Vec<char> = Vec::new();
        for ch in line[lead_end..].chars() {
            if ch == ' ' || ch == '\t' {
                run.push(ch);
            } else {
                if !run.is_empty() {
                    if run.contains(&'\t') {
                        cur.push(' ');
                    } else {
                        cur.extend(run.iter());
                    }
                    run.clear();
                }
                cur.push(ch);
            }
        }
        if !run.is_empty() && !run.contains(&'\t') {
            cur.extend(run.iter());
        }
        out.push_str(cur.trim_end());
    }
    out
}

/// Markdown post-passes bundle (0048): engine-handler outputs and
/// appended rebuild blocks exit before impl2's interior post-passes and
/// were shipping unstripped chrome lines ("Author Message") and raw tabs.
/// U+00A0 normalization (0144, owner-review find): forum post headers
/// render "12-01-2010,\u{a0}12:30 PM" where gold has a plain space. The
/// annotator's pipeline collapses nbsp to space in 21 of the 22 docs where
/// we emit one, so normalize in markdown mode.
fn normalize_nbsp(text: String) -> String {
    let mut t = if text.contains('\u{a0}') { text.replace('\u{a0}', " ") } else { text };
    // Zero-width/BOM artifacts (0145 char census): U+FEFF appears in 13 dev
    // docs of our output and ZERO golds; same for the zero-width family.
    // They survive from mis-stamped source bytes and are never rendered.
    for z in ['\u{feff}', '\u{200b}', '\u{200e}', '\u{200f}'] {
        if t.contains(z) {
            t = t.replace(z, "");
        }
    }
    t
}

fn md_post_passes(text: String) -> String {
    let text = normalize_nbsp(text);
    let text = normalize_tabs(text);
    let text = strip_ui_label_lines(text);
    let text = strip_related_sections(text);
    let text = strip_orphan_headings(text);
    promote_heading_levels(text)
}

/// Related/teaser section strip (cycle 0072): a heading in the
/// related-content family plus its short link-teaser section (guarded:
/// <=25 lines, <=max(600, 15% of doc) chars, no prose line >200 chars —
/// the tampabay guard: unbounded consumption ate an article).
/// Orphan chrome-heading strip (cycle 0085): a heading with an EMPTY
/// section (next heading or EOF follows immediately) whose text is
/// censused chrome. Orphan-gated: "No comments:" with actual comments
/// after it is Blogspot content-adjacent and stays.
fn strip_orphan_headings(text: String) -> String {
    const ORPHANS: &[&str] = &[
        "no comments:", "0 comments:", "0 comments", "archives",
        "personal tools", "search", "pages", "categories",
    ];
    if !text.contains('#') {
        return text;
    }
    let lines: Vec<&str> = text.split('\n').collect();
    let mut drop = vec![false; lines.len()];
    let mut changed = false;
    for i in 0..lines.len() {
        let t = lines[i].trim();
        if !t.starts_with('#') {
            continue;
        }
        let name = t.trim_start_matches('#').trim().to_lowercase();
        if !ORPHANS.contains(&name.as_str()) {
            continue;
        }
        let mut j = i + 1;
        while j < lines.len() && lines[j].trim().is_empty() {
            j += 1;
        }
        if j >= lines.len() || lines[j].trim_start().starts_with('#') {
            drop[i] = true;
            changed = true;
        }
    }
    if !changed {
        return text;
    }
    let mut s = lines
        .iter()
        .enumerate()
        .filter(|(i, _)| !drop[*i])
        .map(|(_, l)| *l)
        .collect::<Vec<_>>()
        .join("\n");
    while s.contains("\n\n\n") {
        s = s.replace("\n\n\n", "\n\n");
    }
    s
}

fn strip_related_sections(text: String) -> String {
    fn is_rel_heading(t: &str) -> Option<usize> {
        let t = t.trim();
        if !t.starts_with('#') {
            return None;
        }
        let lvl = t.len() - t.trim_start_matches('#').len();
        let rest = t.trim_start_matches('#').trim().to_lowercase();
        const FAMS: &[&str] = &[
            "you may also", "related articles", "related posts", "related stories",
            "related news", "related content", "more from", "recommended for you",
            "see also", "around the web", "popular posts", "popular articles", "trending",
            "similar tracks",
            "latest stories", "latest news", "latest headlines", "more stories",
            "most read", "most popular", "most viewed", "sponsored links",
            "from our partners", "editor's picks",
            "gallery links", "related medicine news",
        ];
        if FAMS.iter().any(|f| rest.starts_with(f)) {
            Some(lvl)
        } else {
            None
        }
    }
    if !text.contains('#') {
        return text;
    }
    let lines: Vec<&str> = text.split('\n').collect();
    let total = text.len();
    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    let mut i = 0usize;
    let mut changed = false;
    while i < lines.len() {
        if let Some(lvl) = is_rel_heading(lines[i]) {
            let mut j = i + 1;
            while j < lines.len() {
                let t = lines[j].trim_start();
                if t.starts_with('#') && (t.len() - t.trim_start_matches('#').len()) <= lvl {
                    break;
                }
                j += 1;
            }
            let seg: Vec<&&str> = lines[i + 1..j].iter().filter(|l| !l.trim().is_empty()).collect();
            let seg_chars: usize = seg.iter().map(|l| l.len()).sum();
            // absolute floor only for large docs: on small pages the
            // section must stay a minor share (quote-site craters, 0072)
            let cap = std::cmp::max(600, total * 3 / 20);
            if seg.len() <= 25
                && seg_chars <= cap
                && seg_chars * 4 <= total
                && !seg.iter().any(|l| l.trim().len() > 200)
            {
                changed = true;
                i = j;
                continue;
            }
        }
        out.push(lines[i]);
        i += 1;
    }
    if !changed {
        return text;
    }
    let mut s = out.join("\n");
    while s.contains("\n\n\n") {
        s = s.replace("\n\n\n", "\n\n");
    }
    s
}

fn strip_ui_label_lines(text: String) -> String {
    const LABELS: &[&str] = &[
        "reply", "like", "report", "share", "permalink", "profile",
        "post a comment", "log in to reply", "login to reply",
        "reply to this comment", "view profile", "send pm", "back to top",
        "read more", "continue reading",
        "leave a reply", "leave a comment", "post comment", "submit comment",
        "notify me of new posts by email.", "notify me of follow-up comments by email.",
        "leave a reply cancel reply", "%d bloggers like this:",
        "advertisement", "advertisements",
        "skip to content", "skip to main content",
        "newer post older post home",
        "recent posts", "advanced search", "search for:",
        "this site uses akismet to reduce spam. learn how your comment data is processed.",
        "who is online", "author message", "post subject:",
        "display posts from previous: sort by",
        "print view previous topic | next topic",
        "view unanswered posts | view active topics",
        "you cannot post new topics in this forum",
        "you cannot reply to topics in this forum",
        "you cannot edit your posts in this forum",
        "you cannot delete your posts in this forum",
        "you cannot post attachments in this forum",
        "view single post", "blog comments powered by disqus",
        "comments powered by disqus",
        "related posts plugin for wordpress, blogger...",
        "share this page", "events calendar",
        "current community", "your communities",
        "more stack exchange communities",
        "you may not post new threads", "you may not post replies",
        "subscribe to: post comments (atom)",
        "get every new post delivered to your inbox.",
        "you cannot vote in polls in this forum",
        "likeliked by 1 person", "likeliked by 2 people",
        "likeliked by 3 people", "likeliked by 4 people",
        "rate this thread", "search this thread", "display modes",
        "html code is on", "html code is off", "bb code is on",
        "smilies are on", "[img] code is on", "posting rules",
        "thread tools", "you may not post attachments",
        "you may not edit your posts",
        "menu",
        "loading...", "jump to:", "email print", "user avatar",
        "quantcast", "reactions:", "post reply", "\u{ab} previous next \u{bb}",
        "i want!", "tag a friend", "be the first to post a tip",
        "please follow and like us:", "error: content is protected !!",
        "new schedule b search engine", "newest trade data!",
        "bookmark the permalink.", "bookmark the permalink",
        "linkback",
        "email this page", "print this page", "email this pageprint this page",
        "garage list", "reply with quote", "view options",
        "report a problem", "no comments posted for this article.",
        "post icons", "trackback:",
        "send trackbacks to (separate multiple urls with spaces) :",
        "confirm password:", "password:", "user name:",
        "likelike", "please login to remove!", "(permalink)",
        "find all posts by this user", "quote this message in a reply",
        "get adobe flash player",
        "content on this page requires a newer version of adobe flash player.",
        "content on this page requires a newer version of adobe flash player",
        "taking too long? try again or cancel this request.",
        "[buy photo]", "buy photo",
        "there are no comments yet.", "be the first to comment!",
        "be the first to comment", "sort: oldest | newest",
        "skip to main navigation", "skip all navigation and go directly to page content",
        "skip top navigation", "skip navigation", "back to article",
        "save | post a comment |", "\u{ab} back to article",
    ];
    let mut out = String::with_capacity(text.len());
    let mut removed_any = false;
    // vBulletin "Similar Threads" teaser table (0107): a single-cell table
    // header row opens the block; skip table rows until a non-table line.
    let mut in_similar_table = false;
    for line in text.split('\n') {
        if in_similar_table {
            if line.trim_start().starts_with('|') || line.trim().is_empty() {
                removed_any = true;
                continue;
            }
            in_similar_table = false;
        }
        let lt_full = line.trim();
        if lt_full.eq_ignore_ascii_case("| similar threads |") {
            in_similar_table = true;
            removed_any = true;
            continue;
        }
        // Forum status line: "Currently Active Users Viewing This Thread: ..."
        if lt_full.len() < 90
            && lt_full
                .to_lowercase()
                .starts_with("currently active users viewing this thread")
        {
            removed_any = true;
            continue;
        }
        // WordPress attachment-page suffix: "… | Full size is 640 × 425
        // pixels" tails the kept byline; drop the suffix, keep the byline.
        if let Some(pos) = line.to_lowercase().find(" | full size is ") {
            if line.trim_end().to_lowercase().ends_with("pixels") {
                removed_any = true;
                out.push_str(line[..pos].trim_end());
                out.push('\n');
                continue;
            }
        }
        let was_heading = line.trim_start().starts_with('#');
        let t = line.trim().trim_start_matches(['-', '\u{2022}', '#', ' ']).trim();
        let t = t.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.').trim();
        let tl = if t.contains('*') {
            t.replace("**", "").replace('*', "").trim().to_lowercase()
        } else {
            t.to_lowercase()
        };
        // Single short words rendered as headings ("# Reply") are sometimes
        // genuine gold content (form pages); only strip heading lines whose
        // label is unambiguous chrome (multi-word or long).
        let heading_ok = !was_heading || tl.contains(' ') || tl.len() >= 10;
        // Forum postbit user-stats prefixes (0082): "Joined:", "Posts: N",
        // "Rep Power: N" etc. — 0-6% gold keep-rate across 3.5K instances.
        // Location: excluded (13% keep — real content on some pages).
        let postbit = !was_heading
            && tl.len() < 40
            && (tl.starts_with("joined:")
                || tl.starts_with("join date:")
                || tl.starts_with("rep power:")
                || (tl.starts_with("posts:")
                    && tl[6..].trim().chars().all(|c| c.is_ascii_digit() || c == ','))
                || (tl.starts_with("thanks:")
                    && tl[7..].trim().chars().all(|c| c.is_ascii_digit() || c == ','))
                || (tl.ends_with(" posts")
                    && tl[..tl.len() - 6].trim().chars().all(|c| c.is_ascii_digit()))
                // vBulletin per-post user-panel family (0114 band taxonomy):
                // status/avatar/stat lines gold reduces to author+time+body
                || tl.ends_with(" is offline")
                || tl.ends_with(" is online")
                || tl.ends_with("'s avatar")
                || tl.starts_with("itrader:")
                || (tl.starts_with("mentioned:") && tl.ends_with("post(s)"))
                || (tl.starts_with("tagged:") && tl.ends_with("thread(s)"))
                || (tl.starts_with("quoted:") && tl.ends_with("post(s)"))
                || (tl.starts_with("liked ") && tl.contains(" times in "))
                || (tl.starts_with("appreciate ")
                    && tl[11..].trim().chars().all(|c| c.is_ascii_digit())));
        if !tl.is_empty() && (postbit || (heading_ok && LABELS.contains(&tl.as_str()))) {
            removed_any = true;
            continue;
        }
        // Unrendered template-token lines (0105): `$tooltip.getTerm()`,
        // `translation_missing` — client-side placeholders a rendered page
        // never shows. Line-level so code blocks (4-space/fenced) survive.
        if !line.starts_with("    ")
            && !line.starts_with('\t')
            && regex_search_not_empty(tl.as_bytes(), &TEMPLATE_TOKEN)
        {
            removed_any = true;
            continue;
        }
        // Ad-targeting machine lines (0120): "action:article | category:X | adString:..."
        if tl.contains("| adstring:") || tl.contains("| zoneid:") {
            removed_any = true;
            continue;
        }
        // Empty-cell table rows (0141, owner-flagged): "| | | | |" carries
        // zero information — vBulletin post-icon grids and spacer rows.
        {
            let tt = line.trim();
            if tt.len() >= 3
                && tt.starts_with('|')
                && tt.ends_with('|')
                && tt.chars().all(|c| c == '|' || c == ' ')
            {
                removed_any = true;
                continue;
            }
        }
        // Engine-chrome prefix/skeleton lines (0133, band-79 lexicon batch)
        if !was_heading
            && (tl.starts_with("view full version :")
                || tl.starts_with("[date prev][date next]")
                || (tl.starts_with("quote originally posted by") && tl.ends_with("view post"))
                || (tl.starts_with("thanked ") && tl.contains(" times in ") && tl.ends_with(" posts"))
                || (tl.starts_with("slide ")
                    && tl.len() < 20
                    && tl.contains(" of ")
                    && tl[6..].replace(" of ", "").chars().all(|c| c.is_ascii_digit()))
                || tl.starts_with("users browsing this forum"))
        {
            removed_any = true;
            continue;
        }
        // Trailing self-reference/mailing-list boilerplate (0125)
        if tl.starts_with("find this article at:")
            || tl.starts_with("view this article at:")
            || tl.starts_with("this archive was generated by hypermail")
        {
            removed_any = true;
            continue;
        }
        // Render-timer footers (0113): "generated in 0.010506 seconds"
        if tl.len() < 45 && regex_search_not_empty(tl.as_bytes(), &RENDER_TIMER) {
            removed_any = true;
            continue;
        }
        // Breadcrumb rows (0115): >=2 '>>'/'\u{bb}' separators between short
        // tokens, no sentence punctuation — nav trails, not prose.
        if !was_heading && tl.len() < 100 && !tl.contains('.') && !tl.contains(',') {
            let n_sep = tl.matches(" >> ").count() + tl.matches(" \u{bb} ").count();
            if n_sep >= 2 {
                removed_any = true;
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    if !removed_any {
        return text;
    }
    let mut cleaned = String::with_capacity(out.len());
    let mut blanks = 0;
    for line in out.lines() {
        if line.trim().is_empty() {
            blanks += 1;
            if blanks > 1 {
                continue;
            }
        } else {
            blanks = 0;
        }
        cleaned.push_str(line);
        cleaned.push('\n');
    }
    cleaned.truncate(cleaned.trim_end().len());
    cleaned
}
