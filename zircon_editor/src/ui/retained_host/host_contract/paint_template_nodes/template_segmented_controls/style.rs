mod segmented;
mod tabs;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use segmented::{
    segment_text_color, segmented_control_style, segmented_group_label_color,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use segmented::{
    segmented_background, selected_segment_border_width, selected_segment_underline_color,
    selected_segment_underline_height, SEGMENT_IDLE_BACKGROUND,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use tabs::tab_background;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use tabs::{
    tab_style, tab_text_color,
};
