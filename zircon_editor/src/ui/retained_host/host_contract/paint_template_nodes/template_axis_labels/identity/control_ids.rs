use super::AxisLabelKind;

const SCALE_LINK_CONTROL_ID: &str = "WorkbenchTransformScaleLink";
const TRANSFORM_CONTROL_ID_PREFIX: &str = "WorkbenchTransform";
const TRANSFORM_SCALE_AXIS_CONTROL_ID_PREFIX: &str = "WorkbenchTransformScaleAxis";

const AXIS_X_SUFFIX: &str = "AxisX";
const AXIS_Y_SUFFIX: &str = "AxisY";
const AXIS_Z_SUFFIX: &str = "AxisZ";

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
    if field.ends_with(AXIS_X_SUFFIX) {
        Some(AXIS_X_LABEL)
    } else if field.ends_with(AXIS_Y_SUFFIX) {
        Some(AXIS_Y_LABEL)
    } else if field.ends_with(AXIS_Z_SUFFIX) {
        Some(AXIS_Z_LABEL)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_label_kind_matches_transform_axis_and_scale_link_ids() {
        assert_eq!(
            axis_label_kind_from_control_id("WorkbenchTransformPositionAxisX"),
            Some(AxisLabelKind::Axis("X"))
        );
        assert_eq!(
            axis_label_kind_from_control_id("WorkbenchTransformRotationAxisY"),
            Some(AxisLabelKind::Axis("Y"))
        );
        assert_eq!(
            axis_label_kind_from_control_id("WorkbenchTransformScaleAxisZ"),
            Some(AxisLabelKind::Axis("Z"))
        );
        assert_eq!(
            axis_label_kind_from_control_id("WorkbenchTransformScaleLink"),
            Some(AxisLabelKind::ScaleLink)
        );
    }

    #[test]
    fn transform_scale_axis_prefix_excludes_scale_link() {
        assert!(is_transform_scale_axis_control_id(
            "WorkbenchTransformScaleAxisX"
        ));
        assert!(!is_transform_scale_axis_control_id(
            "WorkbenchTransformScaleLink"
        ));
    }
}
