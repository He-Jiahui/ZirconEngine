use crate::core::asset::{
    AssetContextCommandDescriptor, AssetCreationTemplateDescriptor, AssetToolkitDescriptor,
    AssetTypeContribution, AssetTypeId, AssetTypePresentation, AssetTypeRegistry,
    AssetTypeRegistryError, ThumbnailProviderDescriptor,
};
use crate::core::editor_operation::EditorOperationPath;
use zircon_runtime_interface::resource::ResourceKind;

#[test]
fn plugin_contribution_augments_the_single_materialized_builtin_definition() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let material = AssetTypeId::parse("material").unwrap();
    let open = EditorOperationPath::parse("material.editor.open").unwrap();

    registry
        .apply_contribution(
            "zircon.material_editor",
            AssetTypeContribution::augment(material.clone())
                .with_toolkit(AssetToolkitDescriptor::new("editor.material", open.clone())),
        )
        .unwrap();

    let definition = registry.get(&material).unwrap();
    assert_eq!(definition.runtime_kind(), Some(ResourceKind::Material));
    assert_eq!(definition.toolkit().unwrap().view_id(), "editor.material");
    assert_eq!(definition.toolkit().unwrap().open_operation(), &open);
}

#[test]
fn custom_plugin_type_materializes_when_it_supplies_a_complete_base() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let support = AssetTypeId::parse("support.asset").unwrap();
    let preview = EditorOperationPath::parse("support.asset.preview").unwrap();

    registry
        .apply_contribution(
            "plugin.support",
            AssetTypeContribution::define(
                support.clone(),
                AssetTypePresentation::new(
                    "Support Asset",
                    "SUP",
                    "asset-support",
                    "asset.support",
                ),
                ThumbnailProviderDescriptor::Operation(preview),
            )
            .with_runtime_kind(ResourceKind::Data),
        )
        .unwrap();

    let definition = registry.get(&support).unwrap();
    assert_eq!(definition.id(), &support);
    assert_eq!(definition.runtime_kind(), Some(ResourceKind::Data));
    assert_eq!(definition.presentation().display_name(), "Support Asset");
}

#[test]
fn a_second_toolkit_owner_is_rejected_instead_of_silently_overriding() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let material = AssetTypeId::parse("material").unwrap();

    registry
        .apply_contribution(
            "plugin.first",
            AssetTypeContribution::augment(material.clone()).with_toolkit(
                AssetToolkitDescriptor::new(
                    "editor.material.first",
                    EditorOperationPath::parse("material.first.open").unwrap(),
                ),
            ),
        )
        .unwrap();

    let error = registry
        .apply_contribution(
            "plugin.second",
            AssetTypeContribution::augment(material.clone()).with_toolkit(
                AssetToolkitDescriptor::new(
                    "editor.material.second",
                    EditorOperationPath::parse("material.second.open").unwrap(),
                ),
            ),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        AssetTypeRegistryError::DuplicateFieldOwner {
            asset_type,
            field: "toolkit",
            first_owner,
            second_owner,
        } if asset_type == material
            && first_owner == "plugin.first"
            && second_owner == "plugin.second"
    ));
}

#[test]
fn an_incomplete_custom_type_is_rejected_without_ui_fallback_guessing() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let custom = AssetTypeId::parse("custom.incomplete").unwrap();

    let error = registry
        .apply_contribution(
            "plugin.incomplete",
            AssetTypeContribution::augment(custom.clone()).with_runtime_kind(ResourceKind::Data),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        AssetTypeRegistryError::IncompleteDefinition { asset_type, .. }
            if asset_type == custom
    ));
}

#[test]
fn creation_templates_materialize_inside_the_asset_type_definition() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let material = AssetTypeId::parse("material.graph").unwrap();
    let operation = EditorOperationPath::parse("material.graph.create").unwrap();

    registry
        .apply_contribution(
            "plugin.material",
            AssetTypeContribution::augment(material.clone()).with_creation_template(
                AssetCreationTemplateDescriptor::new(
                    "material.template.graph",
                    "Material Graph",
                    operation.clone(),
                ),
            ),
        )
        .unwrap();

    let templates = registry.get(&material).unwrap().creation_templates();
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].id(), "material.template.graph");
    assert_eq!(templates[0].operation(), &operation);
}

#[test]
fn duplicate_creation_template_ids_report_both_owners() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let material = AssetTypeId::parse("material.graph").unwrap();
    let template = AssetCreationTemplateDescriptor::new(
        "material.template.graph",
        "Material Graph",
        EditorOperationPath::parse("material.graph.create").unwrap(),
    );
    registry
        .apply_contribution(
            "plugin.first",
            AssetTypeContribution::augment(material.clone())
                .with_creation_template(template.clone()),
        )
        .unwrap();

    let error = registry
        .apply_contribution(
            "plugin.second",
            AssetTypeContribution::augment(material.clone()).with_creation_template(template),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        AssetTypeRegistryError::DuplicateEntryOwner {
            asset_type,
            collection: "creation_templates",
            entry_id,
            first_owner,
            second_owner,
        } if asset_type == material
            && entry_id == "material.template.graph"
            && first_owner == "plugin.first"
            && second_owner == "plugin.second"
    ));
}

#[test]
fn context_commands_materialize_inside_the_asset_type_definition() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let material = AssetTypeId::parse("material.graph").unwrap();
    let operation = EditorOperationPath::parse("material.graph.rebuild").unwrap();

    registry
        .apply_contribution(
            "plugin.material",
            AssetTypeContribution::augment(material.clone()).with_context_command(
                AssetContextCommandDescriptor::new(
                    "material.rebuild",
                    "Rebuild Material Graph",
                    operation.clone(),
                ),
            ),
        )
        .unwrap();

    let commands = registry.get(&material).unwrap().context_commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].id(), "material.rebuild");
    assert_eq!(commands[0].operation(), &operation);
}

#[test]
fn duplicate_context_command_ids_report_both_owners() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let material = AssetTypeId::parse("material.graph").unwrap();
    let command = AssetContextCommandDescriptor::new(
        "material.rebuild",
        "Rebuild Material Graph",
        EditorOperationPath::parse("material.graph.rebuild").unwrap(),
    );
    registry
        .apply_contribution(
            "plugin.first",
            AssetTypeContribution::augment(material.clone()).with_context_command(command.clone()),
        )
        .unwrap();

    let error = registry
        .apply_contribution(
            "plugin.second",
            AssetTypeContribution::augment(material.clone()).with_context_command(command),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        AssetTypeRegistryError::DuplicateEntryOwner {
            asset_type,
            collection: "context_commands",
            entry_id,
            first_owner,
            second_owner,
        } if asset_type == material
            && entry_id == "material.rebuild"
            && first_owner == "plugin.first"
            && second_owner == "plugin.second"
    ));
}
