pub struct CPUMemStatsIterator<'a>(pub &'a [u8]);

impl<'a> Iterator for CPUMemStatsIterator<'a> {
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

        while let Some(line) = iter.next_if(|line| !line.starts_with(b"[")) {
            offset += line.len();
        }

        let chunk = &self.0[start..(start + offset)];
        self.0 = &self.0[start + offset..];
        if chunk.len() == 0 {
            None
        } else {
            str::from_utf8(chunk).ok()
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::{
        ingest::cpumemstats_windows::CPUMemStatsIterator,
        parser::cpumemstats_windows::CPUMemoryStats::{self},
        util,
    };

    #[test]
    fn cpumemstats_iterator_empty_file() {
        let input = "".as_bytes();
        let next = CPUMemStatsIterator(&input).next();
        assert!(next.is_none());
    }

    #[test]
    fn cpumemstats_iterator_full_file() {
        let path = PathBuf::from("./test/cpumemstats0.txt");
        let map = util::map_file(path).unwrap();
        let mut cpu_table_count = 0;
        let mut mem_table_count = 0;
        for chunk in CPUMemStatsIterator(&map) {
            dbg!(cpu_table_count);
            dbg!(mem_table_count);
            assert_ne!(chunk.len(), 0);
            let table = CPUMemoryStats::try_from(chunk);
            assert!(
                table.is_ok(),
                "Error during parsing table: {0}",
                table.unwrap_err()
            );

            match table.unwrap() {
                CPUMemoryStats::CPU(_) => cpu_table_count += 1,
                CPUMemoryStats::Memory(_) => mem_table_count += 1,
            };
        }

        assert_eq!(cpu_table_count, 818);
        assert_eq!(mem_table_count, 818);
    }

    #[test]
    fn cpumemstats_iterator_single_chunk() {
        let map = util::map_file("test/cpumemstats_single_chunk.txt").unwrap();
        let mut iter = CPUMemStatsIterator(&map).into_iter();
        let chunk = iter.next();
        assert!(chunk.is_some(), "Error during slicing chunks");
        let chunk = chunk.unwrap();
        eprintln!("CHUNK: {chunk}");
    }
}
