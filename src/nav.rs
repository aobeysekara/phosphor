use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub is_hidden: bool,
}

/// Read directory contents and return sorted entries.
/// Directories are sorted first, then files. Each group is sorted alphabetically
/// (case-insensitive). Returns an error string on failure.
pub fn read_directory(dir: &Path) -> Result<Vec<FileEntry>, String> {
    let read_dir = fs::read_dir(dir)
        .map_err(|e| format!("Cannot read {}: {}", dir.display(), e))?;

    let mut entries: Vec<FileEntry> = Vec::new();

    for result in read_dir {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        let is_hidden = name.starts_with('.');

        let metadata = entry.metadata();
        let (is_dir, size) = match metadata {
            Ok(m) => (m.is_dir(), m.len()),
            Err(_) => (false, 0),
        };

        entries.push(FileEntry {
            name,
            path,
            is_dir,
            size,
            is_hidden,
        });
    }

    entries.sort_by(|a, b| {
        // Directories first
        match (a.is_dir, b.is_dir) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}

/// Format a byte count into a human-readable string.
pub fn format_size(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;

    if bytes >= GIB {
        format!("{:.1}G", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1}M", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1}K", bytes as f64 / KIB as f64)
    } else {
        format!("{}B", bytes)
    }
}
