pub struct CPUMonitoringIterator<'a>(pub &'a [u8]);

impl<'a> Iterator for CPUMonitoringIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.trim_ascii_start().is_empty() {
            return None;
        }

        let mut start = 0;
        let mut offset = 0;
        let mut iter = self.0.split_inclusive(|b| *b == b'\n').peekable();

        while let Some(line) = iter.next() {
            if !line.starts_with(b"[") || !line.trim_ascii_end().ends_with(b"::") {
                start += line.len();
                continue;
            }

            offset += line.len();
            break;
        }

        while let Some(line) = iter.next_if(|l| !l.starts_with(b"[")) {
            offset += line.len();
        }

        let contents = &self.0[start..(start + offset)];
        self.0 = &self.0[start + offset..];
        str::from_utf8(contents).ok()
    }
}
