use std::io::{Read, Seek, SeekFrom};
use std::{io, mem};

pub(crate) fn to_bytes<T>(t: &T) -> &[u8] {
    let len = mem::size_of::<T>();
    let raw = t as *const T as *const u8;
    unsafe { core::slice::from_raw_parts(raw, len) }
}

/// Split the vector of bytes at the `0` elements
///
/// Return the slice before the first
pub(crate) fn split_at_null(bytes: &[u8]) -> (&[u8], usize) {
    let end = bytes.iter()
        .position(|&x| x == 0)
        .unwrap_or(bytes.len());
    (&bytes[..end], end)
}

pub(crate) struct BytesStream<'a> {
    inner: &'a [u8],
    pos: u64,
}

impl<'a> BytesStream<'a> {
    pub fn new(inner: &'a [u8]) -> Self {
        BytesStream { inner, pos: 0 }
    }
}

impl<'a> Read for BytesStream<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let pos = self.pos as usize;
        if pos >= self.inner.len() {
            return Ok(0);
        }
        match (&self.inner[pos..]).read(buf) {
            Ok(size) => {
                self.pos += size as u64;
                Ok(size)
            }
            err => err,
        }
    }
}

impl<'a> Seek for BytesStream<'a> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(start) => {
                self.pos = start;
                Ok(self.pos)
            }
            SeekFrom::End(end) => {
                self.pos = (self.inner.len() as i64 + end) as u64;
                Ok(self.pos)
            }
            SeekFrom::Current(offset) => {
                self.pos = ((self.pos as i64) + offset) as u64;
                Ok(self.pos)
            }
        }
    }
}
