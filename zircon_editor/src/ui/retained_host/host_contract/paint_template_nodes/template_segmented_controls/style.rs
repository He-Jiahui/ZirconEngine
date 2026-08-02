mod segmented;
mod tabs;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use segmented::{
    SEGMENT_IDLE_BACKGROUND, SEGMENT_SELECTED_BACKGROUND, segmented_background,
    selected_segment_border_width, selected_segment_surface_color,
    selected_segment_underline_color, selected_segment_underline_height,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use segmented::{
    segment_text_color, segmented_control_style, segmented_group_label_color,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use tabs::tab_background;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use tabs::{
    tab_style, tab_text_color,
};
