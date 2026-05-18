use std::num::ParseIntError;

use memchr::memmem;

use crate::{error::stacktrace::Parse, parser::scanner::Scanner, util};

#[derive(Debug, Default)]
pub struct Trace<'a>(pub Vec<Element<'a>>);

#[derive(Debug)]
pub enum Element<'a> {
    Lock(Object<'a>),
    Elem { method: &'a [u8], source: Source<'a> },
}

#[derive(Debug)]
pub struct Object<'a> {
    pub class: &'a [u8],
    pub identity: u64,
}

#[derive(Debug)]
pub enum Source<'a> {
    NativeMethod,
    UnknownSource,
    Generated(&'a [u8]),
    Filename { file: &'a [u8], line: u64 },
}

impl<'a> TryFrom<&'a [u8]> for Trace<'a> {
    type Error = Parse;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        if scanner.peek_expect(b"java.lang.Throwable") {
            let _ = scanner.expect(b"java.lang.Throwable");
            scanner.skip_whitespace();
        }

        let mut st = Trace::default();
        st.0.reserve(50);
        while !scanner.is_empty() {
            if scanner.peek_expect(b"at") {
                let _ = scanner.expect(b"at");
            }
            scanner.skip_whitespace();
            let line = match scanner.take_until_inclusive(b"\n") {
                Some(line) => line,
                None => break,
            };
            scanner.skip_whitespace();

            let result = Element::try_from(line)?;
            st.0.push(result);
        }
        Ok(st)
    }
}

impl<'a> TryFrom<&'a [u8]> for Element<'a> {
    type Error = Parse;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        if scanner.peek_expect(b"- locked") {
            scanner.expect(b"- locked").expect("SAFETY: CHECKED");
            scanner.skip_whitespace();
            let result = Object::try_from(scanner.remaining())?;
            return Ok(Element::Lock(result));
        }

        let frame = scanner
            .take_until_inclusive(b"(")
            .ok_or(Parse::ParenNotFound)?;
        let source = scanner
            .take_within(b"(", b")")
            .map_err(|_| Parse::ParenNotFound)?;
        let source = Source::try_from(source)?;
        Ok(Element::Elem { method: frame, source })
    }
}

impl Object<'_> {
    pub fn hex_to_u64(value: &[u8]) -> Result<u64, Parse> {
        let utf8 = str::from_utf8(value)?;
        let result = u64::from_str_radix(utf8, 16);
        result.map_err(|_| Parse::HexUnexpectedInput {
            got: utf8.to_string(),
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for Object<'a> {
    type Error = Parse;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        let class = scanner.take_until(b"@").ok_or(Parse::MissingCommat)?;
        Ok(Object {
            class,
            identity: Object::hex_to_u64(scanner.remaining().trim_ascii())?,
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for Source<'a> {
    type Error = Parse;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match value {
            b"Unknown Source" => return Ok(Source::UnknownSource),
            b"Native Method" => return Ok(Source::NativeMethod),
            _ => {}
        };

        if let Some(pos) = memmem::find(value, b":") {
            let (file, line) = value.split_at(pos);
            Ok(Source::Filename {
                file,
                line: util::parse_u64(&line[1..])?,
            })
        } else {
            Ok(Source::Generated(value))
        }
    }
}
