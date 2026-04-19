use crate::ui::UiDesignerSelectionModel;
use zircon_runtime::ui::template::UiAssetDocument;

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

pub(crate) fn build_source_selection_summary(
    _document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    source: &str,
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
    let (line, excerpt) = source_outline_entry_for_node(source, node_id)
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

pub(crate) fn source_outline_entry_for_node(
    source: &str,
    node_id: &str,
) -> Option<UiAssetSourceOutlineEntry> {
    let lines = source.lines().collect::<Vec<_>>();
    source_outline_entry(node_id, &lines)
}

pub(crate) fn build_source_outline(
    document: &UiAssetDocument,
    source: &str,
) -> Vec<UiAssetSourceOutlineEntry> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut entries = document
        .nodes
        .keys()
        .filter_map(|node_id| source_outline_entry(node_id, &lines))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then_with(|| left.block_label.cmp(&right.block_label))
    });
    entries
}

pub(crate) fn source_outline_node_id_for_line(
    document: &UiAssetDocument,
    source: &str,
    line: usize,
) -> Option<String> {
    let line = line as i32;
    build_source_outline(document, source)
        .into_iter()
        .find(|entry| line >= entry.line && line <= entry.end_line)
        .map(|entry| entry.node_id)
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

fn source_outline_entry(node_id: &str, lines: &[&str]) -> Option<UiAssetSourceOutlineEntry> {
    let block_label = format!("[nodes.{node_id}]");
    let (line, end_line) = find_block_line_range(&block_label, lines)?;
    Some(UiAssetSourceOutlineEntry {
        node_id: node_id.to_string(),
        block_label,
        line,
        end_line,
        excerpt: capture_block_excerpt(lines, (line - 1) as usize),
    })
}

fn find_block_line_range(block_label: &str, lines: &[&str]) -> Option<(i32, i32)> {
    let start_index = lines.iter().position(|line| line.trim() == block_label)?;
    let end_index = lines
        .iter()
        .enumerate()
        .skip(start_index + 1)
        .find(|(_, line)| line.trim_start().starts_with('['))
        .map(|(index, _)| index.saturating_sub(1))
        .unwrap_or_else(|| lines.len().saturating_sub(1));
    Some((start_index as i32 + 1, end_index as i32 + 1))
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
