use super::super::paint_theme::{METRICS, PALETTE};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ROW_HEIGHT: f32 =
    METRICS.row_height;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ROW_TEXT_FONT_SIZE:
    f32 = METRICS.font_body;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ROW_SURFACE_RADIUS:
    f32 = METRICS.radius_control;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ROW_TEXT_INSET_X: f32 =
    METRICS.gap_m;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ROW_TEXT_INSET_Y: f32 =
    METRICS.gap_s;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ROW_RIGHT_RESERVE: f32 =
    METRICS.button_chevron_reserve + METRICS.gap_m;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_BASE_INSET_X: f32 =
    METRICS.button_pad_x;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_DISCLOSURE_SIZE:
    f32 = METRICS.gap_l;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_ICON_SIZE: f32 =
    METRICS.font_large;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_TEXT_GAP: f32 =
    METRICS.gap_s + METRICS.border_width * 2.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_RIGHT_INSET: f32 =
    METRICS.button_pad_x;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_ACTION_SIZE: f32 =
    METRICS.font_large;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_ACTION_GAP: f32 =
    METRICS.gap_l + METRICS.gap_s;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_GUIDE_STEP: f32 =
    METRICS.button_chevron_reserve;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_GUIDE_OFFSET_X:
    f32 = METRICS.gap_s + METRICS.border_width;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TREE_GUIDE_COLOR: [u8;
    4] = PALETTE.track;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PROPERTY_LABEL_WIDTH:
    f32 = ROW_HEIGHT * 3.5;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const COMPONENT_PROPERTY_LABEL_WIDTH:
    f32 = ROW_HEIGHT * 4.0 - METRICS.gap_s;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PROPERTY_LABEL_MIN_WIDTH:
    f32 = METRICS.font_large * 4.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PROPERTY_LABEL_MAX_WIDTH_RATIO:
    f32 = 0.45;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PROPERTY_TEXT_INSET_X:
    f32 = METRICS.gap_s + METRICS.border_width;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PROPERTY_TEXT_INSET_Y:
    f32 = METRICS.gap_s;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PROPERTY_AXIS_WIDTH:
    f32 = METRICS.gap_l;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PROPERTY_AXIS_GAP: f32 =
    METRICS.gap_s;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PROPERTY_GROUP_GAP:
    f32 = METRICS.gap_s + METRICS.border_width * 2.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PROPERTY_FIELD_INSET_Y:
    f32 = METRICS.input_pad[2];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PROPERTY_FIELD_RADIUS:
    f32 = ROW_SURFACE_RADIUS;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_text_line_height(
) -> f32 {
    METRICS.line_height(ROW_TEXT_FONT_SIZE)
}
