use super::*;

#[test]
fn material_property_value_summary_counts_projected_value_kinds() {
    let mut values = BTreeMap::new();
    values.insert(
        "enabled".to_string(),
        RenderMaterialPropertyValue::Bool { value: true },
    );
    values.insert(
        "gain".to_string(),
        RenderMaterialPropertyValue::Float { value: 1.0 },
    );
    values.insert(
        "layer".to_string(),
        RenderMaterialPropertyValue::Int { value: -2 },
    );
    values.insert(
        "flags".to_string(),
        RenderMaterialPropertyValue::UInt { value: 7 },
    );
    values.insert(
        "label".to_string(),
        RenderMaterialPropertyValue::String {
            value: "debug".to_string(),
        },
    );
    values.insert(
        "uv".to_string(),
        RenderMaterialPropertyValue::Vec2 { value: [0.0, 1.0] },
    );
    values.insert(
        "normal".to_string(),
        RenderMaterialPropertyValue::Vec3 {
            value: [0.0, 1.0, 0.0],
        },
    );
    values.insert(
        "tint".to_string(),
        RenderMaterialPropertyValue::Vec4 {
            value: [1.0, 0.5, 0.25, 1.0],
        },
    );

    let summary = RenderMaterialPropertyValueSummary::from_values(&values);

    assert_eq!(summary.total_count, 8);
    assert_eq!(summary.bool_count, 1);
    assert_eq!(summary.float_count, 1);
    assert_eq!(summary.int_count, 1);
    assert_eq!(summary.uint_count, 1);
    assert_eq!(summary.string_count, 1);
    assert_eq!(summary.vec2_count, 1);
    assert_eq!(summary.vec3_count, 1);
    assert_eq!(summary.vec4_count, 1);
    assert_eq!(summary.uniform_eligible_count(), 7);
    assert_eq!(summary.non_uniform_count(), 1);
}

#[test]
fn material_property_value_state_lists_property_names_and_uniform_eligibility() {
    let mut values = BTreeMap::new();
    values.insert(
        "debug_label".to_string(),
        RenderMaterialPropertyValue::String {
            value: "debug".to_string(),
        },
    );
    values.insert(
        "gain".to_string(),
        RenderMaterialPropertyValue::Float { value: 1.0 },
    );

    let states = RenderMaterialPropertyValueState::from_values(&values);

    assert_eq!(states.len(), 2);
    assert_eq!(states[0].name, "debug_label");
    assert_eq!(
        states[0].value,
        RenderMaterialPropertyValue::String {
            value: "debug".to_string()
        }
    );
    assert!(!states[0].is_uniform_eligible());
    assert_eq!(states[1].name, "gain");
    assert_eq!(
        states[1].value,
        RenderMaterialPropertyValue::Float { value: 1.0 }
    );
    assert!(states[1].is_uniform_eligible());
}
