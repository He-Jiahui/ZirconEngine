const SAVE_CUTOUT_ORDER_OFFSET: i32 = 1;

pub(super) fn save_cutout_order(base_order: i32) -> i32 {
    base_order + SAVE_CUTOUT_ORDER_OFFSET
}
