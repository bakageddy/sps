use crate::error::scanner::Error;

pub struct Scanner<'a> {
    pub data: &'a str,
}

impl<'a> Scanner<'a> {
    pub fn new(data: &'a str) -> Self {
        Self { data }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn peek_expect(&self, expected: &str) -> bool {
        if self.is_empty() {
            return false;
        }

        return self.data.starts_with(expected);
    }

    pub fn expect(&mut self, expected: &str) -> Result<(), Error> {
        if self.data.len() < expected.len() {
            return Err(Error::EndOfData);
        }

        self.data = self.data.strip_prefix(expected).ok_or(Error::Expected {
            got: String::from(&self.data[..expected.len()]),
            expected: String::from(expected),
        })?;

        Ok(())
    }

    pub fn skip_whitespace(&mut self) {
        self.data = self.data.trim_start();
    }

    // WARN: DO NOT USE IT DURING PARSING. This is only an convenience function
    pub fn take(&mut self, n: usize) -> Result<&'a str, Error> {
        if n >= self.data.len() {
            return Err(Error::NotEnoughData {
                have: self.data.len(),
                expect: n,
            });
        }
        if !self.data.is_char_boundary(n) {
            return Err(Error::NotACharBoundary {
                n,
                data: String::from(self.data),
            });
        }

        let (before, after) = self.data.split_at(n);
        self.data = after;
        Ok(before)
    }

    pub fn take_until_inclusive(&mut self, delimiter: &str) -> Result<&'a str, Error> {
        let pos = self.data.find(delimiter).ok_or(Error::DelimiterNotFound {
            delimiter: String::from(delimiter),
            data: String::from(self.data),
        })?;
        let before = &self.data[..pos];
        self.data = &self.data[pos..];
        Ok(before)
    }

    pub fn take_until(&mut self, delimiter: &str) -> Option<&'a str> {
        let (before, after) = self
            .data
            .split_once(delimiter)?;
        self.data = after;
        Some(before)
    }

    pub fn peek_until(&mut self, delimiter: &str) -> Option<&'a str> {
        Some(self.data.split_once(delimiter)?.0)
    }

    pub fn take_within(&mut self, open: &str, close: &str) -> Result<&'a str, Error> {
        let (_, rest) = self.data.split_once(open).ok_or(Error::DelimiterNotFound {
            delimiter: String::from(open),
            data: String::from(self.data),
        })?;

        let (within, rest) = rest.split_once(close).ok_or(Error::DelimiterNotFound {
            delimiter: String::from(close),
            data: String::from(rest),
        })?;

        self.data = rest;
        Ok(within)
    }
}
