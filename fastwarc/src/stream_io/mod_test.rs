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

use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::rc::Rc;

/// Test helper simulating an unreliable writer.
pub(crate) struct ErrorWriter {
    pub(crate) fail_on_write: bool,
    pub(crate) fail_on_flush: bool,
}

impl Write for ErrorWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.fail_on_write && !buf.is_empty() {
            Err(io::Error::other("injected write failure"))
        } else {
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.fail_on_flush {
            Err(io::Error::other("injected flush failure"))
        } else {
            Ok(())
        }
    }
}

/// Test helper for testing writer Drop implementations with a shared Vec buffer.
#[derive(Clone, Default)]
pub(crate) struct SharedVecWriter {
    data: Rc<RefCell<Vec<u8>>>,
}

impl SharedVecWriter {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn data(&self) -> Rc<RefCell<Vec<u8>>> {
        Rc::clone(&self.data)
    }
}

impl Write for SharedVecWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
