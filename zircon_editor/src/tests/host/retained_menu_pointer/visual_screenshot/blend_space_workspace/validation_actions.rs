use super::*;

use crate::ui::binding::EditorUiBindingPayload;
use zircon_runtime_interface::ui::binding::UiEventKind;

const VALIDATION_ACTIONS: &[(&str, &str)] = &[
    (
        "WorkbenchValidationLogAll",
        "workbench.extension.blend_space.validation.filter_all",
    ),
    (
        "WorkbenchValidationLogErrors",
        "workbench.extension.blend_space.validation.filter_errors",
    ),
    (
        "WorkbenchValidationLogWarnings",
        "workbench.extension.blend_space.validation.filter_warnings",
    ),
    (
        "WorkbenchValidationLogInfos",
        "workbench.extension.blend_space.validation.filter_infos",
    ),
    (
        "WorkbenchValidationLogClear",
        "workbench.extension.blend_space.validation.clear",
    ),
];

const DIAGNOSTIC_ROWS: &[&str] = &[
    "WorkbenchValidationLogInfoAxesRow",
    "WorkbenchValidationLogWarningRow",
    "WorkbenchValidationLogInfoRangeRow",
    "WorkbenchValidationLogInfoDuplicatesRow",
];

#[test]
fn blend_space_validation_controls_dispatch_unique_registered_actions() {
    let mut bridge = open_blend_space_bridge(1260, 780);
    let mut routes = std::collections::BTreeSet::new();

    for (control_id, expected_action) in VALIDATION_ACTIONS {
        let binding = bridge
            .dispatch_control_state(control_id, UiEventKind::Click)
            .unwrap_or_else(|error| panic!("{control_id} should dispatch without error: {error}"))
            .unwrap_or_else(|| panic!("{control_id} should expose its Click binding"));
        assert!(matches!(
            binding.payload(),
            EditorUiBindingPayload::MenuAction { action_id } if action_id == expected_action
        ));
        assert!(
            routes.insert(*expected_action),
            "duplicate validation route"
        );
    }
}

#[test]
fn blend_space_validation_filters_keep_rows_and_selection_in_sync() {
    let mut bridge = open_blend_space_bridge(1260, 780);

    dispatch(&mut bridge, "WorkbenchValidationLogWarnings");
    assert!(selected(&bridge, "WorkbenchValidationLogWarnings"));
    assert!(!selected(&bridge, "WorkbenchValidationLogAll"));
    assert!(visible(&bridge, "WorkbenchValidationLogWarningRow"));
    for row_id in [
        "WorkbenchValidationLogInfoAxesRow",
        "WorkbenchValidationLogInfoRangeRow",
        "WorkbenchValidationLogInfoDuplicatesRow",
    ] {
        assert!(!visible(&bridge, row_id), "{row_id} should be filtered out");
    }

    dispatch(&mut bridge, "WorkbenchValidationLogInfos");
    assert!(selected(&bridge, "WorkbenchValidationLogInfos"));
    assert!(!selected(&bridge, "WorkbenchValidationLogWarnings"));
    assert!(visible(&bridge, "WorkbenchValidationLogInfoAxesRow"));
    assert!(visible(&bridge, "WorkbenchValidationLogInfoRangeRow"));
    assert!(visible(&bridge, "WorkbenchValidationLogInfoDuplicatesRow"));
    assert!(!visible(&bridge, "WorkbenchValidationLogWarningRow"));

    dispatch(&mut bridge, "WorkbenchValidationLogErrors");
    assert!(selected(&bridge, "WorkbenchValidationLogErrors"));
    for row_id in DIAGNOSTIC_ROWS {
        assert!(
            !visible(&bridge, row_id),
            "{row_id} should be empty for 0 errors"
        );
    }

    dispatch(&mut bridge, "WorkbenchValidationLogAll");
    assert!(selected(&bridge, "WorkbenchValidationLogAll"));
    for row_id in DIAGNOSTIC_ROWS {
        assert!(
            visible(&bridge, row_id),
            "{row_id} should return in All mode"
        );
    }

    dispatch(&mut bridge, "WorkbenchValidationLogClear");
    for row_id in DIAGNOSTIC_ROWS {
        assert!(!visible(&bridge, row_id), "{row_id} should be cleared");
    }
    assert_eq!(
        text(&bridge, "WorkbenchStatusReady"),
        "Validation log cleared"
    );
}

fn dispatch(bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge, control_id: &str) {
    bridge
        .dispatch_control_state(control_id, UiEventKind::Click)
        .unwrap_or_else(|error| panic!("{control_id} should dispatch without error: {error}"))
        .unwrap_or_else(|| panic!("{control_id} should expose its Click binding"));
}

fn selected(bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge, control_id: &str) -> bool {
    bool_property(bridge, control_id, "selected")
}

fn visible(bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge, control_id: &str) -> bool {
    bridge.control_frame(control_id).is_some()
}

fn bool_property(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> bool {
    bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
                .and_then(|metadata| metadata.attributes.get(property))
                .and_then(toml::Value::as_bool)
        })
        .unwrap_or(false)
}

fn text(bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge, control_id: &str) -> String {
    bridge
        .host_projection()
        .node_by_control_id(control_id)
        .and_then(|node| node.text.as_deref())
        .unwrap_or_default()
        .to_string()
}
