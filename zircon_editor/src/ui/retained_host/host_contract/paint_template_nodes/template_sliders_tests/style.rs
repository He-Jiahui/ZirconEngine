use super::super::super::style_selector::{
    WORKBENCH_SLIDER_FILL as SLIDER_FILL, WORKBENCH_SLIDER_THUMB as SLIDER_THUMB,
    WORKBENCH_SLIDER_TRACK_DISABLED as SLIDER_TRACK_DISABLED,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::super::template_slider_geometry::{
    slider_thumb_size, workbench_slider_metrics_from_host,
};
use super::support::{
    pixel_at, positioned_slider_node, resolved_background, slider_accent, slider_thumb_color,
    slider_thumb_halo_color, slider_thumb_outline_color, slider_track_color, slider_visual_hot,
    slider_visual_state,
};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{
    ResolvedButtonStyle, UiPainterResolvedState, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
};

#[test]
fn workbench_slider_uses_shared_selector_for_drop_hover_halo() {
    let mut node = positioned_slider_node("WorkbenchInputSlider", 0.5, 8.0, 8.0, 160.0, 30.0);
    node.drop_hovered = true;

    assert_eq!(
        slider_visual_state(&node),
        UiPainterResolvedState::DropHovered
    );
    assert!(slider_visual_hot(&node));

    let bytes = paint_template_nodes_for_test(190, 48, model_rc(vec![node]));
    assert_ne!(pixel_at(&bytes, 190, 61, 15), [0, 0, 0, 255]);
}

#[test]
fn workbench_slider_defaults_to_slate_bar_thumb_density() {
    let node = positioned_slider_node("WorkbenchInputSlider", 0.75, 8.0, 8.0, 184.0, 30.0);

    assert_eq!(slider_accent(&node), SLIDER_FILL);
    assert_eq!(slider_thumb_size(&node), 8.0);
    assert_eq!(slider_thumb_color(&node), SLIDER_THUMB);
    assert_eq!(slider_thumb_outline_color(&node), PALETTE.border);
}

#[test]
fn workbench_slider_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.border_width = 2.0;
    host.radius_control = 5.0;
    host.font_body = 12.0;
    host.button_pad_x = 13.0;
    host.text_clip_guard = 7.0;
    host.gap_s = 5.0;
    host.gap_m = 10.0;
    host.gap_l = 14.0;
    host.row_height = 30.0;

    let metrics = workbench_slider_metrics_from_host(host);

    assert_eq!(metrics.track_height, 8.0);
    assert_eq!(metrics.track_radius, 4.0);
    assert_eq!(metrics.thumb_size, 10.0);
    assert_eq!(metrics.thumb_halo_size, 20.0);
    assert_eq!(metrics.horizontal_inset, 10.0);
    assert_eq!(metrics.label_width, 56.0);
    assert_eq!(metrics.label_gap, 14.0);
    assert_eq!(metrics.value_width, 49.0);
    assert_eq!(metrics.value_gap, 10.0);
    assert_eq!(metrics.value_min_width, 147.0);
    assert_eq!(metrics.value_text_inset_x, 7.0);
    assert_eq!(metrics.value_radius, 5.0);
    assert_eq!(metrics.tick_width, 2.0);
    assert_eq!(metrics.tick_height, 8.0);
    assert_eq!(metrics.font_size, 14.0);
    assert!((metrics.line_height - 16.8).abs() < 0.001);
}

#[test]
fn workbench_slider_uses_declared_fill_and_thumb_size() {
    let mut node = positioned_slider_node("WorkbenchInputSlider", 0.75, 8.0, 8.0, 184.0, 30.0);
    node.value_color = Color::from_rgb_u8(32, 153, 162);
    node.layout_icon_size = 9.0;
    node.button_style = resolved_background([42, 51, 56, 255]);

    assert_eq!(slider_track_color(&node), [42, 51, 56, 255]);
    assert_eq!(slider_accent(&node), [32, 153, 162, 255]);
    assert_eq!(slider_thumb_size(&node), 9.0);

    node.disabled = true;
    assert_eq!(slider_track_color(&node), SLIDER_TRACK_DISABLED);
}

#[test]
fn workbench_slider_uses_declared_thumb_colors() {
    let mut node = positioned_slider_node("WorkbenchInputSlider", 0.75, 8.0, 8.0, 184.0, 30.0);
    node.icon_color = Color::from_rgb_u8(183, 241, 248);
    node.state_layer_color = Color::from_argb_u8(61, 50, 211, 222);
    node.button_style = ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            border_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(42, 177, 188, 51))),
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    };

    assert_eq!(slider_thumb_color(&node), [183, 241, 248, 255]);
    assert_eq!(slider_thumb_outline_color(&node), [42, 177, 188, 51]);
    assert_eq!(slider_thumb_halo_color(&node), Some([50, 211, 222, 61]));

    let bytes = paint_template_nodes_for_test(220, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 220, 101, 23), [183, 241, 248, 255]);
    assert_ne!(pixel_at(&bytes, 220, 96, 23), [0, 0, 0, 255]);
}
