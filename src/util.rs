use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use memmap2::Mmap;

pub type Result<T> = std::result::Result<T, crate::error::Error>;

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
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidFilename,
                format!("Cannot extract filepath from {path:#?}"),
            ))?
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
    let path = path.as_ref();
    let handle = fs::File::open(&path)?;
    let map = unsafe { memmap2::Mmap::map(&handle)? };
    Ok(map)
}
