use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::{METRICS, PALETTE};
use super::super::super::template_nodes::{
    paint_template_nodes_for_test, push_template_node_commands,
};
use super::super::metrics::{linear_progress_radius, material_feedback_metrics_from_host};
use super::super::palette::material_feedback_palette_from_host;
use super::super::push_material_feedback_primitive_commands;
use super::support::{pixel_at, positioned_backdrop_node, positioned_progress_node};
use crate::ui::layouts::common::model_rc;

#[test]
fn workbench_progress_defaults_to_shared_track_and_accent_fill() {
    let palette = material_feedback_palette_from_host(PALETTE);
    let bytes = paint_template_nodes_for_test(
        220,
        48,
        model_rc(vec![positioned_progress_node(
            "WorkbenchFeedbackProgress",
            0.64,
            8.0,
            16.0,
            184.0,
            12.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 220, 48, 22), palette.accent);
    assert_eq!(pixel_at(&bytes, 220, 150, 22), palette.track);
}

#[test]
fn generic_material_progress_keeps_accent_fallback() {
    let palette = material_feedback_palette_from_host(PALETTE);
    let bytes = paint_template_nodes_for_test(
        220,
        48,
        model_rc(vec![positioned_progress_node(
            "MaterialProgress",
            0.64,
            8.0,
            16.0,
            184.0,
            12.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 220, 48, 22), palette.accent);
}

#[test]
fn material_feedback_palette_projects_from_host_palette() {
    let mut host = PALETTE;
    host.track = [9, 10, 11, 255];
    host.surface_disabled = [12, 13, 14, 255];
    host.text_disabled = [80, 81, 82, 255];
    host.accent = [90, 91, 92, 255];
    host.shadow = [1, 2, 3, 96];

    let palette = material_feedback_palette_from_host(host);

    assert_eq!(palette.track, [9, 10, 11, 255]);
    assert_eq!(palette.disabled_track, [12, 13, 14, 255]);
    assert_eq!(palette.disabled_fill, [80, 81, 82, 255]);
    assert_eq!(palette.accent, [90, 91, 92, 255]);
    assert_eq!(palette.backdrop_scrim, [1, 2, 3, 96]);
}

#[test]
fn material_feedback_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.border_width = 2.0;

    let metrics = material_feedback_metrics_from_host(host);

    assert_eq!(metrics.linear_radius_floor, 4.0);
    assert_eq!(metrics.circular_indeterminate_percent, 0.58);
    assert_eq!(linear_progress_radius(0.0, 12.0, metrics), 4.0);
    assert_eq!(linear_progress_radius(5.0, 12.0, metrics), 5.0);
}

#[test]
fn backdrop_uses_host_shadow_when_background_is_not_declared() {
    let palette = material_feedback_palette_from_host(PALETTE);
    let node = positioned_backdrop_node("WorkbenchBackdropAtlas", 8.0, 8.0, 60.0, 20.0);
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 80.0,
        height: 40.0,
    };
    let clip = origin.clone();
    let mut commands = Vec::new();

    push_template_node_commands(&mut commands, &node, &origin, &clip, None, 0);

    let backdrop = commands
        .iter()
        .find(|command| command.frame.x == 8.0 && command.frame.y == 8.0)
        .expect("backdrop command");
    assert_eq!(backdrop.background_color, Some(palette.backdrop_scrim));
}

#[test]
fn fully_clipped_material_progress_does_not_emit_paint_commands() {
    let node = positioned_progress_node("WorkbenchFeedbackProgress", 0.64, 8.0, 16.0, 184.0, 12.0);
    let rect = FrameRect {
        x: 8.0,
        y: 16.0,
        width: 184.0,
        height: 12.0,
    };
    let clip = FrameRect {
        x: 220.0,
        y: 0.0,
        width: 80.0,
        height: 80.0,
    };
    let mut commands = Vec::new();

    assert!(push_material_feedback_primitive_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        0,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn partially_clipped_material_backdrop_keeps_only_clipped_paint_commands() {
    let node = positioned_backdrop_node("WorkbenchBackdropAtlas", 8.0, 8.0, 120.0, 32.0);
    let rect = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 120.0,
        height: 32.0,
    };
    let clip = FrameRect {
        x: 16.0,
        y: 12.0,
        width: 60.0,
        height: 20.0,
    };
    let mut commands = Vec::new();

    assert!(push_material_feedback_primitive_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        0,
        1.0,
    ));

    assert!(!commands.is_empty());
    assert!(commands.iter().all(|command| {
        command
            .clip_frame
            .as_ref()
            .is_some_and(|clip_frame| frame_is_within(&clip, clip_frame))
    }));
}

fn frame_is_within(outer: &FrameRect, inner: &FrameRect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}
