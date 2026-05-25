.. _fastwarc-manual:

FastWARC
========

FastWARC is a high-performance WARC parsing library written in Rust with bindings available for Python. The API is inspired in large parts by `WARCIO <https://github.com/webrecorder/warcio>`__, but does not aim at being a drop-in replacement.  FastWARC supports compressed and uncompressed WARC/1.0 and WARC/1.1 streams. Supported compression algorithms are Gzip, Zstd, and LZ4.

FastWARC belongs to the :ref:`ChatNoir Resiliparse toolkit <resiliparse-docs-index>` for fast and robust web data processing.

Why FastWARC and not WARCIO?
----------------------------
WARCIO is a fantastic tool for reading and writing WARCs, but it is implemented entirely in Python and thus becomes rather inefficient for large web crawls at the tera- or petabyte scale where a few seconds of additional processing time add up quickly. FastWARC solves these performance issues by being written in efficient, low-level Rust. We also took the opportunity to add support for Zstandard (Zstd) as a new compression option (which has been `proposed as a new standard <https://iipc.github.io/warc-specifications/specifications/warc-zstd/>`__ besides Gzip) and LZ4, a much, much (!) faster compression algorithm than Gzip.

FastWARC's design goals are high speed, a low and fixed memory footprint, and simplicity. For the latter reason, we decided against adding support for the legacy ARC format. If you need that kind of backwards compatibility, use WARCIO instead.


.. _fastwarc-installation:

Installing FastWARC
-------------------
Pre-built FastWARC binaries for Python can be installed from `PyPi <https://pypi.org/project/FastWARC/>`_:

.. code-block:: bash

  pip install fastwarc

Building FastWARC From Source
-----------------------------

To build FastWARC from the `source repository <https://github.com/chatnoir-eu/chatnoir-resiliparse>`__, you need Rust and Cargo installed. If that is not the case, install ``rustup`` first and follow the
instructions:

.. code-block:: bash

  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

To build FastWARC, simply run ``cargo build`` from the root of this repository.

Using FastWARC as a Library
^^^^^^^^^^^^^^^^^^^^^^^^^^^

To use FastWARC in your Rust project, add it as a dependency to your ``Cargo.toml``:

.. code-block:: toml
  :substitutions:

  [dependencies]
  fastwarc = "|project_release_minor|"

Then build your project like normal with ``cargo build``.

Building Python Bindings From Source
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Note:** If you just want to use FastWARC in Python, building the bindings from source is usually necessary, unless there are no pre-built binaries for your platform.

To be able to build FastWARC, you need the Rust toolchain installed (see above). Then to build directly from the GitHub repository, follow these steps:

