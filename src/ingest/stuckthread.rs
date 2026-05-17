use std::path::Path;

use crate::parser::stuckthread::StuckThread;

pub struct StuckThreadIterator<'a>(pub &'a [u8]);

impl<'a> Iterator for StuckThreadIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.trim_ascii_start().is_empty() {
            self.0 = b"";
            return None;
        }

        self.0 = self.0.trim_ascii_start();

        let start = 0;
        let mut offset = 0;
        let mut iter = self.0.split_inclusive(|b| *b == b'\n').peekable();
        if let Some(line) = iter.next() {
            if !line.starts_with(b"[") {
                eprintln!("Unreachable");
                return None;
            }
            offset += line.len();
        }

        while let Some(line) = iter.next_if(|l| !l.starts_with(b"[")) {
            offset += line.len();
        }

        let contents = &self.0[start..start + offset];
        self.0 = &self.0[start + offset..];

        Some(contents)
    }
}
