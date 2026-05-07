use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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

/// Format a Unix permission triple (rwx for user/group/other).
pub fn format_perms(metadata: &fs::Metadata) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode() & 0o777;
        let bits: [(u32, char); 9] = [
            (0o400, 'r'), (0o200, 'w'), (0o100, 'x'),
            (0o040, 'r'), (0o020, 'w'), (0o010, 'x'),
            (0o004, 'r'), (0o002, 'w'), (0o001, 'x'),
        ];
        bits.iter()
            .map(|(b, c)| if mode & b != 0 { *c } else { '-' })
            .collect()
    }
    #[cfg(not(unix))]
    {
        if metadata.permissions().readonly() {
            "ro".to_string()
        } else {
            "rw".to_string()
        }
    }
}

/// Render a `SystemTime` as a relative-from-now description ("3m ago", "2d ago").
pub fn format_modified(t: SystemTime) -> String {
    let dur = match SystemTime::now().duration_since(t) {
        Ok(d) => d,
        Err(_) => return "in the future".to_string(),
    };
    let s = dur.as_secs();
    if s < 60 {
        format!("{}s ago", s)
    } else if s < 3600 {
        format!("{}m ago", s / 60)
    } else if s < 86400 {
        format!("{}h ago", s / 3600)
    } else if s < 604800 {
        format!("{}d ago", s / 86400)
    } else if s < 2592000 {
        format!("{}w ago", s / 604800)
    } else if s < 31536000 {
        format!("{}mo ago", s / 2592000)
    } else {
        format!("{}y ago", s / 31536000)
    }
}

/// Map a file extension to a short human label, falling back to the extension.
pub fn detect_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => "rust",
        Some("toml") => "toml",
        Some("md") => "markdown",
        Some("json") => "json",
        Some("yaml") | Some("yml") => "yaml",
        Some("py") => "python",
        Some("ts") | Some("tsx") => "typescript",
        Some("js") | Some("jsx") => "javascript",
        Some("go") => "go",
        Some("c") => "c",
        Some("cpp") | Some("cc") | Some("hpp") | Some("h") => "c++",
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") => "image",
        Some("txt") => "text",
        Some("sh") | Some("bash") | Some("zsh") => "shell",
        Some("html") | Some("htm") => "html",
        Some("css") => "css",
        Some("lock") => "lockfile",
        Some(_) => "file",
        None => "file",
    }
}
