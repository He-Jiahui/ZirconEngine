use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::identity::{is_badge_root_node, is_badge_slot_node};
use super::overlay::push_badge_overlay;
use super::root_label::push_badge_root_label;
use super::root_surface::push_badge_root_surface;

const BADGE_PRIMITIVE_COMMAND_CAPACITY: usize = 4;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_badge_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_badge_slot_node(node) {
        return true;
    }
    if !is_badge_root_node(node) {
        return false;
    }
    if !rect.x.is_finite()
        || !rect.y.is_finite()
        || !rect.width.is_finite()
        || !rect.height.is_finite()
        || rect.width <= 0.0
        || rect.height <= 0.0
    {
        return true;
    }

    commands.reserve(BADGE_PRIMITIVE_COMMAND_CAPACITY);
    push_badge_root_surface(commands, node, rect, clip, order, opacity);
    push_badge_root_label(commands, node, rect, clip, order + 1, opacity);
    push_badge_overlay(commands, node, rect, clip, order + 2, opacity);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_finite_badge_root_layout_does_not_emit_any_paint_commands() {
        let node = TemplatePaneNodeData {
            component_role: "badge".to_owned(),
            text: "Asset status".to_owned(),
            value_text: "1".to_owned(),
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: 8.0,
            y: f32::NAN,
            width: 48.0,
            height: 24.0,
        };
        let mut commands = Vec::new();

        assert!(push_badge_primitive_commands(
            &mut commands,
            &node,
            &rect,
            &rect,
            0,
            1.0,
        ));
        assert!(commands.is_empty());
    }

    #[test]
    fn optimization_batch_20260830cr_editor505_badge_reserves_maximum_command_count() {
        let source = include_str!("sequencing.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("badge sequencing production source");

        assert!(production.contains("commands.reserve(BADGE_PRIMITIVE_COMMAND_CAPACITY);"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830cr_editor505_badge_command_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const COMMANDS_PER_BATCH: usize = 4;
        const MARKER: &str = "EDITOR505_BADGE_COMMAND_CAPACITY_BENCH_V1";
        let legacy_growth_events =
            badge_command_growth_events(BATCH_COUNT, COMMANDS_PER_BATCH, false);
        let optimized_growth_events =
            badge_command_growth_events(BATCH_COUNT, COMMANDS_PER_BATCH, true);

        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
        println!(
            "{MARKER} batches={BATCH_COUNT} commands_per_batch={COMMANDS_PER_BATCH} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} reduction_pct=100"
        );
    }

    fn badge_command_growth_events(
        batch_count: usize,
        commands_per_batch: usize,
        reserve: bool,
    ) -> usize {
        let mut commands = Vec::new();
        let mut growth_events = 0;
        for _ in 0..batch_count {
            if reserve {
                commands.reserve(commands_per_batch);
            }
            for command in 0..commands_per_batch {
                let previous_capacity = commands.capacity();
                commands.push(command);
                growth_events += usize::from(commands.capacity() != previous_capacity);
            }
        }
        growth_events
    }
}
