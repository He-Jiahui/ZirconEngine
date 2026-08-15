use crate::ui::asset_editor::UiDesignerSelectionModel;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::UiAssetDocument;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiAssetSourceSelectionSummary {
    pub block_label: String,
    pub line: i32,
    pub excerpt: String,
    pub roundtrip_status: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiAssetSourceOutlineEntry {
    pub node_id: String,
    pub block_label: String,
    pub line: i32,
    pub end_line: i32,
    pub excerpt: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct UiAssetSourceOutlineIndex {
    entries: Vec<UiAssetSourceOutlineEntry>,
    entry_indexes: BTreeMap<String, usize>,
    line_segments: Vec<UiAssetSourceOutlineLineSegment>,
}

/// Retains the one immutable outline artifact for the current source and
/// document generation. Consumers borrow it instead of rescanning source.
pub(crate) struct UiAssetSourceOutlineCache {
    source_revision: Option<u64>,
    index: Arc<UiAssetSourceOutlineIndex>,
    #[cfg(test)]
    build_count: usize,
}

impl UiAssetSourceOutlineCache {
    pub(crate) fn new(source_revision: u64, index: Arc<UiAssetSourceOutlineIndex>) -> Self {
        Self {
            source_revision: Some(source_revision),
            index,
            #[cfg(test)]
            build_count: 0,
        }
    }

    pub(crate) fn from_built(source_revision: u64, index: Arc<UiAssetSourceOutlineIndex>) -> Self {
        let mut cache = Self::new(source_revision, index);
        cache.record_build();
        cache
    }

    pub(crate) fn is_current(&self, source_revision: u64) -> bool {
        self.source_revision == Some(source_revision)
    }

    pub(crate) fn replace(&mut self, source_revision: u64, index: UiAssetSourceOutlineIndex) {
        self.replace_shared(source_revision, Arc::new(index));
        self.record_build();
    }

    pub(crate) fn replace_shared(
        &mut self,
        source_revision: u64,
        index: Arc<UiAssetSourceOutlineIndex>,
    ) {
        self.source_revision = Some(source_revision);
        self.index = index;
    }

    pub(crate) fn replace_shared_built(
        &mut self,
        source_revision: u64,
        index: Arc<UiAssetSourceOutlineIndex>,
    ) {
        self.replace_shared(source_revision, index);
        self.record_build();
    }

    pub(crate) fn index(&self) -> &UiAssetSourceOutlineIndex {
        self.index.as_ref()
    }

    fn record_build(&mut self) {
        #[cfg(test)]
        {
            self.build_count += 1;
        }
    }

    #[cfg(test)]
    pub(crate) fn build_count(&self) -> usize {
        self.build_count
    }

    #[cfg(test)]
    pub(crate) fn shares_index_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.index, &other.index)
    }
}

impl UiAssetSourceOutlineIndex {
    pub(crate) fn entries(&self) -> &[UiAssetSourceOutlineEntry] {
        &self.entries
    }

    pub(crate) fn entry_for_node(&self, node_id: &str) -> Option<&UiAssetSourceOutlineEntry> {
        self.entry_indexes
            .get(node_id)
            .and_then(|index| self.entries.get(*index))
    }

    pub(crate) fn index_for_node(&self, node_id: &str) -> Option<usize> {
        self.entry_indexes.get(node_id).copied()
    }

    pub(crate) fn entry_for_line(&self, line: usize) -> Option<&UiAssetSourceOutlineEntry> {
        let line = i32::try_from(line).ok()?;
        let segment_index = self
            .line_segments
            .partition_point(|segment| segment.start_line <= line)
            .checked_sub(1)?;
        let segment = self.line_segments.get(segment_index)?;
        (line <= segment.end_line)
            .then(|| self.entries.get(segment.entry_index))
            .flatten()
    }

    pub(crate) fn node_id_for_line(&self, line: usize) -> Option<&str> {
        self.entry_for_line(line)
            .map(|entry| entry.node_id.as_str())
    }

