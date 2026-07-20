pub struct RunningQueriesIterator<'a>(pub &'a [u8]);

#[derive(Debug, PartialEq, Eq)]
pub enum Entry<'a> {
    Table(&'a str),
    Meta(&'a str),
    Unknown(&'a str),
}

impl<'a> Iterator for RunningQueriesIterator<'a> {
    type Item = Entry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.trim_ascii_start().is_empty() {
            return None;
        }

        enum IteratorState {
            Table,
            Meta,
            Unknown,
        }

        let mut offset = 0;
        let mut state = IteratorState::Unknown;
        let mut iter = self.0.split_inclusive(|b| *b == b'\n').peekable();
        if let Some(line) = iter.next() {
            if line.trim_ascii_start().starts_with(b"|") {
                state = IteratorState::Table;
                offset += line.len();
                while let Some(line) = iter.next_if(|l| l.trim_ascii_start().starts_with(b"|")) {
                    offset += line.len();
                }
            } else if line.trim_ascii_start().starts_with(b"[") {
                state = IteratorState::Meta;
                offset += line.len();
            } else {
                state = IteratorState::Unknown;
                offset += line.len();
            }
        }
        let chunk = &self.0[..offset];
        self.0 = &self.0[offset..];
        let inner = str::from_utf8(chunk).ok()?;
        match state {
            IteratorState::Table => Some(Entry::Table(inner)),
            IteratorState::Meta => Some(Entry::Meta(inner)),
            IteratorState::Unknown => Some(Entry::Unknown(inner)),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ingest::running_queries::{Entry, RunningQueriesIterator},
        util,
    };

    #[test]
    fn running_queries_iterator_table_single_line() {
        let input = r#"|  12688  |  4547.357970     |  4547.365633   |  servicedesk  |  active               |  f        |  SELECT count(*) as orphanentries FROM  notes  WHERE	 (notesid  not in 	(  	SELECT notesid 	FROM  workordernotes  	))  limit 50000                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |  2026-02-18 15:02:14.246541+05:30  |  PostgreSQL JDBC Driver  |                  |                   |               |"#;
        let mut iterator = RunningQueriesIterator(input.as_bytes());
        let item = iterator.next();
        assert!(item.is_some(), "Error during parsing");
        let item = item.unwrap();
        assert_eq!(item, Entry::Table(input));
    }

    #[test]
    fn running_queries_iterator_meta_single_line() {
        let input = r#"[16:18:01.505]|[18-02-2026]|[com.zoho.mickey.db.RunningQueries]|[INFO]|[1316919]| :: "#;
        let mut iterator = RunningQueriesIterator(input.as_bytes());
        let item = iterator.next();
        assert!(item.is_some(), "Error during parsing");
        let item = item.unwrap();
        assert_eq!(item, Entry::Meta(input));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn running_queries_iterator_table_full() {
        let map = util::map_file("test/runningqueries_single_table.txt").unwrap();
        let mut iterator = RunningQueriesIterator(&map);
        let item = iterator.next();
        assert!(item.is_some(), "Error during parsing");
        let item = item.unwrap();
        let Entry::Table(table) = item else {
            std::process::exit(1);
        };
        let mut count = 0;
        for _ in table.lines() {
            count += 1;
        }

        assert_eq!(count, 14);
        assert_eq!(iterator.next(), None);
    }
}
