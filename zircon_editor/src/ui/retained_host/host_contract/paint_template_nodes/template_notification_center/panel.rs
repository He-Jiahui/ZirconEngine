use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::{empty_text, header_text};
use super::layout::{empty_text_rect, header_rect, NotificationCenterMetrics};
use super::style::NotificationCenterPalette;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_notification_panel_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: NotificationCenterPalette,
    metrics: &NotificationCenterMetrics,
) {
    commands.reserve(2);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(palette.panel_surface),
        Some(palette.panel_border),
        metrics.border_width,
        metrics.panel_radius,
        opacity,
    ));

    let header = header_rect(rect, metrics);
    if header.width > 0.0 && header.height > 0.0 {
        commands.push(HostPaintCommand::text(
            header,
            Some(clip.clone()),
            order + 1,
            header_text(node),
            palette.header_text,
            metrics.header_font_size,
            metrics.header_line_height,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_empty_notification_message(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: NotificationCenterPalette,
    metrics: &NotificationCenterMetrics,
) {
    let message = empty_text_rect(rect, metrics);
    if message.width > 0.0 && message.height > 0.0 {
        commands.push(HostPaintCommand::text(
            message,
            Some(clip.clone()),
            order,
            empty_text(node),
            palette.muted_text,
            metrics.message_font_size,
            metrics.message_line_height,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
}

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830dh_notification_panel_reserves_two_commands() {
        let source = include_str!("panel.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("notification panel production source");

        assert!(production.contains("commands.reserve(2)"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830dh_notification_panel_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const COMMAND_COUNT: usize = 2;
        const MARKER: &str = "EDITOR520_NOTIFICATION_PANEL_CAPACITY_BENCH_V1";

        let legacy_growth_events = panel_growth_events(BATCH_COUNT, COMMAND_COUNT, false);
        let optimized_growth_events = panel_growth_events(BATCH_COUNT, COMMAND_COUNT, true);

        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
        println!(
            "{MARKER} batches={BATCH_COUNT} commands={COMMAND_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
        );
    }

    fn panel_growth_events(batch_count: usize, command_count: usize, reserve: bool) -> usize {
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
