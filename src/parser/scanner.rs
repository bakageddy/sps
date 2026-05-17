use crate::error::scanner::Error;
use memchr::memmem;

pub struct Scanner<'a> {
    data: &'a [u8],
}

impl<'a> Scanner<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn peek_expect(&self, expected: &[u8]) -> bool {
        self.data.starts_with(expected)
    }

    pub fn expect(&mut self, expected: &[u8]) -> Result<(), Error> {
        if self.data.len() < expected.len() {
            return Err(Error::EndOfData);
        }

        match self.data.strip_prefix(expected) {
            Some(rest) => {
                self.data = rest;
                Ok(())
            }
            None => Err(Error::Expected {
                got: String::from_utf8_lossy(&self.data[..expected.len()]).into_owned(),
                expected: String::from_utf8_lossy(expected).into_owned(),
            }),
        }
    }

    pub fn skip_whitespace(&mut self) {
        self.data = self.data.trim_ascii_start();
    }

    /// Take exactly `n` bytes. Errors if fewer than `n` remain.
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        if n > self.data.len() {
            return Err(Error::NotEnoughData {
                have: self.data.len(),
                expect: n,
            });
        }

        let (before, after) = self.data.split_at(n);
        self.data = after;
        Ok(before)
    }

    pub fn take_until_inclusive(&mut self, delimiter: &[u8]) -> Option<&'a [u8]> {
        let pos = memmem::find(self.data, delimiter)?;
        let (before, after) = self.data.split_at(pos);
        self.data = after;
        Some(before)
    }

    pub fn take_until(&mut self, delimiter: &[u8]) -> Option<&'a [u8]> {
        let pos = memmem::find(self.data, delimiter)?;
        let before = &self.data[..pos];
        self.data = &self.data[pos + delimiter.len()..];
        Some(before)
    }

    pub fn peek_until(&self, delimiter: &[u8]) -> Option<&'a [u8]> {
        let pos = memmem::find(self.data, delimiter)?;
        Some(&self.data[..pos])
    }

    pub fn take_within(&mut self, open: &[u8], close: &[u8]) -> Result<&'a [u8], Error> {
        let open_pos = memmem::find(self.data, open).ok_or_else(|| Error::DelimiterNotFound {
            delimiter: String::from_utf8_lossy(open).into_owned(),
            data: String::from_utf8_lossy(self.data).into_owned(),
        })?;
        let rest = &self.data[open_pos + open.len()..];

        let close_pos = memmem::find(rest, close).ok_or_else(|| Error::DelimiterNotFound {
            delimiter: String::from_utf8_lossy(close).into_owned(),
            data: String::from_utf8_lossy(rest).into_owned(),
        })?;

        let within = &rest[..close_pos];
        self.data = &rest[close_pos + close.len()..];
        Ok(within)
    }

    pub fn remaining(self) -> &'a [u8] {
        self.data
    }
}
