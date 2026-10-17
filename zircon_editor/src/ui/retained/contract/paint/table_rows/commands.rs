//! 按工作台视觉语言决定完整表格接管，正文为空时允许普通回退；文本入口主动跳过已由专用入口绘制的行。
//! 命令消费固定单元格序列，表头、尾行和选中行的偏移需先在同一行框上应用。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::actions::push_table_action;
use super::cells::{push_table_cells, table_cells};
use super::geometry::has_paintable_table_row_extent;
use super::identity::{is_table_row, is_workbench_table_row};
use super::layers::{action_slot_order, cells_order};
use super::surface::{push_table_row_surface, table_paint_rect};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_table_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if push_label_collection_rows(commands, node, rect, clip, order, opacity) {
        return true;
    }
    if !is_workbench_table_row(node) {
        return false;
    }

    let rect = table_paint_rect(node, rect);
    if !has_paintable_table_row_extent(&rect) {
        return true;
    }
    let Some(clip) = intersect(&rect, clip) else {
        return true;
    };
    let cells = table_cells(node);
    if cells.is_empty() {
        return false;
    }
    push_table_row_surface(commands, node, &rect, &clip, order, opacity);
    push_table_cells(
        commands,
        node,
        &rect,
        &clip,
        cells_order(order),
        opacity,
        &cells,
    );
    push_table_action(
        commands,
        node,
        &rect,
        &clip,
        action_slot_order(order),
        opacity,
    );
    true
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_table_row_text_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if has_label_collection_rows(node) || is_workbench_table_row(node) || !is_table_row(node) {
        return false;
    }
    if intersect(rect, clip).is_none() {
        return true;
    }

    let cells = table_cells(node);
    if cells.is_empty() {
        return false;
    }

    push_table_cells(commands, node, rect, clip, order, opacity, &cells);
    true
}

fn has_label_collection_rows(node: &TemplatePaneNodeData) -> bool {
    is_table_row(node)
        && node.collection_rows.row_count() != 0
        && table_cells(node).is_empty()
        && node.collection_rows.iter().any(|row| {
            !row.row_identity_field.is_empty()
                && !row.identity_kind.is_empty()
                && !row.identity_text.is_empty()
        })
}

