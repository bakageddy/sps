use self::error::Error;
use crate::parser::tokenizer::Tokenizer;
use std::borrow::Cow;
use std::str::Utf8Error;

#[derive(Debug)]
pub struct HealthMeterParser<'a>(&'a str, ParserState);

#[derive(Debug)]
pub enum ParserState {
    Initial,
}

#[derive(Debug)]
pub struct HCell<'a> {
    pub key: Cow<'a, str>,
    pub val: Option<Cow<'a, str>>,
}

impl<'a> HealthMeterParser<'a> {
    pub fn new(data: &'a str) -> Self {
        Self(data, ParserState::Initial)
    }
}

impl<'a> TryFrom<&'a [u8]> for HealthMeterParser<'a> {
    type Error = Utf8Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let data = std::str::from_utf8(value)?;
        Ok(Self::new(data))
    }
}

impl<'a> Iterator for HealthMeterParser<'a> {
    type Item = Result<HCell<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.0.contains("<tr>") {
            return None;
        }

        let mut tok = Tokenizer::new(self.0);
        tok.skip_whitespace();
        if tok.is_empty() {
            return None;
        }

        self.1 = ParserState::Initial;
        if let Err(e) = tok.take_until_fallible("<tr>") {
            self.0 = tok.remaining();
            return Some(Err(Error::from(e)));
        }

        tok.skip_whitespace();

        if let Err(e) = tok.expect("<td") {
            self.0 = tok.remaining();
            return Some(Err(Error::from(e)));
        }

        let key: Cow<'a, str> = match tok.take_within(">", "</td>") {
            Ok(k) => k.into(),
            Err(e) => {
                self.0 = tok.remaining();
                return Some(Err(Error::from(e)));
            }
        };

        if key.is_empty() {
            self.0 = tok.remaining();
            return Some(Err(Error::InvalidKey));
        }

        tok.skip_whitespace();
        if !tok.peek("<td") {
            self.0 = tok.remaining();
            return Some(Err(Error::InvalidKey));
        }
        if let Err(e) = tok.take_until_fallible("<td") {
            self.0 = tok.remaining();
            return Some(Err(Error::from(e)));
        }

        let val: Cow<'a, str> = match tok.take_within(">", "</td>") {
            Ok(v) => v.into(),
            Err(e) => {
                self.0 = tok.remaining();
                return Some(Err(Error::from(e)));
            }
        };

        let val = if val.is_empty() {
            None
        } else {
            Some(val)
        };

        tok.skip_whitespace();
        if let Err(e) = tok.expect("</tr>") {
            self.0 = tok.remaining();
            return Some(Err(Error::from(e)));
        }

        self.0 = tok.remaining();
        Some(Ok(HCell { key, val }))
    }
}

#[cfg(test)]
pub mod test {
    use crate::{parser::healthmeter::HealthMeterParser, util};
    use std::ops::Deref;

    #[test]
    fn healthmeter_single_table() {
        let map = util::map_file("test/healthmeter/single_table.html").unwrap();
        let parser = HealthMeterParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for result in parser {
            if let Err(e) = result {
                dbg!(e);
                continue;
            }

            count += 1;
        }
        assert_eq!(count, 73);
    }

    #[test]
    fn healthmeter_full_file() {
        let map = util::map_file("test/healthmeter/HealthMeter.html").unwrap();
        let parser = HealthMeterParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for result in parser {
            if let Err(e) = result {
                dbg!(e);
                continue;
            }

            count += 1;
        }
        assert_eq!(count, 942);
    }
}

pub mod error {
    use crate::parser::tokenizer;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid Format in HealthMeter: {0}")]
        InvalidFormat(#[from] tokenizer::error::Error),
        #[error("Invalid Key in HealthMeter")]
        InvalidKey,
    }
}
