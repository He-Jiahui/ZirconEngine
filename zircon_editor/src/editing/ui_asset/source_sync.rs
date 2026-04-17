use zircon_editor_ui::UiDesignerSelectionModel;
use zircon_ui::UiAssetDocument;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct UiAssetSourceSelectionSummary {
    pub block_label: String,
    pub line: i32,
    pub excerpt: String,
    pub roundtrip_status: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct UiAssetSourceOutlineEntry {
    pub node_id: String,
    pub block_label: String,
    pub line: i32,
    pub excerpt: String,
}

pub(super) fn build_source_selection_summary(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    source: &str,
    diagnostics: &[String],
) -> UiAssetSourceSelectionSummary {
    let Some(node_id) = selection.primary_node_id.as_deref() else {
        return UiAssetSourceSelectionSummary {
            line: -1,
            roundtrip_status: invalid_prefix(diagnostics, "no node selected"),
            ..Default::default()
        };
    };
    let outline_entries = build_source_outline(document, source);
    let block_label = format!("[nodes.{node_id}]");
    let (line, excerpt) = outline_entries
        .iter()
        .find(|entry| entry.node_id == node_id)
        .map(|entry| (entry.line, entry.excerpt.clone()))
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

pub(super) fn build_source_outline(
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

fn source_outline_entry(node_id: &str, lines: &[&str]) -> Option<UiAssetSourceOutlineEntry> {
    let block_label = format!("[nodes.{node_id}]");
    let line = lines
        .iter()
        .position(|line| line.trim() == block_label)
        .map(|index| index as i32 + 1)?;
    Some(UiAssetSourceOutlineEntry {
        node_id: node_id.to_string(),
        block_label,
        line,
        excerpt: capture_block_excerpt(lines, (line - 1) as usize),
    })
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

fn invalid_prefix(diagnostics: &[String], message: &str) -> String {
    if diagnostics.is_empty() {
        message.to_string()
    } else {
        format!("source invalid, preview uses last valid snapshot; {message}")
    }
}
