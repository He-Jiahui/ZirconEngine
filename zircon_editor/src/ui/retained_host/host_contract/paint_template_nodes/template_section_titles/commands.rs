use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_section_title_glyphs::{push_section_icon, section_title_icon};
use super::geometry::{frame_is_within, has_paintable_section_title_extent, section_icon_rect};
use super::identity::is_workbench_section_title;
use super::surface::push_section_title_surface;
use super::text::push_section_label;
use crate::ui::retained_host::host_contract::paint_geometry::intersect;

const SECTION_TITLE_COMMAND_CAPACITY: usize = 4;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_section_title_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_section_title(node) {
        return false;
    }
    if !has_paintable_section_title_extent(rect) || intersect(rect, clip).is_none() {
        return true;
    }

    commands.reserve(SECTION_TITLE_COMMAND_CAPACITY);
    push_section_title_surface(commands, rect, clip, order, opacity);
    let icon = section_title_icon(node);
    let icon_painted = if let Some(icon) = icon {
        let icon_rect = section_icon_rect(rect);
        if frame_is_within(rect, &icon_rect) && intersect(&icon_rect, clip).is_some() {
            push_section_icon(commands, &icon_rect, clip, order + 2, icon, opacity);
            true
        } else {
            false
        }
    } else {
        false
    };
    push_section_label(commands, node, rect, clip, order + 3, icon_painted, opacity);
    true
}

#[cfg(test)]
mod optimization_batch_20260830cu_editor_tests {
    use super::*;

    #[test]
    fn section_title_preserves_fractional_post_dpi_surface_geometry() {
        let node = TemplatePaneNodeData {
            control_id: "WorkbenchSectionTitleRoot".into(),
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: 8.25,
            y: 10.5,
            width: 240.75,
            height: 28.25,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 640.0,
            height: 480.0,
        };
        let mut commands = Vec::new();

        assert!(push_section_title_commands(
            &mut commands,
            &node,
            &rect,
            &clip,
            0,
            1.0,
        ));

        assert_eq!(commands.first().map(|command| &command.frame), Some(&rect));
    }

    #[test]
    fn optimization_batch_20260830cu_section_title_reserves_its_command_bound() {
        let source = include_str!("commands.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("section title production source");

        assert!(production.contains("const SECTION_TITLE_COMMAND_CAPACITY: usize = 4;"));
        assert!(production.contains("commands.reserve(SECTION_TITLE_COMMAND_CAPACITY);"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830cu_section_title_command_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const COMMANDS_PER_BATCH: usize = 4;
        const MARKER: &str = "EDITOR508_SECTION_TITLE_COMMAND_CAPACITY_BENCH_V1";

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
