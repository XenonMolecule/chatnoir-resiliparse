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
    LXB_TAG_DD,
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
                make_big_block: matches!(local_name, LXB_TAG_P | LXB_TAG_H1 | LXB_TAG_H2 | LXB_TAG_H3 | LXB_TAG_H4),
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
                            if output.last() == Some(&b'|') {
                                // empty row: drop the dangling "|"
                                output.pop();
                                rstrip_in_place(&mut output);
                            } else {
                                output.extend_from_slice(b" |");
                                if md_row_index == 0 {
                                    md_row0_cells = md_cell_index.max(1);
                                    output.push(b'\n');
                                    output.push(b'|');
                                    for _ in 0..md_row0_cells {
                                        output.extend_from_slice(b" --- |");
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
            if opts.preserve_formatting == FormattingOpts::Markdown
                && output.ends_with(b"```")
                && !element_text.starts_with(b"\n")
            {
                output.push(b'\n');
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
        r"(?:^|[\s_-])(?:cookie(?:-?(?:bar|banner|notice|consent))?|consent|gdpr|breadcrumbs?|share-?(?:this|bar|buttons?|links?|post)?|sharing|addthis|sharedaddy|sociable|log-?in|sign-?in|sign-?up|subscribe|newsletter|search-?(?:form|box|bar)|site-?footer|tag-?(?:cloud|list|links)|post-?tags|cat-?links|meta-?(?:nav|links)|read-?next|around-?the-?web|you-?may-?(?:also-?)?like|outbrain|taboola|sponsored-?(?:links|content))(?:$|[\s_-])",
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
/// contain (`place-login-pop` wrapping 45KB of page) must never be vetoed.
unsafe fn is_small_chrome_sized(node: *mut lxb_dom_node_t) -> bool {
    unsafe { get_collapsed_string(&get_node_text(node)).len() <= 1500 }
}

static BYLINE_CLS: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"(?:^|[\s_-])(?:author|byline|by-?line|posted|vcard|entry-meta|post-?meta)(?:$|[\s_-])")
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
            return false;
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
                && query_selector_all_raw(doc, body_ptr, b"div[id^=\"post_message_\"]").len() >= 2;
            if generator.starts_with(b"vbulletin") || vb_markup {
                if let Some(out) = extract_vbulletin(doc, opts) {
                    lxb_html_document_destroy(doc);
                    return out;
                }
            }
            if let Some(out) = extract_phpbb2(doc, opts) {
                lxb_html_document_destroy(doc);
                return out;
            }
            if let Some(out) = extract_phpbb(doc, opts) {
                lxb_html_document_destroy(doc);
                return out;
            }
            if generator.starts_with(b"ubb.threads") {
                if let Some(out) = extract_ubb(doc, opts) {
                    lxb_html_document_destroy(doc);
                    return out;
                }
            }
            if let Some(out) = extract_invision(doc, opts) {
                lxb_html_document_destroy(doc);
                return out;
            }
            if let Some(out) = extract_smf(doc, opts) {
                lxb_html_document_destroy(doc);
                return out;
            }
        }

        let mut page_has_card_grid = false;
        let tpl_set: Option<HashSet<*mut lxb_dom_node_t>> =
            if opts.main_content && opts.preserve_formatting == FormattingOpts::Markdown {
                let body: *mut lxb_dom_node_t = (*doc).body.cast();
                if body.is_null() {
                    None
                } else {
                    let (mut v, grid) = tpl_vetoes(body);
                    page_has_card_grid = grid;
                    if MODEL_VETO_ENABLED {
                        for m in model_vetoes(body) {
                            v.insert(m);
                        }
                    }
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
        let tpl_ref = tpl_set.as_ref();
        let (mut result, mut dropped_nodes) =
            extract_plain_text_from_doc_impl(doc, opts, RelaxFlags::default(), tpl_ref);
        // Gold mirrors each theme's native comment rendering; rebuild only
        // when the native walk LOSES attribution (>=half the authors absent).
        let mut wp_comments: Option<String> = None;
        if let Some((block, vetoes, authors)) = wp_candidate {
            let missing = authors.iter().filter(|a| !result.contains(a.as_str())).count();
            if missing * 2 >= authors.len() {
                let mut set2 = tpl_set.clone().unwrap_or_default();
                for v in &vetoes {
                    set2.insert(*v);
                }
                let (r2, d2) = extract_plain_text_from_doc_impl(doc, opts, RelaxFlags::default(), Some(&set2));
                result = r2;
                dropped_nodes = d2;
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
                t.bytes().filter(|b| !matches!(b, b'|' | b'#' | b'*')).count()
            }
            let result_content_len = content_len(&result);

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
            let mut rescued = false;
            if !is_error_stub
                && result_content_len < RESCUE_NEAR_EMPTY_ABS
                && body_text_len(doc) > RESCUE_BODY_FACTOR * result_content_len.max(1)
            {
                let fallback_opts = ExtractOpts {
                    main_content: false,
                    ..opts.clone()
                };
                let fallback = extract_plain_text_from_doc(doc, &fallback_opts, RelaxFlags::default(), tpl_ref);
                if fallback.len() > RESCUE_KEEP_FACTOR * result.len().max(1) {
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
                    let retry = extract_plain_text_from_doc(doc, opts, relax, tpl_ref);
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
                result.push_str(&block);
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
        let mut out = String::new();
        for (idx, b) in blocks.iter().enumerate() {
            let f = build_block_features(b, page_text, page_ld);
            let text = get_collapsed_string(&get_node_text(b.ptr));
            let text_snip: String = String::from_utf8_lossy(&text)
                .chars()
                .take(600)
                .collect::<String>()
                .replace('\t', " ")
                .replace('\n', " ");
            out.push_str(&format!(
                "{{\"i\":{},\"tag\":{},\"depth\":{},\"text_len\":{},\"link_len\":{},\"n_links\":{},\"page_text\":{},\"page_ld\":{:.4},\"punct\":{:.4},\"digit\":{:.4},\"upper\":{:.4},\"avgw\":{:.3},\"nav\":{},\"footer\":{},\"header\":{},\"sidebar\":{},\"social\":{},\"article\":{},\"chrome\":{},\"byline\":{},\"widget\":{},\"recommended\":{},\"comments\":{},\"text\":{}}}\n",
                idx, b.tag, b.depth, b.text_len, b.link_len, b.n_a, page_text, page_ld,
                f.punct, f.digit, f.upper, f.avgw,
                f.nav as u8, f.footer as u8, f.header as u8, f.sidebar as u8, f.social as u8,
                f.article as u8, f.chrome as u8, f.byline as u8, f.widget as u8,
                f.recommended as u8, f.comments as u8,
                serde_escape(&text_snip),
            ));
        }
        lxb_html_document_destroy(doc);
        out
    }
}

unsafe fn build_block_features(b: &RawBlock, page_text: usize, page_ld: f64) -> block_model::BlockFeatures {
    unsafe {
        let cls = get_node_attr(b.ptr, b"class");
        let id = get_node_attr(b.ptr, b"id");
        let mut combo = cls.to_vec();
        combo.push(b' ');
        combo.extend_from_slice(id);
        let tl = b.text_len.max(1);
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
            nav: regex_search_not_empty(&combo, &NAV_CLS) as u8 as f64,
            footer: regex_search_not_empty(&combo, &FOOTER_CLS) as u8 as f64,
            header: regex_search_not_empty(&combo, &HEADER_CLS) as u8 as f64,
            sidebar: regex_search_not_empty(&combo, &SIDEBAR_CLS) as u8 as f64,
            social: regex_search_not_empty(&combo, &SOCIAL_CLS) as u8 as f64,
            article: regex_search_not_empty(&combo, &ARTICLE_CLS) as u8 as f64,
            chrome: regex_search_not_empty(&combo, &MD_CHROME_CLS) as u8 as f64,
            byline: regex_search_not_empty(&combo, &BYLINE_CLS) as u8 as f64,
            widget: regex_search_not_empty(&combo, &WIDGETISH_CLS) as u8 as f64,
            recommended: regex_search_not_empty(&combo, &RECOMMENDED_CLS) as u8 as f64,
            comments: regex_search_not_empty(&combo, &COMMENTS_CLS) as u8 as f64,
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

#[allow(dead_code)]
struct TplNode {
    ptr: *mut lxb_dom_node_t,
    text_len: usize,
    link_len: usize,
    n_imgs: usize,
    n_a: usize,
    punct: usize,
    digits: usize,
    upper: usize,
    words: usize,
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
        let mut punct = 0usize;
        let mut digits = 0usize;
        let mut upper = 0usize;
        let mut words = 0usize;
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
                    for &b in t {
                        if c_isspace(b) {
                            in_word = false;
                        } else if !in_word {
                            words += 1;
                            in_word = true;
                        }
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
                        punct += c.punct;
                        digits += c.digits;
                        upper += c.upper;
                        words += c.words;
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
        TplNode { ptr: node, text_len, link_len, n_imgs, n_a, punct, digits, upper, words, sig1, sig2 }
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
const MODEL_VETO_THRESHOLD: f64 = 0.10;
const MODEL_VETO_ENABLED: bool = false; // flipped on when the exported model lands

/// Learned block vetoes (markdown config): score every classifier decision
/// point with the exported GBM; confident-junk blocks are skipped.
unsafe fn model_vetoes(body: *mut lxb_dom_node_t) -> HashSet<*mut lxb_dom_node_t> {
    unsafe {
        let mut dummy_v = HashSet::new();
        let mut dummy_c: Vec<(*mut lxb_dom_node_t, usize)> = Vec::new();
        let mut blocks: Vec<RawBlock> = Vec::new();
        let mut coll = Some(&mut blocks);
        let totals = tpl_scan(body, false, 0, &mut dummy_v, &mut dummy_c, &mut coll);
        let page_text = totals.text_len.max(1);
        let page_ld = totals.link_len as f64 / page_text as f64;
        let mut out = HashSet::new();
        for b in &blocks {
            let f = build_block_features(b, page_text, page_ld);
            if block_model::score_block(&f) < MODEL_VETO_THRESHOLD {
                out.insert(b.ptr);
            }
        }
        out
    }
}

/// Returns the veto set plus whether the page carries a LARGE repeated-
/// structure container (>=3000B) — the positive signal that this is a
/// listing/card-grid page (cycle 0023 uses it to gate the listing rescue).
unsafe fn tpl_vetoes(body: *mut lxb_dom_node_t) -> (HashSet<*mut lxb_dom_node_t>, bool) {
    unsafe {
        let mut vetoes = HashSet::new();
        let mut candidates: Vec<(*mut lxb_dom_node_t, usize)> = Vec::new();
        let totals = tpl_scan(body, false, 0, &mut vetoes, &mut candidates, &mut None);
        let large_repeated = candidates.iter().any(|&(_, tl)| tl >= 3000)
            || (totals.text_len > 0
                && totals.link_len as f64 / totals.text_len as f64 > TPL_PAGE_LINK_DENSITY_MAX);
        if totals.text_len < TPL_MIN_PAGE_TEXT
            || totals.link_len as f64 / totals.text_len as f64 > TPL_PAGE_LINK_DENSITY_MAX
        {
            // Listing-like or thin page: on thin pages whatever repeats is
            // usually the content (package-instruction pages, profiles).
            vetoes.clear();
            return (vetoes, large_repeated);
        }
        // container-fraction guard, applied now that body totals are known
        for (n, tl) in candidates {
            if (tl as f64) > TPL_MAX_CONTAINER_FRAC * totals.text_len as f64 {
                vetoes.remove(&n);
            }
        }
        (vetoes, large_repeated)
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
            let author = author_nodes
                .first()
                .map(|&n| String::from_utf8_lossy(&get_collapsed_string(&get_node_text(n))).trim().to_string())
                .unwrap_or_default();
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
        let items = query_selector_all_raw(doc, body, b"li.comment, div.comment[id^=\"comment\"]");
        if items.len() < 2 {
            return None;
        }
        let mut out = String::new();
        let mut vetoes: Vec<*mut lxb_dom_node_t> = Vec::new();
        let mut authors: Vec<String> = Vec::new();
        let mut attributed = 0;
        for c in &items {
            let c = *c;
            let author = query_selector_all_raw(
                doc,
                c,
                b".comment-author .fn, cite.fn, .comment-author cite, .c-head a.url, .comment-author b, .comment-author a",
            )
            .first()
            .map(|&n| collapsed_text(n))
            .unwrap_or_default();
            let mut date = query_selector_all_raw(
                doc,
                c,
                b".comment-metadata, .comment-meta, .commentmetadata, .c-head span, time",
            )
            .first()
            .map(|&n| collapsed_text(n))
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
                b".comment-content, .c-body, .commenttext, .comment-text, .comment-body",
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

unsafe fn extract_plain_text_from_node_opts(
    doc: *mut lxb_html_document_t,
    root: *mut lxb_dom_node_t,
    opts: &ExtractOpts,
) -> String {
    unsafe { extract_plain_text_from_doc_impl2(doc, Some(root), opts, RelaxFlags::default(), None).0 }
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
        if names.len() < 2 || bodies.len() < 2 || names.len() != bodies.len() {
            return None;
        }
        let details = query_selector_all_raw(doc, body, b"span.postdetails");
        let mut out = String::new();
        let mut posts = 0;
        for (i, (&n, &b)) in names.iter().zip(bodies.iter()).enumerate() {
            let author = collapsed_text(n);
            let text = extract_plain_text_from_node(doc, b, opts);
            if text.trim().is_empty() {
                continue;
            }
            let mut date = String::new();
            if let Some(&d) = details.get(i * details.len() / names.len().max(1)) {
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
        if posts < 2 {
            return None;
        }
        // Coverage guard: on odd skins (PNphpBB2) span.postbody matches
        // signatures, not bodies — the rebuild must carry a meaningful share
        // of the page text or the generic walk is better.
        let body_total = get_collapsed_string(&get_node_text(body)).len();
        if out.len() * 4 < body_total {
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
];

unsafe fn extract_plain_text_from_doc(
    doc: *mut lxb_html_document_t,
    opts: &ExtractOpts,
    relax: RelaxFlags,
    tpl: Option<&HashSet<*mut lxb_dom_node_t>>,
) -> String {
    unsafe { extract_plain_text_from_doc_impl(doc, opts, relax, tpl).0 }
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
        extract_plain_text_from_doc_impl2(doc, Some(root), &sub_opts, RelaxFlags::default(), None).0
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
    unsafe { extract_plain_text_from_doc_impl2(doc, None, opts, relax, tpl) }
}

unsafe fn extract_plain_text_from_doc_impl2(
    doc: *mut lxb_html_document_t,
    root_override: Option<*mut lxb_dom_node_t>,
    opts: &ExtractOpts,
    relax: RelaxFlags,
    tpl: Option<&HashSet<*mut lxb_dom_node_t>>,
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
        (decode_utf8_ignore(output), dropped_nodes)
    }
}