    fn from_entries(entries: Vec<UiAssetSourceOutlineEntry>) -> Self {
        let entry_indexes = entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (entry.node_id.clone(), index))
            .collect();
        let line_segments = build_line_segments(&entries);
        Self {
            entries,
            entry_indexes,
            line_segments,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct UiAssetSourceOutlineLineSegment {
    start_line: i32,
    end_line: i32,
    entry_index: usize,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct UiAssetSourceOutlineLinePriority {
    start_line: i32,
    span: std::cmp::Reverse<i32>,
    entry_index: usize,
}

#[derive(Default)]
struct UiAssetSourceOutlineLineEvents {
    starts: Vec<UiAssetSourceOutlineLinePriority>,
    ends: Vec<UiAssetSourceOutlineLinePriority>,
}

fn build_line_segments(
    entries: &[UiAssetSourceOutlineEntry],
) -> Vec<UiAssetSourceOutlineLineSegment> {
    let mut events = BTreeMap::<i32, UiAssetSourceOutlineLineEvents>::new();
    for (entry_index, entry) in entries.iter().enumerate() {
        if entry.end_line < entry.line {
            continue;
        }
        let end_line = entry.end_line;
        let priority = UiAssetSourceOutlineLinePriority {
            start_line: entry.line,
            span: std::cmp::Reverse(end_line.saturating_sub(entry.line)),
            entry_index,
        };
        events.entry(entry.line).or_default().starts.push(priority);
        if let Some(after_end_line) = end_line.checked_add(1) {
            events
                .entry(after_end_line)
                .or_default()
                .ends
                .push(priority);
        }
    }

    let boundaries = events.keys().copied().collect::<Vec<_>>();
    let mut active = BTreeSet::new();
    let mut segments = Vec::new();
    for (boundary_index, start_line) in boundaries.iter().copied().enumerate() {
        let events_at_line = &events[&start_line];
        for priority in &events_at_line.ends {
            active.remove(priority);
        }
        for priority in &events_at_line.starts {
            active.insert(*priority);
        }
        let Some(next_start_line) = boundaries.get(boundary_index + 1).copied() else {
            if let Some(priority) = active.last().copied() {
                segments.push(UiAssetSourceOutlineLineSegment {
                    start_line,
                    end_line: i32::MAX,
                    entry_index: priority.entry_index,
                });
            }
            continue;
        };
        let Some(priority) = active.last().copied() else {
            continue;
        };
        let end_line = next_start_line.saturating_sub(1);
        if start_line <= end_line {
            segments.push(UiAssetSourceOutlineLineSegment {
                start_line,
                end_line,
                entry_index: priority.entry_index,
            });
        }
    }
    segments
}

pub(crate) fn build_source_selection_summary(
    outline: &UiAssetSourceOutlineIndex,
    selection: &UiDesignerSelectionModel,
    diagnostics: &[String],
    preferred_line_offset: Option<usize>,
) -> UiAssetSourceSelectionSummary {
    let Some(node_id) = selection.primary_node_id.as_deref() else {
        return UiAssetSourceSelectionSummary {
            line: -1,
            roundtrip_status: invalid_prefix(diagnostics, "no node selected"),
            ..Default::default()
        };
    };
    let block_label = format!("[nodes.{node_id}]");
    let (line, excerpt) = outline
        .entry_for_node(node_id)
        .map(|entry| {
            (
                resolved_source_line(&entry, preferred_line_offset),
                entry.excerpt.clone(),
            )
        })
        .unwrap_or((-1, String::new()));
    let roundtrip_status = if line > 0 {
        invalid_prefix(diagnostics, &format!("selection maps to line {line}"))
    } else {
        invalid_prefix(
            diagnostics,
            &format!("selection block {block_label} was not found in source"),
        )
    };
    UiAssetSourceSelectionSummary {
        block_label,
        line,
        excerpt,
        roundtrip_status,
    }
}

pub(crate) fn source_line_for_byte_offset(source: &str, byte_offset: usize) -> usize {
    let clamped = byte_offset.min(source.len());
    source.as_bytes()[..clamped]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count()
        + 1
}

pub(crate) fn source_byte_offset_for_line(source: &str, line: usize) -> usize {
    if line <= 1 {
        return 0;
    }
    let mut current_line = 1usize;
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            current_line += 1;
            if current_line == line {
                return index + 1;
            }
        }
    }
    source.len()
}

pub(crate) fn build_source_outline_index(
    document: &UiAssetDocument,
    source: &str,
) -> UiAssetSourceOutlineIndex {
    build_source_outline_index_for_node_ids(
        source,
        document
            .iter_nodes()
            .map(|node| node.node_id.as_str())
            .collect::<BTreeSet<_>>(),
    )
}

fn build_source_outline_index_for_node_ids<'node>(
    source: &str,
    node_ids: impl IntoIterator<Item = &'node str>,
) -> UiAssetSourceOutlineIndex {
    let node_ids = node_ids.into_iter().collect::<BTreeSet<_>>();
    if node_ids.is_empty() {
        return UiAssetSourceOutlineIndex::default();
    }

    let lines = source.lines().collect::<Vec<_>>();
    let mut headers = Vec::new();
    let mut direct_entries = BTreeMap::new();
    let mut tree_candidates = BTreeMap::new();
    let mut seen_tree_node_ids = BTreeSet::new();
    let mut last_array_headers = BTreeMap::new();
    let mut last_non_array_header = None;
    let mut pending_direct = None::<(String, usize)>;

    for (line_index, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with('[') {
            if let Some((node_id, start_index)) = pending_direct.take() {
                direct_entries.entry(node_id.clone()).or_insert_with(|| {
                    direct_outline_entry(&node_id, &lines, start_index, line_index)
                });
            }
        }

        if let Some(header) = parse_header_line(line_index, line) {
            let header_index = headers.len();
            if header.is_array {
                last_array_headers.insert(header.path.clone(), header_index);
            } else {
                last_non_array_header = Some(TreeNodeHeader {
                    range_header_index: wrapping_array_header_index(
                        &last_array_headers,
                        &header.path,
                    )
                    .unwrap_or(header_index),
                });
                if let Some(node_id) = direct_node_id(line) {
                    if node_ids.contains(node_id) && !direct_entries.contains_key(node_id) {
                        pending_direct = Some((node_id.to_owned(), line_index));
                    }
                }
            }
            headers.push(header);
            continue;
        }

        let Some(node_id) = tree_node_id(line) else {
            continue;
        };
        if !node_ids.contains(node_id) || !seen_tree_node_ids.insert(node_id) {
            continue;
        }
        let Some(node_header) = last_non_array_header else {
            continue;
        };
        tree_candidates.insert(
            node_id.to_owned(),
            TreeNodeCandidate {
                node_line_index: line_index,
                range_header_index: node_header.range_header_index,
            },
        );
    }

    if let Some((node_id, start_index)) = pending_direct {
        direct_entries
            .entry(node_id.clone())
            .or_insert_with(|| direct_outline_entry(&node_id, &lines, start_index, lines.len()));
    }

    let header_end_lines = header_range_end_lines(&headers, lines.len());
    let mut entries = node_ids
        .into_iter()
        .filter_map(|node_id| {
            direct_entries.remove(node_id).or_else(|| {
                tree_candidates.remove(node_id).map(|candidate| {
                    tree_outline_entry(node_id, &lines, &headers, &header_end_lines, candidate)
                })
            })
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then_with(|| left.block_label.cmp(&right.block_label))
    });
    UiAssetSourceOutlineIndex::from_entries(entries)
}

