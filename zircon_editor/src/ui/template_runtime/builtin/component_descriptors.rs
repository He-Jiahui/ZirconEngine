use crate::ui::template::{
    EditorComponentCatalogManifestError, EditorComponentDescriptor,
    parse_editor_component_catalog_manifest,
};

pub(crate) const BUILTIN_COMPONENT_CATALOG_MANIFEST_ID: &str =
    "res://ui/editor/components/catalog.toml";
const BUILTIN_COMPONENT_CATALOG_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/components/catalog.toml"
));

pub(crate) fn builtin_component_descriptors()
-> Result<Vec<EditorComponentDescriptor>, EditorComponentCatalogManifestError> {
    parse_editor_component_catalog_manifest(BUILTIN_COMPONENT_CATALOG_MANIFEST)
}

#[cfg(test)]
fn builtin_component_descriptors_for_tests() -> Vec<EditorComponentDescriptor> {
    builtin_component_descriptors().expect("builtin component catalog asset should remain valid")
}

#[cfg(test)]
fn primitive_root_prop_default(
    document_id: &str,
    property_name: &str,
) -> crate::ui::template::EditorPropLiteral {
    let relative_path = document_id
        .strip_prefix("res://")
        .expect("builtin primitive documents should use res:// identifiers");
    let source_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(relative_path);
    let source = std::fs::read_to_string(&source_path)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", source_path.display()));
    let document: toml::Value = toml::from_str(&source)
        .unwrap_or_else(|error| panic!("could not parse {}: {error}", source_path.display()));
    let property = document
        .get("nodes")
        .and_then(toml::Value::as_table)
        .and_then(|nodes| nodes.get("root"))
        .and_then(toml::Value::as_table)
        .and_then(|root| root.get("props"))
        .and_then(toml::Value::as_table)
        .and_then(|props| props.get(property_name))
        .unwrap_or_else(|| {
            panic!(
                "{} should expose root prop `{property_name}`",
                source_path.display()
            )
        });
    match property {
        toml::Value::String(value) => crate::ui::template::EditorPropLiteral::Text(value.clone()),
        toml::Value::Boolean(value) => crate::ui::template::EditorPropLiteral::Boolean(*value),
        toml::Value::Integer(value) => crate::ui::template::EditorPropLiteral::Integer(*value),
        toml::Value::Float(value) => crate::ui::template::EditorPropLiteral::Float(*value),
        toml::Value::Array(values) => crate::ui::template::EditorPropLiteral::TextList(
            values
                .iter()
                .map(|value| {
                    value.as_str().unwrap_or_else(|| {
                        panic!(
                            "{} root prop `{property_name}` must contain only text values",
                            source_path.display()
                        )
                    })
                })
                .map(str::to_string)
                .collect(),
        ),
        _ => panic!(
            "{} root prop `{property_name}` must be a supported literal default",
            source_path.display()
        ),
    }
}

#[cfg(test)]
#[path = "component_descriptors/dialog_contract_tests.rs"]
mod dialog_contract_tests;

#[cfg(test)]
#[path = "component_descriptors/feedback_container_contract_tests.rs"]
mod feedback_container_contract_tests;

#[cfg(test)]
#[path = "component_descriptors/feedback_state_contract_tests.rs"]
mod feedback_state_contract_tests;

#[cfg(test)]
#[path = "component_descriptors/tooltip_contract_tests.rs"]
mod tooltip_contract_tests;

#[cfg(test)]
mod tests {
    use super::{
        BUILTIN_COMPONENT_CATALOG_MANIFEST_ID, builtin_component_descriptors_for_tests,
        primitive_root_prop_default,
    };
    use crate::ui::template::{EditorComponentTier, EditorPropDefault, EditorPropLiteral};
    use std::{
        collections::BTreeSet,
        fs,
        path::{Path, PathBuf},
    };
    use zircon_runtime_interface::ui::layout::UiSlotKind;

    fn descriptors() -> Vec<crate::ui::template::EditorComponentDescriptor> {
        builtin_component_descriptors_for_tests()
    }

