use std::collections::BTreeMap;

use toml::Value;

use super::projected_popup_frame;

fn attributes(values: &[(&str, Value)]) -> BTreeMap<String, Value> {
    values
        .iter()
        .map(|(key, value)| ((*key).to_string(), value.clone()))
        .collect()
}

#[test]
fn closed_popup_keeps_authored_frame() {
    let frame = projected_popup_frame(
        &BTreeMap::new(),
        "menu",
        false,
        Some(10.0),
        Some(20.0),
        1.0,
        2.0,
        30.0,
        40.0,
    );

    assert_eq!(frame.x, 1.0);
    assert_eq!(frame.y, 2.0);
    assert_eq!(frame.width, 30.0);
    assert_eq!(frame.height, 40.0);
}

#[test]
fn menu_popup_uses_default_anchor_and_transform_origins() {
    let attrs = attributes(&[
        ("popup_anchor_width", Value::Float(100.0)),
        ("popup_anchor_height", Value::Float(30.0)),
    ]);

    let frame = projected_popup_frame(
        &attrs,
        "menu",
        true,
        Some(10.0),
        Some(20.0),
        0.0,
        0.0,
        40.0,
        50.0,
    );

    assert_eq!(frame.x, 10.0);
    assert_eq!(frame.y, 50.0);
}

#[test]
fn tooltip_popper_placement_applies_gap_and_offsets() {
    let attrs = attributes(&[
        ("placement", Value::String("top-start".to_string())),
        ("popup_anchor_width", Value::Float(100.0)),
        ("popup_anchor_height", Value::Float(20.0)),
        ("popup_offset_x", Value::Float(4.0)),
        ("popup_offset_y", Value::Float(-2.0)),
    ]);

    let frame = projected_popup_frame(
        &attrs,
        "tooltip",
        true,
        Some(50.0),
        Some(80.0),
        0.0,
        0.0,
        40.0,
        10.0,
    );

    assert_eq!(frame.x, 54.0);
    assert_eq!(frame.y, 60.0);
}
