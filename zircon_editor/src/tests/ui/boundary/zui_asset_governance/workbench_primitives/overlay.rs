use toml::Value;

use super::super::support::{editor_asset_root, load_zui_document};
use super::{DRAG_OVERLAY_REQUIRED_PROPS, WORKBENCH_OVERLAY_PRIMITIVE_CONTRACTS};

#[test]
fn workbench_overlay_primitives_expose_popup_shell_contract() {
    let mut offenders = Vec::new();

    for contract in WORKBENCH_OVERLAY_PRIMITIVE_CONTRACTS {
        let path = editor_asset_root()
            .join("ui/editor/components")
            .join(contract.file_name);
        let document = load_zui_document(&path);
        let Some(component) = document.components.get(contract.component_name) else {
            offenders.push(format!(
                "{} should declare component `{}`",
                path.display(),
                contract.component_name
            ));
            continue;
        };
        let Some(root_node) = document.nodes.get(&component.root) else {
            offenders.push(format!(
                "{} component `{}` references missing root `{}`",
                path.display(),
                contract.component_name,
                component.root
            ));
            continue;
        };

        for prop in contract.required_props {
            if !root_node.props.contains_key(*prop) {
                offenders.push(format!(
                    "{} component `{}` should expose popup-shell prop `{prop}`",
                    path.display(),
                    contract.component_name
                ));
            }
        }
        if root_node.props.get("placement").and_then(Value::as_str) != Some(contract.placement) {
            offenders.push(format!(
                "{} component `{}` should set placement `{}`",
                path.display(),
                contract.component_name,
                contract.placement
            ));
        }
        if root_node
            .props
            .get("close_on_backdrop_click")
            .and_then(Value::as_bool)
            != Some(true)
        {
            offenders.push(format!(
                "{} component `{}` should keep outside-click dismissal enabled",
                path.display(),
                contract.component_name
            ));
        }
        if root_node
            .props
            .get("disable_portal")
            .and_then(Value::as_bool)
            != Some(false)
        {
            offenders.push(format!(
                "{} component `{}` should stay attached to the overlay portal layer",
                path.display(),
                contract.component_name
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "workbench overlay primitives must expose the retained popup shell contract before higher-level surfaces compose them: {offenders:#?}"
    );
}

#[test]
fn notification_center_primitive_leaves_anchor_geometry_to_its_owner() {
    let path = editor_asset_root().join(
        "ui/editor/components/workbench/primitives/feedback/workbench_notification_center.zui",
    );
    let document = load_zui_document(&path);
    let root = document
        .nodes
        .get("root")
        .expect("WorkbenchNotificationCenter root node should exist");

    for property in [
        "popup_anchor_x",
        "popup_anchor_y",
        "popup_anchor_width",
        "popup_anchor_height",
    ] {
        assert!(
            !root.props.contains_key(property),
            "notification center primitive must not own `{property}`"
        );
    }
}

#[test]
fn workbench_drag_overlay_exposes_drag_visual_contract() {
    let path = editor_asset_root()
        .join("ui/editor/components")
        .join("workbench/primitives/feedback/workbench_drag_overlay.zui");
    let document = load_zui_document(&path);
    let component = document
        .components
        .get("WorkbenchDragOverlay")
        .expect("WorkbenchDragOverlay component should be declared");
    let root = document
        .nodes
        .get(&component.root)
        .expect("WorkbenchDragOverlay root node should exist");

    let missing_props = DRAG_OVERLAY_REQUIRED_PROPS
        .iter()
        .filter(|prop| !root.props.contains_key(**prop))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing_props.is_empty(),
        "WorkbenchDragOverlay should expose DragOverlay descriptor props for retained/native projection: {missing_props:#?}"
    );
    assert_eq!(
        root.props.get("payload_kind").and_then(Value::as_str),
        Some("asset")
    );
    assert_eq!(
        root.props
            .get("drop_indicator_edge")
            .and_then(Value::as_str),
        Some("bottom")
    );
    assert_eq!(
        root.props.get("disable_portal").and_then(Value::as_bool),
        Some(false)
    );
}
