use crate::{Axis3, AxisDirection, ScalarPrecision, SpaceKind};

#[test]
fn coordinate_vocabulary_preserves_axis_and_space_semantics() {
    assert_eq!(
        AxisDirection::Positive(Axis3::Y),
        AxisDirection::Positive(Axis3::Y)
    );
    assert!(SpaceKind::ViewRelative.is_render_space());
    assert!(!SpaceKind::World.is_render_space());
}

#[test]
fn scalar_precision_reports_storage_width() {
    assert_eq!(ScalarPrecision::F32.bytes(), core::mem::size_of::<f32>());
    assert_eq!(ScalarPrecision::F64.bytes(), core::mem::size_of::<f64>());
}
