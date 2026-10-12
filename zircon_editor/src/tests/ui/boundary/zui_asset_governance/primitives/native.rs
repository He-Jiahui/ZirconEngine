use std::collections::BTreeSet;

use toml::Value;
use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime::ui::v2::UiV2PrototypeStoreFileCache;
use zircon_runtime_interface::ui::v2::UiV2AssetKind;

use super::super::support::{editor_asset_root, load_zui_document};
use super::primitive_contracts::WORKBENCH_PRIMITIVE_CONTRACTS;

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
    let path = editor_asset_root().join("ui/editor/component_showcase.zui");
    let mut cache = UiV2PrototypeStoreFileCache::new();
    let outcome = cache
        .load_store(std::iter::once(path))
        .expect("registered native Showcase must compile through the real cache");
    assert!(outcome.unresolved_imports.is_empty());
    let document = &outcome.root_document;
    assert_eq!(document.asset.kind, UiV2AssetKind::View);
    assert_eq!(document.root_node_id(), Some("component_showcase_root"));
    assert!(
        document.components.is_empty(),
        "Showcase is an independent View"
    );
    let mut mounted_native_components = BTreeSet::new();
    for (asset_id, component_name, root_node) in [
        (
            "res://ui/editor/components/showcase/showcase_visual_section.zui",
            "ShowcaseVisualSection",
            "visual_section_root",
        ),
        (
            "res://ui/editor/components/showcase/showcase_input_section.zui",
            "ShowcaseInputSection",
            "input_section_root",
        ),
        (
            "res://ui/editor/components/showcase/selection_section.zui",
            "ShowcaseSelectionSection",
            "selection_section_root",
        ),
        (
            "res://ui/editor/components/showcase/collections_section.zui",
            "ShowcaseCollectionsSection",
            "collections_section_root",
        ),
    ] {
        let import = format!("{asset_id}#{component_name}");
        assert!(
            document
                .imports
                .widgets
                .iter()
                .any(|candidate| candidate == &import),
            "Showcase must import {import}"
        );
        assert!(
            document
                .nodes
                .values()
                .any(|node| node.component == component_name),
            "Showcase must mount {component_name}"
        );
        let section = outcome
            .store
            .get(asset_id)
            .expect("imported Showcase section must be loaded");
        assert_eq!(section.asset.kind, UiV2AssetKind::Component);
        assert_eq!(section.components.len(), 1);
        assert_eq!(section.components[component_name].root, root_node);
        assert!(section.nodes.contains_key(root_node));
        for node in section.nodes.values() {
            mounted_native_components.insert(node.component.clone());
        }
    }
    let expected = [
        "ArrayField",
        "AssetField",
        "Badge",
        "Button",
        "Checkbox",
        "ColorField",
        "ComboBox",
        "CommandPalette",
        "ConfirmDialog",
        "ContextActionMenu",
        "ContextMenu",
        "Dialog",
        "Dropdown",
        "DropdownPopup",
        "EnumField",
        "FlagsField",
        "Foldout",
        "Group",
        "HelpRow",
        "Icon",
        "IconButton",
        "Image",
        "InputField",
        "InspectorSection",
        "InstanceField",
        "Label",
        "ListRow",
        "MapField",
        "NotificationCenter",
        "NumberField",
        "ObjectField",
        "PagedList",
        "ProgressBar",
        "PropertyRow",
        "Radio",
        "RangeField",
        "RangeSlider",
        "RichLabel",
        "ScrollableBox",
        "SearchSelect",
        "SegmentedControl",
        "Separator",
        "Skeleton",
        "Slider",
        "Spinner",
        "SvgIcon",
        "Tab",
        "TableRow",
        "Tabs",
        "TextField",
        "ToggleButton",
        "TreeRow",
        "Vector2Field",
        "Vector3Field",
        "Vector4Field",
        "VirtualList",
        "WorldSpaceSurface",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(
        mounted_native_components
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        expected,
        "current Showcase must preserve the complete native atom/collection/property sample set"
    );
    let catalog = UiComponentDescriptorRegistry::editor_showcase_shared();
    for component in &mounted_native_components {
        assert!(
            catalog.descriptor(component).is_some(),
            "sample {component} must use the shared native catalog used by surface input"
        );
    }
    assert!(
        mounted_native_components.len() >= 39,
        "the native gallery must retain broad low-level sample coverage"
    );
}
