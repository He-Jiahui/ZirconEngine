use super::super::identity::{AxisLabelKind, axis_label_kind};
use super::support::label_node;

#[test]
fn axis_label_kind_matches_transform_axis_labels_and_scale_link() {
    assert_eq!(
        axis_label_kind(&label_node("WorkbenchTransformPositionAxisX", "X")),
        Some(AxisLabelKind::Axis("X"))
    );
    assert_eq!(
        axis_label_kind(&label_node("WorkbenchTransformScaleLink", "")),
        Some(AxisLabelKind::ScaleLink)
    );
    assert_eq!(
        axis_label_kind(&label_node("WorkbenchTransformScaleX", "1.00")),
        None
    );
}