.. code:: bash

  # Clone repository
  git clone https://github.com/chatnoir-eu/chatnoir-resiliparse.git
  cd chatnoir-resiliparse

  # Install Rust toolchain (if not already installed)
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

  # Create a fresh venv first (recommended)
  python3 -m venv venv && source venv/bin/activate

  # Option 1: Build and install in editable mode (best for development)
  python3 -m pip install -e ./fastwarc-py

  # Option 2 (alternative): Build and install wheels in separate steps (best for redistribution)
  python3 -m pip wheel -w build ./fastwarc-py
  ls ./build/*.whl | xargs python3 -m pip install

Iterating WARC Files
--------------------

.. important::

  With the Rust port in version 1.0, the Python API has changed slightly. The old names (e.g. ``GZipStream`` instead of ``GzipReader``) still mostly work through compatibility shims, but will eventually be removed. So please make sure you update your scripts! If you run into deeper issues, the old Cython bindings are still available as a submodule named ``fastwarc.legacy``.


The central class for stream-processing WARC files is :class:`.ArchiveIterator`:

.. code-block:: python

  from fastwarc import ArchiveIterator

  # Explicit file-like object
  with open('warcfile.warc.gz', 'rb') as f:
      for record in ArchiveIterator(f):
          print(record.record_id)

  # File path
  for record in ArchiveIterator('warcfile.warc.gz'):
      print(record.record_id)

This will iterate over all records in the file and print out their IDs. You can pass any file-like Python object or a file path as a string to :class:`.ArchiveIterator`. The stream or file can be either uncompressed or a Gzip-, Zstd-, or LZ4-compressed WARC. FastWARC will try to auto-detect the stream format (unless you pass ``stream_detect=False``). But if you already know the compression algorithm beforehand, you can optimize the process a little by explicitly passing a :class:`.GzipReader`, :class:`.ZstdReader`, or :class:`.LZ4Reader` object instead:

.. code-block:: python

  from fastwarc.stream_io import *

  # Gzip:
  stream = GzipReader('warcfile.warc.gz')

  # Zstandard:
  stream = ZstdReader('warcfile.warc.zst')

  # LZ4:
  stream = LZ4Reader('warcfile.warc.lz4')

Instead of a filename, you can also pass any kind of file-like object:

.. code-block:: python

  stream = GzipReader(open('warcfile.warc.gz', 'rb'))

If `fsspec <https://filesystem-spec.readthedocs.io/>`__ is installed (which is a dependency if you installed FastWARC as ``fastwarc[fsspec]``), you can also use a (remote) URL instead of a local file path:

.. code-block:: python

  from fastwarc.warc import ArchiveIterator

  # Read remote S3 object (with optional credentials, needs s3fs installed)
  creds = {'key': '...',
           'secret': '...',
           'endpoint_url': '...'}
  for record in ArchiveIterator('s3://mybucket/warcfile.warc.gz', fsspec_args=creds):
      print(record.record_id)

  # Or without stream auto-detection:
  for record in ArchiveIterator(GzipReader('s3://mybucket/warcfile.warc.gz', fsspec_args=creds)):
      print(record.record_id)

Create your own :class:`fsspec.core.OpenFile` object if you need more control:

.. code-block:: python

  from fsspec import open as fsspec_open

  with fsspec_open('s3://mybucket/warcfile.warc.gz', 'rb', **creds) as f:
      for record in ArchiveIterator(f):
          print(record.record_id)


Filtering Records
-----------------
FastWARC provides several ways in which you can filter and efficiently skip records you are not interested in. These filters are checked very early in the parsing process, right after the WARC header block has been read. Multiple types of filters can be combined.

Record Type Filter
^^^^^^^^^^^^^^^^^^
If you want only records of a certain type, you can skip all other records efficiently by specifying a bitmask of the desired record types:

.. code-block:: python

  from fastwarc import ArchiveIterator, WarcRecordType

  for record in ArchiveIterator(stream, record_types=WarcRecordType.request | WarcRecordType.response):
      pass

This will skip all records with a ``WARC-Type`` other than ``request`` or ``response``.

Content-Length Filter
^^^^^^^^^^^^^^^^^^^^^
You can automatically skip any records whose ``Content-Length`` exceeds or is lower than a certain value:

.. code-block:: python

  from fastwarc import ArchiveIterator

  # Skip all records that are larger than 500 KiB
  for record in ArchiveIterator(stream, max_content_length=512000):
      pass

  # Skip all records that are smaller than 128 bytes
  for record in ArchiveIterator(stream, min_content_length=128):
      pass


Function Filter
^^^^^^^^^^^^^^^
If the above-mentioned filter mechanisms are not sufficient, you can pass a function object that accepts as its only parameter a :class:`.WarcRecord` and returns a ``bool`` value as a filter predicate. This filter type is much slower than the previous filters, but probably still more efficient than checking the same thing later on in the loop. Be aware that since the record body hasn't been seen yet, you cannot access any information beyond what is in the record headers.

FastWARC comes with a handful of existing filters that you can use:

.. code-block:: python

  from fastwarc.warc import *

  # Skip any non-HTTP records
  for record in ArchiveIterator(stream, func_filter=is_http):
      pass

  # Skip records without a block digest
  for record in ArchiveIterator(stream, func_filter=has_block_digest):
      pass

  # Skip records that are not WARC/1.1
  for record in ArchiveIterator(stream, func_filter=is_warc_11):
      pass

The full list of pre-defined function filters is: :func:`.is_warc_10`, :func:`.is_warc_11`, :func:`.has_block_digest`, :func:`.has_valid_block_digest`, :func:`.has_payload_digest`, :func:`.has_valid_payload_digest`, :func:`.is_http`, :func:`.is_concurrent`. Besides these, you can pass any Python callable that accepts a :class:`.WarcRecord` and returns a ``bool``:

.. code-block:: python

  # Skip records which haven't been identified as HTML pages
  for record in ArchiveIterator(stream, func_filter=lambda r: r.headers.get('WARC-Identified-Payload-Type') == 'text/html'):
      pass

  # Skip records without any sort of digest header
  for record in ArchiveIterator(stream, func_filter=lambda r: has_block_digest(r) and has_payload_digest(r)):
      pass

Digest Filter
^^^^^^^^^^^^^
Unlike all the other filter predicates, :func:`has_valid_block_digest` and :func:`has_valid_payload_digest` are executed only *after* the content is available, which is needed for the digest calculation. These filters will skip records without or with an invalid block or payload digest. You can also set ``verify_digests=True`` as a shorthand for ``func_filter=has_valid_block_digest``:

.. code-block:: python

  for record in ArchiveIterator(stream, verify_digests=True):
      pass

.. warning::

  These are the most expensive filters of all, as they will create an in-memory copy of the whole record. See :ref:`verifying-record-digests` for more information on how digest verification works.

Record Properties
-----------------
The :class:`.ArchiveIterator` returns objects of type :class:`.WarcRecord`, which have various properties:

.. code-block:: python

  for record in ArchiveIterator(stream):
      record.headers              # Dict-like object containing the WARC headers
      record.record_id            # Shorthand for record.headers['WARC-Record-ID']
      record.record_type          # Shorthand for record.headers['WARC-Type']
      record.record_date          # Parsed record.headers['WARC-Date']
      record.content_length       # Effective record payload length
      record.stream_pos           # Record start offset in the (uncompressed) stream
      record.is_http              # Boolean indicating whether record is an HTTP record
      record.is_http_parsed       # Boolean indicating whether the record has been parsed as HTTP
      record.http_headers         # Dict-like object containing the parsed HTTP headers
      record.http_content_type    # Plain HTTP Content-Type without charset
      record.http_charset         # HTTP charset from the Content-Type header (if any)
      record.http_date            # Parsed HTTP Date header
      record.http_last_modified   # Parsed HTTP Last-Modified header
      record.reader               # A BufferedReader for the record content

      # Read and return up to 1024 bytes from the record stream
      body = record.reader.read(1024)

      # Consume and return the remaining record bytes
      body += record.reader.read()

      # Or: Consume rest of stream without allocating a buffer for it (i.e., skip over)
      record.reader.consume()

As you can see, HTTP request and response records are parsed automatically for convenience. If not needed, you can disable this behaviour by passing ``parse_http=False`` to the :class:`.ArchiveIterator` constructor to avoid unnecessary processing. :attr:`record.reader <.WarcRecord.reader>` will then start at the beginning of the HTTP header block instead of the HTTP body. You can parse HTTP headers later on a per-record basis by calling :meth:`record.parse_http() <.WarcRecord.parse_http>` as long as the :class:`.BufferedReader` hasn't been consumed at that point.


.. _verifying-record-digests:

Verifying Record Digests
------------------------
If a record has digest headers, you can verify the consistency of the record contents and/or its HTTP payload:

.. code-block:: python

  for record in ArchiveIterator(stream, parse_http=False):
      if 'WARC-Block-Digest' in record.headers:
          print('Block digest OK:', record.verify_block_digest())

      if 'WARC-Payload-Digest' in record.headers:
          record.parse_http()    # It's safe to call this even if the record has no HTTP payload
          print('Payload digest OK:', record.verify_payload_digest())

Note that both :meth:`~.WarcRecord.verify_block_digest` and :meth:`~.WarcRecord.verify_payload_digest` will simply return ``False`` if the headers do not exist, so check that first. Also keep in mind that the block verification will fail if the reader has been (partially) consumed, so automatic HTTP parsing has to be turned off for this to work.

.. warning::

  Calling either of these two methods will create an in-memory copy of the remaining record stream to preserve its contents for further processing (that's why verifying the HTTP payload digest after verifying the block digest worked in the first place).

If your records are very large, you need to ensure that they fit into memory entirely (e.g. by checking :attr:`record.content_length <.WarcRecord.content_length>`). If you do not want to preserve the stream contents, you can set ``consume=True`` as a parameter. This will avoid the creation of a stream copy altogether and fully consume the rest of the record instead.


.. _fastwarc-clueweb:

ClueWeb Notes
-------------
FastWARC is a standards-compliant WARC parser. Unfortunately, the `ClueWeb <https://lemurproject.org/>`__ authors were somewhat creative with the standard. If you work with these datasets, you will inevitably notice certain defects in the files that result in premature stream aborts. This applies to both the old ClueWeb09 as well as the new ClueWeb22. Following is a list of known ClueWeb WARC defects and how to work around them:

ClueWeb09
^^^^^^^^^
The `ClueWeb09 <https://lemurproject.org/clueweb09.php/>`__ uses a non-standard WARC version and the WARC headers are separated by ``LF`` instead of ``CRLF`` line breaks. In addition, some header values contain line breaks without proper continuation indents and non-ASCII characters. To prevent FastWARC from choking on this creative WARC flavour, pass ``quirks_mode=True`` to your :class:`.ArchiveIterator`.

ClueWeb22
^^^^^^^^^
`ClueWeb22 <https://lemurproject.org/clueweb22.php/>`__ WARCs are a bit more predictable than ClueWeb09 WARCs, but have non-trivial defects nonetheless.

*First*, the initial ``warcinfo`` records are missing the required ``Content-Length`` header, so we have to rely on heuristics to determine where the record ends. If ``strict_mode=False`` is set and the WARC is read from a Gzip-compressed stream, FastWARC will attempt to use the internal buffer boundaries for determining the record end. If you are reading the WARC as an uncompressed file, FastWARC has to seek forward to the next valid ``WARC/1.1`` version line. In this case the record body will be skipped as empty. Without ``quirks_mode=True``, FastWARC will stop after the first header block.

*Second*, all records are of type ``response`` with ``Content-Type: application/http; msgtype=response``, yet they contain only the HTML body and not the full HTTP response (the correct record type would be ``resource`` with ``Content-Type: text/html``). This incorrect type description will trigger FastWARC's automatic HTTP parsing, which will result in empty or incomplete record bodies. To avoid this, explicitly set ``parse_http=False``.

.. _fastwarc-benchmarks:

Benchmarks
----------
Depending on your CPU, your storage speed, and the WARC compression algorithm, you can typically expect speedups between 1.3x and 6.5x over WARCIO.

The :ref:`fastwarc-cli` comes with a benchmarking tool for measuring WARC record decompression and parsing performance on your own machine. The benchmarking results can be compared directly with WARCIO. Here are three example runs on an AMD Ryzen Threadripper 2920X (with NVMe SSD) over five `Common Crawl <https://commoncrawl.org/>`__ WARCs:

**Uncompressed WARC:**

.. code-block:: console

  $ fastwarc benchmark CC-MAIN-*.warc --bench-warcio

  Benchmarking read performance from 5 input path(s)...
  FastWARC: 630,245 records read in 5.81 seconds (108,487.93 records/s).
  WARCIO:   630,245 records read in 37.19 seconds (16,945.51 records/s).
  Time difference: -31.38 seconds, speedup: 6.40

**Gzip WARC:**

.. code-block:: console

  $ fastwarc benchmark CC-MAIN-*.warc.gz --bench-warcio

  Benchmarking read performance from 5 input path(s)...
  FastWARC: 630,245 records read in 60.52 seconds (10,413.38 records/s).
  WARCIO:   630,245 records read in 97.56 seconds (6,460.06 records/s).
  Time difference: -37.04 seconds, speedup: 1.61

**LZ4 WARC:**

.. code-block:: console

  $ fastwarc benchmark CC-MAIN-*.warc.lz4

  Benchmarking read performance from 5 input path(s)...
  FastWARC: 630,245 records read in 12.65 seconds (49,825.44 records/s).

(Direct comparison not possible, since WARCIO does not support LZ4.)

The read benchmarking tool has additional options, such as reading WARCs directly from a remote S3 data source
using `Boto3 <https://boto3.amazonaws.com/v1/documentation/api/latest/index.html>`__.
