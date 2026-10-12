use std::collections::BTreeSet;

use super::super::support::{editor_asset_root, load_zui_document};
use super::shell_contracts::WORKBENCH_SHELL_SURFACE_CONTRACTS;

#[test]
fn workbench_shell_surface_component_assets_keep_bottom_up_composition_contract() {
    let mut offenders = Vec::new();

    for contract in WORKBENCH_SHELL_SURFACE_CONTRACTS {
        let path = editor_asset_root()
            .join("ui/editor")
            .join(contract.file_name);
        let document = load_zui_document(&path);
        if document.asset.kind != contract.asset_kind {
            offenders.push(format!(
                "{} should remain a {:?} asset",
                path.display(),
                contract.asset_kind
            ));
        }
        let component = contract.component_name.map(|name| {
            document
                .components
                .get(name)
                .unwrap_or_else(|| panic!("{} should declare component {name}", path.display()))
        });
        if let Some(component) = component {
            if component.root != contract.root_node {
                offenders.push(format!(
                    "{} component root changed from {} to {}",
                    path.display(),
                    contract.root_node,
                    component.root
                ));
            }
        } else {
            assert!(
                document.components.is_empty(),
                "{} must remain a View document",
                path.display()
            );
            assert_eq!(
                document.root_node_id(),
                contract.document_root,
                "{} canonical document root changed",
                path.display()
            );
        }
        let Some(root_node) = document.nodes.get(contract.root_node) else {
            offenders.push(format!(
                "{} references missing root {}",
                path.display(),
                contract.root_node
            ));
            continue;
        };

        if root_node.component != contract.root_component {
            offenders.push(format!(
                "{} component `{:?}` root node should render `{}` but renders `{}`",
                path.display(),
                contract.component_name,
                contract.root_component,
                root_node.component
            ));
        }
        if root_node.control_id.as_deref() != Some(contract.root_control_id) {
            offenders.push(format!(
                "{} component `{:?}` root node should expose control id `{}`",
                path.display(),
                contract.component_name,
                contract.root_control_id
            ));
        }
        for required_class in contract.root_classes {
            if component.is_some_and(|component| {
                !component
                    .default_classes
                    .iter()
                    .any(|candidate| candidate.as_str() == *required_class)
            }) {
                offenders.push(format!(
                    "{} component `{:?}` should expose default class `{}`",
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
                    "{} component `{:?}` root node should expose class `{}`",
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
                "{} component `{:?}` root node should declare width and height layout contracts",
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
                    "{} component `{:?}` should import `{}`",
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
                    "{} component `{:?}` should mount `{}` instead of duplicating that lower-level component inline",
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
        if contract.root_node == "main_band" {
            assert!(
                document.nodes["editor_content"]
                    .children
                    .iter()
                    .any(|child| child.node == "main_band"),
                "canonical window must mount its main_band node"
            );
        }
        for required_control_id in contract.required_control_ids {
            if !control_ids.contains(*required_control_id) {
                offenders.push(format!(
                    "{} component `{:?}` should keep control id `{}`",
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
        "workbench surfaces must keep real View/Component roots, current native/component composition, and control identities: {offenders:#?}"
    );
}
