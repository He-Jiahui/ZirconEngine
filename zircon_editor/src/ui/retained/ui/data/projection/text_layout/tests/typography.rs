use super::*;

#[test]
fn projected_typography_preserves_explicit_shrink_overflow() {
    let attributes = BTreeMap::from([(
        "overflow".to_string(),
        toml::Value::String("shrink".to_string()),
    )]);

    assert_eq!(
        projected_typography(&attributes, "button").overflow,
        "shrink"
    );
}

#[test]
fn authored_font_metadata_survives_projection_without_family_substitution() {
    let attributes = BTreeMap::from([
        ("font_size".into(), toml::Value::Float(17.0)),
        ("line_height".into(), toml::Value::Float(25.0)),
        (
            "font_family".into(),
            toml::Value::String("Authored Resource Family".into()),
        ),
    ]);
    let projected = projected_typography(&attributes, "input");
    assert_eq!(projected.font_size, 17.0);
    assert_eq!(projected.line_height, 25.0);
    assert_eq!(projected.font_family, "Authored Resource Family");
}

#[test]
fn authored_field_offsets_distinguish_zero_missing_and_invalid_values() {
    let attributes = BTreeMap::from([
        ("layout_padding_left".into(), toml::Value::Float(0.0)),
        ("layout_padding_right".into(), toml::Value::Float(7.0)),
        ("layout_padding_top".into(), toml::Value::Float(-1.0)),
    ]);
    let offsets = super::super::offsets::projected_layout_offsets(&attributes);
    assert_eq!(
        offsets.authored_field_padding,
        [Some(0.0), Some(7.0), None, None]
    );
}

#[test]
fn field_label_width_preserves_logical_zero_and_rejects_invalid_values() {
    for (value, expected) in [(0.0, Some(0.0)), (92.0, Some(92.0)), (-1.0, None)] {
        let attributes = BTreeMap::from([("layout_label_width".into(), toml::Value::Float(value))]);
        let offsets = super::super::offsets::projected_layout_offsets(&attributes);
        assert_eq!(offsets.authored_field_label_width, expected);
    }
    assert_eq!(
        super::super::offsets::projected_layout_offsets(&BTreeMap::new())
            .authored_field_label_width,
        None
    );
}
