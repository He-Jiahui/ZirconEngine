mod bounds;
mod dropdown;
mod metrics;
mod rows;
mod template;

pub(crate) use dropdown::{
    dropdown_option_popup_frame, dropdown_option_popup_frame_within, dropdown_option_row_frame,
    dropdown_option_row_frame_within,
};
pub(crate) use rows::{menu_item_row_at_y, menu_item_row_frame};
pub(crate) use template::{
    template_option_popup_frame_within, template_option_row_frame_within,
    template_option_rows_use_projected_frame,
};

#[cfg(test)]
#[path = "template_popup_layout_tests.rs"]
mod tests;
