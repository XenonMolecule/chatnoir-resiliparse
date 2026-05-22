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

use crate::stream_io::LimitedBufReader;
use crate::stream_io::gzip::GzipReader;
use crate::stream_io::lz4::Lz4Reader;
use crate::stream_io::traits::{IntoWarcReader, WarcRead};
use crate::stream_io::zstd::ZstdReader;
use crate::warc::record::{AutoDecode, ReaderType, WarcRecord};
use std::cell::RefCell;
use std::io::BufRead;
use std::io::{self, BufReader};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[doc(hidden)]
/// Convenience wrapper for iterating [`WarcRecord`] instances from a stream.
///
/// This is an internal generic interface for implementing the single-threaded and
/// thread-safe iterator variants.
#[derive(Debug)]
pub struct ArchiveIteratorImpl<R> {
    cur: R,
    stream_started: bool,
    options: ArchiveIteratorOptions,
}

/// Options for constructing a new [`ArchiveIterator`] or [`ArchiveIteratorThreadSafe`]..
///
/// # Options
///
/// * `parse_http` - automatically parse HTTP headers.
/// * `decode_http_payload` - automatically decode transfer- or content-encoded HTTP payloads.
///   Requires `parse_http=true`.to have an effect.
/// * `verify_digests` - automatically verify record digest.
/// * `quirks_mode` - enable strict record parsing (set to `false` for quirks mode --
///   required for some ClueWebs).
/// * `stream_detect` - automatically detect the stream compression type. If enabled, automatically
///   detects the stream compression type based on the file extension or magic bytes.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ArchiveIteratorOptions {
    pub stream_detect: bool,
    pub parse_http: bool,
    pub decode_http_payload: AutoDecode,
    pub verify_digests: bool,
    pub quirks_mode: bool,
}

impl Default for ArchiveIteratorOptions {
    fn default() -> Self {
        Self {
            stream_detect: true,
            parse_http: true,
            decode_http_payload: AutoDecode::None,
            verify_digests: false,
            quirks_mode: false,
        }
    }
}

/// Convenience wrapper for iterating [`WarcRecord`] instances from a stream.
///
/// Using the record iterator is roughly equivalent to creating an initial record instance
/// with [`WarcRecord::from_reader()`] and then retrieving each new record by calling
/// [`WarcRecord::next()`] on the previous record. The main difference is that this iterator
/// returns shared mutable references to allow iteration over multiple records using a single
/// iterator instance. This enables the usage within loops and allows applying map functions
/// to multiple records in a row.
pub type ArchiveIterator = ArchiveIteratorImpl<Rc<RefCell<WarcRecord>>>;

/// Thread-safe convenience wrapper for iterating [`WarcRecord`] instances from a stream.
///
/// This is a thread-safe variant of [`ArchiveIterator`].
///
/// Using the record iterator is roughly equivalent to creating an initial record instance
/// with [`WarcRecord::from_reader()`] and then retrieving each new record by calling
/// [`WarcRecord::next()`] on the previous record. The main difference is that this iterator
/// returns shared mutable references to allow iteration over multiple records using a single
/// iterator instance. This enables the usage within loops and allows applying map functions
/// to multiple records in a row.
pub type ArchiveIteratorThreadSafe = ArchiveIteratorImpl<Arc<Mutex<WarcRecord>>>;

/// Reference-counted handle for holding [`WarcRecord`] instances.
///
/// This is a common accessor trait for the two reference-counted types
/// `Rc<RefCell<WarcRecord>>` and `Arc<Mutex<WarcRecord>>` used by [`ArchiveIterator`]
/// and [`ArchiveIteratorThreadSafe`], respectively.
pub trait SharedWarcRecord: Clone {
    /// Create a new reference-counted [`WarcRecord`] handle.
    fn new(record: WarcRecord) -> Self;

    /// Replace the currently held [`WarcRecord`] with another one.
    fn replace(&self, record: WarcRecord);

    /// Execute the closure with a mutable reference to the held record.
    /// This is to abstract from the individual accessors of the underlying reference
    /// counting mechanism ([`Rc::borrow_mut()`] vs. [`Mutex::lock()::unwrap()`](Mutex::lock()).
    fn with_mut<R>(&self, f: impl FnOnce(&mut WarcRecord) -> R) -> R;
}

impl SharedWarcRecord for Rc<RefCell<WarcRecord>> {
    fn new(record: WarcRecord) -> Self {
        Rc::new(RefCell::new(record))
    }

    fn replace(&self, record: WarcRecord) {
        *self.borrow_mut() = record;
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut WarcRecord) -> R) -> R {
        f(&mut self.borrow_mut())
    }
}

