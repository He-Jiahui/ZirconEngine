use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};

use super::super::geometry::SampleGridGeometry;
use super::super::metrics::{
    sample_grid_metrics_from_host, POINT_EDGE_INSET, POINT_INTERIOR_RADIUS, POINT_RADIUS,
};
use super::super::palette::sample_grid_palette_from_host;
use super::super::points::push_sample_points;
use super::support::sample_grid_node;

#[test]
fn sample_grid_default_frame_metrics_preserve_the_chart_baseline() {
    let metrics = sample_grid_metrics_from_host(METRICS);

    assert_eq!(metrics.outer_radius, 2.0);
    assert_eq!(metrics.plot_radius, 1.0);
    assert_eq!(metrics.border_width, 1.0);
    assert_eq!(metrics.grid_line_width, 1.0);
    assert_eq!(metrics.selected_label_border_width, 1.0);
    assert_eq!(metrics.selected_label_radius, 2.0);
}

#[test]
fn sample_grid_frame_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.radius_control = 8.0;
    host.border_width = 1.5;

    let metrics = sample_grid_metrics_from_host(host);

    assert_eq!(metrics.outer_radius, 4.0);
    assert_eq!(metrics.plot_radius, 2.0);
    assert_eq!(metrics.border_width, 1.5);
    assert_eq!(metrics.grid_line_width, 1.5);
    assert_eq!(metrics.selected_label_border_width, 1.5);
    assert_eq!(metrics.selected_label_radius, 4.0);
}

#[test]
fn sample_grid_zero_radius_theme_does_not_retain_plot_rounding() {
    let mut host = METRICS;
    host.radius_control = 0.0;
    host.border_width = 2.0;

    let metrics = sample_grid_metrics_from_host(host);

    assert_eq!(metrics.outer_radius, 0.0);
    assert_eq!(metrics.plot_radius, 0.0);
    assert_eq!(metrics.selected_label_radius, 0.0);
    assert_eq!(metrics.border_width, 2.0);
}

#[test]
fn selected_label_command_consumes_projected_border_and_radius_metrics() {
    let mut host = METRICS;
    host.radius_control = 8.0;
    host.border_width = 1.5;
    let metrics = sample_grid_metrics_from_host(host);
    let palette = sample_grid_palette_from_host(PALETTE);
    let node = sample_grid_node(360.0, 260.0);
    let frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 360.0,
        height: 260.0,
    };
    let geometry = SampleGridGeometry::from_frame(&frame);
    let mut commands = Vec::new();

    push_sample_points(
        &mut commands,
        &node,
        &geometry,
        &frame,
        0,
        1.0,
        metrics,
        palette,
    );

    let label_surface = commands
        .iter()
        .find(|command| {
            command.background_color == Some(palette.selected_label_surface)
                && command.border_color == Some(palette.selected_point)
        })
        .expect("the selected point should emit its Runtime Text label surface");
    assert_eq!(label_surface.border_width, 1.5);
    assert_eq!(label_surface.corner_radius, 4.0);
}

#[test]
fn sample_point_size_matches_the_unreal_key_baseline() {
    assert_eq!(POINT_RADIUS * 2 + 1, 11);
    assert_eq!(POINT_INTERIOR_RADIUS * 2 + 1, 7);
    assert!(POINT_INTERIOR_RADIUS < POINT_RADIUS);
    assert!(POINT_EDGE_INSET > POINT_RADIUS as f32);
}

#[test]
fn selected_and_ordinary_points_emit_the_same_unreal_key_geometry() {
    let metrics = sample_grid_metrics_from_host(METRICS);
    let palette = sample_grid_palette_from_host(PALETTE);
    let node = sample_grid_node(360.0, 260.0);
    let frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 360.0,
        height: 260.0,
    };
    let geometry = SampleGridGeometry::from_frame(&frame);
    let mut commands = Vec::new();

    push_sample_points(
        &mut commands,
        &node,
        &geometry,
        &frame,
        0,
        1.0,
        metrics,
        palette,
    );

    assert_eq!(commands.len(), 38);
    let key_commands = commands
        .iter()
        .filter(|command| command.z_index == 7)
        .collect::<Vec<_>>();
    assert_eq!(key_commands.len(), (POINT_RADIUS * 2 + 1) as usize * 2);
    assert!(key_commands.iter().all(|command| {
        matches!(
            command.background_color,
            Some(color) if color == palette.selected_point || color == palette.point
        )
    }));

    let key_width_sequences = [palette.selected_point, palette.point].map(|color| {
        commands
            .iter()
            .filter(|command| command.z_index == 7 && command.background_color == Some(color))
            .map(|command| {
                assert_eq!(command.frame.height, 1.0);
                command.frame.width
            })
            .collect::<Vec<_>>()
    });
    assert_eq!(key_width_sequences[0], key_width_sequences[1]);
    assert_eq!(
        key_width_sequences[0],
        vec![1.0, 3.0, 5.0, 7.0, 9.0, 11.0, 9.0, 7.0, 5.0, 3.0, 1.0]
    );
    let center_commands = commands
        .iter()
        .filter(|command| command.z_index == 8)
        .collect::<Vec<_>>();
    assert_eq!(
        center_commands.len(),
        (POINT_INTERIOR_RADIUS * 2 + 1) as usize * 2
    );
    assert!(center_commands
        .iter()
        .all(|command| command.background_color == Some(palette.plot_surface)));
    let interior_width_sequences = center_commands
        .chunks_exact((POINT_INTERIOR_RADIUS * 2 + 1) as usize)
        .map(|commands| {
            commands
                .iter()
                .map(|command| command.frame.width)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(interior_width_sequences[0], interior_width_sequences[1]);
    assert_eq!(
        interior_width_sequences[0],
        vec![1.0, 3.0, 5.0, 7.0, 5.0, 3.0, 1.0]
    );
    assert_eq!(
        commands
            .iter()
            .filter(|command| command.z_index == 9)
            .count(),
        1
    );
    assert_eq!(
        commands
            .iter()
            .filter(|command| command.z_index == 10)
            .count(),
        1
    );
}

#[test]
fn compact_point_commands_use_the_parent_plot_intersection_clip() {
    let metrics = sample_grid_metrics_from_host(METRICS);
    let palette = sample_grid_palette_from_host(PALETTE);
    let node = sample_grid_node(80.0, 60.0);
    let geometry = SampleGridGeometry {
        outer: FrameRect::default(),
        plot: FrameRect {
            x: 10.0,
            y: 20.0,
            width: 8.0,
            height: 10.0,
        },
    };
    let parent_clip = FrameRect {
        x: 12.0,
        y: 18.0,
        width: 10.0,
        height: 8.0,
    };
    let expected_clip = FrameRect {
        x: 12.0,
        y: 20.0,
        width: 6.0,
        height: 6.0,
    };
    let mut commands = Vec::new();

    push_sample_points(
        &mut commands,
        &node,
        &geometry,
        &parent_clip,
        0,
        1.0,
        metrics,
        palette,
    );

    assert!(!commands.is_empty());
    assert!(commands
        .iter()
        .all(|command| command.clip_frame.as_ref() == Some(&expected_clip)));
    assert!(commands.iter().all(|command| command.z_index <= 8));
    assert!(commands.iter().all(|command| command.text.is_none()));
}
