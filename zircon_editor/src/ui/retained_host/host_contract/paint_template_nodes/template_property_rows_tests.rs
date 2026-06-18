use super::*;

#[test]
fn property_axis_values_group_units_with_axis_value() {
    assert_eq!(
        property_axis_values("X 0 deg   Y 90 deg   Z -12.5 deg"),
        vec![
            PropertyAxisValue {
                axis: "X".into(),
                value: "0 deg".into(),
            },
            PropertyAxisValue {
                axis: "Y".into(),
                value: "90 deg".into(),
            },
            PropertyAxisValue {
                axis: "Z".into(),
                value: "-12.5 deg".into(),
            },
        ]
    );
}

#[test]
fn component_property_input_rows_use_split_property_row_painter() {
    let node = TemplatePaneNodeData {
        control_id: MESH_PROPERTY_ROW.into(),
        role: "InputField".into(),
        component_role: "input-field".into(),
        text: "Visible".into(),
        value_text: "true".into(),
        ..TemplatePaneNodeData::default()
    };

    assert!(is_property_row(&node));
    assert_eq!(
        property_label_width(
            &node,
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: 360.0,
                height: 28.0,
            },
        ),
        COMPONENT_PROPERTY_LABEL_WIDTH
    );
}
