use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

use super::super::super::data::{
    visit_paint_workbench_rows, FrameRect, HostPaneInteractionStateData, HostTextInputFocusData,
    TemplatePaneNodeData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::render_commands::{draw_host_paint_commands, HostPaintCommand};
use super::super::template_nodes::{push_template_node_commands, template_node_intersects_clip};
use super::clip::effective_template_clip;
use super::hover::{apply_template_hover_to_node, template_hover_targets_node};
use super::transform::TemplateNodePaintTransform;

fn push_untransformed_template_node_commands(
    commands: &mut Vec<HostPaintCommand>,
    source_node: &TemplatePaneNodeData,
    origin: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
    row: i32,
    interaction: Option<&HostPaneInteractionStateData>,
) -> bool {
    if let Some(interaction) =
        interaction.filter(|interaction| template_hover_targets_node(source_node, interaction))
    {
        let mut node = source_node.clone();
        apply_template_hover_to_node(&mut node, interaction);
        push_template_node_commands(commands, &node, origin, clip, text_input_focus, row);
        true
    } else {
        push_template_node_commands(commands, source_node, origin, clip, text_input_focus, row);
        false
    }
}

pub(in crate::ui::retained_host::host_contract) fn draw_template_nodes(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> bool {
    draw_template_nodes_with_transform(frame, nodes, origin, clip, text_input_focus, None)
}

pub(in crate::ui::retained_host::host_contract) fn draw_template_nodes_with_transform(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
    transform: Option<&dyn TemplateNodePaintTransform>,
) -> bool {
    let Some(effective_clip) = effective_template_clip(frame, clip) else {
        return false;
    };

    let mut commands: Vec<HostPaintCommand> = Vec::new();
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "template_nodes_collect_commands");
        zircon_runtime::profile_counter!("editor", "template_node_count", nodes.row_count());
        let mut visited = 0_usize;
        let mut cloned = 0_usize;
        let mut damage_rejected = 0_usize;
        let mut collect_row = |row: usize, commands: &mut Vec<HostPaintCommand>| {
            visited = visited.saturating_add(1);
            let Some(source_node) = nodes.get(row) else {
                return;
            };
            if transform.is_none() {
                if !template_node_intersects_clip(source_node, origin, &effective_clip) {
                    damage_rejected = damage_rejected.saturating_add(1);
                    return;
                }
                if push_untransformed_template_node_commands(
                    commands,
                    source_node,
                    origin,
                    &effective_clip,
                    text_input_focus,
                    row as i32,
                    frame.pane_interaction_state(),
                ) {
                    cloned = cloned.saturating_add(1);
                }
                return;
            }
            let source_node = source_node.clone();
            cloned = cloned.saturating_add(1);
            let Some(transform) = transform else {
                return;
            };
            let Some((mut node, node_clip)) =
                transform.transform_row(row, source_node, effective_clip.clone())
            else {
                return;
            };
            if !template_node_intersects_clip(&node, origin, &node_clip) {
                damage_rejected = damage_rejected.saturating_add(1);
                return;
            }
            if let Some(interaction) = frame.pane_interaction_state() {
                apply_template_hover_to_node(&mut node, interaction);
            }
            // Region repaint must avoid generating commands for off-damage nodes:
            // image commands can rasterize previews before the final primitive clip runs.
            push_template_node_commands(
                commands,
                &node,
                origin,
                &node_clip,
                text_input_focus,
                row as i32,
            );
        };
        let streamed_rows = transform.is_some_and(|transform| {
            transform.stream_row_visit_indices(nodes.row_count(), &effective_clip, &mut |row| {
                collect_row(row, &mut commands)
            })
        });
        let streamed_index_rows = !streamed_rows
            && transform.is_none()
            && visit_paint_workbench_rows(nodes, origin, &effective_clip, &mut |row| {
                collect_row(row, &mut commands)
            });
        if !streamed_rows && !streamed_index_rows {
            let visit_rows = transform.and_then(|transform| {
                transform.row_visit_indices(nodes.row_count(), &effective_clip)
            });
            match visit_rows {
                Some(rows) => {
                    for row in rows {
                        collect_row(row, &mut commands);
                    }
                }
                None => {
                    for row in 0..nodes.row_count() {
                        collect_row(row, &mut commands);
                    }
                }
            }
        }
        record_current_ui_perf_counter(UiPerfCounter::TemplateNodeVisitCount, visited as f64);
        record_current_ui_perf_counter(UiPerfCounter::TemplateNodeCloneCount, cloned as f64);
        record_current_ui_perf_counter(
            UiPerfCounter::TemplateNodeDamageRejectCount,
            damage_rejected as f64,
        );
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "template_nodes_draw_commands");
        zircon_runtime::profile_counter!("editor", "template_command_count", commands.len());
        draw_host_paint_commands(frame, &commands)
    }
}

pub(in crate::ui::retained_host::host_contract) fn has_template_nodes(
    nodes: &ModelRc<TemplatePaneNodeData>,
) -> bool {
    nodes.row_count() > 0
}
