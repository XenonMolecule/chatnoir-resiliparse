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

use crate::stream_io::traits::BufReadSeek;
use encoding::all::WINDOWS_1252;
use encoding::{DecoderTrap, EncoderTrap, Encoding};
use memchr::{memchr, memmem};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::{fmt, io};

/// Case-insensitive Cow String type for header keys
#[derive(Debug, Eq, Clone)]
pub struct CaseInsensitiveKey<'a>(Cow<'a, str>);

/// Helper shorthand for constructing a [`CaseInsensitiveKey`] from a string slice.
pub fn ci<S: AsRef<str> + ?Sized>(s: &S) -> CaseInsensitiveKey<'_> {
    CaseInsensitiveKey::from(s.as_ref())
}

impl PartialEq for CaseInsensitiveKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialEq<str> for CaseInsensitiveKey<'_> {
    fn eq(&self, other: &str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl PartialEq<&str> for CaseInsensitiveKey<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl PartialEq<CaseInsensitiveKey<'_>> for str {
    fn eq(&self, other: &CaseInsensitiveKey<'_>) -> bool {
        self.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialEq<CaseInsensitiveKey<'_>> for &str {
    fn eq(&self, other: &CaseInsensitiveKey<'_>) -> bool {
        self.eq_ignore_ascii_case(&other.0)
    }
}

impl std::hash::Hash for CaseInsensitiveKey<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state);
    }
}

impl From<CaseInsensitiveKey<'_>> for String {
    fn from(key: CaseInsensitiveKey<'_>) -> Self {
        key.0.to_string()
    }
}

impl From<String> for CaseInsensitiveKey<'_> {
    fn from(key: String) -> Self {
        Self(Cow::Owned(key))
    }
}

impl<'a> From<&'a str> for CaseInsensitiveKey<'a> {
    fn from(key: &'a str) -> Self {
        Self(Cow::Borrowed(key))
    }
}

impl<'a> From<Cow<'a, str>> for CaseInsensitiveKey<'a> {
    fn from(key: Cow<'a, str>) -> Self {
        Self(key)
    }
}

impl<'a> Deref for CaseInsensitiveKey<'a> {
    type Target = Cow<'a, str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default, Debug, Eq, PartialEq, Clone)]
pub enum HeaderEncoding {
    #[default]
    Unicode,
    Latin1,
}

/// Header key or value offsets used in CoW headers.
pub(super) type HeaderOffset = (usize, usize);

/// Individual CoW header value (used for status line).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CowHeaderValue {
    Offsets(HeaderOffset),
    Owned(Vec<u8>),
}

#[inline]
fn _offset_slice(buf: &[u8], offsets: HeaderOffset) -> &[u8] {
    &buf[offsets.0..offsets.1]
}

impl CowHeaderValue {
    #[inline]
    fn as_slice<'a>(&'a self, raw_header_block: &'a [u8]) -> &'a [u8] {
        match self {
            CowHeaderValue::Offsets(offsets) => _offset_slice(raw_header_block, *offsets),
            CowHeaderValue::Owned(value) => value.as_slice(),
        }
    }
}

/// CoW header (key, value) tuple.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CowHeaderTuple {
    Offsets(HeaderOffset, HeaderOffset),
    Owned(Vec<u8>, Vec<u8>),
}

impl CowHeaderTuple {
    #[inline]
    fn key<'a>(&'a self, raw_header_block: &'a [u8]) -> &'a [u8] {
        match self {
            CowHeaderTuple::Offsets(k, _) => _offset_slice(raw_header_block, *k),
            CowHeaderTuple::Owned(k, _) => k.as_slice(),
        }
    }

    #[inline]
    fn value<'a>(&'a self, raw_header_block: &'a [u8]) -> &'a [u8] {
        match self {
            CowHeaderTuple::Offsets(_, v) => _offset_slice(raw_header_block, *v),
            CowHeaderTuple::Owned(_, v) => v.as_slice(),
        }
    }
}

