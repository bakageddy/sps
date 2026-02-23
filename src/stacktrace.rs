use tracing::warn;

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
    type Error = StackTraceParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if !value.starts_with("java.lang.Throwable") {
            return Err(StackTraceParseError::ThrowableNotFound);
        }
        let (_, value) = value.split_once("\n").expect("java.lang.Throwable is always followed by a newline");

        let mut st = StackTrace::default();
        let mut line_no: usize = 2;
        for line in value.lines() {
            let line = line.trim();
            if !line.starts_with("at ") {
                return Err(StackTraceParseError::AtNotFound {line: line_no});
            }

            let line = line.strip_prefix("at ").expect("Unreachable");
            let result = StackTraceElement::try_from(line);
            match result {
                Ok(elem) => st.traces.push(elem),
                Err(e) => return Err(StackTraceParseError::ElementParseError {
                    inner: e,
                    line: line_no
                })
            }

            line_no += 1;
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
    type Error = StackTraceElementParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (function_name, rest) = value
            .split_once("(")
            .ok_or(StackTraceElementParseError::OpenParenNotFound)?;

        let (raw_source, _) = rest
            .split_once(")")
            .ok_or(StackTraceElementParseError::CloseParenNotFound)?;
        let parsed_source = StackTraceSource::try_from(raw_source)
            .map_err(|e| StackTraceElementParseError::ElementSourceParseError(e))?;

        Ok(StackTraceElement::new(function_name, parsed_source))
    }
}


#[derive(Debug)]
pub enum StackTraceSource<'a> {
    NativeMethod,
    UnknownSource,
    FileName { file: &'a str, line: usize },
}

impl<'a> TryFrom<&'a str> for StackTraceSource<'a> {
    type Error = StackTraceSourceParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "Unknown Source" => return Ok(StackTraceSource::UnknownSource),
            "Native Method" => return Ok(StackTraceSource::NativeMethod),
            _ => {}
        };

        let (file_str, line_str) = value
            .split_once(":")
            .ok_or(StackTraceSourceParseError::ColonNotFound)?;
        let line = line_str
            .parse::<usize>()
            .map_err(|_| StackTraceSourceParseError::LineNumberParseError)?;
        if !file_str.ends_with("java") {
            return Err(StackTraceSourceParseError::SourceTypeNotRecognized);
        }
        Ok(StackTraceSource::FileName {
            file: file_str,
            line,
        })
    }
}

#[derive(Debug)]
pub enum StackTraceParseError {
    ThrowableNotFound,
    AtNotFound {
        line: usize,
    },
    ElementParseError {
        inner: StackTraceElementParseError,
        line: usize,
    },
}

#[derive(Debug)]
pub enum StackTraceSourceParseError {
    SourceTypeNotRecognized,
    LineNumberParseError,
    ColonNotFound,
}

#[derive(Debug)]
pub enum StackTraceElementParseError {
    OpenParenNotFound,
    CloseParenNotFound,
    ElementSourceParseError(StackTraceSourceParseError),
}
