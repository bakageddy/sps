pub struct StuckQueryTableIteratorMSSQL<'a>(pub &'a [u8]);

impl StuckQueryTableIteratorMSSQL<'_> {
    const PREAMBLE: &'static [u8] = b"Running queries information during stuck thread :";
}

impl<'a> Iterator for StuckQueryTableIteratorMSSQL<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.trim_ascii_start().is_empty() {
            return None;
        }

        let mut start = 0;
        let mut offset = 0;
        let mut iter = self.0.split_inclusive(|b| *b == b'\n').peekable();
        while let Some(line) = iter.next() {
            if !line.trim_ascii_end().ends_with(StuckQueryTableIteratorMSSQL::PREAMBLE) {
                start += line.len();
                continue;
            }
            offset += line.len();
            break;
        }

        while let Some(line) = iter.next_if(|line| {
            !line.trim_ascii_end().ends_with(StuckQueryTableIteratorMSSQL::PREAMBLE)
        }) {
            offset += line.len();
        }

        let contents = &self.0[start .. (start + offset)];
        self.0 = &self.0[start + offset ..];
        str::from_utf8(contents).ok()
    }
}
