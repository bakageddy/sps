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
    use crate::{
        ingest::cpumonitoring::CPUMonitoringIterator, parser::cpumonitoring::CPUThread, util,
    };

    #[test]
    fn cpumonitoring_iterator_empty_file() {
        let file_contents = "";
        let chunk = CPUMonitoringIterator(file_contents.as_bytes()).next();
        assert_eq!(chunk, None);
    }

    #[test]
    fn cpumonitoring_iterator_file() {
        let map = util::map_file("test/cpumonitoring_full_file.txt").unwrap();
        let mut count = 0;
        for chunk in CPUMonitoringIterator(&map) {
            assert_ne!(chunk.len(), 0);
            let thread = CPUThread::try_from(chunk);
            assert!(
                thread.is_ok(),
                "Error during parsing thread: {:?}",
                thread.unwrap_err()
            );
            let thread = thread.unwrap();
            assert!(thread.trace.is_some());
            assert!(thread.cpu >= 0.5);

            count += 1;
        }

        assert_eq!(count, 15);
    }

    #[test]
    fn cpumonitoring_iterator_file_emptythreads() {
        let map = util::map_file("test/cpumonitoring_full_file_empty.txt").unwrap();
        let mut count = 0;
        for chunk in CPUMonitoringIterator(&map) {
            assert_ne!(chunk.len(), 0);
            let thread = CPUThread::try_from(chunk);
            assert!(
                thread.is_ok(),
                "Error during parsing thread: {:?}",
                thread.unwrap_err()
            );

            let thread = thread.unwrap();
            assert!(thread.trace.is_none());
            assert!(thread.cpu < 0.5);
            count += 1;
        }

        assert_eq!(count, 117);
    }

    #[test]
    fn cpumonitoring_iterator_full_file() {
        let map = util::map_file("test/CPUMonitoring0.txt").unwrap();
        let mut count = 0;
        for chunk in CPUMonitoringIterator(&map) {
            assert_ne!(chunk.len(), 0);
            let thread = CPUThread::try_from(chunk);
            if thread.is_ok() {
                count += 1;
            }
        }

        assert_eq!(count, 1494);
    }
}
