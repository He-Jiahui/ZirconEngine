use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::{current_host_metrics, current_host_palette};
use super::super::super::render_commands::HostPaintCommand;
use super::rows::tree_view_header_height;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const TREE_TITLE_INSET: f32 = 8.0;
const TREE_CONTENT_INSET: f32 = 8.0;

pub(super) fn push_tree_view_content(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let metrics = current_host_metrics();
    let palette = current_host_palette();
    let title_font = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        metrics.font_body
    };
    let body_font = metrics.font_small.max(9.0);
    let title_line_height = metrics.line_height(title_font).max(title_font);
    let body_line_height = metrics.line_height(body_font).max(body_font);

    let header_height = tree_view_header_height(rect, node);
    if !node.text.trim().is_empty() {
        push_text(
            commands,
            FrameRect {
                x: rect.x + TREE_TITLE_INSET,
                y: rect.y + TREE_TITLE_INSET * 0.5,
                width: (rect.width - TREE_TITLE_INSET * 2.0).max(0.0),
                height: (header_height - TREE_TITLE_INSET * 0.5).max(title_line_height),
            },
            clip,
            order + 3,
            node.text.to_string(),
            palette.text,
            title_font,
            title_line_height,
            opacity,
        );
    }

    let items = node
        .collection_items
        .iter()
        .filter_map(|value| parse_item(value.as_str()))
        .take(8)
        .collect::<Vec<_>>();
    if items.is_empty() {
        if !node.value_text.trim().is_empty() {
            commands.push(HostPaintCommand::wrapped_text(
                FrameRect {
                    x: rect.x + TREE_CONTENT_INSET,
                    y: rect.y + header_height + rect.height * 0.35,
                    width: (rect.width - TREE_CONTENT_INSET * 2.0).max(0.0),
                    height: body_line_height * 2.0,
                },
                Some(clip.clone()),
                order + 4,
                node.value_text.to_string(),
                palette.text_muted,
                body_font,
                body_line_height,
                UiTextRunPaintStyle::default(),
                opacity,
            ));
        }
        return;
    }

    let body_height = (rect.height - header_height).max(0.0);
    let row_height = (body_height * 0.82 / items.len() as f32).max(body_line_height + 2.0);
    for (index, item) in items.into_iter().enumerate() {
        let selected = item.selected || (node.selected && index == 0);
        let expanded = item.expanded || (node.expanded && index == 0);
        let y = rect.y + header_height + body_height * 0.08 + row_height * index as f32;
        let marker = if expanded {
            '▾'
        } else if selected {
            '●'
        } else {
            '○'
        };
        let indent = item.depth.clamp(0, 3) as f32 * 10.0;
        push_text(
            commands,
            FrameRect {
                x: rect.x + TREE_CONTENT_INSET + indent,
                y: y + 2.0,
                width: (rect.width - TREE_CONTENT_INSET * 2.0 - indent).max(0.0),
                height: (row_height - 3.0).max(body_line_height),
            },
            clip,
            order + 5 + index as i32,
            format!("{marker}  {}", item.label),
            if selected {
                palette.text
            } else if expanded {
                palette.info
            } else {
                palette.text_muted
            },
            body_font,
            body_line_height,
            opacity,
        );
    }
}

struct TreeItem {
    depth: i32,
    label: String,
    selected: bool,
    expanded: bool,
}

fn parse_item(value: &str) -> Option<TreeItem> {
    let parts = value
        .split('|')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }
    let mut index = 0;
    let status = if is_status(parts[0]) {
        index += 1;
        parts[0].to_ascii_lowercase()
    } else {
        "normal".to_string()
    };
    let depth = parts
        .get(index)
        .and_then(|value| value.parse::<i32>().ok())
        .map(|value| {
            index += 1;
            value
        })
        .unwrap_or(0);
    let label = parts.get(index..)?.join(" · ");
    (!label.is_empty()).then_some(TreeItem {
        selected: status == "selected",
        expanded: matches!(status.as_str(), "expanded" | "open"),
        depth,
        label,
    })
}

fn is_status(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "selected" | "expanded" | "open" | "normal" | "leaf"
    )
}

#[allow(clippy::too_many_arguments)]
fn push_text(
    commands: &mut Vec<HostPaintCommand>,
    frame: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: String,
    color: [u8; 4],
    font_size: f32,
    line_height: f32,
    opacity: f32,
) {
    if frame.width <= 0.0 || frame.height <= 0.0 || text.trim().is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        frame,
        Some(clip.clone()),
        order,
        text,
        color,
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

#[cfg(test)]
#[path = "tests/content.rs"]
mod tests;