impl SharedWarcRecord for Arc<Mutex<WarcRecord>> {
    fn new(record: WarcRecord) -> Self {
        Arc::new(Mutex::new(record))
    }

    fn replace(&self, record: WarcRecord) {
        *self.lock().unwrap() = record;
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut WarcRecord) -> R) -> R {
        f(&mut self.lock().unwrap())
    }
}

impl<R> ArchiveIteratorImpl<R>
where
    R: SharedWarcRecord,
{
    /// Create a new WARC record iterator from a buffered WARC stream reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - buffered reader instance to attach to records.
    pub fn new(reader: impl IntoWarcReader) -> Self {
        Self::with_options(reader, ArchiveIteratorOptions::default())
    }

    /// Create a new WARC record iterator from a buffered WARC stream reader with options.
    ///
    /// # Arguments
    ///
    /// * `reader` - buffered reader instance to attach to records.
    /// * `options` - custom iterator options.
    pub fn with_options(reader: impl IntoWarcReader, options: ArchiveIteratorOptions) -> Self {
        let mut empty = WarcRecord::new();
        if options.quirks_mode {
            empty.quirks_mode = true;
        }
        empty.attach_reader(reader);
        Self {
            cur: R::new(empty),
            stream_started: false,
            options,
        }
    }
    /// Create an archive iterator from a WARC file path.
    ///
    /// If the file name ends in `.warc.gz`, `.warc.zst`, or `.warc.lz4`, the correct
    /// decompressor will automatically be chosen. No further autodetection based on
    /// magic bytes will be performed after that.
    ///
    /// # Arguments
    ///
    /// * `path` - path to WARC file.
    pub fn from_path(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
        Self::from_path_with_options(path, Default::default())
    }

    /// Create an archive iterator from a WARC file path with the supplied options..
    ///
    /// If `options.stream_detect` is `true`, and the file name ends in `.gz`, `.zst`,
    /// or `.lz4`, the correct decompressor will automatically be chosen.
    /// No further autodetection based on magic bytes will be performed after that.
    ///
    /// # Arguments
    ///
    /// * `path` - path to WARC file.
    /// * `options` - options for constructing the iterator.
    pub fn from_path_with_options(
        path: impl AsRef<std::path::Path>,
        mut options: ArchiveIteratorOptions,
    ) -> io::Result<Self> {
        let reader = BufReader::new(std::fs::File::open(&path)?);
        if options.stream_detect {
            let reader: Box<dyn WarcRead> = match path.as_ref().extension().and_then(|e| e.to_str()) {
                Some("gz") => Box::new(GzipReader::new(reader)),
                Some("zst") => Box::new(ZstdReader::new(reader)),
                Some("lz4") => Box::new(Lz4Reader::new(reader)),
                _ => Box::new(reader.into_warc_reader()),
            };
            options.stream_detect = false;
            return Ok(Self::with_options(reader, options));
        }
        Ok(Self::with_options(reader, options))
    }

    /// Change the iterator options.
    pub fn set_options(&mut self, options: ArchiveIteratorOptions) {
        self.options = options;
    }

    /// Enable or disable automatic stream compression detection.
    pub fn set_stream_detect(&mut self, stream_detect: bool) {
        self.options.stream_detect = stream_detect;
    }

    /// Consuming setter: Enable or disable automatic stream compression detection.
    pub fn with_stream_detect(mut self, stream_detect: bool) -> Self {
        self.set_stream_detect(stream_detect);
        self
    }

    /// Enable or disable automatic HTTP parsing on iterated records.
    pub fn set_parse_http(&mut self, parse_http: bool) {
        self.options.parse_http = parse_http;
    }

    /// Consuming setter: Enable or disable automatic HTTP parsing on iterated records.
    pub fn with_parse_http(mut self, parse_http: bool) -> Self {
        self.set_parse_http(parse_http);
        self
    }

    /// Enable or disable automatic decoding of transfer- or content-encoded HTTP payloads.
    /// Has no effect if HTTP parsing is disabled.
    pub fn set_decode_http_payload(&mut self, auto_decode: AutoDecode) {
        self.options.decode_http_payload = auto_decode;
    }

    /// Consuming setter: Enable or disable automatic decoding of transfer- or content-encoded
    /// HTTP payloads. Has no effect if HTTP parsing is disabled.
    pub fn with_decode_http_payload(mut self, auto_decode: AutoDecode) -> Self {
        self.set_decode_http_payload(auto_decode);
        self
    }

    /// Enable or disable skipping records with missing or invalid block digests.
    pub fn set_verify_digests(&mut self, verify_digests: bool) {
        self.options.verify_digests = verify_digests;
    }

    /// Consuming setter: Enable or disable skipping records with missing or invalid block digests.
    pub fn with_verify_digests(mut self, verify_digests: bool) -> Self {
        self.set_verify_digests(verify_digests);
        self
    }

    /// Enable or disable lenient record parsing. Enabling this ("quirks mode") may be
    /// required for some ClueWebs.
    pub fn set_quirks_mode(&mut self, quirks_mode: bool) {
        self.options.quirks_mode = quirks_mode;
        if self.options.quirks_mode {
            self.cur.with_mut(|record| record.quirks_mode = true);
        }
    }

    /// Consuming setter: Enable or disable lenient record parsing. Enabling this ("quirks mode")
    /// may be required for some ClueWebs.
    pub fn with_quirks_mode(mut self, quirks_mode: bool) -> Self {
        self.set_quirks_mode(quirks_mode);
        self
    }

    /// Create a new WARC record iterator with a filter predicate.
    ///
    /// # Arguments
    ///
    /// * `reader` - buffered reader instance to attach to records
    /// * `filter` - boolean filter predicate (must take a [`&mut WarcRecord`] as parameter)
    pub fn with_filter<F>(reader: impl IntoWarcReader, filter: F) -> FilteredArchiveIteratorImpl<R, F>
    where
        F: Fn(&mut WarcRecord) -> bool,
    {
        FilteredArchiveIteratorImpl {
            inner: ArchiveIteratorImpl::new(reader),
            pred: filter,
        }
    }

    /// Unwrap [`ArchiveIterator`] and return the reader instance.
    ///
    /// # Returns
    ///
    /// Reader instances originally attached to this iterator.
    pub fn into_inner(self) -> Option<Box<dyn WarcRead>> {
        self.cur.with_mut(WarcRecord::detach_reader)
    }

    fn detect_stream_compression_type(&mut self) -> io::Result<()> {
        self.cur.with_mut(|r| -> io::Result<()> {
            let magic_bytes = r.reader_mut().unwrap().fill_buf()?.get(..4).map(|b| b.to_vec());
            let Some(ReaderType::Original(reader)) = r.reader.take() else {
                return Err(io::Error::other("Inconsistent reader state."));
            };
            let reader = match magic_bytes.as_deref() {
                Some(b) if &b[..3] == b"\x1F\x8B\x08" => LimitedBufReader::new(Box::new(GzipReader::new(reader)), None),
                Some(b"\x28\xB5\x2F\xFD") => LimitedBufReader::new(Box::new(ZstdReader::new(reader)), None),
                Some(b"\x04\x22\x4D\x18") => LimitedBufReader::new(Box::new(Lz4Reader::new(reader)), None),
                _ => reader,
            };
            r.reader = Some(ReaderType::Original(reader));
            Ok(())
        })
    }
}

