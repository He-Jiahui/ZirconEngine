use super::super::template_row_metrics::{
    row_text_line_height, ROW_SURFACE_RADIUS, ROW_TEXT_FONT_SIZE,
    TREE_ACTION_GAP as SHARED_TREE_ACTION_GAP, TREE_ACTION_SIZE as SHARED_TREE_ACTION_SIZE,
    TREE_BASE_INSET_X as SHARED_TREE_BASE_INSET_X,
    TREE_DISCLOSURE_SIZE as SHARED_TREE_DISCLOSURE_SIZE,
    TREE_GUIDE_COLOR as SHARED_TREE_GUIDE_COLOR, TREE_GUIDE_OFFSET_X as SHARED_TREE_GUIDE_OFFSET_X,
    TREE_GUIDE_STEP as SHARED_TREE_GUIDE_STEP, TREE_ICON_SIZE as SHARED_TREE_ICON_SIZE,
    TREE_RIGHT_INSET as SHARED_TREE_RIGHT_INSET, TREE_TEXT_GAP as SHARED_TREE_TEXT_GAP,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_FONT_SIZE: f32 =
    ROW_TEXT_FONT_SIZE;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_ROW_RADIUS: f32 =
    ROW_SURFACE_RADIUS;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_GUIDE_COLOR: [u8;
    4] = SHARED_TREE_GUIDE_COLOR;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_BASE_INSET_X: f32 =
    SHARED_TREE_BASE_INSET_X;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_DISCLOSURE_SIZE:
    f32 = SHARED_TREE_DISCLOSURE_SIZE;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_ICON_SIZE: f32 =
    SHARED_TREE_ICON_SIZE;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_TEXT_GAP: f32 =
    SHARED_TREE_TEXT_GAP;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_RIGHT_INSET: f32 =
    SHARED_TREE_RIGHT_INSET;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_ACTION_SIZE: f32 =
    SHARED_TREE_ACTION_SIZE;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_ACTION_GAP: f32 =
    SHARED_TREE_ACTION_GAP;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_GUIDE_STEP: f32 =
    SHARED_TREE_GUIDE_STEP;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_GUIDE_OFFSET_X:
    f32 = SHARED_TREE_GUIDE_OFFSET_X;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_line_height() -> f32 {
    row_text_line_height()
}
