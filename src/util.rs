use std::fs;
use std::io;
use std::num::ParseIntError;
use std::path::Path;
use std::path::PathBuf;
use memmap2::Advice;
use time::PrimitiveDateTime;

use memmap2::Mmap;

pub type Result<T> = std::result::Result<T, crate::error::Error>;

pub fn get_sorted_threaddumps<P>(root: P) -> io::Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let root = root.as_ref();
    let root = root.canonicalize()?;
    if !root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            format!("{root:#?} is not a directory"),
        ));
    }

    let mut entries = Vec::with_capacity(10);
    for entry in root.read_dir()? {
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
    let root = root.as_ref();
    let root = root.canonicalize()?;
    if !root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            format!("{root:#?} is not a directory"),
        ));
    }

    let mut entries = Vec::new();
    for entry in root.read_dir()? {
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

pub fn map_file<P>(path: P) -> self::Result<Mmap>
where
    P: AsRef<Path>,
{
    let handle = fs::File::open(path)?;
    let map = unsafe { memmap2::Mmap::map(&handle)? };
    let _ = map.advise(Advice::WillNeed)?;
    Ok(map)
}

pub fn parse_u64(value: &[u8]) -> std::result::Result<u64, ParseIntError> {
    String::from_utf8_lossy(value).parse()
}

pub fn parse_u32(value: &[u8]) -> std::result::Result<u32, ParseIntError> {
    String::from_utf8_lossy(value).parse()
}

pub fn parse_comma_separated_u32(value: &[u8]) -> std::result::Result<u32, ParseIntError> {
    String::from_utf8_lossy(value).replace(',', "").parse()
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