/// Internal helper for trimming slice offsets to exclude leading and trailing white space.
#[inline]
fn _trim_ascii_offsets(value: &[u8]) -> HeaderOffset {
    let start = value.len() - value.trim_ascii_start().len();
    let end = value.trim_ascii_end().len();
    (start, end)
}

/// Internal helper for trimming leading and trailing white space and
/// removing internal CR and LF characters. Also strips `':'` if `is_key == true`.
fn _sanitize_header_value(value: &[u8], is_key: bool) -> Vec<u8> {
    let mut value_sanitized = Vec::with_capacity(value.len());
    value_sanitized.extend(value.trim_ascii().iter().flat_map(|b| match b {
        b'\r' => None,
        b'\n' => Some(b' '),
        b':' if is_key => None,
        other => Some(*other),
    }));
    value_sanitized
}

/// Multimap structure representing a WARC or HTTP header block.
///
/// Headers can be set or retrieved by key and the whole header block can be
/// serialized to an [`io::BufRead`] stream.
///
/// WARC headers should be created with [`HeaderEncoding::Unicode`].
/// HTTP headers should use [`HeaderEncoding::Latin1`]. However, in either case,
/// you should still avoid non-ASCII characters).
#[derive(Default, Debug, Clone)]
pub struct HeaderMap {
    pub(super) encoding: HeaderEncoding,
    pub(super) raw_header_block: Vec<u8>,
    pub(super) status_line: Option<CowHeaderValue>,
    pub(super) headers: Vec<CowHeaderTuple>,
}

impl HeaderMap {
    /// Create a new header map with the specified encoding.
    ///
    /// # Arguments
    ///
    /// * `encoding` - Header source encoding
    pub fn new(encoding: HeaderEncoding) -> Self {
        HeaderMap {
            encoding,
            raw_header_block: Vec::new(),
            status_line: None,
            headers: Vec::new(),
        }
    }

    /// Get the header encoding.
    pub fn encoding(&self) -> HeaderEncoding {
        self.encoding.clone()
    }

