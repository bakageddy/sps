use std::str::Utf8Error;

use crate::error::scanner::Error;

pub struct Scanner<'a> {
    data: &'a str,
}

impl<'a> Scanner<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, Utf8Error> {
        let data = str::from_utf8(data)?;
        Ok(Self { data })
    }

    pub fn new(data: &'a str) -> Self {
        Self { data }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn peek_expect(&self, expected: &str) -> bool {
        self.data.starts_with(expected)
    }

    pub fn expect(&mut self, expected: &str) -> Result<(), Error> {
        if self.data.len() < expected.len() {
            return Err(Error::EndOfData);
        }

        match self.data.strip_prefix(expected) {
            Some(rest) => {
                self.data = rest;
                Ok(())
            }
            None => Err(Error::Expected {
                got: String::from(&self.data[..expected.len()]),
                expected: String::from(expected),
            }),
        }
    }

    pub fn skip_whitespace(&mut self) {
        self.data = self.data.trim_start();
    }

    /// Take exactly `n` bytes. Errors if fewer than `n` remain.
    pub fn take(&mut self, n: usize) -> Result<&'a str, Error> {
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

    pub fn take_until_exclusive(&mut self, delimiter: &str) -> Option<&'a str> {
        let pos = self.data.find(delimiter)?;
        let (before, after) = self.data.split_at(pos);
        self.data = after;
        Some(before)
    }

    pub fn take_until(&mut self, delimiter: &str) -> Option<&'a str> {
        let pos = self.data.find(delimiter)?;
        let before = &self.data[..pos];
        self.data = &self.data[pos + delimiter.len()..];
        Some(before)
    }

    pub fn peek_until(&self, delimiter: &str) -> Option<&'a str> {
        let pos = self.data.find(delimiter)?;
        Some(&self.data[..pos])
    }

    pub fn take_within(&mut self, open: &str, close: &str) -> Result<&'a str, Error> {
        let open_pos = self
            .data
            .find(open)
            .ok_or_else(|| Error::DelimiterNotFound {
                delimiter: String::from(open),
                data: String::from(self.data),
            })?;
        let rest = &self.data[open_pos + open.len()..];

        let close_pos = rest.find(close).ok_or_else(|| Error::DelimiterNotFound {
            delimiter: String::from(close),
            data: String::from(rest),
        })?;

        let within = &rest[..close_pos];
        self.data = &rest[close_pos + close.len()..];
        Ok(within)
    }

    /// [NOTE] Takes the items within a set of delimiters, but leaves the `close`
    /// delimiter behind.
    /// Useful for scanning tables
    pub fn take_within_exclusive(&mut self, open: &str, close: &str) -> Result<&'a str, Error> {
        let open_pos = self
            .data
            .find(open)
            .ok_or_else(|| Error::DelimiterNotFound {
                delimiter: String::from(open),
                data: String::from(self.data),
            })?;
        let rest = &self.data[open_pos + open.len()..];

        let close_pos = rest.find(close).ok_or_else(|| Error::DelimiterNotFound {
            delimiter: String::from(close),
            data: String::from(rest),
        })?;

        let within = &rest[..close_pos];
        self.data = &rest[close_pos..];
        Ok(within)
    }

    pub fn remaining(self) -> &'a str {
        self.data
    }

    pub fn remaining_bytes(self) -> &'a [u8] {
        self.data.as_bytes()
    }
}

#[cfg(test)]
mod test {
    use crate::parser::scanner::Scanner;

    #[test]
    fn scanner_take_within_exclusive() {
        let data = "| data | another_type_of_data |";
        let mut scanner = Scanner::new(data);
        let first = scanner.take_within_exclusive("|", "|");
        assert!(
            first.is_ok(),
            "Error during scanning: {:?}",
            first.unwrap_err()
        );
        let first = first.unwrap().trim();
        assert_eq!(first, "data");
        let second = scanner.take_within_exclusive("|", "|");
        assert!(
            second.is_ok(),
            "Error during scanning: {:?}",
            second.unwrap_err()
        );
        let second = second.unwrap().trim();
        assert_eq!(second, "another_type_of_data");

        let remaining = scanner.remaining();
        assert_eq!(remaining, "|");
    }
}
