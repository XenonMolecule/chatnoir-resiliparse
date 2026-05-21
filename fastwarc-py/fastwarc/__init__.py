# Copyright 2026 Janek Bevendorff
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import importlib
import sys

_native = importlib.import_module("._fastwarc", __name__)
_legacy_shims = importlib.import_module(".legacy._shims", __name__)

from ._fastwarc import *  # noqa: F401,F403

stream_io = _native.stream_io
warc = _native.warc

sys.modules[__name__ + ".stream_io"] = stream_io
sys.modules[__name__ + ".warc"] = warc

# Patch legacy shims onto stream_io
for name in _legacy_shims.__all__:
    setattr(stream_io, name, getattr(_legacy_shims, name))
stream_io.__all__ += getattr(_legacy_shims, "__all__", ())

FileStream = _legacy_shims.FileStream
GZipStream = _legacy_shims.GZipStream
LZ4Stream = _legacy_shims.LZ4Stream
FastWARCError = _legacy_shims.FastWARCError
StreamError = _legacy_shims.StreamError

__all__ = [
    *_native.__all__,
    "FileStream",
    "GZipStream",
    "LZ4Stream",
    "FastWARCError",
    "StreamError",
]