    /// Encode a header value as either UTF-8 or Latin1.
    /// Decoding is lossy. Invalid characters are replaced.
    ///
    /// # Arguments
    ///
    /// * `s` - Byte sequence to decode
    #[inline]
    fn _encode<'a>(&self, s: &'a str) -> Cow<'a, [u8]> {
        match &self.encoding {
            HeaderEncoding::Unicode => Cow::Borrowed(s.as_bytes()),
            HeaderEncoding::Latin1 => Cow::Owned(WINDOWS_1252.encode(s, EncoderTrap::Ignore).unwrap_or_default()),
        }
    }

    /// Decode a header value as either Unicode or Latin1.
    /// Decoding is lossy. Invalid characters are replaced.
    ///
    /// # Arguments
    ///
    /// * `b` - Byte sequence to decode
    #[inline]
    fn _decode<'a>(&self, b: &'a [u8]) -> Cow<'a, str> {
        match &self.encoding {
            HeaderEncoding::Unicode => String::from_utf8_lossy(b),
            HeaderEncoding::Latin1 => Cow::Owned(WINDOWS_1252.decode(b, DecoderTrap::Ignore).unwrap_or_default()),
        }
    }

    /// Parse a WARC or HTTP header block from a stream and populate the header map.
    ///
    /// The default maximum accepted header length is 32 KiB. If headers are longer, an error is returned.
    /// Use [`Self::parse_with_with_opts()`] to set a different maximum.
    ///
    /// # Arguments
    ///
    /// * `reader` - Buffered reader
    /// * `has_status_line` - Whether the first line is a status line or already a header.
    ///
    /// # Returns
    ///
    /// Number of bytes read from the reader or IO error
    #[inline]
    pub fn parse(&mut self, reader: &mut dyn BufReadSeek, has_status_line: bool) -> Result<usize, io::Error> {
        self.parse_with_with_opts(reader, has_status_line, 32 << 10, false)
    }

    /// Internal raw header block reader implementation with clean hot path (no quirks mode)..
    #[inline]
    fn read_raw_header_block(
        reader: &mut dyn BufReadSeek,
        raw_header_block: &mut Vec<u8>,
        max_header_len: usize,
    ) -> Result<usize, io::Error> {
        let mut pending_crlf = 0usize;
        let mut bytes_consumed = 0usize;
        let crlf_finder = memmem::Finder::new("\r\n\r\n");

        loop {
            let in_buf = reader.fill_buf()?;
            if in_buf.is_empty() {
                return Ok(bytes_consumed);
            }

            if pending_crlf > 0 {
                // CRLF bytes pending, check for split EOH marker.
                let sep_remaining = 4 - pending_crlf;
                if in_buf.len() >= sep_remaining && in_buf[..sep_remaining] == b"\r\n\r\n"[pending_crlf..] {
                    // Ensure we have only one CRLF at the end.
                    raw_header_block.truncate(raw_header_block.len() - pending_crlf);
                    raw_header_block.extend_from_slice(b"\r\n");
                    reader.consume(sep_remaining);
                    bytes_consumed += sep_remaining;
                    return Ok(bytes_consumed);
                }
            }

            let eoh = crlf_finder.find(in_buf);
            let n = eoh.unwrap_or(in_buf.len());
            if raw_header_block.len() + n > max_header_len {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Maximum header length exceeded."));
            }
            raw_header_block.extend_from_slice(&in_buf[..n]);
            reader.consume(n);
            bytes_consumed += n;
            if eoh.is_some() {
                // Add one line separator, but consume both.
                raw_header_block.extend_from_slice(b"\r\n");
                reader.consume(4);
                bytes_consumed += 4;
                return Ok(bytes_consumed);
            }

            // Count trailing CRLF bytes for split EOH marker detection.
            pending_crlf = match raw_header_block.as_slice() {
                [.., b'\r', b'\n', b'\r'] => 3,
                [.., b'\r', b'\n'] => 2,
                [.., b'\r'] => 1,
                _ => 0,
            };
        }
    }

    /// Internal raw header block reader implementation with quirks mode handling in the hot path.
    #[inline]
    fn read_raw_header_block_quirks_mode(
        reader: &mut dyn BufReadSeek,
        raw_header_block: &mut Vec<u8>,
        max_header_len: usize,
    ) -> Result<usize, io::Error> {
        let mut pending_crlf = 0usize;
        let mut pending_lf = 0usize;
        let mut bytes_consumed = 0usize;
        let crlf_finder = memmem::Finder::new("\r\n\r\n");
        let lf_finder = memmem::Finder::new("\n\n");

        loop {
            let in_buf = reader.fill_buf()?;
            if in_buf.is_empty() {
                return Ok(bytes_consumed);
            }

            // CRLF bytes pending, check for split EOH marker.
            let crlf_eoh_split =
                if pending_crlf > 0 {
                    let sep_remaining = 4 - pending_crlf;
                    (in_buf.len() >= sep_remaining && in_buf[..sep_remaining] == b"\r\n\r\n"[pending_crlf..])
                        .then_some((raw_header_block.len() - pending_crlf, pending_crlf, sep_remaining, 4usize))
                } else {
                    None
                };
            // LF bytes pending, check for split EOH marker.
            let lf_eoh_split = if pending_lf > 0 {
                let sep_remaining = 2 - pending_lf;
                (in_buf.len() >= sep_remaining && in_buf[..sep_remaining] == b"\n\n"[pending_lf..]).then_some((
                    raw_header_block.len() - pending_lf,
                    pending_lf,
                    sep_remaining,
                    2usize,
                ))
            } else {
                None
            };
            let eoh_split = match (crlf_eoh_split, lf_eoh_split) {
                (Some(a), None) => Some(a),
                (Some(a), Some(b)) => Some(if a.0 <= b.0 { a } else { b }),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };
            // Split EOH found.
            if let Some((pos, _, sep_remaining, sep_len)) = eoh_split {
                raw_header_block.truncate(pos);
                // Add one line separator, but consume both.
                if sep_len == 2 {
                    raw_header_block.push(b'\n');
                } else {
                    raw_header_block.extend_from_slice(b"\r\n");
                }
                reader.consume(sep_remaining);
                bytes_consumed += sep_remaining;
                return Ok(bytes_consumed);
            }

            // Check for non-split EOH.
            let crlf_eoh = crlf_finder.find(in_buf).map(|pos| (pos, 4usize));
            let lf_eoh = lf_finder.find(in_buf).map(|pos| (pos, 2usize));
            let eoh = match (crlf_eoh, lf_eoh) {
                (Some(a), None) => Some(a),
                (Some(a), Some(b)) => Some(if a.0 <= b.0 { a } else { b }),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };
            let n = eoh.map_or(in_buf.len(), |(pos, _)| pos);
            if raw_header_block.len() + n > max_header_len {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Maximum header length exceeded."));
            }
            raw_header_block.extend_from_slice(&in_buf[..n]);
            reader.consume(n);
            bytes_consumed += n;
            // EOH found.
            if let Some((_, sep_len)) = eoh {
                // Add one line separator, but consume both.
                if sep_len == 2 {
                    raw_header_block.push(b'\n');
                } else {
                    raw_header_block.extend_from_slice(b"\r\n");
                }
                reader.consume(sep_len);
                bytes_consumed += sep_len;
                return Ok(bytes_consumed);
            }

            // Count trailing CRLF bytes for split EOH marker detection.
            pending_crlf = match raw_header_block.as_slice() {
                [.., b'\r', b'\n', b'\r'] => 3,
                [.., b'\r', b'\n'] => 2,
                [.., b'\r'] => 1,
                _ => 0,
            };
            // Check for trailing LF byte for split quirks EOH marker detection.
            pending_lf = (raw_header_block.last() == Some(&b'\n')).into();
        }
    }

    /// Parse a WARC or HTTP header block from a stream and populate the header map.
    ///
    /// If a parsed header exceeds `max_header_len`, an error is returned.
    ///
    /// Quirks mode allows parsing of headers terminated with only LF instead of CRLF.
    ///
    /// # Arguments
    ///
    /// * `reader` - Buffered reader
    /// * `has_status_line` - Whether the first line is a status line or already a header.
    /// * `max_header_len` - Maximum accepted header length in bytes.
    /// * `quirks_mode` - Whether to allow parsing of headers terminated with only LF instead of CRLF.
    ///
    /// # Returns
    ///
    /// Number of bytes read from the reader or IO error
    pub fn parse_with_with_opts(
        &mut self,
        reader: &mut dyn BufReadSeek,
        has_status_line: bool,
        max_header_len: usize,
        quirks_mode: bool,
    ) -> Result<usize, io::Error> {
        self.clear();
        self.raw_header_block
            .reserve(768usize.saturating_sub(self.raw_header_block.capacity()));
        let bytes_consumed = if quirks_mode {
            Self::read_raw_header_block_quirks_mode(reader, &mut self.raw_header_block, max_header_len)?
        } else {
            Self::read_raw_header_block(reader, &mut self.raw_header_block, max_header_len)?
        };
        self.headers.reserve(bytes_consumed / 64);

        let mut expect_first_line = has_status_line;
        let finder = memmem::Finder::new("\r\n");
        let mut pos = 0;
        while pos < self.raw_header_block.len() {
            let line_start = pos;
            let eol_match = if quirks_mode {
                memchr(b'\n', &self.raw_header_block[pos..])
            } else {
                finder.find(&self.raw_header_block[pos..])
            };
            let eol = eol_match.unwrap_or(self.raw_header_block.len() - pos);
            let mut line = &self.raw_header_block[pos..pos + eol];
            pos = if quirks_mode {
                line = line.strip_suffix(b"\r").unwrap_or(line);
                pos + eol + 1
            } else {
                pos + eol + 2
            };

            let offsets = memchr(b':', line).map(|colon_pos| {
                let key = _trim_ascii_offsets(&line[..colon_pos]);
                let value = _trim_ascii_offsets(&line[colon_pos + 1..]);
                (
                    (line_start + key.0, line_start + key.1),
                    (line_start + colon_pos + 1 + value.0, line_start + colon_pos + 1 + value.1),
                )
            });

            // Status line.
            if expect_first_line {
                let trimmed = _trim_ascii_offsets(line);
                self.status_line = Some(CowHeaderValue::Offsets((line_start + trimmed.0, line_start + trimmed.1)));
                expect_first_line = false;
                continue;
            }

            // Indented continuation line.
            if matches!(line.first(), Some(b' ' | b'\t')) {
                let trimmed = self.raw_header_block[line_start..pos - if quirks_mode { 1 } else { 2 }]
                    .strip_suffix(b"\r")
                    .unwrap_or(&self.raw_header_block[line_start..pos - if quirks_mode { 1 } else { 2 }])
                    .trim_ascii();
                match self.headers.last_mut() {
                    Some(last) => match last {
                        CowHeaderTuple::Offsets(ko, vo) => {
                            let key = _offset_slice(&self.raw_header_block, *ko).to_vec();
                            let mut value = _offset_slice(&self.raw_header_block, *vo).to_vec();
                            value.reserve(trimmed.len() + 1);
                            value.extend_from_slice(b" ");
                            value.extend_from_slice(trimmed);
                            *last = CowHeaderTuple::Owned(key, value);
                        }
                        CowHeaderTuple::Owned(_, value) => {
                            value.reserve(trimmed.len() + 1);
                            value.extend_from_slice(b" ");
                            value.extend_from_slice(trimmed);
                        }
                    },
                    None => self.headers.push(CowHeaderTuple::Owned(Vec::new(), trimmed.to_vec())),
                }
                continue;
            }

            if let Some((key, value)) = offsets {
                self.headers.push(CowHeaderTuple::Offsets(key, value));
            }
        }

        Ok(bytes_consumed)
    }

    /// Get the header status line.
    pub fn status_line(&self) -> Option<Cow<'_, str>> {
        self.status_line
            .as_ref()
            .map(|s| self._decode(s.as_slice(&self.raw_header_block)))
    }

    /// Get the raw status line as bytes.
    pub fn status_line_bytes(&self) -> Option<Cow<'_, [u8]>> {
        self.status_line
            .as_ref()
            .map(|s| Cow::Borrowed(s.as_slice(&self.raw_header_block)))
    }

    /// Set status line contents.
    ///
    /// # Arguments
    ///
    /// * `status_line` - New status line
    pub fn set_status_line(&mut self, status_line: impl AsRef<str>) {
        self.set_status_line_bytes(self._encode(status_line.as_ref()).as_ref());
    }

    /// Set status line contents.
    ///
    /// # Arguments
    ///
    /// * `status_line` - New status line
    pub fn set_status_line_bytes(&mut self, status_line: impl AsRef<[u8]>) {
        let status_line = status_line.as_ref();
        let mut status_line_sanitized = Vec::with_capacity(status_line.len());
        status_line_sanitized.extend(_sanitize_header_value(status_line, true));
        self.status_line = Some(CowHeaderValue::Owned(status_line_sanitized));
    }

    /// HTTP status code (unset if header block is not an HTTP header block).
    pub fn status_code(&self) -> Option<u16> {
        let Some(s) = &self.status_line else {
            return None;
        };
        let s = s.as_slice(&self.raw_header_block);
        if !s.starts_with(b"HTTP/") {
            return None;
        }
        let mut parts = s.splitn(3, |&b| b == b' ');
        // Skip HTTP/
        parts.next()?;
        self._decode(parts.next()?).parse::<u16>().ok()
    }

    /// HTTP reason phrase.
    /// Returns None if the header block is not an HTTP header block or no reason phrase was given.
    pub fn reason_phrase(&self) -> Option<Cow<'_, str>> {
        let Some(s) = &self.status_line else {
            return None;
        };
        let s = s.as_slice(&self.raw_header_block);
        if !s.starts_with(b"HTTP/") {
            return None;
        }
        let mut parts = s.splitn(3, |&b| b == b' ');
        // Skip HTTP/ and status code
        parts.next()?;
        parts.next()?;
        Some(self._decode(parts.next()?))
    }

    /// Get value for a (case-insensitive) header key.
    /// If the header is present multiple times, only the first occurrence is returned.
    /// Use [`Self::get_multiple()`] if you want all values.
    ///
    /// Returns `None` if the header is not present.
    ///
    /// # Arguments
    ///
    /// * `key` - Header key
    pub fn get(&self, key: impl AsRef<str>) -> Option<Cow<'_, str>> {
        self.headers
            .iter()
            .find(|h| {
                h.key(&self.raw_header_block)
                    .eq_ignore_ascii_case(key.as_ref().as_bytes())
            })
            .map(|h| self._decode(h.value(&self.raw_header_block)))
    }

    /// Get all values for a (case-insensitive) header key.
    /// Returns a vector of all values for the given key. Can return more than
    /// one element if the header is present multiple times.
    ///
    /// # Arguments
    ///
    /// * `key` - Header key
    pub fn get_multiple(&self, key: impl AsRef<str>) -> Vec<Cow<'_, str>> {
        self.headers
            .iter()
            .filter(|h| {
                h.key(&self.raw_header_block)
                    .eq_ignore_ascii_case(key.as_ref().as_bytes())
            })
            .map(|h| self._decode(h.value(&self.raw_header_block)))
            .collect()
    }

    /// Get byte value for a (case-insensitive) header key.
    /// If the header is present multiple times, only the first occurrence is returned.
    /// Use [`Self::get_bytes_multiple()`] if you want all values.
    ///
    /// Returns `None` if the header is not present.
    ///
    /// # Arguments
    ///
    /// * `key` - Header key
    pub fn get_bytes(&self, key: impl AsRef<[u8]>) -> Option<Cow<'_, [u8]>> {
        let key = key.as_ref();
        self.headers
            .iter()
            .find(|h| h.key(&self.raw_header_block).eq_ignore_ascii_case(key))
            .map(|h| Cow::Borrowed(h.value(&self.raw_header_block)))
    }

    /// Get all byte values for a (case-insensitive) header key.
    /// Returns a vector of all values for the given key. Can return more than
    /// one element if the header is present multiple times.
    ///
    /// # Arguments
    ///
    /// * `key` - Header key as bytes
    pub fn get_bytes_multiple(&self, key: impl AsRef<[u8]>) -> Vec<Cow<'_, [u8]>> {
        let key = key.as_ref();
        self.headers
            .iter()
            .filter(|h| h.key(&self.raw_header_block).eq_ignore_ascii_case(key))
            .map(|h| Cow::Borrowed(h.value(&self.raw_header_block)))
            .collect()
    }

    /// Check if a (case-insensitive) header key exists.
    ///
    /// # Arguments
    ///
    /// * `key` - Header key
    pub fn contains_key(&self, key: impl AsRef<str>) -> bool {
        let key_bytes = self._encode(key.as_ref());
        self.headers
            .iter()
            .any(|h| h.key(&self.raw_header_block).eq_ignore_ascii_case(key_bytes.as_ref()))
    }

    /// Check if a (case-insensitive) header key exists.
    ///
    /// # Arguments
    ///
    /// * `key` - Header key as bytes
    pub fn contains_key_bytes(&self, key: impl AsRef<[u8]>) -> bool {
        let key = key.as_ref();
        self.headers
            .iter()
            .any(|h| h.key(&self.raw_header_block).eq_ignore_ascii_case(key))
    }

    /// Insert a new header and overwrite any existing header(s) if the key already exists.
    ///
    /// Insertion is not efficient and causes a full traversal of all headers.
    /// If a header already exists, its first occurrence will be updated and
    /// all following occurrences will be dropped. If duplicate headers are not a problem,
    /// use [`Self::append()`] instead for better efficiency.
    ///
    /// All data is represented internally as bytes to avoid encoding/decoding overhead
    /// and potential errors. Therefore, if you have data as bytes already, using
    /// [`Self::set_bytes()`] is slightly more efficient.
    ///
    /// # Arguments
    ///
    /// * `key` - Header key
    /// * `value` - Header value
    pub fn set(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) {
        let key = self._encode(key.as_ref());
        let value = self._encode(value.as_ref());
        self.set_bytes(key.as_ref(), value.as_ref());
    }

    /// Insert a new header and overwrite any existing header(s) if the key already exists.
    ///
    /// Insertion is not efficient and causes a full traversal of all headers.
    /// If a header already exists, its first occurrence will be updated and
    /// all following occurrences will be dropped. If duplicate headers are not a problem,
    /// use [`Self::append_bytes()`] instead for better efficiency.
    ///
    /// # Arguments
    ///
    /// * `key` - Header key as bytes
    /// * `value` - Header value as bytes
    pub fn set_bytes(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
        let key = key.as_ref();
        let value = value.as_ref();
        let mut key_lower = Vec::with_capacity(key.len());
        key_lower.extend(_sanitize_header_value(&key.to_ascii_lowercase(), true));

        let mut found = false;
        self.headers.retain_mut(|h| {
            if h.key(self.raw_header_block.as_slice()).to_ascii_lowercase() != key_lower {
                true
            } else if !found {
                *h = CowHeaderTuple::Owned(_sanitize_header_value(key, true), _sanitize_header_value(value, false));
                found = true;
                true
            } else {
                false
            }
        });
        if !found {
            self.headers
                .push(CowHeaderTuple::Owned(_sanitize_header_value(key, true), _sanitize_header_value(value, false)));
        }
    }

    /// Append a header.
    ///
    /// Appending a new header is efficient and does not check for existing headers
    /// with the same name.
    ///
    /// All data is represented internally as bytes to avoid encoding/decoding overhead
    /// and potential errors. Therefore, if you have data as bytes already, using
    /// [`Self::append_bytes()`] is slightly more efficient.
    ///
    /// # Arguments
    ///
    /// * `key` - Header key
    /// * `value` - Header value
    pub fn append(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) {
        let key = self._encode(key.as_ref());
        let value = self._encode(value.as_ref());
        self.append_bytes(key.as_ref(), value.as_ref());
    }

    /// Append a header.
    ///
    /// Appending a new header is efficient and does not check for existing headers
    /// with the same name.
    ///
    /// # Arguments
    ///
    /// * `key` - Header key as bytes
    /// * `value` - Header value as bytes
    pub fn append_bytes(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
        let key = key.as_ref();
        let value = value.as_ref();
        self.headers
            .push(CowHeaderTuple::Owned(_sanitize_header_value(key, true), _sanitize_header_value(value, false)));
    }

    /// Internal function for appending a header without sanitization
    /// (assumes data is already sanitized). Still trims leading and trailing white space.
    pub(super) fn append_bytes_no_sanitize(&mut self, key: &[u8], value: &[u8]) {
        self.headers
            .push(CowHeaderTuple::Owned(key.trim_ascii().to_vec(), value.trim_ascii().to_vec()));
    }

    /// Remove a header if it exists.
    pub fn remove(&mut self, key: impl AsRef<str>) {
        let key = self._encode(key.as_ref());
        self.remove_bytes(key.as_ref());
    }

    /// Remove a header if it exists.
    pub fn remove_bytes(&mut self, key: impl AsRef<[u8]>) {
        let key = key.as_ref();
        let key = _sanitize_header_value(key, true);
        let raw_header_block = self.raw_header_block.as_slice();
        self.headers
            .retain(|h| !h.key(raw_header_block).eq_ignore_ascii_case(key.as_slice()));
    }

    /// Iterator of keys and values.
    pub fn items(&self) -> impl Iterator<Item = (Cow<'_, str>, Cow<'_, str>)> {
        self.headers
            .iter()
            .map(|h| (self._decode(h.key(&self.raw_header_block)), self._decode(h.value(&self.raw_header_block))))
    }

    /// Zero-copy iterator of keys and values as bytes.
    pub fn items_bytes(&self) -> impl Iterator<Item = (Cow<'_, [u8]>, Cow<'_, [u8]>)> {
        self.headers
            .iter()
            .map(|h| (Cow::Borrowed(h.key(&self.raw_header_block)), Cow::Borrowed(h.value(&self.raw_header_block))))
    }

    /// Iterator of header keys.
    pub fn keys(&'_ self) -> impl Iterator<Item = CaseInsensitiveKey<'_>> {
        self.headers
            .iter()
            .map(|h| CaseInsensitiveKey(self._decode(h.key(&self.raw_header_block))))
    }

    /// Zero-copy iterator of header keys as bytes.
    pub fn keys_bytes(&self) -> impl Iterator<Item = Cow<'_, [u8]>> {
        self.headers
            .iter()
            .map(|h| Cow::Borrowed(h.key(&self.raw_header_block)))
    }

    /// Iterator of header values.
    pub fn values(&self) -> impl Iterator<Item = Cow<'_, str>> {
        self.headers
            .iter()
            .map(|h| self._decode(h.value(&self.raw_header_block)))
    }

    /// Zero-copy iterator of header values as bytes.
    pub fn values_bytes(&self) -> impl Iterator<Item = Cow<'_, [u8]>> {
        self.headers
            .iter()
            .map(|h| Cow::Borrowed(h.value(&self.raw_header_block)))
    }

    /// Return the headers as a [`HashMap`] of Unicode strings.
    ///
    /// If multiple headers have the same key, their values will be concatenated with `","`.
    pub fn to_map(&'_ self) -> HashMap<CaseInsensitiveKey<'_>, String> {
        let mut map: HashMap<CaseInsensitiveKey, String> = HashMap::new();
        self.items().for_each(|(k, v)| {
            map.entry(CaseInsensitiveKey(k))
                .and_modify(|v_| {
                    v_.push(',');
                    v_.push_str(&v);
                })
                .or_insert(v.to_string());
        });
        map
    }

    /// Get the number of headers.
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    /// Check if the header map is empty.
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    /// Clear all headers and the status line.
    pub fn clear(&mut self) {
        self.raw_header_block.clear();
        self.headers.clear();
        self.status_line = None;
    }

    /// Write the header block onto a stream.
    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes_written = 0usize;
        if let Some(s) = &self.status_line
            && !s.as_slice(&self.raw_header_block).is_empty()
        {
            let status_line = s.as_slice(&self.raw_header_block);
            writer.write_all(status_line)?;
            writer.write_all(b"\r\n")?;
            bytes_written += status_line.len() + 2;
        }
        for header in &self.headers {
            let key = header.key(&self.raw_header_block);
            let value = header.value(&self.raw_header_block);
            if !key.is_empty() {
                writer.write_all(key)?;
                writer.write_all(b": ")?;
                bytes_written += key.len() + 2;
            }
            writer.write_all(value)?;
            writer.write_all(b"\r\n")?;
            bytes_written += value.len() + 2;
        }
        // Header end
        writer.write_all(b"\r\n")?;
        bytes_written += 2;

        Ok(bytes_written)
    }
}

impl PartialEq for HeaderMap {
    fn eq(&self, other: &Self) -> bool {
        self.encoding == other.encoding
            && self.status_line.as_ref().map(|s| s.as_slice(&self.raw_header_block))
                == other.status_line.as_ref().map(|s| s.as_slice(&other.raw_header_block))
            && self.headers.len() == other.headers.len()
            && self.headers.iter().zip(other.headers.iter()).all(|(left, right)| {
                left.key(&self.raw_header_block) == right.key(&other.raw_header_block)
                    && left.value(&self.raw_header_block) == right.value(&other.raw_header_block)
            })
    }
}

impl Eq for HeaderMap {}

impl Display for HeaderMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buf = Vec::with_capacity(self.len() * 40);
        let _ = self.write(&mut buf);
        let headers_str = match self.encoding {
            HeaderEncoding::Unicode => String::from_utf8_lossy(&buf),
            HeaderEncoding::Latin1 => Cow::Owned(WINDOWS_1252.decode(&buf, DecoderTrap::Ignore).unwrap_or_default()),
        };
        write!(f, "{}", headers_str)
    }
}

// ===========================================================
// Tests
// ===========================================================

#[cfg(test)]
#[path = "header_test.rs"]
mod header_test;