    #[test]
    fn builtin_editor_components_preserve_composite_and_region_panel_boundaries() {
        let descriptors = descriptors();
        let component_ids = descriptors
            .iter()
            .map(|descriptor| descriptor.component_id.as_str())
            .collect::<BTreeSet<_>>();
        for component_id in [
            "UiHostWindow",
            "ActivityDrawerWindow",
            "MenuBar",
            "ActivityRail",
            "DocumentHost",
            "StatusBar",
            "SceneViewportToolbar",
            "AssetSurfaceControls",
            "WelcomeSurfaceControls",
            "InspectorSurfaceControls",
            "PaneSurfaceControls",
            "ConsolePaneBody",
            "InspectorPaneBody",
            "HierarchyPaneBody",
            "AnimationSequencePaneBody",
            "AnimationGraphPaneBody",
            "RuntimeDiagnosticsPaneBody",
            "PerformanceTimelinePaneBody",
            "ModulePluginsPaneBody",
            "BuildExportPaneBody",
            "GeneratedBottomPaneBody",
        ] {
            assert!(
                component_ids.contains(component_id),
                "the asset-backed catalog must preserve builtin component `{component_id}`"
            );
        }
        let tier = |component_id: &str| {
            descriptors
                .iter()
                .find(|descriptor| descriptor.component_id == component_id)
                .map(|descriptor| descriptor.tier)
        };

        assert_eq!(tier("UiHostWindow"), Some(EditorComponentTier::RegionPanel));
        assert_eq!(
            tier("SceneViewportToolbar"),
            Some(EditorComponentTier::Composite)
        );
        assert_eq!(
            tier("InspectorPaneBody"),
            Some(EditorComponentTier::RegionPanel)
        );

        let menu_bar = descriptors
            .iter()
            .find(|descriptor| descriptor.component_id == "MenuBar")
            .expect("MenuBar should remain in the builtin component catalog");
        assert_eq!(menu_bar.slots.len(), 1);
        assert_eq!(menu_bar.slots[0].name, "actions");
        assert_eq!(menu_bar.slots[0].kind, UiSlotKind::Linear);
        assert!(menu_bar.slots[0].multiple);
        assert!(menu_bar.slots[0].accepts.contains("WorkbenchIconButton"));

        let document_host = descriptors
            .iter()
            .find(|descriptor| descriptor.component_id == "DocumentHost")
            .expect("DocumentHost should remain in the builtin component catalog");
        assert!(document_host.slots.iter().any(|slot| {
            slot.name == "tabs" && slot.required && slot.accepts.contains("DocumentTabs")
        }));
        assert!(document_host.slots.iter().any(|slot| {
            slot.name == "content" && slot.required && slot.accepts.contains("PaneSurface")
        }));

        let activity_drawer = descriptors
            .iter()
            .find(|descriptor| descriptor.component_id == "ActivityDrawerWindow")
            .expect("ActivityDrawerWindow should be discoverable as a region component");
        assert_eq!(activity_drawer.tier, EditorComponentTier::RegionPanel);
        assert_eq!(activity_drawer.slots.len(), 7);
        for slot_name in [
            "left_top_activity",
            "left_bottom_activity",
            "right_top_activity",
            "right_bottom_activity",
            "bottom_left_activity",
            "bottom_right_activity",
            "content",
        ] {
            let slot = activity_drawer
                .slots
                .iter()
                .find(|slot| slot.name == slot_name)
                .unwrap_or_else(|| panic!("activity drawer catalog should expose `{slot_name}`"));
            assert_eq!(slot.kind, UiSlotKind::Container);
            assert!(slot.multiple);
            assert_eq!(slot.accepts.len(), 1);
            assert!(slot.accepts.contains("Container"));
        }

        let panel_header = descriptors
            .iter()
            .find(|descriptor| descriptor.component_id == "WorkbenchPanelHeader")
            .expect("WorkbenchPanelHeader should remain discoverable as a shared composite");
        assert_eq!(panel_header.tier, EditorComponentTier::Composite);
        assert!(panel_header.slots.iter().any(|slot| {
            slot.name == "title"
                && slot.kind == UiSlotKind::Linear
                && slot.required
                && !slot.multiple
                && slot.accepts.contains("WorkbenchCaption")
                && slot.accepts.contains("WorkbenchSectionTitle")
        }));
        assert!(panel_header.slots.iter().any(|slot| {
            slot.name == "actions"
                && slot.kind == UiSlotKind::Linear
                && !slot.required
                && slot.multiple
                && slot.accepts.contains("WorkbenchCaption")
                && slot.accepts.contains("WorkbenchIconButton")
        }));

        let property_editor_row = descriptors
            .iter()
            .find(|descriptor| descriptor.component_id == "WorkbenchPropertyEditorRow")
            .expect("WorkbenchPropertyEditorRow should remain discoverable as a shared composite");
        assert_eq!(property_editor_row.tier, EditorComponentTier::Composite);
        assert_eq!(property_editor_row.slots.len(), 1);
        let value_slot = &property_editor_row.slots[0];
        assert_eq!(value_slot.name, "value");
        assert_eq!(value_slot.kind, UiSlotKind::Linear);
        assert!(value_slot.required);
        assert!(!value_slot.multiple);
        for component in ["WorkbenchDropdown", "WorkbenchField", "WorkbenchToggle"] {
            assert!(
                value_slot.accepts.contains(component),
                "property editor value slot should accept `{component}`"
            );
        }

        let tab_strip = descriptors
            .iter()
            .find(|descriptor| descriptor.component_id == "WorkbenchTabStrip")
            .expect("WorkbenchTabStrip should remain discoverable as an interactive primitive");
        assert_eq!(tab_strip.tier, EditorComponentTier::Primitive);
        assert_eq!(tab_strip.slots.len(), 1);
        let default_slot = &tab_strip.slots[0];
        assert_eq!(default_slot.name, "default");
        assert_eq!(default_slot.kind, UiSlotKind::Linear);
        assert!(!default_slot.required);
        assert!(default_slot.multiple);
        assert_eq!(default_slot.accepts.len(), 1);
        assert!(default_slot.accepts.contains("WorkbenchTab"));
    }