impl<S> Iterator for ArchiveIteratorImpl<S>
where
    S: SharedWarcRecord,
{
    type Item = Result<S, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.stream_started
            && self.options.stream_detect
            && let Err(e) = self.detect_stream_compression_type()
        {
            return Some(Err(e));
        }
        self.stream_started = true;

        loop {
            let next = self.cur.with_mut(WarcRecord::next)?;
            return match next {
                Ok(n) => {
                    self.cur = S::new(n);
                    let keep_record = self.cur.with_mut(|record| {
                        if self.options.verify_digests && !record.verify_block_digest(false).unwrap_or(false) {
                            return Ok(false);
                        }
                        if self.options.parse_http
                            && record.is_http()
                            && let Err(e) = record.parse_http_with_decode_opts(self.options.decode_http_payload)
                        {
                            return Err(e);
                        }
                        Ok(true)
                    });
                    match keep_record {
                        Ok(keep) if !keep => continue,
                        Err(e) => return Some(Err(e)),
                        _ => {}
                    };
                    Some(Ok(self.cur.clone()))
                }
                Err(e) => Some(Err(e)),
            };
        }
    }
}

#[doc(hidden)]
/// Filtered wrapper for [`ArchiveIterator`] that filters records based on a predicate.
/// Use [`ArchiveIterator::with_filter()`] to construct a [`FilteredArchiveIterator`].
///
/// This is an internal generic interface for implementing the single-threaded and
/// thread-safe iterator variants.
pub struct FilteredArchiveIteratorImpl<S, F>
where
    F: Fn(&mut WarcRecord) -> bool,
{
    inner: ArchiveIteratorImpl<S>,
    pred: F,
}