fn direct_outline_entry(
    node_id: &str,
    lines: &[&str],
    start_index: usize,
    end_index: usize,
) -> UiAssetSourceOutlineEntry {
    UiAssetSourceOutlineEntry {
        node_id: node_id.to_owned(),
        block_label: format!("[nodes.{node_id}]"),
        line: start_index as i32 + 1,
        end_line: end_index.saturating_sub(1) as i32 + 1,
        excerpt: capture_block_excerpt(lines, start_index),
    }
}

fn tree_outline_entry(
    node_id: &str,
    lines: &[&str],
    headers: &[SourceHeaderLine],
    header_end_lines: &[usize],
    candidate: TreeNodeCandidate,
) -> UiAssetSourceOutlineEntry {
    let block_label = format!("[nodes.{node_id}]");
    let excerpt_start = headers[candidate.range_header_index].line_index;
    let mut excerpt = capture_block_excerpt(lines, excerpt_start);
    if !excerpt.contains(&block_label) {
        excerpt = if excerpt.is_empty() {
            block_label.clone()
        } else {
            format!("{block_label}\n{excerpt}")
        };
    }
    UiAssetSourceOutlineEntry {
        node_id: node_id.to_owned(),
        block_label,
        line: candidate.node_line_index as i32 + 1,
        end_line: header_end_lines[candidate.range_header_index] as i32 + 1,
        excerpt,
    }
}

