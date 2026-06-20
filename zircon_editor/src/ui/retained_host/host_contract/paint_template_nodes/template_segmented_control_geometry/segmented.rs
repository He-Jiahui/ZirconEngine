mod body;
mod group;
mod item;
mod selected;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use body::segmented_body_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use group::segmented_group_label_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use item::{
    segment_divider_rect, segment_label_rect, segment_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selected::{
    selected_segment_rect, selected_segment_underline_rect,
};