/// Filtered wrapper for [`ArchiveIterator`] that filters records based on a predicate.
/// Use [`ArchiveIterator::with_filter()`] to construct a [`FilteredArchiveIterator`].
pub type FilteredArchiveIterator<F> = FilteredArchiveIteratorImpl<Rc<RefCell<WarcRecord>>, F>;

/// Filtered wrapper for [`ArchiveIterator`] that filters records based on a predicate.
/// Use [`ArchiveIterator::with_filter()`] to construct a [`FilteredArchiveIterator`].
///
/// This is a thread-safe variant of [`FilteredArchiveIterator`].
pub type FilteredArchiveIteratorThreadSafe<F> = FilteredArchiveIteratorImpl<Arc<Mutex<WarcRecord>>, F>;

impl<S, F> Iterator for FilteredArchiveIteratorImpl<S, F>
where
    S: SharedWarcRecord,
    F: Fn(&mut WarcRecord) -> bool,
{
    type Item = Result<S, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.inner.next()?;
            return match next {
                Ok(n) => {
                    if !n.with_mut(|record| (self.pred)(record)) {
                        continue;
                    }
                    Some(Ok(n))
                }
                Err(e) => Some(Err(e)),
            };
        }
    }
}

impl<S, F> Deref for FilteredArchiveIteratorImpl<S, F>
where
    S: SharedWarcRecord,
    F: Fn(&mut WarcRecord) -> bool,
{
    type Target = ArchiveIteratorImpl<S>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S, F> DerefMut for FilteredArchiveIteratorImpl<S, F>
where
    S: SharedWarcRecord,
    F: Fn(&mut WarcRecord) -> bool,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Filter predicates to be used with [`ArchiveIterator::with_filter()`].
pub mod filter {
    use crate::warc::record::{WarcRecord, WarcRecordType};

    /// Filter predicate for checking if a record is a WARC/1.0 record.
    pub fn is_warc_10(record: &mut WarcRecord) -> bool {
        record
            .headers()
            .status_line_bytes()
            .is_some_and(|s| s.as_ref() == b"WARC/1.0")
    }

    /// Filter predicate for checking if a record is a WARC/1.1 record.
    pub fn is_warc_11(record: &mut WarcRecord) -> bool {
        record
            .headers()
            .status_line_bytes()
            .is_some_and(|s| s.as_ref() == b"WARC/1.1")
    }

    /// Filter predicate for checking if a record has a block digest.
    pub fn has_block_digest(record: &mut WarcRecord) -> bool {
        record.headers().contains_key_bytes(b"WARC-Block-Digest")
    }

    /// Filter predicate for checking if a record's WARC-Block-Digest is valid.
    /// This is a mutating filter that freezes the record!
    pub fn has_valid_block_digest(record: &mut WarcRecord) -> bool {
        record.verify_block_digest(false).unwrap_or(false)
    }

    /// Filter predicate for checking if a record has a payload digest.
    pub fn has_payload_digest(record: &mut WarcRecord) -> bool {
        record.headers().contains_key_bytes(b"WARC-Payload-Digest")
    }

    /// Filter predicate for checking if a record's WARC-Payload-Digest is valid.
    /// This is a mutating filter that freezes the record!
    pub fn has_valid_payload_digest(record: &mut WarcRecord) -> bool {
        record.verify_payload_digest(false).unwrap_or(false)
    }

    /// Filter predicate for checking if a record is an HTTP record.
    pub fn is_http(record: &mut WarcRecord) -> bool {
        record.is_http()
    }

    /// Filter predicate for checking if a record is concurrent to another record.
    pub fn is_concurrent(record: &mut WarcRecord) -> bool {
        record.headers().contains_key_bytes(b"WARC-Concurrent-To")
    }

    /// Parameterized filter predicate for checking if a record's Content-Length is less than or equal to `max`.
    pub fn has_record_type(record_type: WarcRecordType) -> impl Fn(&mut WarcRecord) -> bool {
        move |r: &mut WarcRecord| r.record_type() == record_type
    }

    /// Parameterized filter predicate for checking if a record's Content-Length is less than or equal to `max`.
    pub fn has_content_length_lte(max: u64) -> impl Fn(&mut WarcRecord) -> bool {
        move |r: &mut WarcRecord| r.content_length() <= max
    }

    /// Parameterized filter predicate for checking if a record's Content-Length is greater than or equal to `min`.
    pub fn has_content_length_gte(min: u64) -> impl Fn(&mut WarcRecord) -> bool {
        move |r: &mut WarcRecord| r.content_length() >= min
    }
}

// ===========================================================
// Tests
// ===========================================================

#[cfg(test)]
#[path = "iter_test.rs"]
mod iter_test;
