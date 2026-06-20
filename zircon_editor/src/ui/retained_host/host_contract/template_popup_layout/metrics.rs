use super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract) const TEMPLATE_POPUP_ROW_GAP: f32 = 4.0;
const MIN_TEMPLATE_POPUP_ROW_HEIGHT: f32 = 24.0;

pub(in crate::ui::retained_host::host_contract) fn dropdown_option_row_height(
    control_frame: &FrameRect,
) -> f32 {
    control_frame.height.max(MIN_TEMPLATE_POPUP_ROW_HEIGHT)
}

pub(in crate::ui::retained_host::host_contract) fn menu_item_row_height(
    menu_frame: &FrameRect,
    row_count: usize,
) -> Option<f32> {
    (row_count > 0)
        .then_some((menu_frame.height / row_count as f32).max(MIN_TEMPLATE_POPUP_ROW_HEIGHT))
}
