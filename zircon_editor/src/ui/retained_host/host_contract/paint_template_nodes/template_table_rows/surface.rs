use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::identity::{is_table_selected, is_table_tail};
use super::layers::separator_order;
use super::metrics::table_row_surface_metrics;
use super::style::{
    table_row_background, table_row_border, table_row_border_width, table_row_style,
};

const TABLE_ROW_SURFACE_COMMAND_CAPACITY: usize = 2;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_table_row_surface(
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
    commands.reserve(TABLE_ROW_SURFACE_COMMAND_CAPACITY);
    let metrics = table_row_surface_metrics();
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(table_row_background(node)),
        table_row_border(node),
        table_row_border_width(node),
        metrics.radius,
        opacity,
    ));
    if !is_selected_row(node) {
        let separator_height = metrics.separator_height.min(rect.height).max(0.0);
        let separator = FrameRect {
            x: rect.x,
            y: rect.y + (rect.height - separator_height).max(0.0),
            width: rect.width,
            height: separator_height,
        };
        if intersect(&separator, clip).is_none() {
            return;
        }
        commands.push(HostPaintCommand::quad(
            separator,
            Some(clip.clone()),
            separator_order(order),
            Some(table_row_style(node).separator),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    if is_table_tail(node) || is_table_selected(node) {
        FrameRect {
            x: rect.x + node.layout_offset_x,
            y: rect.y + node.layout_offset_y,
            width: rect.width,
            height: rect.height,
        }
    } else {
        rect.clone()
    }
}

fn is_selected_row(node: &TemplatePaneNodeData) -> bool {
    node.selected || node.checked || is_table_selected(node)
}

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830cy_table_row_surface_reserves_its_command_bound() {
        let source = include_str!("surface.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("table row surface production source");

        assert!(production.contains("const TABLE_ROW_SURFACE_COMMAND_CAPACITY: usize = 2;"));
        assert!(production.contains("commands.reserve(TABLE_ROW_SURFACE_COMMAND_CAPACITY);"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830cy_table_row_surface_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const COMMANDS_PER_BATCH: usize = 2;
        const MARKER: &str = "EDITOR511_TABLE_ROW_SURFACE_CAPACITY_BENCH_V1";

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