    #[test]
    fn builtin_catalog_exposes_core_interaction_parameters() {
        let descriptors = descriptors();
        let default_for = |component_id: &str, property_name: &str| {
            descriptors
                .iter()
                .find(|descriptor| descriptor.component_id == component_id)
                .and_then(|descriptor| {
                    descriptor
                        .props
                        .iter()
                        .find(|property| property.name == property_name)
                })
                .map(|property| {
                    (
                        descriptor.document_id.as_str(),
                        &property.value_type,
                        &property.default,
                    )
                })
                .unwrap_or_else(|| {
                    panic!("builtin catalog should expose `{property_name}` on `{component_id}`")
                })
        };

        for (component_id, property_name, value_type, default) in [
            ("WorkbenchAlert", "text", "text", "Unsaved scene changes"),
            ("WorkbenchAlert", "severity", "enum", "warning"),
            ("WorkbenchAlert", "variant", "enum", "outlined"),
            ("WorkbenchCaption", "text", "text", "Caption"),
            ("WorkbenchChip", "text", "text", "Chip"),
            ("WorkbenchDivider", "orientation", "enum", "horizontal"),
            ("WorkbenchDivider", "variant", "enum", "fullWidth"),
            (
                "WorkbenchIcon",
                "icon",
                "icon_ref",
                "zircon_editor_shell/controls/add.svg",
            ),
            ("WorkbenchIcon", "label", "text", "Icon"),
            ("WorkbenchLabel", "text", "text", "Label"),
            ("WorkbenchListRow", "text", "text", "List item"),
            ("WorkbenchListRow", "value", "text", "item"),
            ("WorkbenchListRow", "selected", "boolean", "false"),
            ("WorkbenchButton", "text", "text", "Button"),
            ("WorkbenchButton", "button_variant", "enum", "outlined"),
            ("WorkbenchCheckbox", "text", "text", "Checkbox"),
            ("WorkbenchCheckbox", "checked", "boolean", "false"),
            (
                "WorkbenchIconButton",
                "icon",
                "icon_ref",
                "zircon_editor_shell/toolbar/select.svg",
            ),
            ("WorkbenchIconButton", "label", "text", "Tool"),
            ("WorkbenchField", "value", "text", ""),
            ("WorkbenchField", "placeholder", "text", ""),
            ("WorkbenchDropdown", "value", "text", "default"),
            ("WorkbenchDropdown", "value_text", "text", "Default"),
            ("WorkbenchNumberField", "value_text", "text", "42"),
            ("WorkbenchNumberField", "placeholder", "text", "0"),
            ("WorkbenchPropertyRow", "text", "text", "Property"),
            ("WorkbenchPropertyRow", "value", "text", "Value"),
            ("WorkbenchRadio", "text", "text", "Radio option"),
            ("WorkbenchRadio", "checked", "boolean", "false"),
            ("WorkbenchSearchInput", "query", "text", ""),
            ("WorkbenchSearchInput", "placeholder", "text", "Search"),
            ("WorkbenchSectionTitle", "text", "text", "Section"),
            ("WorkbenchSegmentedControl", "value", "text", "center"),
            (
                "WorkbenchSegmentedControl",
                "selection_state",
                "enum",
                "single",
            ),
            ("WorkbenchStatusItem", "text", "text", "Status"),
            ("WorkbenchTab", "text", "text", "Tab"),
            ("WorkbenchTab", "selected", "boolean", "false"),
            (
                "WorkbenchToast",
                "text",
                "text",
                "Operation completed successfully",
            ),
            ("WorkbenchToast", "severity", "enum", "info"),
            ("WorkbenchToast", "variant", "enum", "outlined"),
            ("WorkbenchToggle", "text", "text", "Switch"),
            ("WorkbenchToggle", "checked", "boolean", "false"),
        ] {
            let (document_id, actual_value_type, actual_default) =
                default_for(component_id, property_name);
            let expected_default = if value_type == "boolean" {
                EditorPropLiteral::Boolean(match default {
                    "false" => false,
                    "true" => true,
                    _ => panic!(
                        "boolean property `{component_id}.{property_name}` has invalid default"
                    ),
                })
            } else {
                EditorPropLiteral::Text(default.to_string())
            };
            assert_eq!(actual_value_type, value_type);
            assert_eq!(
                actual_default,
                &EditorPropDefault::Literal(expected_default.clone()),
                "{component_id}.{property_name} should preserve its authored default"
            );
            assert_eq!(
                primitive_root_prop_default(document_id, property_name),
                expected_default,
                "{component_id}.{property_name} should match its primitive .zui root prop"
            );
        }
    }

