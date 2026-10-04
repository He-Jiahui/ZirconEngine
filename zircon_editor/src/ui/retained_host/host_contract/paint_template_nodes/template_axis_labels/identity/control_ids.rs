//! 变换面板约定的 control ID 同时区分 X/Y/Z 标签和比例链接标记；后缀规则限定在 Axis 后，避免值字段误作标签。
//! 工作台 ZUI 作者须保持这些 ID 与实际轴槽对应。

use super::AxisLabelKind;

const SCALE_LINK_CONTROL_ID: &str = "WorkbenchTransformScaleLink";
const TRANSFORM_CONTROL_ID_PREFIX: &str = "WorkbenchTransform";
const TRANSFORM_SCALE_AXIS_CONTROL_ID_PREFIX: &str = "WorkbenchTransformScaleAxis";

const AXIS_X_LABEL: &str = "X";
const AXIS_Y_LABEL: &str = "Y";
const AXIS_Z_LABEL: &str = "Z";

pub(super) fn axis_label_kind_from_control_id(control_id: &str) -> Option<AxisLabelKind> {
    if control_id == SCALE_LINK_CONTROL_ID {
        return Some(AxisLabelKind::ScaleLink);
    }
    transform_axis_label(control_id).map(AxisLabelKind::Axis)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_transform_scale_axis_control_id(
    control_id: &str,
) -> bool {
    control_id.starts_with(TRANSFORM_SCALE_AXIS_CONTROL_ID_PREFIX)
}

fn transform_axis_label(control_id: &str) -> Option<&'static str> {
    let field = control_id.strip_prefix(TRANSFORM_CONTROL_ID_PREFIX)?;
    let (axis_offset, axis) = field.char_indices().next_back()?;
    if !field[..axis_offset].ends_with("Axis") {
        return None;
    }
    match axis {
        'X' => Some(AXIS_X_LABEL),
        'Y' => Some(AXIS_Y_LABEL),
        'Z' => Some(AXIS_Z_LABEL),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/control_ids.rs"]
mod tests;
