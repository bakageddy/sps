pub struct ThreadDumpIterator<'a>(pub &'a [u8]);

impl<'a> Iterator for ThreadDumpIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.trim_ascii().is_empty() {
            return None;
        }

        let mut iter = self.0.split_inclusive(|c| *c == b'\n').peekable();
        let mut start = 0;
        let mut offset = 0;
        while let Some(line) = iter.next() {
            if !line.starts_with(b"Thread dump") {
                start += line.len();
                continue;
            }
            offset += line.len();
            while let Some(line) = iter.next_if(|l| !l.trim_ascii().starts_with(b"TriggeredTime")) {
                offset += line.len();
            }
            break;
        }
        let contents = &self.0[start..start + offset];
        self.0 = &self.0[start + offset..];

        Some(contents)
    }
}