    #[test]
    fn builtin_catalog_exposes_native_numeric_and_menu_parameters() {
        let descriptors = descriptors();
        let default_for = |component_id: &str, property_name: &str| {
            descriptors
                .iter()
                .find(|descriptor| descriptor.component_id == component_id)
                .and_then(|descriptor| {
                    descriptor
                        .props
                        .iter()
                        .find(|property| property.name == property_name)
                })
                .map(|property| {
                    (
                        descriptor.document_id.as_str(),
                        &property.value_type,
                        &property.default,
                    )
                })
                .unwrap_or_else(|| {
                    panic!("builtin catalog should expose `{property_name}` on `{component_id}`")
                })
        };

        for (component_id, property_name, value_type, expected_default) in [
            (
                "WorkbenchSlider",
                "value",
                "number",
                EditorPropLiteral::Float(50.0),
            ),
            (
                "WorkbenchSlider",
                "min",
                "number",
                EditorPropLiteral::Float(0.0),
            ),
            (
                "WorkbenchSlider",
                "max",
                "number",
                EditorPropLiteral::Float(100.0),
            ),
            (
                "WorkbenchSlider",
                "step",
                "number",
                EditorPropLiteral::Float(1.0),
            ),
            (
                "WorkbenchRangeSlider",
                "range_min",
                "number",
                EditorPropLiteral::Float(20.0),
            ),
            (
                "WorkbenchRangeSlider",
                "value",
                "number",
                EditorPropLiteral::Float(80.0),
            ),
            (
                "WorkbenchRangeSlider",
                "min",
                "number",
                EditorPropLiteral::Float(0.0),
            ),
            (
                "WorkbenchRangeSlider",
                "max",
                "number",
                EditorPropLiteral::Float(100.0),
            ),
            (
                "WorkbenchRangeSlider",
                "step",
                "number",
                EditorPropLiteral::Float(1.0),
            ),
            (
                "WorkbenchRangeSlider",
                "large_step",
                "number",
                EditorPropLiteral::Float(10.0),
            ),
            (
                "WorkbenchPopupMenu",
                "value",
                "text",
                EditorPropLiteral::Text("Open".to_string()),
            ),
            (
                "WorkbenchPopupMenu",
                "menu_items",
                "text_list",
                EditorPropLiteral::TextList(vec![
                    "New|icon=plus".to_string(),
                    "Open|icon=folder".to_string(),
                    "Save|icon=save".to_string(),
                    "Delete|danger,icon=trash".to_string(),
                    "More Tools|submenu".to_string(),
                ]),
            ),
            (
                "WorkbenchPopupMenu",
                "popup_open",
                "boolean",
                EditorPropLiteral::Boolean(true),
            ),
        ] {
            let (document_id, actual_value_type, actual_default) =
                default_for(component_id, property_name);
            assert_eq!(actual_value_type, value_type);
            assert_eq!(
                actual_default,
                &EditorPropDefault::Literal(expected_default.clone()),
                "{component_id}.{property_name} should preserve its authored native default"
            );
            assert_eq!(
                primitive_root_prop_default(document_id, property_name),
                expected_default,
                "{component_id}.{property_name} should match its primitive .zui root prop"
            );
        }
    }

