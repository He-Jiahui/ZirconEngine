use super::super::super::template_property_axis_values::{property_axis_values, PropertyAxisValue};

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
