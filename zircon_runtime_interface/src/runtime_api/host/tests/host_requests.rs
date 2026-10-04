use super::*;

#[test]
fn ime_host_request_keeps_its_viewport_and_logical_cursor_space() {
    let viewport = ZrRuntimeViewportHandle::new(7);
    let request = ZrRuntimeImeHostRequestV1::set_cursor_area(ZrRuntimeImeCursorAreaV1::new(
        12.0, 34.0, 2.0, 18.0,
    ))
    .with_target_viewport(viewport);

    assert_eq!(request.target_viewport, Some(viewport));
    assert_eq!(
        request.cursor_area.map(|area| area.coordinate_space),
        Some(ZrRuntimeImeCoordinateSpaceV1::WindowLogical)
    );

    let encoded = serde_json::to_value(&request).expect("serialize IME host request");
    assert_eq!(encoded["target_viewport"], serde_json::json!(7));
    assert_eq!(
        encoded["cursor_area"]["coordinate_space"],
        serde_json::json!("WindowLogical")
    );
}

#[test]
fn ime_host_request_decodes_legacy_payload_without_a_viewport_target() {
    let legacy_request = serde_json::json!({
        "kind": "Enable",
        "cursor_area": null,
        "surrounding_text": null,
    });

    let request: ZrRuntimeImeHostRequestV1 =
        serde_json::from_value(legacy_request).expect("decode legacy IME host request");

    assert_eq!(request.target_viewport, None);
}

#[test]
fn legacy_ime_cursor_area_defaults_to_window_logical_coordinates() {
    let legacy_area = serde_json::json!({
        "x": 12.0,
        "y": 34.0,
        "width": 2.0,
        "height": 18.0,
    });

    let area: ZrRuntimeImeCursorAreaV1 =
        serde_json::from_value(legacy_area).expect("decode legacy IME cursor area");

    assert_eq!(
        area.coordinate_space,
        ZrRuntimeImeCoordinateSpaceV1::WindowLogical
    );
}

#[test]
fn legacy_ime_surrounding_text_defaults_to_no_composition_range() {
    let legacy_text = serde_json::json!({
        "value": "search",
        "cursor": 6,
        "anchor": 0,
    });

    let text: ZrRuntimeImeSurroundingTextV1 =
        serde_json::from_value(legacy_text).expect("decode legacy surrounding text");

    assert_eq!(text.composition_range, None);
}

#[test]
fn project_scene_transition_accepts_only_canonical_project_resource_uris() {
    let request = ZrRuntimeProjectSceneTransitionRequestV1::try_new(
        17,
        "res://scenes/eastbrook_mvp.scene.toml",
        ZrRuntimeProjectSceneTransitionPolicyV1::ReplaceActive,
    )
    .expect("canonical project scene URI");

    assert_eq!(request.request_id, 17);
    assert_eq!(request.scene_uri, "res://scenes/eastbrook_mvp.scene.toml");
    for rejected in [
        "C:/project/scenes/main.scene.toml",
        "res://",
        "res:///absolute.scene.toml",
        "res://scenes/../secret.scene.toml",
        "res://scenes\\main.scene.toml",
        "res://scenes//main.scene.toml",
        "res://scenes/main.scene.toml?variant=debug",
    ] {
        assert!(
            ZrRuntimeProjectSceneTransitionRequestV1::try_new(
                18,
                rejected,
                ZrRuntimeProjectSceneTransitionPolicyV1::ReplaceActive,
            )
            .is_err(),
            "host path or non-canonical URI must be rejected: {rejected}"
        );
    }
}