    #[test]
    fn builtin_catalog_exposes_overlay_content_and_selection_parameters() {
        let descriptors = descriptors();
        let default_for = |component_id: &str, property_name: &str| {
            descriptors
                .iter()
                .find(|descriptor| descriptor.component_id == component_id)
                .and_then(|descriptor| {
                    descriptor
                        .props
                        .iter()
                        .find(|property| property.name == property_name)
                })
                .map(|property| {
                    (
                        descriptor.document_id.as_str(),
                        &property.value_type,
                        &property.default,
                    )
                })
                .unwrap_or_else(|| {
                    panic!("builtin catalog should expose `{property_name}` on `{component_id}`")
                })
        };

        for (component_id, property_name, value_type, expected_default) in [
            (
                "WorkbenchContextMenu",
                "popup_open",
                "boolean",
                EditorPropLiteral::Boolean(true),
            ),
            (
                "WorkbenchContextMenu",
                "options",
                "text_list",
                EditorPropLiteral::TextList(vec![
                    "Open".to_string(),
                    "Rename".to_string(),
                    "Duplicate".to_string(),
                    "Delete".to_string(),
                    "More Tools".to_string(),
                ]),
            ),
            (
                "WorkbenchContextMenu",
                "context_target",
                "text",
                EditorPropLiteral::Text("SceneNode:Camera".to_string()),
            ),
            (
                "WorkbenchDropdownPopup",
                "popup_open",
                "boolean",
                EditorPropLiteral::Boolean(true),
            ),
            (
                "WorkbenchDropdownPopup",
                "options",
                "text_list",
                EditorPropLiteral::TextList(vec![
                    "Scene".to_string(),
                    "Assets".to_string(),
                    "Console".to_string(),
                    "Render".to_string(),
                ]),
            ),
            (
                "WorkbenchDropdownPopup",
                "selected_options",
                "text_list",
                EditorPropLiteral::TextList(vec!["Assets".to_string()]),
            ),
            (
                "WorkbenchCommandPalette",
                "popup_open",
                "boolean",
                EditorPropLiteral::Boolean(true),
            ),
            (
                "WorkbenchCommandPalette",
                "query",
                "text",
                EditorPropLiteral::Text(String::new()),
            ),
            (
                "WorkbenchCommandPalette",
                "placeholder",
                "text",
                EditorPropLiteral::Text("Search commands".to_string()),
            ),
            (
                "WorkbenchCommandPalette",
                "commands",
                "text_list",
                EditorPropLiteral::TextList(vec![
                    "open_scene|label=Open Scene|shortcut=Ctrl+O".to_string(),
                    "build_project|label=Build Project|shortcut=Ctrl+B".to_string(),
                    "toggle_console|label=Toggle Console|shortcut=Ctrl+`".to_string(),
                ]),
            ),
            (
                "WorkbenchCommandPalette",
                "selected_command_id",
                "text",
                EditorPropLiteral::Text("build_project".to_string()),
            ),
        ] {
            let (document_id, actual_value_type, actual_default) =
                default_for(component_id, property_name);
            assert_eq!(actual_value_type, value_type);
            assert_eq!(
                actual_default,
                &EditorPropDefault::Literal(expected_default.clone()),
                "{component_id}.{property_name} should preserve its authored overlay default"
            );
            assert_eq!(
                primitive_root_prop_default(document_id, property_name),
                expected_default,
                "{component_id}.{property_name} should match its primitive .zui root prop"
            );
        }
    }