fn capture_block_excerpt(lines: &[&str], start: usize) -> String {
    let mut excerpt = Vec::new();
    for line in lines.iter().skip(start) {
        if !excerpt.is_empty() && line.trim_start().starts_with('[') {
            break;
        }
        excerpt.push((*line).to_string());
        if excerpt.len() >= 8 {
            break;
        }
    }
    excerpt.join("\n")
}

fn direct_node_id(line: &str) -> Option<&str> {
    line.trim()
        .strip_prefix("[nodes.")?
        .strip_suffix(']')
        .filter(|node_id| !node_id.is_empty())
}

fn tree_node_id(line: &str) -> Option<&str> {
    line.trim().strip_prefix("node_id = \"")?.strip_suffix('"')
}

fn wrapping_array_header_index(
    last_array_headers: &BTreeMap<String, usize>,
    node_path: &str,
) -> Option<usize> {
    let mut parent_end = 0;
    let mut matching_wrapper = None;
    for segment in node_path.split('.') {
        if segment == "node" {
            matching_wrapper =
                matching_wrapper.max(last_array_headers.get(&node_path[..parent_end]).copied());
        }
        let delimiter = if parent_end == 0 { 0 } else { 1 };
        parent_end = parent_end
            .saturating_add(delimiter)
            .saturating_add(segment.len());
    }
    matching_wrapper
}

fn header_range_end_lines(headers: &[SourceHeaderLine], line_count: usize) -> Vec<usize> {
    let mut end_lines = vec![line_count.saturating_sub(1); headers.len()];
    let mut active_headers = Vec::<usize>::new();
    let mut open_headers = Vec::new();
    let mut active_array_headers = BTreeMap::<String, BTreeSet<usize>>::new();
    for (next_index, next_header) in headers.iter().enumerate() {
        if next_header.is_array {
            if let Some(repeated_headers) = active_array_headers.remove(&next_header.path) {
                for active_index in repeated_headers {
                    if open_headers[active_index] {
                        end_lines[active_index] = next_header.line_index.saturating_sub(1);
                        open_headers[active_index] = false;
                    }
                }
            }
        }

        while let Some(active_index) = active_headers.last().copied() {
            let active_header = &headers[active_index];
            if header_path_contains(&active_header.path, &next_header.path) {
                break;
            }
            active_headers.pop();
            if !open_headers[active_index] {
                continue;
            }
            end_lines[active_index] = next_header.line_index.saturating_sub(1);
            open_headers[active_index] = false;
            if active_header.is_array {
                remove_active_array_header(
                    &mut active_array_headers,
                    &active_header.path,
                    active_index,
                );
            }
        }

        active_headers.push(next_index);
        open_headers.push(true);
        if next_header.is_array {
            active_array_headers
                .entry(next_header.path.clone())
                .or_default()
                .insert(next_index);
        }
    }
    end_lines
}

fn remove_active_array_header(
    active_array_headers: &mut BTreeMap<String, BTreeSet<usize>>,
    path: &str,
    header_index: usize,
) {
    let should_remove = active_array_headers
        .get_mut(path)
        .map(|headers| {
            headers.remove(&header_index);
            headers.is_empty()
        })
        .unwrap_or(false);
    if should_remove {
        active_array_headers.remove(path);
    }
}

fn header_path_contains(parent_path: &str, child_path: &str) -> bool {
    child_path == parent_path
        || child_path
            .strip_prefix(parent_path)
            .is_some_and(|suffix| suffix.starts_with('.'))
}

#[derive(Clone, Copy, Debug)]
struct TreeNodeCandidate {
    node_line_index: usize,
    range_header_index: usize,
}

#[derive(Clone, Copy, Debug)]
struct TreeNodeHeader {
    range_header_index: usize,
}

