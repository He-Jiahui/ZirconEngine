use super::super::super::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ICON_TINT: [u8; 4] =
    PALETTE.text;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ICON_TINT_ACTIVE: [u8;
    4] = PALETTE.focus_ring;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ICON_TINT_DISABLED:
    [u8; 4] = PALETTE.text_disabled;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ICON_TINT_ERROR: [u8;
    4] = PALETTE.error;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ICON_TINT_WARNING:
    [u8; 4] = PALETTE.warning;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_image_tint(
    is_icon_like: bool,
    active: bool,
    disabled: bool,
    text_tone: &str,
    validation_level: &str,
    style_tint: Option<[u8; 4]>,
) -> Option<[u8; 4]> {
    if !is_icon_like {
        return None;
    }
    if disabled {
        return Some(ICON_TINT_DISABLED);
    }
    if validation_level.eq_ignore_ascii_case("error") || text_tone.eq_ignore_ascii_case("error") {
        return Some(ICON_TINT_ERROR);
    }
    if validation_level.eq_ignore_ascii_case("warning") || text_tone.eq_ignore_ascii_case("warning")
    {
        return Some(ICON_TINT_WARNING);
    }
    if let Some(style_tint) = style_tint {
        return Some(style_tint);
    }
    if active {
        return Some(ICON_TINT_ACTIVE);
    }
    Some(ICON_TINT)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tint_non_transparent_pixels(
    rgba: &mut [u8],
    tint: [u8; 4],
) {
    for pixel in rgba.chunks_exact_mut(4) {
        if pixel[3] == 0 {
            continue;
        }
        pixel[0] = tint[0];
        pixel[1] = tint[1];
        pixel[2] = tint[2];
    }
}
