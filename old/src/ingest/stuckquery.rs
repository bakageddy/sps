use crate::ingest::kind::DBKind;

pub enum StuckQueryIterator<'a> {
    PGSQL(PGSQLIterator<'a>),
    MSSQL(MSSQLIterator<'a>),
}

struct PGSQLIterator<'a>(&'a [u8]);
struct MSSQLIterator<'a>(&'a [u8]);

impl<'a> StuckQueryIterator<'a> {
    pub fn new(kind: DBKind, body: &'a [u8]) -> Self {
        match kind {
            DBKind::PGSQL => StuckQueryIterator::PGSQL(PGSQLIterator(body)),
            DBKind::MSSQL => StuckQueryIterator::MSSQL(MSSQLIterator(body)),
        }
    }
}

impl<'a> Iterator for StuckQueryIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::PGSQL(iter) => iter.next(),
            Self::MSSQL(iter) => iter.next(),
        }
    }
}

impl PGSQLIterator<'_> {
    const PREAMBLE: &'static [u8] = b"Running queries information during stuck thread :";
}

impl<'a> Iterator for PGSQLIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.trim_ascii_start().is_empty() {
            return None;
        }

        let mut start = 0;
        let mut offset = 0;
        let mut iter = self.0.split_inclusive(|b| *b == b'\n').peekable();
        while let Some(line) = iter.next() {
            if !line.trim_ascii_end().ends_with(Self::PREAMBLE) {
                start += line.len();
                continue;
            }

            offset += line.len();
            while let Some(line) =
                iter.next_if(|line| !line.trim_ascii_end().ends_with(Self::PREAMBLE))
            {
                offset += line.len();
            }
            break;
        }

        let contents = &self.0[start..(start + offset)];
        self.0 = &self.0[start + offset..];
        str::from_utf8(contents).ok()
    }
}

impl<'a> MSSQLIterator<'a> {
    const PREAMBLE: &'static [u8] = b"Running queries information during stuck thread :";
}

impl<'a> Iterator for MSSQLIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.trim_ascii_start().is_empty() {
            return None;
        }

        let mut start = 0;
        let mut offset = 0;
        let mut iter = self.0.split_inclusive(|b| *b == b'\n').peekable();
        while let Some(line) = iter.next() {
            if !line.trim_ascii_end().ends_with(Self::PREAMBLE) {
                start += line.len();
                continue;
            }
            offset += line.len();
            break;
        }

        while let Some(line) = iter.next_if(|line| !line.trim_ascii_end().ends_with(Self::PREAMBLE))
        {
            offset += line.len();
        }

        let contents = &self.0[start..(start + offset)];
        self.0 = &self.0[start + offset..];
        str::from_utf8(contents).ok()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ingest::stuckquery::{MSSQLIterator, PGSQLIterator},
        parser::stuckquery::MSSQLTable,
        util,
    };

    #[test]
    fn stuckquery_pgsql_streamer_basic() {
        let map = util::map_file("test/stuckqueries_pgsql_basic.txt").unwrap();
        let mut count = 0;
        for table in PGSQLIterator(&map) {
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
        for table in PGSQLIterator(&map) {
            assert!(table.len() != 0);
            count += 1;
            println!("=========================================================");
            println!("{table}");
            println!("=========================================================");
        }

        assert!(count == TOTAL_TABLES);
    }

    #[test]
    fn stuckquery_mssql_full_file() {
        let map = util::map_file("test/stuckqueries_mssql_full.txt").unwrap();
        let mut count = 0;
        for chunk in MSSQLIterator(&map) {
            assert_ne!(chunk.len(), 0);
            let table = MSSQLTable::try_from(chunk);
            assert!(
                table.is_ok(),
                "Error during parsing table: {}",
                table.unwrap_err()
            );
            count += 1;
        }
        assert_eq!(count, 4);
    }

    #[test]
    fn stuckquery_mssql_empty_file() {
        let bytes = "".as_bytes();
        let empty_chunk = MSSQLIterator(bytes).next();
        assert!(empty_chunk.is_none())
    }
}
