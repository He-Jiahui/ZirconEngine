use super::super::super::style_selector::WORKBENCH_SLIDER_TRACK_DISABLED as SLIDER_TRACK_DISABLED;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::super::template_slider_geometry::slider_thumb_size;
use super::support::{
    pixel_at, positioned_slider_node, resolved_background, slider_accent, slider_thumb_color,
    slider_thumb_halo_color, slider_thumb_outline_color, slider_track_color, slider_visual_hot,
    slider_visual_state,
};
use crate::ui::layouts::common::model_rc;
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

    assert_eq!(pixel_at(&bytes, 220, 104, 23), [183, 241, 248, 255]);
    assert_ne!(pixel_at(&bytes, 220, 96, 23), [0, 0, 0, 255]);
}
