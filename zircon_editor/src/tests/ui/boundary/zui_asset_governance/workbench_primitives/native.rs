use std::collections::BTreeSet;

use toml::Value;
use zircon_runtime_interface::ui::v2::UiV2AssetKind;

use super::super::support::{editor_asset_root, load_zui_document};
use super::WORKBENCH_PRIMITIVE_CONTRACTS;

#[test]
fn workbench_primitive_component_assets_keep_native_component_contract() {
    let mut offenders = Vec::new();

    for contract in WORKBENCH_PRIMITIVE_CONTRACTS {
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

        if document.asset.kind != UiV2AssetKind::Component {
            offenders.push(format!(
                "{} should remain a component asset",
                path.display()
            ));
        }
        if component.root != "root" {
            offenders.push(format!(
                "{} component `{}` should use the stable `root` node",
                path.display(),
                contract.component_name
            ));
        }
        if root_node.component != contract.root_component {
            offenders.push(format!(
                "{} component `{}` root node should render `{}` but renders `{}`",
                path.display(),
                contract.component_name,
                contract.root_component,
                root_node.component
            ));
        }
        let expected_control_id = format!("{}Root", contract.component_name);
        if root_node.control_id.as_deref() != Some(expected_control_id.as_str()) {
            offenders.push(format!(
                "{} component `{}` root node should expose control id `{}Root`",
                path.display(),
                contract.component_name,
                contract.component_name
            ));
        }
        if !component
            .default_classes
            .iter()
            .any(|class| class == "workbench-primitive")
        {
            offenders.push(format!(
                "{} component `{}` should expose the shared `workbench-primitive` class",
                path.display(),
                contract.component_name
            ));
        }
        if root_node
            .layout
            .as_ref()
            .is_none_or(|layout| !layout.contains_key("width") || !layout.contains_key("height"))
        {
            offenders.push(format!(
                "{} component `{}` root node should declare width and height layout contracts",
                path.display(),
                contract.component_name
            ));
        }
        if contract.interactive {
            for prop in [
                "input_interactive",
                "input_clickable",
                "input_hoverable",
                "input_focusable",
            ] {
                if root_node.props.get(prop).and_then(Value::as_bool) != Some(true) {
                    offenders.push(format!(
                        "{} component `{}` root node should set `{prop} = true`",
                        path.display(),
                        contract.component_name
                    ));
                }
            }
        }
    }

    assert!(
        WORKBENCH_PRIMITIVE_CONTRACTS.len() >= 39,
        "workbench primitive contract should cover the low-level atom/collection/property/shell-leaf set"
    );
    assert!(
        offenders.is_empty(),
        "workbench primitive .zui component assets must stay componentized, layout-explicit, and input-ready before module assembly: {offenders:#?}"
    );
}

#[test]
fn workbench_component_drawer_composes_workbench_primitive_assets() {
    let path = editor_asset_root()
        .join("ui/editor/components")
        .join("workbench/shell/workbench_component_drawer.zui");
    let document = load_zui_document(&path);
    let widget_imports = document
        .imports
        .widgets
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let node_components = document
        .nodes
        .values()
        .map(|node| node.component.as_str())
        .collect::<BTreeSet<_>>();

    let mut offenders = Vec::new();
    for contract in WORKBENCH_PRIMITIVE_CONTRACTS
        .iter()
        .filter(|contract| contract.sampled_in_component_drawer)
    {
        let import = format!(
            "res://ui/editor/components/{}#{}",
            contract.file_name, contract.component_name
        );
        if !widget_imports.contains(import.as_str()) {
            offenders.push(format!(
                "{} should import `{}` for componentized low-level samples",
                path.display(),
                import
            ));
        }
        if !node_components.contains(contract.component_name) {
            offenders.push(format!(
                "{} should mount `{}` instead of duplicating its structure inline",
                path.display(),
                contract.component_name
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "workbench component drawer must keep composing low-level Workbench primitives through .zui imports: {offenders:#?}"
    );
}
