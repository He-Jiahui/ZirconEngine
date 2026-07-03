use zircon_runtime_interface::ui::surface::UiTextDirection;

pub(crate) fn resolve_direction(text: &str, requested: UiTextDirection) -> UiTextDirection {
    match requested {
        UiTextDirection::LeftToRight | UiTextDirection::RightToLeft => requested,
        UiTextDirection::Auto | UiTextDirection::Mixed => {
            first_strong_direction(text).unwrap_or(UiTextDirection::LeftToRight)
        }
    }
}

pub(super) fn is_rtl_direction(direction: UiTextDirection) -> bool {
    matches!(direction, UiTextDirection::RightToLeft)
}

// UAX#9 P2/P3 paragraph direction: use the first strong character until full
// bidi level resolution replaces this low-fidelity visual-order scaffold.
fn first_strong_direction(text: &str) -> Option<UiTextDirection> {
    text.chars().find_map(|ch| {
        if is_rtl_char(ch) {
            Some(UiTextDirection::RightToLeft)
        } else if is_ltr_char(ch) {
            Some(UiTextDirection::LeftToRight)
        } else {
            None
        }
    })
}

pub(super) fn is_ltr_char(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch.is_ascii_digit()
}

pub(super) fn is_rtl_char(ch: char) -> bool {
    matches!(ch as u32, 0x0590..=0x08FF | 0xFB1D..=0xFDFF | 0xFE70..=0xFEFF)
}
