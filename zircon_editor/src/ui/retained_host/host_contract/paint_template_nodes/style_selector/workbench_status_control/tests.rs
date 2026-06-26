use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn status_signal_unavailable_states_mute_icon_and_text() {
    let mut disabled = TemplatePaneNodeData::default();
    disabled.disabled = true;
    disabled.hovered = true;
    disabled.label_color = Color::from_rgb_u8(242, 195, 86);
    disabled.value_color = Color::from_rgb_u8(135, 146, 153);

    let disabled_style =
        select_workbench_status_signal_style(&disabled, WorkbenchStatusSignalKind::Warning);
    assert_eq!(disabled_style.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled_style.icon_fill, PALETTE.text_disabled);
    assert_eq!(disabled_style.text, PALETTE.text_disabled);

    let mut loading = TemplatePaneNodeData::default();
    loading.hovered = true;
    loading.button_style.loading = true;
    loading.label_color = Color::from_rgb_u8(88, 184, 102);
    loading.value_color = Color::from_rgb_u8(143, 154, 160);

    let loading_style =
        select_workbench_status_signal_style(&loading, WorkbenchStatusSignalKind::Success);
    assert_eq!(loading_style.state, UiPainterResolvedState::Loading);
    assert_eq!(loading_style.icon_fill, PALETTE.text_disabled);
    assert_eq!(loading_style.text, PALETTE.text_disabled);
}
