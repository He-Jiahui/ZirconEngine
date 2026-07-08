mod geometry;
mod label;
mod shortcut;
mod style;

use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use label::push_popup_row_label;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use shortcut::push_popup_row_shortcut;

pub(super) fn popup_row_text_style() -> UiTextRunPaintStyle {
    UiTextRunPaintStyle::default()
}
