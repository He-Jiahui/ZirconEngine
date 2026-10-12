use super::*;
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn strip_axis_prefix_accepts_native_axis_labels() {
    assert_eq!(strip_axis_prefix("X 42.0", "X"), "42.0");
    assert_eq!(strip_axis_prefix("90 deg", "Y"), "90 deg");
}

#[test]
fn format_axis_row_value_keeps_reference_spacing() {
    assert_eq!(
        format_axis_row_value("12.0", "3.5", "-8.0"),
        "X 12.0   Y 3.5   Z -8.0"
    );
}

#[test]
fn transform_commit_emits_a_typed_finite_scalar() {
    let bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    let binding = bridge
        .transform_axis_commit_binding(POSITION_X, "Inspector/TransformPositionXCommit", "X 4.25")
        .unwrap()
        .expect("position commit should resolve");
    let EditorUiBindingPayload::InspectorFieldBatch { changes, .. } = binding.payload() else {
        panic!("position commit must dispatch an inspector field batch");
    };

    assert_eq!(changes[0].field_id, "transform.translation.x");
    assert_eq!(changes[0].value, UiBindingValue::Float(4.25));

    let binding = bridge
        .transform_axis_commit_binding(SCALE_Z, "Inspector/TransformScaleZCommit", "Z 2.5")
        .unwrap()
        .expect("scale commit should resolve");
    let EditorUiBindingPayload::InspectorFieldBatch { changes, .. } = binding.payload() else {
        panic!("scale commit must dispatch an inspector field batch");
    };

    assert_eq!(changes[0].field_id, "transform.scale.z");
    assert_eq!(changes[0].value, UiBindingValue::Float(2.5));
    assert_eq!(
        bridge
            .transform_axis_commit_binding(SCALE_Z, "Inspector/TransformScaleZCommit", "Z NaN",)
            .unwrap_err(),
        "Inspector transform Z value `NaN` must be a finite number"
    );
}
