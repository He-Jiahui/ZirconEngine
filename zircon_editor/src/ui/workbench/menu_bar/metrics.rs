pub(crate) const WORKBENCH_MENU_SLOT_FONT_SIZE: f32 = 12.0;
pub(crate) const WORKBENCH_MENU_SLOT_MIN_WIDTH: f32 = 40.0;
pub(crate) const WORKBENCH_MENU_SLOT_MAX_WIDTH: f32 = 128.0;

const WORKBENCH_MENU_SLOT_CHROME_RESERVE: f32 = 24.0;

pub(crate) fn workbench_menu_slot_width_from_label_width(label_width: f32) -> f32 {
    let label_width = if label_width.is_finite() {
        label_width.max(0.0)
    } else {
        0.0
    };

    (label_width + WORKBENCH_MENU_SLOT_CHROME_RESERVE)
        .clamp(WORKBENCH_MENU_SLOT_MIN_WIDTH, WORKBENCH_MENU_SLOT_MAX_WIDTH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workbench_menu_slot_width_clamps_measured_label_width() {
        assert_eq!(
            workbench_menu_slot_width_from_label_width(1.0),
            WORKBENCH_MENU_SLOT_MIN_WIDTH
        );
        assert_eq!(
            workbench_menu_slot_width_from_label_width(10_000.0),
            WORKBENCH_MENU_SLOT_MAX_WIDTH
        );
        assert_eq!(
            workbench_menu_slot_width_from_label_width(f32::NAN),
            WORKBENCH_MENU_SLOT_MIN_WIDTH
        );
    }
}
