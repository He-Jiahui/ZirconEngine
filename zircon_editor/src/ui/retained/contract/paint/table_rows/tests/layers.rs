use super::*;

#[test]
fn table_row_layers_keep_surface_separator_cells_action_order() {
    let surface = 18;
    let action_slot = action_slot_order(surface);

    assert!(surface < separator_order(surface));
    assert!(separator_order(surface) < cells_order(surface));
    assert!(cells_order(surface) < action_slot);
    assert!(action_slot < action_icon_order(action_slot));
}
