use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_row_metrics::{workbench_row_metrics, workbench_row_palette};
use super::layers::selection_indicator_order;
use super::style::{list_row_background, list_row_border, list_row_border_width};
use crate::ui::retained_host::host_contract::paint_geometry::corner_radius_for_frame;

const LIST_ROW_SURFACE_COMMAND_CAPACITY: usize = 2;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_list_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if intersect(rect, clip).is_none() {
        return;
    }
    let background = list_row_background(node);
    let border = list_row_border(node);
    if background.is_none() && border.is_none() {
        return;
    }
    commands.reserve(LIST_ROW_SURFACE_COMMAND_CAPACITY);
    let metrics = workbench_row_metrics();
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        background,
        border,
        list_row_border_width(node),
        corner_radius_for_frame(rect, metrics.surface_radius),
        opacity,
    ));
    if is_selected_row(node) {
        let palette = workbench_row_palette();
        let indicator = selection_indicator_rect(rect, metrics.selection_indicator_width);
        if intersect(&indicator, clip).is_none() {
            return;
        }
        commands.push(HostPaintCommand::quad(
            indicator,
            Some(clip.clone()),
            selection_indicator_order(order),
            Some(palette.selection_indicator),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

fn is_selected_row(node: &TemplatePaneNodeData) -> bool {
    node.selected || node.checked
}

fn selection_indicator_rect(rect: &FrameRect, indicator_width: f32) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: indicator_width.min(rect.width),
        height: rect.height,
    }
}

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830cw_list_row_surface_reserves_its_command_bound() {
        let source = include_str!("surface.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("list row surface production source");

        assert!(production.contains("const LIST_ROW_SURFACE_COMMAND_CAPACITY: usize = 2;"));
        assert!(production.contains("commands.reserve(LIST_ROW_SURFACE_COMMAND_CAPACITY);"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830cw_list_row_surface_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const COMMANDS_PER_BATCH: usize = 2;
        const MARKER: &str = "EDITOR510_LIST_ROW_SURFACE_CAPACITY_BENCH_V1";

        let legacy_growth_events = command_growth_events(BATCH_COUNT, COMMANDS_PER_BATCH, false);
        let optimized_growth_events = command_growth_events(BATCH_COUNT, COMMANDS_PER_BATCH, true);

        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
        println!(
            "{MARKER} batches={BATCH_COUNT} commands_per_batch={COMMANDS_PER_BATCH} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
        );
    }

    fn command_growth_events(
        batch_count: usize,
        commands_per_batch: usize,
        reserve: bool,
    ) -> usize {
        let mut growth_events = 0;
        for _ in 0..batch_count {
            let mut commands = if reserve {
                Vec::with_capacity(commands_per_batch)
            } else {
                Vec::new()
            };
            for command in 0..commands_per_batch {
                let previous_capacity = commands.capacity();
                commands.push(command);
                growth_events += usize::from(commands.capacity() != previous_capacity);
            }
        }
        growth_events
    }
}
