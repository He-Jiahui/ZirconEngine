// 核对 UE 主窗口真实输入、选择和反馈控件的宿主投影。
use super::*;

#[test]
fn componentized_workbench_window_template_bridge_projects_input_selection_and_feedback_controls() {
    let _guard = env_lock().lock().unwrap();
    let (bridge, nodes) = componentized_workbench_projection_fixture();
    let search = template_contract_node(&nodes, "WorkbenchSceneSearchField");
    assert_eq!(search.role.as_str(), "SearchField");
    assert_eq!(search.search_query.as_str(), "");
    assert!(!search.disabled);
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneSearchField", "placeholder").as_deref(),
        Some("Search")
    );

    let dropdowns: &[(&str, &str, &[(&str, &str, bool)])] = &[
        (
            "SetProjectionMode",
            "Perspective",
            &[
                ("Perspective", "Perspective", true),
                ("Orthographic", "Orthographic", false),
            ],
        ),
        (
            "SetDisplayMode",
            "Lit",
            &[
                ("Shaded", "Lit", true),
                ("WireOverlay", "Wireframe Overlay", false),
                ("WireOnly", "Wireframe", false),
            ],
        ),
        (
            "SetTransformSpace",
            "Local",
            &[("Local", "Local", true), ("Global", "World", false)],
        ),
    ];
    for (control_id, value, options) in dropdowns {
        let dropdown = template_contract_node(&nodes, control_id);
        assert_eq!(dropdown.role.as_str(), "Dropdown");
        assert_eq!(dropdown.component_role.as_str(), "dropdown");
        assert_eq!(dropdown.value_text.as_str(), *value);
        assert_eq!(dropdown.selection_state.as_str(), "single");
        assert!(!dropdown.popup_open);
        assert!(!dropdown.disabled);
        assert_eq!(dropdown.structured_options.row_count(), options.len());
        for (row, (id, label, selected)) in options.iter().enumerate() {
            let option = template_contract_option(&dropdown.structured_options, row);
            assert_eq!(option.id.as_str(), *id);
            assert_eq!(option.label.as_str(), *label);
            assert_eq!(option.selected, *selected);
            assert!(!option.disabled);
        }
    }

    for (control_id, selected) in [
        ("TranslateMode", true),
        ("RotateMode", false),
        ("ToggleTranslateSnap", true),
    ] {
        let button = template_contract_node(&nodes, control_id);
        assert_eq!(button.role.as_str(), "IconButton");
        assert_eq!(button.component_role.as_str(), "icon-button");
        assert_eq!(button.selected, selected);
        assert_eq!(button.checked, selected);
        assert!(!button.disabled);
    }
    for (control_id, text) in [
        ("WorkbenchStatusReady", "Ready"),
        ("WorkbenchStatusErrors", "0 Errors"),
    ] {
        let status = template_contract_node(&nodes, control_id);
        assert_eq!(status.role.as_str(), "Label");
        assert_eq!(status.text.as_str(), text);
    }
    for control_id in [
        "WorkbenchSceneSearchField",
        "SetProjectionMode",
        "SetDisplayMode",
        "SetTransformSpace",
        "TranslateMode",
        "RotateMode",
        "ToggleTranslateSnap",
        "WorkbenchStatusReady",
        "WorkbenchStatusErrors",
    ] {
        let frame = &template_contract_node(&nodes, control_id).frame;
        assert!(frame.x.is_finite() && frame.y.is_finite());
        assert!(frame.width.is_finite() && frame.width > 0.0);
        assert!(frame.height.is_finite() && frame.height > 0.0);
    }

    // Closed popups belong to the live surface and are excluded from visible paint rows.
    let context_menu = bridge
        .host_projection()
        .node_by_control_id("WorkbenchContextMenu")
        .expect("UE context menu must be mounted");
    assert_eq!(context_menu.component, "ContextMenu");
    assert!(!context_menu.popup_open);
    assert!(!control_bool(&bridge, "WorkbenchContextMenu", "open"));
    assert!(context_menu.options.is_empty());
    assert!(context_menu.menu_items.is_empty());
    assert_eq!(
        control_string(&bridge, "WorkbenchContextMenu", "context_target").as_deref(),
        Some("")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchContextMenu", "context_target_path").as_deref(),
        Some("")
    );
}
