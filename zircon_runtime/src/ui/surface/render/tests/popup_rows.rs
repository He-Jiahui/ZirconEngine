use super::*;

#[test]
fn popup_surface_and_rows_keep_distinct_radius_tiers() {
    let palette = popup_row_palette();
    let controls = EditorDesignTokens::workbench_dark().controls;
    let metadata = UiTemplateNodeMetadata {
        style_overrides: toml::from_str("corner_radius = 14.0").unwrap(),
        ..Default::default()
    };

    assert_eq!(palette.panel_radius, controls.panel_radius);
    assert_eq!(palette.row_radius, controls.small_radius);

    let mut commands = Vec::new();
    push_popup_background(
        &mut commands,
        UiNodeId(1),
        &metadata,
        UiFrame::new(0.0, 0.0, 160.0, 120.0),
        None,
        0,
        1.0,
    );

    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].z_index, -2);
    assert_eq!(commands[1].z_index, -1);
    assert_eq!(commands[2].z_index, 0);
    assert_eq!(commands[2].style.corner_radius, 14.0);
    assert_eq!(commands[0].frame, UiFrame::new(-2.0, 0.0, 164.0, 124.0));
    assert_eq!(commands[1].frame, UiFrame::new(-1.0, 0.0, 162.0, 122.0));
}

#[test]
fn popup_shadow_layers_stay_behind_the_authoritative_background() {
    let frame = UiFrame::new(10.0, 20.0, 160.0, 120.0);

    assert_eq!(
        popup_shadow_frame(frame, 2.0, 2.0),
        UiFrame::new(8.0, 20.0, 164.0, 124.0)
    );
    assert_eq!(
        popup_shadow_frame(frame, 1.0, 1.0),
        UiFrame::new(9.0, 20.0, 162.0, 122.0)
    );
}

#[test]
fn popup_attribute_id_set_borrows_small_inputs_and_indexes_large_inputs() {
    let small = toml::Value::Array(vec![
        toml::Value::String("first".to_string()),
        toml::Value::String("second".to_string()),
    ]);
    let small_set = PopupAttributeIdSet::new(Some(&small));
    assert!(matches!(small_set, PopupAttributeIdSet::Linear(_)));
    assert!(small_set.contains("second"));
    assert!(!small_set.contains("missing"));

    let large = toml::Value::Array(
        (0..=POPUP_ATTRIBUTE_LINEAR_SCAN_LIMIT)
            .map(|index| toml::Value::String(format!("item-{index}")))
            .collect(),
    );
    let large_set = PopupAttributeIdSet::new(Some(&large));
    assert!(matches!(large_set, PopupAttributeIdSet::Indexed(_)));
    assert!(large_set.contains("item-8"));
    assert!(!large_set.contains("missing"));
}

#[test]
fn popup_row_frame_consumes_authored_padding_and_spacing() {
    let metadata = UiTemplateNodeMetadata {
        attributes: toml::from_str(
            r#"
layout_padding_left = 8.0
layout_padding_right = 8.0
layout_padding_top = 4.0
layout_padding_bottom = 4.0
layout_spacing = 4.0
"#,
        )
        .unwrap(),
        ..Default::default()
    };
    let popup = UiFrame::new(10.0, 20.0, 190.0, 148.0);

    assert_frame_near(
        popup_row_frame(&metadata, popup, 5, 0).expect("first row"),
        UiFrame::new(18.0, 24.0, 174.0, 24.8),
    );
    assert_frame_near(
        popup_row_frame(&metadata, popup, 5, 4).expect("last row"),
        UiFrame::new(18.0, 139.2, 174.0, 24.8),
    );
    assert!((menu_row_height(&metadata, popup, 5).unwrap() - 24.8).abs() <= 0.001);
    assert!((popup_rows_height(&metadata, 5, 24.8).unwrap() - 148.0).abs() <= 0.001);
}

fn assert_frame_near(actual: UiFrame, expected: UiFrame) {
    for (actual, expected) in [
        (actual.x, expected.x),
        (actual.y, expected.y),
        (actual.width, expected.width),
        (actual.height, expected.height),
    ] {
        assert!((actual - expected).abs() <= 0.001, "{actual} != {expected}");
    }
}
