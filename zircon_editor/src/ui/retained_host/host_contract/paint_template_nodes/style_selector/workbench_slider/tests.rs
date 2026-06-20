use super::palette::WORKBENCH_SLIDER_TRACK_DISABLED;
use super::selection::select_workbench_slider_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn slider_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.pressed = true;
    node.drop_hovered = true;
    node.button_style.loading = true;
    node.validation_level = "warning".into();
    node.value_color = Color::from_rgb_u8(53, 199, 208);
    node.icon_color = Color::from_rgb_u8(201, 242, 246);
    node.label_color = Color::from_rgb_u8(174, 189, 196);
    node.state_layer_color = Color::from_argb_u8(58, 53, 199, 208);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(54, 64, 70, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(53, 199, 208, 255)));

    let style = select_workbench_slider_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.track, WORKBENCH_SLIDER_TRACK_DISABLED);
    assert_eq!(style.fill, PALETTE.text_disabled);
    assert_eq!(style.thumb, PALETTE.text_disabled);
    assert_eq!(style.thumb_outline, PALETTE.border_disabled);
    assert_eq!(style.thumb_halo, None);
    assert_eq!(style.value_surface, PALETTE.surface_disabled);
    assert_eq!(style.value_border, PALETTE.border_disabled);
    assert_eq!(style.range_value_border, PALETTE.border_disabled);
    assert_eq!(style.label_text, PALETTE.text_disabled);
    assert_eq!(style.value_text, PALETTE.text_disabled);
    assert_eq!(style.tick, PALETTE.border_disabled);
}
