//! 分段组的主体、组标签、单项和选中区域几何门面；由 segments/labels 共享。

#[path = "../../../retained/contract/paint/segmented/body.rs"]
mod body;
#[path = "../../../retained/contract/paint/segmented/group.rs"]
mod group;
#[path = "../../../retained/contract/paint/segmented/item.rs"]
mod item;
#[path = "../../../retained/contract/paint/segmented/selected.rs"]
mod selected;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use body::segmented_body_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use group::segmented_group_label_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use item::{
    segment_divider_rect, segment_label_rect, segment_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selected::{
    selected_segment_rect, selected_segment_underline_rect,
};
