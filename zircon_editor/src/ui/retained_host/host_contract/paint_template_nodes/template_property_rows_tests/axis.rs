use super::super::super::template_property_axis_values::{PropertyAxisValue, property_axis_values};
use super::super::layout::{
    axis_field_rect, axis_label_rect, label_text_rect, property_label_width,
    property_value_area_rect, scalar_field_rect, value_text_rect,
};
use super::support::frame;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

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
fn axis_slots_remain_inside_a_narrow_value_area() {
    let value_area = frame(48.0, 12.0, 18.0, 16.0);

    for count in 2..=4 {
        for index in 0..count {
            assert_rect_is_contained_by(axis_label_rect(&value_area, count, index), &value_area);
            assert_rect_is_contained_by(axis_field_rect(&value_area, count, index), &value_area);
        }
    }
}

#[test]
fn regular_axis_slots_keep_the_authored_label_and_gap_density() {
    let value_area = frame(0.0, 0.0, 175.0, 28.0);
    let label = axis_label_rect(&value_area, 3, 0);
    let field = axis_field_rect(&value_area, 3, 0);

    assert_eq!(label.width, 12.0);
    assert_eq!(field.x, 16.0);
    assert!((field.width - 38.333_332).abs() < 0.000_1);
}

#[test]
fn property_row_text_and_scalar_field_stay_inside_a_short_row() {
    let row = frame(0.0, 0.0, 18.0, 4.0);
    let value_area = property_value_area_rect(&row, 8.0);
    let scalar_field = scalar_field_rect(&value_area);

    assert_rect_is_contained_by(label_text_rect(&row, 8.0), &row);
    assert_rect_is_contained_by(value_area, &row);
    assert_rect_is_contained_by(scalar_field.clone(), &value_area);
    assert_rect_is_contained_by(value_text_rect(&scalar_field), &scalar_field);
}

#[test]
fn regular_property_row_insets_keep_the_authored_density() {
    let row = frame(12.0, 16.0, 360.0, 28.0);
    let label = label_text_rect(&row, 105.0);
    let value_area = property_value_area_rect(&row, 105.0);
    let scalar_field = scalar_field_rect(&value_area);
    let value = value_text_rect(&scalar_field);

    assert_eq!(label, frame(17.0, 20.0, 97.5, 20.0));
    assert_eq!(scalar_field, frame(117.0, 19.0, 250.0, 22.0));
    assert_eq!(value, frame(122.0, 23.0, 240.0, 14.0));
}

#[test]
fn property_label_and_value_area_stay_inside_a_subpixel_row() {
    let row = frame(4.0, 8.0, 0.5, 4.0);
    let label_width = property_label_width(&TemplatePaneNodeData::default(), &row);
    let value_area = property_value_area_rect(&row, label_width);

    assert_rect_is_contained_by(label_text_rect(&row, label_width), &row);
    assert_rect_is_contained_by(value_area, &row);
}

fn assert_rect_is_contained_by(rect: FrameRect, parent: &FrameRect) {
    let epsilon = 0.000_1;
    assert!(
        rect.x >= parent.x - epsilon,
        "rect starts before its parent"
    );
    assert!(rect.y >= parent.y - epsilon, "rect starts above its parent");
    assert!(
        rect.x + rect.width <= parent.x + parent.width + epsilon,
        "rect exceeds its parent's right edge"
    );
    assert!(
        rect.y + rect.height <= parent.y + parent.height + epsilon,
        "rect exceeds its parent's bottom edge"
    );
}