#[derive(Clone, Debug)]
struct SourceHeaderLine {
    line_index: usize,
    path: String,
    is_array: bool,
}

fn parse_header_line(line_index: usize, line: &str) -> Option<SourceHeaderLine> {
    let trimmed = line.trim();
    if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
        return Some(SourceHeaderLine {
            line_index,
            path: trimmed[2..trimmed.len().saturating_sub(2)].to_string(),
            is_array: true,
        });
    }
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return Some(SourceHeaderLine {
            line_index,
            path: trimmed[1..trimmed.len().saturating_sub(1)].to_string(),
            is_array: false,
        });
    }
    None
}

fn resolved_source_line(
    entry: &UiAssetSourceOutlineEntry,
    preferred_line_offset: Option<usize>,
) -> i32 {
    let line_offset = preferred_line_offset
        .unwrap_or_default()
        .min((entry.end_line - entry.line).max(0) as usize);
    entry.line + line_offset as i32
}

fn invalid_prefix(diagnostics: &[String], message: &str) -> String {
    if diagnostics.is_empty() {
        message.to_string()
    } else {
        format!("source invalid, preview uses last valid snapshot; {message}")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_source_outline_index_for_node_ids, UiAssetSourceOutlineEntry,
        UiAssetSourceOutlineIndex,
    };

    #[test]
    fn outline_index_preserves_direct_block_ranges_and_line_queries() {
        let source = "[nodes.root]\nkind = \"container\"\nname = \"Root\"\n[nodes.label]\nkind = \"label\"\n";
        let index = build_source_outline_index_for_node_ids(source, ["root", "label"]);

        assert_eq!(index.entries().len(), 2);
        assert_eq!(
            index.entry_for_node("root").map(|entry| entry.line),
            Some(1)
        );
        assert_eq!(
            index.entry_for_node("root").map(|entry| entry.end_line),
            Some(3)
        );
        assert_eq!(index.node_id_for_line(2), Some("root"));
        assert_eq!(index.node_id_for_line(4), Some("label"));
    }

    #[test]
    fn outline_index_preserves_tree_ranges_and_node_line_queries() {
        let source = "[[nodes]]\n[nodes.node]\nnode_id = \"root\"\n[nodes.node.style]\ncolor = \"white\"\n[[nodes]]\n[nodes.node]\nnode_id = \"label\"\n";
        let index = build_source_outline_index_for_node_ids(source, ["root", "label"]);

        assert_eq!(
            index.entry_for_node("root").map(|entry| entry.line),
            Some(3)
        );
        assert_eq!(
            index.entry_for_node("root").map(|entry| entry.end_line),
            Some(5)
        );
        assert_eq!(index.node_id_for_line(4), Some("root"));
        assert_eq!(index.node_id_for_line(8), Some("label"));
    }

    #[test]
    fn outline_index_preserves_direct_blocks_over_tree_fallbacks() {
        let source =
            "[nodes.root]\nkind = \"container\"\n[[nodes]]\n[nodes.node]\nnode_id = \"root\"\n";
        let index = build_source_outline_index_for_node_ids(source, ["root"]);

        assert_eq!(
            index.entry_for_node("root").map(|entry| entry.line),
            Some(1)
        );
        assert_eq!(
            index.entry_for_node("root").map(|entry| entry.end_line),
            Some(2)
        );
        assert_eq!(index.node_id_for_line(4), None);
    }

    #[test]
    fn outline_index_stops_direct_ranges_at_malformed_header_boundaries() {
        let source = "[nodes.root]\nkind = \"container\"\n[broken\nname = \"not root\"\n";
        let index = build_source_outline_index_for_node_ids(source, ["root"]);

        assert_eq!(
            index.entry_for_node("root").map(|entry| entry.end_line),
            Some(2)
        );
        assert_eq!(index.node_id_for_line(3), None);
        assert_eq!(index.node_id_for_line(4), None);
    }

    #[test]
    fn outline_index_freezes_tree_wrapper_before_the_node_id_line() {
        let source = "[[nodes]]\n[nodes.node]\n[[nodes]]\nnode_id = \"root\"\n";
        let index = build_source_outline_index_for_node_ids(source, ["root"]);

        assert_eq!(
            index.entry_for_node("root").map(|entry| entry.end_line),
            Some(2)
        );
        assert_eq!(index.node_id_for_line(4), None);
    }

    #[test]
    fn outline_index_only_matches_complete_node_path_segments_for_tree_wrappers() {
        let source = "[[nodes]]\n[nodes.nodelet]\nnode_id = \"root\"\n[[nodes]]\n";
        let index = build_source_outline_index_for_node_ids(source, ["root"]);

        assert_eq!(
            index
                .entry_for_node("root")
                .map(|entry| entry.excerpt.as_str()),
            Some("[nodes.root]\n[nodes.nodelet]\nnode_id = \"root\"")
        );
    }

    #[test]
    fn outline_index_uses_the_complete_parent_path_for_tree_wrappers() {
        let source = "[[nodes]]\n[nodes.node]\nnode_id = \"root\"\n[[nodes]]\n";
        let index = build_source_outline_index_for_node_ids(source, ["root"]);

        assert_eq!(
            index
                .entry_for_node("root")
                .map(|entry| entry.excerpt.as_str()),
            Some("[[nodes]]\n[nodes.node]\nnode_id = \"root\"")
        );
    }

    #[test]
    fn outline_index_preserves_the_first_unmapped_tree_node_occurrence() {
        let source = "node_id = \"root\"\n[[nodes]]\n[nodes.node]\nnode_id = \"root\"\n";
        let index = build_source_outline_index_for_node_ids(source, ["root"]);

        assert_eq!(index.entry_for_node("root"), None);
    }

    #[test]
    fn outline_index_prefers_the_most_specific_precompiled_line_segment() {
        let index = UiAssetSourceOutlineIndex::from_entries(vec![
            UiAssetSourceOutlineEntry {
                node_id: "root".to_string(),
                block_label: "[nodes.root]".to_string(),
                line: 1,
                end_line: 10,
                excerpt: String::new(),
            },
            UiAssetSourceOutlineEntry {
                node_id: "child".to_string(),
                block_label: "[nodes.child]".to_string(),
                line: 4,
                end_line: 6,
                excerpt: String::new(),
            },
            UiAssetSourceOutlineEntry {
                node_id: "same_start".to_string(),
                block_label: "[nodes.same_start]".to_string(),
                line: 4,
                end_line: 5,
                excerpt: String::new(),
            },
        ]);

        assert_eq!(index.node_id_for_line(3), Some("root"));
        assert_eq!(index.node_id_for_line(4), Some("same_start"));
        assert_eq!(index.node_id_for_line(6), Some("child"));
        assert_eq!(index.node_id_for_line(7), Some("root"));
    }

    #[test]
    fn outline_index_skips_empty_tree_ranges_instead_of_clamping_them_to_a_node_line() {
        let index = UiAssetSourceOutlineIndex::from_entries(vec![UiAssetSourceOutlineEntry {
            node_id: "root".to_string(),
            block_label: "[nodes.root]".to_string(),
            line: 4,
            end_line: 3,
            excerpt: String::new(),
        }]);

        assert_eq!(index.node_id_for_line(3), None);
        assert_eq!(index.node_id_for_line(4), None);
    }

    #[test]
    fn outline_index_exposes_the_sorted_entry_position_for_a_node() {
        let source = "[nodes.root]\nkind = \"container\"\n[nodes.label]\nkind = \"label\"\n";
        let index = build_source_outline_index_for_node_ids(source, ["root", "label"]);

        assert_eq!(index.index_for_node("root"), Some(0));
        assert_eq!(index.index_for_node("label"), Some(1));
        assert_eq!(index.index_for_node("missing"), None);
    }

    #[test]
    fn header_ranges_close_repeated_array_headers_without_rescanning_active_ranges() {
        let source = "[[nodes]]\n[nodes.node]\nnode_id = \"root\"\n[[nodes]]\n[nodes.node]\nnode_id = \"label\"\n";
        let index = build_source_outline_index_for_node_ids(source, ["root", "label"]);

        assert_eq!(
            index.entry_for_node("root").map(|entry| entry.end_line),
            Some(3)
        );
        assert_eq!(
            index.entry_for_node("label").map(|entry| entry.end_line),
            Some(6)
        );
    }
}
