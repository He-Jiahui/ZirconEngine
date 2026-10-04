use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::{current_host_metrics, current_host_palette};
use super::super::super::render_commands::HostPaintCommand;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const GRID_TITLE_INSET: f32 = 8.0;
const GRID_CONTENT_INSET: f32 = 8.0;

pub(super) fn push_data_grid_content(
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

    if !node.text.trim().is_empty() {
        let title_frame = FrameRect {
            x: rect.x + GRID_TITLE_INSET,
            y: rect.y + GRID_TITLE_INSET * 0.5,
            width: (rect.width - GRID_TITLE_INSET * 2.0).max(0.0),
            height: (title_line_height + 2.0).min(rect.height.max(0.0)),
        };
        push_text(
            commands,
            title_frame,
            clip,
            order + 3,
            node.text.to_string(),
            palette.text,
            title_font,
            title_line_height,
            opacity,
        );
    }

    let columns = node
        .options
        .iter()
        .map(|value| value.to_string())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .take(5)
        .collect::<Vec<_>>();
    let column_count = columns.len();
    if column_count > 0 {
        let column_width = (rect.width - GRID_CONTENT_INSET * 2.0).max(0.0) / column_count as f32;
        for (index, column) in columns.into_iter().enumerate() {
            let frame = FrameRect {
                x: rect.x + GRID_CONTENT_INSET + column_width * index as f32,
                y: rect.y + rect.height * 0.17,
                width: (column_width - 4.0).max(0.0),
                height: body_line_height,
            };
            push_text(
                commands,
                frame,
                clip,
                order + 4,
                column,
                palette.text_muted,
                body_font,
                body_line_height,
                opacity,
            );
        }
    }

    let rows = node
        .collection_items
        .iter()
        .filter_map(|value| parse_row(value.as_str()))
        .take(6)
        .collect::<Vec<_>>();
    if rows.is_empty() {
        if !node.value_text.trim().is_empty() {
            let frame = FrameRect {
                x: rect.x + GRID_CONTENT_INSET,
                y: rect.y + rect.height * 0.52,
                width: (rect.width - GRID_CONTENT_INSET * 2.0).max(0.0),
                height: body_line_height * 2.0,
            };
            commands.push(HostPaintCommand::wrapped_text(
                frame,
                Some(clip.clone()),
                order + 5,
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

    let row_height = (rect.height * 0.67 / rows.len() as f32).max(body_line_height + 2.0);
    let content_width = (rect.width - GRID_CONTENT_INSET * 2.0).max(0.0);
    let effective_column_count = column_count.max(1);
    let column_width = content_width / effective_column_count as f32;
    for (row_index, row) in rows.into_iter().enumerate() {
        let selected = row.selected || (node.selected && row_index == 0);
        let y = rect.y + rect.height * 0.26 + row_height * row_index as f32;
        for (cell_index, cell) in row
            .cells
            .into_iter()
            .take(effective_column_count)
            .enumerate()
        {
            let frame = FrameRect {
                x: rect.x + GRID_CONTENT_INSET + column_width * cell_index as f32,
                y: y + 2.0,
                width: (column_width - 4.0).max(0.0),
                height: (row_height - 3.0).max(body_line_height),
            };
            push_text(
                commands,
                frame,
                clip,
                order + 6 + row_index as i32,
                cell,
                if selected && cell_index == 0 {
                    palette.text
                } else {
                    palette.text_muted
                },
                body_font,
                body_line_height,
                opacity,
            );
        }
    }
}

struct DataGridRow {
    selected: bool,
    cells: Vec<String>,
}

fn parse_row(value: &str) -> Option<DataGridRow> {
    let parts = value
        .split('|')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }
    let selected = parts
        .first()
        .is_some_and(|status| status.eq_ignore_ascii_case("selected"));
    let start = if is_row_status(parts[0]) { 1 } else { 0 };
    let cells = parts
        .into_iter()
        .skip(start)
        .map(str::to_string)
        .collect::<Vec<_>>();
    (!cells.is_empty()).then_some(DataGridRow { selected, cells })
}

fn is_row_status(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "selected" | "normal" | "success" | "warning" | "error" | "pending"
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
