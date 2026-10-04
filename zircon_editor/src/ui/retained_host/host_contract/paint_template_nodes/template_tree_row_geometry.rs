//! 树行各绘制阶段共享同一套几何与密度指标，保证展开符、对象图标、标题和操作槽衔接。

mod actions;
mod labels;
mod metrics;
mod rows;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use actions::{
    tree_action_button_rect, tree_action_icon_rect, tree_action_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use labels::{
    tree_label_rect, tree_label_rect_for_variant,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    tree_font_size, tree_guide_color, tree_guide_opacity, tree_line_height, tree_metrics,
    tree_row_radius,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use rows::{
    tree_disclosure_rect, tree_guide_rect, tree_guide_x, tree_icon_rect,
};
