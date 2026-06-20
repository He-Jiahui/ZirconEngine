mod fields;
mod labels;
mod metrics;
mod shadows;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use fields::{
    chevron_rect, field_rect, leading_affordance_rect, nested_select_field_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use labels::nested_label_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    INSPECTOR_CHEVRON_SIZE, INSPECTOR_COUNT_WIDTH, INSPECTOR_FIELD_RIGHT_PAD,
    INSPECTOR_FIELD_TEXT_X, INSPECTOR_LABEL_WIDTH, INSPECTOR_ROW_TEXT_Y,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use shadows::{
    shadow_check_content_offset_x, shadow_check_rect,
};
