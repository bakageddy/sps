use memmap2::Advice;
use std::fs;
use std::io;
use std::iter::Sum;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use time::PrimitiveDateTime;

use memmap2::Mmap;

pub type Result<T> = std::result::Result<T, crate::error::Error>;

pub fn get_sorted_threaddumps<P>(root: P) -> io::Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let mut entries = Vec::with_capacity(10);
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        let filename = path
            .file_name()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidFilename,
                    format!("Cannot extract filepath from {path:#?}"),
                )
            })?
            .to_string_lossy();

        if filename.starts_with("threaddump") {
            entries.push(path);
        }
    }

    entries.sort_by_key(|p| {
        p.file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_prefix("threaddump"))
            .and_then(|f| f.strip_suffix(".txt"))
            .and_then(|n| n.parse::<u32>().ok())
    });

    entries.reverse();
    Ok(entries)
}

pub fn get_sorted_stuckthreads<P>(root: P) -> io::Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let mut entries = Vec::new();
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        let filename = path
            .file_name()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidFilename,
                    format!("Cannot extract filepath from {path:#?}"),
                )
            })?
            .to_string_lossy();
        if filename.starts_with("stuckthreads") {
            entries.push(path);
        }
    }

    entries.sort_by_key(|p| {
        p.file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_prefix("stuckthreads"))
            .and_then(|f| f.strip_suffix(".txt"))
            .and_then(|n| n.parse::<u32>().ok())
    });
    entries.reverse();
    Ok(entries)
}

pub fn get_sorted_stuckqueries<P>(root: P) -> io::Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let mut entries = Vec::with_capacity(5);
    let dir = fs::read_dir(&root)?;
    for entry in dir {
        let path = entry?.path();
        let filename = path
            .file_name()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidFilename,
                    format!("Cannot extract filepath from {path:#?}"),
                )
            })?
            .to_string_lossy();
        if filename.starts_with("stuckqueries") {
            entries.push(path);
        }
    }

    entries.sort_by_key(|p| {
        p.file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_prefix("stuckqueries"))
            .and_then(|f| f.strip_suffix(".txt"))
            .and_then(|n| n.parse::<u32>().ok())
    });
    entries.reverse();
    Ok(entries)
}

pub fn map_file<P>(path: P) -> self::Result<Mmap>
where
    P: AsRef<Path>,
{
    let handle = fs::File::open(path)?;
    let map = unsafe { memmap2::Mmap::map(&handle)? };
    let _ = map.advise(Advice::WillNeed)?;
    Ok(map)
}

pub fn parse_num<T>(value: &str) -> std::result::Result<T, <T as FromStr>::Err>
where
    T: FromStr + Sum + Default + PartialOrd,
{
    value.parse::<T>()
}

pub fn parse_comma_separated_u32(value: &str) -> Option<u32> {
    let mut result = 0u32;
    for c in value.bytes() {
        match c {
            b'0'..=b'9' => {
                result *= 10;
                result += (c - b'0') as u32;
            }
            b',' => continue,
            _ => return None,
        };
    }
    Some(result)
}

pub trait ToUnixMillis {
    fn to_unix_millis(&self) -> Option<u64>;
}

impl ToUnixMillis for PrimitiveDateTime {
    fn to_unix_millis(&self) -> Option<u64> {
        let result = self.assume_utc().unix_timestamp_nanos() / 1_000_000;
        u64::try_from(result).ok()
    }
}
