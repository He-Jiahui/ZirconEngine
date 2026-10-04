#[test]
fn input_data_rejects_style_and_visibility_changes() {
    assert!(validate(&serde_json::json!({"componentInput": {"value": "Player"}})).is_ok());
    for property in ["visibility", "background_color", "text", "rows"] {
        let mut input = Map::new();
        input.insert(property.into(), Value::String("replacement".into()));
        assert!(validate(&serde_json::json!({"componentInput": input})).is_err());
    }
}

#[test]
fn workbench_state_accepts_only_a_source_scoped_identity_selector() {
    assert!(validate(&serde_json::json!({
        "workbenchState": {
            "sourcePath": "zircon_editor/assets/ui/editor/windows/workbench_window.zui",
            "controlId": "WorkbenchViewport",
            "sourceNodeId": "viewport",
            "instancePath": "[]"
        }
    }))
    .is_ok());
    assert!(validate(&serde_json::json!({
        "workbenchState": {"sourcePath": "one.zui", "controlId": "viewport", "state": "selected"}
    }))
    .is_err());
    assert!(validate(&serde_json::json!({
        "workbenchState": {"sourcePath": "one.zui", "controlId": "viewport", "instancePath": ""}
    }))
    .is_err());
    assert!(validate(&serde_json::json!({
        "workbenchState": {"sourcePath": "one.zui", "controlId": "viewport"},
        "componentInput": {"value": "mixed"}
    }))
    .is_err());
}

#[test]
fn workbench_presentation_snapshot_can_stand_without_transient_state() {
    assert!(validate(&serde_json::json!({
        "workbenchPresentation": {"schema": "dev.zircon.editor.workbench-presentation"}
    }))
    .is_ok());
    assert!(validate(&serde_json::json!({
        "workbenchPresentation": [],
    }))
    .is_err());
    assert!(validate(&serde_json::json!({
        "workbenchPresentation": {"schema": "dev.zircon.editor.workbench-presentation"},
        "componentInput": {"value": "mixed"}
    }))
    .is_err());
}

#[test]
fn workbench_scroll_target_is_an_exact_optional_source_scoped_selector() {
    let valid = serde_json::json!({
        "workbenchState": {
            "sourcePath": "zircon_editor/assets/ui/editor/windows/workbench_window.zui",
            "controlId": "ToolbarOverflow",
            "sourceNodeId": "toolbar.overflow",
            "instancePath": "[]",
            "scrollTarget": {
                "sourcePath": "zircon_editor/assets/ui/editor/windows/workbench_window.zui",
                "controlId": "HierarchyTree",
                "sourceNodeId": "hierarchy.tree",
                "instancePath": "[]"
            }
        },
        "workbenchPresentation": {"schema": "dev.zircon.editor.workbench-presentation"}
    });
    assert!(validate(&valid).is_ok());

    for target in [
        serde_json::json!({"sourcePath": "one.zui"}),
        serde_json::json!({"sourcePath": "one.zui", "controlId": "tree", "unexpected": true}),
        serde_json::json!({"sourcePath": "one.zui", "controlId": "tree", "instancePath": ""}),
    ] {
        assert!(validate(&serde_json::json!({
            "workbenchState": {
                "sourcePath": "one.zui",
                "controlId": "overflow",
                "scrollTarget": target
            }
        }))
        .is_err());
    }

    assert!(validate(&serde_json::json!({
        "workbenchState": {"sourcePath": "one.zui", "controlId": "overflow"},
        "workbenchPresentation": []
    }))
    .is_err());
}

#[test]
fn workbench_state_accepts_bounded_authored_text_overrides() {
    let override_value = serde_json::json!({
        "sourcePath": "zircon_editor/assets/ui/editor/windows/workbench_window.zui",
        "sourceNodeId": "inspector.title",
        "instancePath": "[]",
        "controlId": "InspectorTitle",
        "property": "text",
        "value": "属性检查器标题以及较长的中文内容"
    });
    assert!(validate(&serde_json::json!({
        "workbenchState": {
            "sourcePath": "zircon_editor/assets/ui/editor/windows/workbench_window.zui",
            "controlId": "InspectorTitle",
            "textOverrides": [override_value.clone()]
        }
    }))
    .is_ok());

    let mut invalid_override = override_value;
    invalid_override["extra"] = Value::Bool(true);
    assert!(validate(&serde_json::json!({
        "workbenchState": {
            "sourcePath": "one.zui",
            "controlId": "InspectorTitle",
            "textOverrides": [invalid_override]
        }
    }))
    .is_err());

    let oversized = "x".repeat(4097);
    for value in ["\ncontrol", "  ", oversized.as_str()] {
        assert!(validate(&serde_json::json!({
            "workbenchState": {
                "sourcePath": "one.zui",
                "controlId": "InspectorTitle",
                "textOverrides": [{
                    "sourcePath": "one.zui",
                    "sourceNodeId": "inspector.title",
                    "controlId": "InspectorTitle",
                    "property": "text",
                    "value": value
                }]
            }
        }))
        .is_err());
    }
}
