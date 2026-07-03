use super::palette::workbench_text_field_palette;
use super::state::is_unavailable_text_field_state;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_text(
    state: UiPainterResolvedState,
    label_is_placeholder: bool,
) -> [u8; 4] {
    let palette = workbench_text_field_palette();
    if is_unavailable_text_field_state(state) {
        palette.disabled_text
    } else if label_is_placeholder {
        palette.placeholder
    } else {
        palette.text
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_stepper(
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_text_field_palette();
    if is_unavailable_text_field_state(state) {
        palette.disabled_text
    } else {
        palette.placeholder
    }
}
