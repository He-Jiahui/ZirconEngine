use std::ops::Range;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::is_command_palette;
use super::layers::{empty_message_order, row_order};
use super::layout::{command_palette_metrics, min_frame_extent, paint_rect, row_rect};
use super::panel::{push_command_palette_empty_message, push_command_palette_panel_commands};
use super::rows::push_command_row_commands;

const COMMAND_PALETTE_PANEL_COMMAND_UPPER_BOUND: usize = 4;
const COMMAND_PALETTE_EMPTY_COMMAND_UPPER_BOUND: usize = 1;
const COMMAND_PALETTE_ROW_COMMAND_UPPER_BOUND: usize = 4;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_command_palette_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_command_palette(node) {
        return false;
    }
    if !node.popup_open {
        return true;
    }

    let rect = paint_rect(rect);
    let min_frame_extent = min_frame_extent();
    if rect.width <= min_frame_extent || rect.height <= min_frame_extent {
        return true;
    }

    let row_count = node.structured_options.row_count();
    let visible_rows = command_palette_visible_rows(&rect, clip, row_count);
    let body_command_upper_bound = if row_count == 0 {
        COMMAND_PALETTE_EMPTY_COMMAND_UPPER_BOUND
    } else {
        visible_rows
            .len()
            .saturating_mul(COMMAND_PALETTE_ROW_COMMAND_UPPER_BOUND)
    };
    let command_upper_bound =
        COMMAND_PALETTE_PANEL_COMMAND_UPPER_BOUND.saturating_add(body_command_upper_bound);
    commands.reserve(command_upper_bound);

    push_command_palette_panel_commands(commands, node, &rect, clip, order, opacity);

    if row_count == 0 {
        push_command_palette_empty_message(
            commands,
            &rect,
            clip,
            empty_message_order(order),
            opacity,
        );
        return true;
    }

    for row in visible_rows {
        let Some(option) = node.structured_options.get(row) else {
            continue;
        };
        push_command_row_commands(
            commands,
            option,
            &row_rect(&rect, row),
            clip,
            row_order(order, row),
            opacity,
        );
    }

    true
}

const COMMAND_PALETTE_PAINT_OVERSCAN_ROWS: usize = 1;

fn command_palette_visible_rows(
    panel: &FrameRect,
    clip: &FrameRect,
    row_count: usize,
) -> Range<usize> {
    if row_count == 0
        || panel.width <= 0.0
        || panel.height <= 0.0
        || clip.width <= 0.0
        || clip.height <= 0.0
        || clip.x >= panel.x + panel.width
        || clip.x + clip.width <= panel.x
    {
        return 0..0;
    }

    let metrics = command_palette_metrics();
    let list_top = panel.y + metrics.list_top;
    let list_bottom =
        (list_top + row_count as f32 * metrics.row_height).min(panel.y + panel.height);
    let visible_top = clip.y.max(list_top);
    let visible_bottom = (clip.y + clip.height).min(list_bottom);
    if visible_bottom <= visible_top {
        return 0..0;
    }

    let first_visible = ((visible_top - list_top) / metrics.row_height)
        .floor()
        .max(0.0) as usize;
    let visible_end = ((visible_bottom - list_top) / metrics.row_height)
        .ceil()
        .max(0.0) as usize;
    let visible_end = visible_end.min(row_count);
    if first_visible >= visible_end {
        return 0..0;
    }

    first_visible.saturating_sub(COMMAND_PALETTE_PAINT_OVERSCAN_ROWS)
        ..visible_end
            .saturating_add(COMMAND_PALETTE_PAINT_OVERSCAN_ROWS)
            .min(row_count)
}

#[cfg(test)]
mod tests {
    use super::super::layout::command_palette_metrics;
    use super::*;

    #[test]
    fn visible_rows_include_exactly_one_overscan_row_on_each_side() {
        let metrics = command_palette_metrics();
        let panel = FrameRect {
            x: 100.0,
            y: 50.0,
            width: 560.0,
            height: metrics.list_top + metrics.row_height * 40.0,
        };
        let clip = FrameRect {
            x: panel.x,
            y: panel.y + metrics.list_top + metrics.row_height * 10.25,
            width: panel.width,
            height: metrics.row_height * 2.5,
        };

        assert_eq!(command_palette_visible_rows(&panel, &clip, 40), 9..14);
    }

    #[test]
    fn visible_rows_are_empty_when_clip_is_horizontally_disjoint() {
        let panel = FrameRect {
            x: 100.0,
            y: 50.0,
            width: 560.0,
            height: 280.0,
        };
        let clip = FrameRect {
            x: panel.x + panel.width + 1.0,
            y: panel.y,
            width: 20.0,
            height: panel.height,
        };

        assert_eq!(command_palette_visible_rows(&panel, &clip, 40), 0..0);
    }

    #[test]
    fn painter_source_has_no_full_row_loop_or_cloning_row_access() {
        let source = include_str!("commands.rs");
        let full_loop = ["for row in ", "0..row_count"].concat();
        let cloning_access = ["structured_options", ".row_data(row)"].concat();
        let borrowed_access = ["structured_options", ".get(row)"].concat();

        assert!(!source.contains(&full_loop));
        assert!(!source.contains(&cloning_access));
        assert!(source.contains(&borrowed_access));
    }

    #[test]
    fn optimization_batch_20260830dg_command_palette_reserves_visible_command_upper_bound() {
        let source = include_str!("commands.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("command palette production source");

        assert!(production.contains("COMMAND_PALETTE_PANEL_COMMAND_UPPER_BOUND"));
        assert!(production.contains("COMMAND_PALETTE_ROW_COMMAND_UPPER_BOUND"));
        assert!(production.contains("commands.reserve(command_upper_bound)"));
        assert_eq!(
            production.matches("command_palette_visible_rows(").count(),
            2
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830dg_command_palette_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const PANEL_COMMAND_COUNT: usize = 4;
        const VISIBLE_ROW_COUNT: usize = 8;
        const ROW_COMMAND_COUNT: usize = 4;
        const COMMAND_COUNT: usize = PANEL_COMMAND_COUNT + VISIBLE_ROW_COUNT * ROW_COMMAND_COUNT;
        const MARKER: &str = "EDITOR519_COMMAND_PALETTE_CAPACITY_BENCH_V1";

        let legacy_growth_events = command_growth_events(BATCH_COUNT, COMMAND_COUNT, false);
        let optimized_growth_events = command_growth_events(BATCH_COUNT, COMMAND_COUNT, true);

        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
        println!(
            "{MARKER} batches={BATCH_COUNT} panel_commands={PANEL_COMMAND_COUNT} \
             visible_rows={VISIBLE_ROW_COUNT} row_commands={ROW_COMMAND_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
        );
    }

    fn command_growth_events(batch_count: usize, command_count: usize, reserve: bool) -> usize {
        let mut growth_events = 0;
        for _ in 0..batch_count {
            let mut commands = Vec::new();
            if reserve {
                commands.reserve(command_count);
            }
            for command in 0..command_count {
                let previous_capacity = commands.capacity();
                commands.push(command);
                growth_events += usize::from(commands.capacity() != previous_capacity);
            }
        }
        growth_events
    }
}
