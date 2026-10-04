use super::*;

#[test]
fn menu_slot_typography_uses_workbench_body_role() {
    assert_eq!(
        WORKBENCH_MENU_SLOT_FONT_SIZE,
        EditorTypographyTokens::WORKBENCH_BODY_SIZE
    );
}

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

#[test]
fn low_positive_menu_measure_uses_scaled_readability_floor() {
    let scale = 1.25;
    let width =
        workbench_menu_slot_width_for_paint(4.0, WORKBENCH_MENU_SLOT_FONT_SIZE * scale, 0.0);

    assert_eq!(width, WORKBENCH_MENU_SLOT_MIN_WIDTH * scale);
}

#[test]
fn longer_measured_menu_label_gets_scaled_chrome_reserve() {
    let scale = 1.25;
    let width =
        workbench_menu_slot_width_for_paint(72.0, WORKBENCH_MENU_SLOT_FONT_SIZE * scale, 8.0);

    assert_eq!(width, 122.0);
    assert!(width > 72.0 * scale);
}

#[test]
fn scaled_menu_slot_keeps_painted_text_and_insets_inside_the_hit_region() {
    let label_width = 82.0;
    let inset = 12.0;
    let width = workbench_menu_slot_width_for_paint(
        label_width,
        WORKBENCH_MENU_SLOT_FONT_SIZE * 2.0,
        inset,
    );
    assert!(width >= label_width + inset * 2.0);
    assert!(width > workbench_menu_slot_width_from_label_width(label_width));
}
