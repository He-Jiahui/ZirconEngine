use crate::text::TextStyle;
use zircon_runtime_interface::ui::surface::UiResolvedStyle;

pub(crate) fn text_style(value: &UiResolvedStyle) -> TextStyle {
    value.into()
}
