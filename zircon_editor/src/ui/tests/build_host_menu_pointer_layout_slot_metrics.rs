use super::*;
use crate::ui::workbench::menu_bar::WORKBENCH_MENU_SLOT_FONT_SIZE;

#[test]
fn menu_pointer_stencil_uses_the_same_scaled_slot_width_as_the_scene() {
    let metrics = MenuLabelSlotMetrics {
        font_size: WORKBENCH_MENU_SLOT_FONT_SIZE * 1.5,
        logical_font_size: WORKBENCH_MENU_SLOT_FONT_SIZE,
        horizontal_inset: 9.0,
    };
    let stencil =
        std::array::from_fn(|index| UiFrame::new(8.0 + index as f32 * 50.0, 2.0, 40.0, 22.0));
    let frames = menu_button_frames_from_stencil(&stencil, &["File", "Selection"], 2, metrics);
    assert_eq!(frames[0].width, menu_label_slot_width("File", metrics));
    assert_eq!(frames[1].width, menu_label_slot_width("Selection", metrics));
    assert!(frames[1].x >= frames[0].x + frames[0].width);
}

#[test]
fn unresolved_font_measure_keeps_first_frame_menu_labels_readable() {
    let metrics = MenuLabelSlotMetrics {
        font_size: WORKBENCH_MENU_SLOT_FONT_SIZE * 1.25,
        logical_font_size: WORKBENCH_MENU_SLOT_FONT_SIZE,
        horizontal_inset: 7.5,
    };
    let unresolved_file = menu_label_slot_width_from_measured("File", metrics, 0.0);
    let unresolved_selection = menu_label_slot_width_from_measured("Selection", metrics, 0.0);
    let resolved_selection = menu_label_slot_width_from_measured("Selection", metrics, 72.0);

    assert!(unresolved_file > 40.0 * 1.25);
    assert!(unresolved_selection > unresolved_file);
    assert_eq!(
        resolved_selection,
        workbench_menu_slot_width_for_paint(72.0, metrics.font_size, metrics.horizontal_inset)
    );
}
