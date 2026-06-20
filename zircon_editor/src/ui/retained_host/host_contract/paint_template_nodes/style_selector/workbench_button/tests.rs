use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{ButtonInteractionState, UiRgbaColor, UiStyleColor};

#[test]
fn button_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.button_style.loading = true;
    node.label_brightness = 1.5;
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(41, 164, 184, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(28, 135, 152, 255)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(8, 24, 27, 255)));

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Primary, true);

    assert_eq!(style.interaction, ButtonInteractionState::Loading);
    assert_eq!(style.surface, PALETTE.surface_disabled);
    assert_eq!(style.border, PALETTE.border_disabled);
    assert_eq!(style.border_width, 1.0);
    assert_eq!(style.text, PALETTE.text_disabled);
    assert_eq!(style.glyph, PALETTE.text_disabled);
}
