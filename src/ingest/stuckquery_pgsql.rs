pub struct StuckQueryTableIteratorPGSQL<'a>(pub &'a [u8]);

impl StuckQueryTableIteratorPGSQL<'_> {
    const PREAMBLE: &'static [u8] = b"Running queries information during stuck thread :";
}

impl<'a> Iterator for StuckQueryTableIteratorPGSQL<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.trim_ascii_start().is_empty() {
            return None;
        }

        let mut start = 0;
        let mut offset = 0;
        let mut iter = self.0.split_inclusive(|b| *b == b'\n').peekable();
        while let Some(line) = iter.next() {
            if !line.trim_ascii_end().ends_with(StuckQueryTableIteratorPGSQL::PREAMBLE) {
                start += line.len();
                continue;
            }

            offset += line.len();
            while let Some(line) = iter.next_if(|line| {
                !line
                    .trim_ascii_end()
                    .ends_with(StuckQueryTableIteratorPGSQL::PREAMBLE)
            }) {
                offset += line.len();
            }
            break;
        }

        let contents = &self.0[start..(start + offset)];
        self.0 = &self.0[start + offset..];
        str::from_utf8(contents).ok()
    }
}

#[cfg(test)]
mod test {
    use crate::{ingest::stuckquery_pgsql::StuckQueryTableIteratorPGSQL, util};

    #[test]
    fn stuckquery_pgsql_streamer_basic() {
        let map = util::map_file("test/stuckqueries_pgsql_basic.txt").unwrap();
        let mut count = 0;
        for table in StuckQueryTableIteratorPGSQL(&map) {
            count += 1;
            assert!(table.len() != 0);
            println!("========================================================");
            println!("{table}");
            println!("========================================================");
        }

        assert!(count == 3)
    }

    #[test]
    fn stuckquery_pgsql_streamer_full() {
        let map = util::map_file("test/stuckqueries_pgsql_full.txt").unwrap();
        const TOTAL_TABLES: u32 = 8;
        let mut count = 0;
        for table in StuckQueryTableIteratorPGSQL(&map) {
            assert!(table.len() != 0);
            count += 1;
            println!("=========================================================");
            println!("{table}");
            println!("=========================================================");
        }

        assert!(count == TOTAL_TABLES);
    }
}