fn push_label_collection_rows(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !has_label_collection_rows(node) {
        return false;
    }
    let count = node.collection_rows.row_count();
    let height = if node.virtualization_enabled && node.virtualization_item_extent > 0.0 {
        node.virtualization_item_extent
    } else {
        rect.height / count as f32
    };
    if !height.is_finite() || height <= 0.0 || !has_paintable_table_row_extent(rect) {
        return true;
    }
    let metrics = crate::ui::retained_host::host_contract::paint_theme::current_host_metrics();
    let line = metrics.line_height(metrics.font_body);
    for (index, row) in node.collection_rows.iter().enumerate() {
        if row.row_identity_field.is_empty()
            || row.identity_kind.is_empty()
            || row.identity_text.is_empty()
        {
            continue;
        }
        let row_rect = FrameRect {
            x: rect.x,
            y: rect.y + index as f32 * height,
            width: rect.width,
            height,
        };
        let Some(row_clip) = intersect(&row_rect, clip) else {
            continue;
        };
        let mut row_node = node.clone();
        row_node.selected = row.identity_text.as_str() == node.value_text.as_str();
        row_node.checked = false;
        push_table_row_surface(commands, &row_node, &row_rect, &row_clip, order, opacity);
        for (line_index, text) in row.label.lines().enumerate() {
            let text_rect = FrameRect {
                x: row_rect.x + metrics.gap_s,
                y: row_rect.y + metrics.gap_s + line_index as f32 * line,
                width: (row_rect.width - metrics.gap_s * 2.0).max(0.0),
                height: line,
            };
            if text_rect.width <= 0.0 || intersect(&text_rect, &row_clip).is_none() {
                continue;
            }
            commands.push(HostPaintCommand::text(
                text_rect,
                Some(row_clip.clone()),
                cells_order(order),
                text.to_owned(),
                super::style::table_cell_color(&row_node, 0),
                metrics.font_body,
                line,
                zircon_runtime_interface::ui::surface::UiTextRunPaintStyle::default(),
                opacity,
            ));
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::host_contract::{
        data::{HostWindowPresentationData, TemplateNodeFrameData, TemplatePaneCollectionRowData},
        hit_test_workbench_window_template_node_with_index, HostWorkbenchHitIndex,
    };

    #[test]
    fn structured_collection_rows_paint_labels_and_preserve_typed_hit_geometry() {
        for (virtualized, extent, total_height, second_y) in
            [(false, 0.0, 256.0, 148.0), (true, 128.0, 300.0, 148.0)]
        {
            let node = TemplatePaneNodeData {
                role: "Table".into(),
                component_role: "table".into(),
                control_id: "ModulePluginFeatureRows".into(),
                frame: TemplateNodeFrameData {
                    x: 10.0,
                    y: 20.0,
                    width: 220.0,
                    height: total_height,
                },
                collection_rows: model_rc(vec![
                    TemplatePaneCollectionRowData {
                        source_index: 0,
                        row_identity_field: "feature_id".into(),
                        identity_kind: "string".into(),
                        identity_text: "first.feature".into(),
                        label: "First\nDesired: No\nEffective: Unknown\nRequired: No\nBlocked: No"
                            .into(),
                    },
                    TemplatePaneCollectionRowData {
                        source_index: 1,
                        row_identity_field: "feature_id".into(),
                        identity_kind: "string".into(),
                        identity_text: "second.feature".into(),
                        label:
                            "Second\nDesired: Yes\nEffective: Unknown\nRequired: No\nBlocked: No"
                                .into(),
                    },
                ]),
                virtualization_enabled: virtualized,
                virtualization_item_extent: extent,
                value_text: "second.feature".into(),
                ..Default::default()
            };
            let rect = FrameRect {
                x: 10.0,
                y: 20.0,
                width: 220.0,
                height: total_height,
            };
            let mut commands = vec![];
            assert!(push_table_row_commands(
                &mut commands,
                &node,
                &rect,
                &rect,
                0,
                1.0
            ));
            let text: Vec<_> = commands
                .iter()
                .filter(|command| {
                    command
                        .text
                        .as_deref()
                        .map_or(false, |text| !text.is_empty())
                })
                .collect();
            assert_eq!(text.len(), 10);
            assert_eq!(text[0].text.as_deref(), Some("First"));
            assert_eq!(text[5].text.as_deref(), Some("Second"));
            let gap =
                crate::ui::retained_host::host_contract::paint_theme::current_host_metrics().gap_s;
            assert_eq!(text[5].frame.y, second_y + gap);
            let presentation = HostWindowPresentationData {
                workbench_window_nodes: model_rc(vec![node]),
                ..Default::default()
            };
            let index = HostWorkbenchHitIndex::from_presentation(&presentation);
            let hit = hit_test_workbench_window_template_node_with_index(
                &presentation,
                &index,
                24.0,
                second_y + 1.0,
            )
            .unwrap();
            assert_eq!(hit.table_row_source_index, Some(1));
            assert_eq!(hit.table_row_identity_kind.as_str(), "string");
            assert_eq!(hit.table_row_identity_text.as_str(), "second.feature");
        }
        let mut node = TemplatePaneNodeData {
            role: "Table".into(),
            component_role: "table".into(),
            collection_rows: model_rc(vec![TemplatePaneCollectionRowData {
                label: "Decorative".into(),
                ..Default::default()
            }]),
            ..Default::default()
        };
        assert!(!has_label_collection_rows(&node));
        node.collection_rows = model_rc(vec![]);
        assert!(!has_label_collection_rows(&node));
    }
}