    #[test]
    fn builtin_catalog_includes_every_workbench_primitive_asset_as_a_primitive() {
        let descriptors = descriptors();
        let component_ids = descriptors
            .iter()
            .map(|descriptor| descriptor.component_id.as_str())
            .collect::<BTreeSet<_>>();
        let primitive_descriptors = descriptors
            .iter()
            .filter(|descriptor| descriptor.tier == EditorComponentTier::Primitive)
            .collect::<Vec<_>>();
        let primitive_ids = primitive_descriptors
            .iter()
            .map(|descriptor| descriptor.component_id.as_str())
            .collect::<BTreeSet<_>>();
        let primitive_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/ui/editor/components/workbench/primitives");

        assert_eq!(
            component_ids.len(),
            descriptors.len(),
            "builtin component identities must remain globally unique"
        );
        assert_eq!(
            primitive_ids.len(),
            primitive_descriptors.len(),
            "every primitive catalog identity must be unique"
        );
        assert_eq!(
            primitive_descriptors
                .iter()
                .map(|descriptor| {
                    descriptor
                        .document_id
                        .strip_prefix("res://ui/editor/components/workbench/primitives/")
                        .expect("primitive descriptors should stay in the primitive asset root")
                        .to_string()
                })
                .collect::<BTreeSet<_>>(),
            workbench_primitive_asset_paths(&primitive_root),
            "the catalog asset must cover every physical workbench primitive asset"
        );
        for descriptor in primitive_descriptors {
            assert_eq!(descriptor.binding_namespace, descriptor.component_id);
        }
    }

    #[test]
    fn builtin_catalog_uses_the_typed_editor_metadata_path() {
        assert_eq!(
            BUILTIN_COMPONENT_CATALOG_MANIFEST_ID,
            "res://ui/editor/components/catalog.toml"
        );
        assert!(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("assets/ui/editor/components/catalog.toml")
                .is_file(),
            "the catalog manifest must remain packaged editor metadata"
        );
        let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        for descriptor in descriptors() {
            let relative_path = descriptor
                .document_id
                .strip_prefix("res://")
                .expect("builtin catalog documents should use res:// identifiers");
            assert!(
                asset_root.join(relative_path).is_file(),
                "builtin component {} must reference a packaged document: {}",
                descriptor.component_id,
                descriptor.document_id
            );
        }
    }

    #[test]
    fn builtin_component_asset_entries_resolve_to_declared_component_definitions() {
        let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        let reusable_descriptors = descriptors()
            .into_iter()
            .filter(|descriptor| {
                descriptor
                    .document_id
                    .starts_with("res://ui/editor/components/")
            })
            .collect::<Vec<_>>();
        assert!(
            !reusable_descriptors.is_empty(),
            "the built-in catalog should retain reusable component asset entries"
        );

        for descriptor in reusable_descriptors {
            let relative_path = descriptor
                .document_id
                .strip_prefix("res://")
                .expect("builtin component documents should use res:// identifiers");
            let source_path = asset_root.join(relative_path);
            let source = fs::read_to_string(&source_path).unwrap_or_else(|error| {
                panic!("could not read {}: {error}", source_path.display())
            });
            let document: toml::Value = toml::from_str(&source).unwrap_or_else(|error| {
                panic!("could not parse {}: {error}", source_path.display())
            });
            assert!(
                document
                    .get("components")
                    .and_then(toml::Value::as_table)
                    .is_some_and(|components| components.contains_key(&descriptor.component_id)),
                "reusable catalog component {} must be declared by {}",
                descriptor.component_id,
                source_path.display()
            );
        }
    }

    fn workbench_primitive_asset_paths(root: &Path) -> BTreeSet<String> {
        let mut paths = BTreeSet::new();
        collect_workbench_primitive_asset_paths(root, root, &mut paths);
        paths
    }

    fn collect_workbench_primitive_asset_paths(
        root: &Path,
        directory: &Path,
        paths: &mut BTreeSet<String>,
    ) {
        for entry in fs::read_dir(directory).expect("workbench primitive directory should exist") {
            let entry = entry.expect("workbench primitive directory entries should be readable");
            let path = entry.path();
            if path.is_dir() {
                collect_workbench_primitive_asset_paths(root, &path, paths);
            } else if path.extension().is_some_and(|extension| extension == "zui") {
                let relative_path = path
                    .strip_prefix(root)
                    .expect("primitive assets should stay under the primitive root")
                    .to_string_lossy()
                    .replace('\\', "/");
                paths.insert(relative_path);
            }
        }
    }
}
