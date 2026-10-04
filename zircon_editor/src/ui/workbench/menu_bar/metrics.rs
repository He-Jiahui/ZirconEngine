use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

pub(crate) const WORKBENCH_MENU_SLOT_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE;
pub(crate) const WORKBENCH_MENU_SLOT_MIN_WIDTH: f32 = 72.0;
pub(crate) const WORKBENCH_MENU_SLOT_MAX_WIDTH: f32 = 128.0;

const WORKBENCH_MENU_SLOT_CHROME_RESERVE: f32 = 40.0;

pub(crate) fn workbench_menu_slot_width_from_label_width(label_width: f32) -> f32 {
    workbench_menu_slot_width_for_paint(label_width, WORKBENCH_MENU_SLOT_FONT_SIZE, 0.0)
}

pub(crate) fn workbench_menu_slot_width_for_paint(
    label_width: f32,
    font_size: f32,
    horizontal_inset: f32,
) -> f32 {
    let label_width = if label_width.is_finite() {
        label_width.max(0.0)
    } else {
        0.0
    };
    let scale = font_size / WORKBENCH_MENU_SLOT_FONT_SIZE;
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let horizontal_inset = if horizontal_inset.is_finite() {
        horizontal_inset.max(0.0)
    } else {
        0.0
    };
    let chrome_reserve = (WORKBENCH_MENU_SLOT_CHROME_RESERVE * scale).max(horizontal_inset * 2.0);

    (label_width + chrome_reserve).clamp(
        WORKBENCH_MENU_SLOT_MIN_WIDTH * scale,
        WORKBENCH_MENU_SLOT_MAX_WIDTH * scale,
    )
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;
