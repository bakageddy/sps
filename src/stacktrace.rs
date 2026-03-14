use crate::error::stacktrace::Parse;
use crate::scanner::Scanner;

#[derive(Debug)]
pub struct StackTrace<'a> {
    pub traces: Vec<StackTraceElement<'a>>,
}

impl Default for StackTrace<'_> {
    fn default() -> Self {
        Self { traces: Vec::new() }
    }
}

impl<'a> TryFrom<&'a str> for StackTrace<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.expect("java.lang.Throwable").map_err(|e| Parse::ThrowableNotFound)?;
        scanner.skip_whitespace();

        let mut st = StackTrace::default();
        while !scanner.is_empty() {
            scanner.expect("at").map_err(|e| Parse::AtNotFound)?;
            let line = match scanner.take_until_inclusive("\n") {
                Ok(line) => line,
                Err(_) => break,
            };
            scanner.skip_whitespace();

            let result = StackTraceElement::try_from(line)?;
            st.traces.push(result);
        }

        Ok(st)
    }
}

#[derive(Debug)]
pub struct StackTraceElement<'a> {
    pub function_name: &'a str,
    pub stacktrace_source: StackTraceSource<'a>,
}

impl<'a> StackTraceElement<'a> {
    pub fn new(function_name: &'a str, stacktrace_source: StackTraceSource<'a>) -> Self {
        Self {
            function_name,
            stacktrace_source,
        }
    }
}

impl<'a> TryFrom<&'a str> for StackTraceElement<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        let function_name = scanner.take_until_inclusive("(")?;
        let raw_source = scanner.take_within("(", ")")?;
        let parsed_source = StackTraceSource::try_from(raw_source)?;
        Ok(StackTraceElement::new(function_name, parsed_source))
    }
}

#[derive(Debug)]
pub enum StackTraceSource<'a> {
    NativeMethod,
    UnknownSource,
    Generated { inner: &'a str },
    FileName { file: &'a str, line: usize },
}

impl<'a> TryFrom<&'a str> for StackTraceSource<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "Unknown Source" => return Ok(StackTraceSource::UnknownSource),
            "Native Method" => return Ok(StackTraceSource::NativeMethod),
            _ => {}
        };

        if value.contains('$') {
            return Ok(StackTraceSource::Generated { inner: value });
        }

        let (file_str, line_str) = match value.split_once(':') {
            Some(res) => res,
            None => {
                return Err(Parse::ColonNotFound);
            }
        };

        let line = line_str.parse::<usize>().map_err(|_| Parse::LineNumber)?;
        if !file_str.ends_with("java") {
            return Err(Parse::SourceTypeNotRecognized);
        }
        Ok(StackTraceSource::FileName {
            file: file_str,
            line,
        })
    }
}
