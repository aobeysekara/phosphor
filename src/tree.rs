use std::cmp::Ordering;
use std::fs;
use std::path::Path;

const TREE_MAX_DEPTH: usize = 2;
const TREE_MAX_NODES: usize = 200;
/// Cap entries read from a single directory to avoid pathological allocation
/// on directories with millions of children.
const MAX_ENTRIES_PER_DIR: usize = 1000;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
    pub is_last: bool,
    /// For each ancestor depth, was that ancestor the last child of its parent?
    pub ancestors_last: Vec<bool>,
}

/// Build a flat, render-ordered list of nodes representing the directory tree
/// rooted at `root`. Truncates at `TREE_MAX_DEPTH` levels and `TREE_MAX_NODES`
/// total nodes. When `show_hidden` is false, dot-files are skipped.
pub fn build(root: &Path, show_hidden: bool) -> Vec<TreeNode> {
    let mut nodes = Vec::new();

    let root_name = root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.display().to_string());

    nodes.push(TreeNode {
        name: root_name,
        depth: 0,
        is_dir: true,
        is_last: true,
        ancestors_last: Vec::new(),
    });

    walk(root, 1, &[], show_hidden, &mut nodes);
    nodes
}

fn walk(
    dir: &Path,
    depth: usize,
    ancestors_last: &[bool],
    show_hidden: bool,
    nodes: &mut Vec<TreeNode>,
) {
    if depth > TREE_MAX_DEPTH || nodes.len() >= TREE_MAX_NODES {
        return;
    }

    let read_dir = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };

    let mut entries: Vec<(std::path::PathBuf, bool, String)> = read_dir
        .filter_map(|r| r.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if !show_hidden && name.starts_with('.') {
                return None;
            }
            let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
            Some((e.path(), is_dir, name))
        })
        .take(MAX_ENTRIES_PER_DIR)
        .collect();

    entries.sort_by(|a, b| match (a.1, b.1) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => a.2.to_lowercase().cmp(&b.2.to_lowercase()),
    });

    let total = entries.len();
    for (i, (path, is_dir, name)) in entries.into_iter().enumerate() {
        if nodes.len() >= TREE_MAX_NODES {
            return;
        }
        let is_last = i + 1 == total;
        nodes.push(TreeNode {
            name,
            depth,
            is_dir,
            is_last,
            ancestors_last: ancestors_last.to_vec(),
        });
        if is_dir {
            let mut next_ancestors = ancestors_last.to_vec();
            next_ancestors.push(is_last);
            walk(&path, depth + 1, &next_ancestors, show_hidden, nodes);
        }
    }
}

/// Format a node into a single line, e.g. `│   ├── app.rs`.
pub fn render_line(node: &TreeNode, max_width: usize) -> String {
    let mut prefix = String::new();
    for &is_last in &node.ancestors_last {
        prefix.push_str(if is_last { "    " } else { "│   " });
    }
    if node.depth > 0 {
        prefix.push_str(if node.is_last { "└── " } else { "├── " });
    }
    let suffix = if node.is_dir { "/" } else { "" };
    let line = format!("{}{}{}", prefix, node.name, suffix);

    if line.chars().count() > max_width {
        truncate_with_ellipsis(&line, max_width)
    } else {
        line
    }
}

fn truncate_with_ellipsis(s: &str, max: usize) -> String {
    if max <= 1 {
        return s.chars().take(max).collect();
    }
    let kept: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", kept)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(name: &str, depth: usize, is_dir: bool, is_last: bool, anc: Vec<bool>) -> TreeNode {
        TreeNode {
            name: name.to_string(),
            depth,
            is_dir,
            is_last,
            ancestors_last: anc,
        }
    }

    #[test]
    fn root_has_no_prefix() {
        assert_eq!(render_line(&node("phosphor", 0, true, true, vec![]), 80), "phosphor/");
    }

    #[test]
    fn first_level_uses_tee() {
        assert_eq!(
            render_line(&node("src", 1, true, false, vec![]), 80),
            "├── src/",
        );
    }

    #[test]
    fn last_first_level_uses_corner() {
        assert_eq!(
            render_line(&node("README.md", 1, false, true, vec![]), 80),
            "└── README.md",
        );
    }

    #[test]
    fn nested_under_non_last_carries_pipe() {
        assert_eq!(
            render_line(&node("app.rs", 2, false, false, vec![false]), 80),
            "│   ├── app.rs",
        );
    }

    #[test]
    fn nested_under_last_carries_blank() {
        assert_eq!(
            render_line(&node("ui.rs", 2, false, true, vec![true]), 80),
            "    └── ui.rs",
        );
    }

    #[test]
    fn long_lines_get_ellipsised() {
        let n = node("very_long_filename_that_overflows.rs", 1, false, true, vec![]);
        let rendered = render_line(&n, 12);
        assert_eq!(rendered.chars().count(), 12);
        assert!(rendered.ends_with('…'));
    }
}
