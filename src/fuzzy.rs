use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

use crate::nav::FileEntry;

/// Filter entries by fuzzy-matching against the query.
/// Returns indices into the original entries vec, ordered by score (best first).
pub fn filter_entries(entries: &[FileEntry], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..entries.len()).collect();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );

    let mut buf = Vec::new();
    let mut scored: Vec<(usize, u32)> = entries
        .iter()
        .enumerate()
        .filter_map(|(i, entry)| {
            let haystack = Utf32Str::new(&entry.name, &mut buf);
            pattern
                .score(haystack, &mut matcher)
                .map(|score| (i, score))
        })
        .collect();

    // Sort by score descending, then by original index ascending for stability
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    scored.into_iter().map(|(i, _)| i).collect()
}
