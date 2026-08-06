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

//! Golden parity tests for the `extract_plain_text` port.
//!
//! The `.expected.txt` fixtures are the **byte-exact outputs of the Cython
//! reference implementation** (`resiliparse.extract.html2text`,
//! resiliparse-py 1.0.9) with `main_content=True` on five lpv11 dev documents
//! (frozen in research cycle 0002, where the port was verified byte-identical
//! on 10,999 dev+train docs). Any diff here means the port has silently
//! drifted from the reference — do not update the fixtures without a
//! deliberate, logged decision.

use resiliparse::extract::html2text::{ExtractOpts, extract_plain_text};
use std::fs;
use std::path::PathBuf;

fn fixture(name: &str, ext: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/extract_golden")
        .join(format!("{name}.{ext}"));
    fs::read_to_string(path).unwrap()
}

fn check_golden(name: &str) {
    let html = fixture(name, "html");
    let expected = fixture(name, "expected.txt");
    let opts = ExtractOpts {
        main_content: true,
        ..Default::default()
    };
    assert_eq!(extract_plain_text(&html, &opts), expected, "golden mismatch for {name}");
}

#[test]
fn golden_table() {
    check_golden("table");
}

#[test]
fn golden_code() {
    check_golden("code");
}

#[test]
fn golden_list() {
    check_golden("list");
}

#[test]
fn golden_plain() {
    check_golden("plain");
}

#[test]
fn golden_big() {
    check_golden("big");
}

#[test]
fn basic_formatting() {
    let html = "<html><body><main><h1>t</h1><p>hi</p><ul><li>a</li><li>b</li></ul></main>";
    // Without main_content (the UL is dropped by the shallow-list rule otherwise).
    assert_eq!(
        extract_plain_text(html, &ExtractOpts::default()),
        "t\n\nhi\n\n  \u{2022} a\n  \u{2022} b"
    );
    // With main_content: body_depth < 4 drops the list.
    let opts = ExtractOpts {
        main_content: true,
        ..Default::default()
    };
    assert_eq!(extract_plain_text(html, &opts), "t\n\nhi");
}
