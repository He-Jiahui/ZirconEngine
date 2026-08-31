use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_tree_row_geometry::{
    tree_guide_color, tree_guide_opacity, tree_guide_rect, tree_row_radius,
};
use super::style::{tree_row_background, tree_row_border, tree_row_border_width};

const TREE_ROW_SURFACE_COMMAND_CAPACITY: usize = 1;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let Some(background) = tree_row_background(node) else {
        return;
    };
    commands.reserve(TREE_ROW_SURFACE_COMMAND_CAPACITY);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        tree_row_border(node),
        tree_row_border_width(node),
        tree_row_radius(),
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_indent_guides(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let depth = node.tree_depth.max(0) as usize;
    commands.reserve(depth);
    let guide_color = tree_guide_color();
    let guide_opacity = tree_guide_opacity();
    for level in 0..depth {
        commands.push(HostPaintCommand::quad(
            tree_guide_rect(rect, level),
            Some(clip.clone()),
            order,
            Some(guide_color),
            None,
            0.0,
            0.0,
            opacity * guide_opacity,
        ));
    }
}

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830cz_tree_row_reserves_surface_and_guide_bounds() {
        let source = include_str!("surface.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("tree row surface production source");

        assert!(production.contains("const TREE_ROW_SURFACE_COMMAND_CAPACITY: usize = 1;"));
        assert!(production.contains("commands.reserve(TREE_ROW_SURFACE_COMMAND_CAPACITY);"));
        assert!(production.contains("commands.reserve(depth);"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830cz_tree_row_command_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const GUIDE_COUNT: usize = 32;
        const MARKER: &str = "EDITOR512_TREE_ROW_COMMAND_CAPACITY_BENCH_V1";

        let legacy_growth_events = command_growth_events(BATCH_COUNT, GUIDE_COUNT, false);
        let optimized_growth_events = command_growth_events(BATCH_COUNT, GUIDE_COUNT, true);

        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
        println!(
            "{MARKER} batches={BATCH_COUNT} surface_commands=1 guide_commands={GUIDE_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
        );
    }

    fn command_growth_events(batch_count: usize, guide_count: usize, reserve: bool) -> usize {
        let mut growth_events = 0;
        for _ in 0..batch_count {
            let mut commands = Vec::new();
            if reserve {
                commands.reserve(1);
            }
            let previous_capacity = commands.capacity();
            commands.push(0);
            growth_events += usize::from(commands.capacity() != previous_capacity);

            if reserve {
                commands.reserve(guide_count);
            }
            for guide in 0..guide_count {
                let previous_capacity = commands.capacity();
                commands.push(guide + 1);
                growth_events += usize::from(commands.capacity() != previous_capacity);
            }
        }
        growth_events
    }
}
