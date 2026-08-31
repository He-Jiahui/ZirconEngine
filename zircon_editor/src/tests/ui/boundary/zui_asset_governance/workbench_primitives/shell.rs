use std::collections::BTreeSet;

use zircon_runtime_interface::ui::v2::UiV2AssetKind;

use super::super::support::{editor_asset_root, load_zui_document};
use super::WORKBENCH_SHELL_SURFACE_CONTRACTS;

#[test]
fn workbench_shell_surface_component_assets_keep_bottom_up_composition_contract() {
    let mut offenders = Vec::new();

    for contract in WORKBENCH_SHELL_SURFACE_CONTRACTS {
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
        if component.root != contract.root_node {
            offenders.push(format!(
                "{} component `{}` should use stable root node `{}`",
                path.display(),
                contract.component_name,
                contract.root_node
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
        if root_node.control_id.as_deref() != Some(contract.root_control_id) {
            offenders.push(format!(
                "{} component `{}` root node should expose control id `{}`",
                path.display(),
                contract.component_name,
                contract.root_control_id
            ));
        }
        for required_class in contract.root_classes {
            if !component
                .default_classes
                .iter()
                .any(|candidate| candidate.as_str() == *required_class)
            {
                offenders.push(format!(
                    "{} component `{}` should expose default class `{}`",
                    path.display(),
                    contract.component_name,
                    required_class
                ));
            }
            if !root_node
                .classes
                .iter()
                .any(|candidate| candidate.as_str() == *required_class)
            {
                offenders.push(format!(
                    "{} component `{}` root node should expose class `{}`",
                    path.display(),
                    contract.component_name,
                    required_class
                ));
            }
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

        let widget_imports = document
            .imports
            .widgets
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for required_import in contract.required_widget_imports {
            if !widget_imports.contains(*required_import) {
                offenders.push(format!(
                    "{} component `{}` should import `{}`",
                    path.display(),
                    contract.component_name,
                    required_import
                ));
            }
        }

        let node_components = document
            .nodes
            .values()
            .map(|node| node.component.as_str())
            .collect::<BTreeSet<_>>();
        for required_component in contract.required_mounted_components {
            if !node_components.contains(*required_component) {
                offenders.push(format!(
                    "{} component `{}` should mount `{}` instead of duplicating that lower-level component inline",
                    path.display(),
                    contract.component_name,
                    required_component
                ));
            }
        }

        let control_ids = document
            .nodes
            .values()
            .filter_map(|node| node.control_id.as_deref())
            .collect::<BTreeSet<_>>();
        for required_control_id in contract.required_control_ids {
            if !control_ids.contains(*required_control_id) {
                offenders.push(format!(
                    "{} component `{}` should keep control id `{}`",
                    path.display(),
                    contract.component_name,
                    required_control_id
                ));
            }
        }
    }

    assert!(
        WORKBENCH_SHELL_SURFACE_CONTRACTS.len() >= 7,
        "workbench shell surface contract should cover the activity rail, toolbar, panels, status bar, and main band"
    );
    assert!(
        offenders.is_empty(),
        "workbench shell surface .zui assets must stay componentized and compose lower-level Workbench primitives before module assembly: {offenders:#?}"
    );
}
