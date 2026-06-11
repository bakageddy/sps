use crate::{error::stacktrace::Parse, parser::scanner::Scanner};

#[derive(Debug, Default)]
pub struct Trace<'a>(pub Vec<Element<'a>>);

#[derive(Debug)]
pub enum Element<'a> {
    Lock(Object<'a>),
    Elem { method: &'a str, source: Source<'a> },
}

#[derive(Debug)]
pub struct Object<'a> {
    pub class: &'a str,
    pub identity: u64,
}

#[derive(Debug)]
pub enum Source<'a> {
    NativeMethod,
    UnknownSource,
    Generated(&'a str),
    Filename { file: &'a str, line: u64 },
}

impl<'a> TryFrom<&'a str> for Trace<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        // NOTE: Thread Dump stack traces do not have the java.lang.Throwable PREAMBLE
        if scanner.peek_expect("java.lang.Throwable") {
            let _ = scanner.expect("java.lang.Throwable");
            scanner.skip_whitespace();
        }

        let mut st = Trace::default();
        st.0.reserve(50);
        while !scanner.is_empty() {
            // NOTE: Thread Dump stack traces do not have the at PREAMBLE
            if scanner.peek_expect("at") {
                let _ = scanner.expect("at");
            }
            scanner.skip_whitespace();
            let line = match scanner.take_until_inclusive("\n") {
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

impl<'a> TryFrom<&'a str> for Element<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        if scanner.peek_expect("- locked") {
            scanner.expect("- locked").expect("SAFETY: CHECKED");
            scanner.skip_whitespace();
            let result = Object::try_from(scanner.remaining())?;
            return Ok(Element::Lock(result));
        }

        let frame = scanner
            .take_until_inclusive("(")
            .ok_or(Parse::ParenNotFound)?;
        let source = scanner
            .take_within("(", ")")
            .map_err(|_| Parse::ParenNotFound)?;
        let source = Source::try_from(source)?;
        Ok(Element::Elem {
            method: frame,
            source,
        })
    }
}

impl Object<'_> {
    pub fn hex_to_u64(value: &str) -> Result<u64, Parse> {
        let result = u64::from_str_radix(value, 16);
        result.map_err(|_| Parse::InvalidHexadecimalCharacter {
            got: value.to_string(),
        })
    }
}

impl<'a> TryFrom<&'a str> for Object<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        let class = scanner.take_until("@").ok_or(Parse::MissingCommat)?;
        Ok(Object {
            class,
            identity: Object::hex_to_u64(scanner.remaining())?,
        })
    }
}

impl<'a> TryFrom<&'a str> for Source<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "Unknown Source" => return Ok(Source::UnknownSource),
            "Native Method" => return Ok(Source::NativeMethod),
            _ => {}
        };

        if let Some(pos) = value.find(":") {
            let (file, line) = value.split_at(pos);
            Ok(Source::Filename {
                file,
                line: line[1..].parse()?,
            })
        } else {
            Ok(Source::Generated(value))
        }
    }
}
