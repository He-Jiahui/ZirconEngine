const SEPARATOR_OFFSET: i32 = 1;
const CELLS_OFFSET: i32 = 2;
const ACTION_SLOT_OFFSET: i32 = 3;
const ACTION_ICON_OFFSET: i32 = 1;

pub(super) fn separator_order(surface_order: i32) -> i32 {
    surface_order + SEPARATOR_OFFSET
}

pub(super) fn cells_order(surface_order: i32) -> i32 {
    surface_order + CELLS_OFFSET
}

pub(super) fn action_slot_order(surface_order: i32) -> i32 {
    surface_order + ACTION_SLOT_OFFSET
}

pub(super) fn action_icon_order(slot_order: i32) -> i32 {
    slot_order + ACTION_ICON_OFFSET
}

#[cfg(test)]
mod tests {
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
}
