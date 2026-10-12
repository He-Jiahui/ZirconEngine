#[path = "popup_layout/bounds.rs"]
mod bounds;
#[path = "popup_layout/dropdown.rs"]
mod dropdown;
#[path = "popup_layout/metrics.rs"]
mod metrics;
#[path = "popup_layout/rows.rs"]
mod rows;
#[path = "popup_layout/template.rs"]
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
#[path = "tests/popup_layout.rs"]
mod tests;
