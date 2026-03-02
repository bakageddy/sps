use crate::error::stacktrace::Element;
use crate::error::stacktrace::Parse;
use crate::error::stacktrace::Source;

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
        if !value.starts_with("java.lang.Throwable") {
            return Err(Parse::ThrowableNotFound)?;
        }
        let (_, value) = value.split_once("\n").expect("SAFETY: checked");

        let mut st = StackTrace::default();
        for line in value.lines() {
            let line = line.trim();
            if !line.starts_with("at ") {
                return Err(Parse::AtNotFound)?;
            }

            let line = line.strip_prefix("at ").expect("SAFETY: checked");
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
    type Error = Element;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (function_name, rest) = value.split_once("(").ok_or(Element::OpenParenNotFound)?;

        let (raw_source, _) = rest.split_once(")").ok_or(Element::CloseParenNotFound)?;
        let parsed_source =
            StackTraceSource::try_from(raw_source).map_err(|e| Element::SourceError(e))?;

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
    type Error = Source;

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
                eprintln!("{}", value);
                return Err(Source::ColonNotFound);
            },
        };

        // let (file_str, line_str) = value.split_once(":").ok_or(Source::ColonNotFound)?;
        let line = line_str.parse::<usize>().map_err(|_| Source::LineNumber)?;
        if !file_str.ends_with("java") {
            return Err(Source::SourceTypeNotRecognized);
        }
        Ok(StackTraceSource::FileName {
            file: file_str,
            line,
        })
    }
}
