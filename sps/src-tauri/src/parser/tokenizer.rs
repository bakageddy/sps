use crate::parser::tokenizer::error::Error;
use std::str::Utf8Error;

pub trait Parser<'a> {
    type Error;

    fn parse(data: &'a str) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

pub struct Tokenizer<'a>(&'a str);

impl<'a> Tokenizer<'a> {
    pub fn new(data: &'a str) -> Self {
        Self(data)
    }

    pub fn from_bytes(data: &'a [u8]) -> Result<Self, Utf8Error> {
        let parsed = str::from_utf8(data)?;
        Ok(Self(parsed))
    }

    pub fn skip_whitespace(&mut self) {
        self.0 = self.0.trim_start();
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn expect(&mut self, expect: &str) -> Result<(), Error> {
        if self.0.len() < expect.len() {
            return Err(Error::NotEnoughData);
        }
        self.0 = self.0.strip_prefix(expect).ok_or_else(|| Error::Expected {
            got: String::from(&self.0[..expect.len()]),
            expected: String::from(expect),
        })?;
        Ok(())
    }

    pub fn take_until_exclusive(&mut self, needle: &str) -> Result<&'a str, Error> {
        let idx = self
            .0
            .find(needle)
            .ok_or_else(|| Error::DelimiterNotFound(String::from(needle)))?;
        let (data, rest) = self.0.split_at(idx);
        self.0 = rest;
        Ok(data)
    }

    pub fn take_until(&mut self, needle: &str) -> Option<&'a str> {
        let (data, rest) = self.0.split_once(needle)?;
        self.0 = rest;
        Some(data)
    }

    pub fn get_line(&mut self) -> Option<&'a str> {
        self.take_until("\n")
    }

    pub fn peek_until(&mut self, needle: &str) -> Option<&'a str> {
        let (data, _) = self.0.split_once(needle)?;
        Some(data)
    }

    pub fn peek_line(&mut self) -> Option<&'a str> {
        self.peek_until("\n")
    }

    pub fn take_within(&mut self, open: &str, close: &str) -> Result<&'a str, Error> {
        let (_, rest) = self
            .0
            .split_once(open)
            .ok_or_else(|| Error::DelimiterNotFound(String::from(open)))?;
        let (data, rest) = rest
            .split_once(close)
            .ok_or_else(|| Error::DelimiterNotFound(String::from(close)))?;
        self.0 = rest;
        Ok(data)
    }

    pub fn take_within_exclusive(&mut self, open: &str, close: &str) -> Result<&'a str, Error> {
        let (_, rest) = self
            .0
            .split_once(open)
            .ok_or_else(|| Error::DelimiterNotFound(String::from(open)))?;

        let idx = rest
            .find(close)
            .ok_or_else(|| Error::DelimiterNotFound(String::from(close)))?;

        let within = &rest[..idx];
        self.0 = &rest[idx..];

        Ok(within)
    }

    pub fn remaining(self) -> &'a str {
        return self.0;
    }
}

pub mod error {
    #[non_exhaustive]
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Not Enough Data in the Tokenizer")]
        NotEnoughData,
        #[error("Tokenizer expected: {expected}, got: {got}")]
        Expected { expected: String, got: String },
        #[error("Delimiter not found: {0}")]
        DelimiterNotFound(String),
    }
}
