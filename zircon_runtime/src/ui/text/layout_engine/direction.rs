use zircon_runtime_interface::ui::surface::UiTextDirection;

use crate::text::shaping::resolve_bidi_base_direction;

pub(crate) fn resolve_direction(text: &str, requested: UiTextDirection) -> UiTextDirection {
    resolve_bidi_base_direction(text, requested.into()).into()
}

pub(super) fn is_rtl_direction(direction: UiTextDirection) -> bool {
    matches!(direction, UiTextDirection::RightToLeft)
}
