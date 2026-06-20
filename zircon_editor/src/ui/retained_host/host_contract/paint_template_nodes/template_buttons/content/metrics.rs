pub(super) const BUTTON_FONT_SIZE: f32 = 12.0;
pub(super) const BUTTON_LINE_HEIGHT: f32 = BUTTON_FONT_SIZE * 1.2;
pub(super) const BUTTON_TEXT_INSET_X: f32 = 12.0;
pub(super) const BUTTON_ICON_GAP: f32 = 7.0;
pub(super) const BUTTON_CHEVRON_RESERVE: f32 = 18.0;

pub(super) fn estimated_label_width(label: &str) -> f32 {
    label.chars().count() as f32 * BUTTON_FONT_SIZE * 0.56
}
