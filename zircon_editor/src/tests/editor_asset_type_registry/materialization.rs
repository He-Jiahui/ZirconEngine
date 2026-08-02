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

#[test]
fn failed_delta_preserves_definition_and_generation_atomically() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let material = AssetTypeId::parse("material.graph").unwrap();
    let before = registry.get(&material).unwrap().clone();
    let before_generation = registry.generation();
    let duplicate = AssetContextCommandDescriptor::new(
        "material.rebuild.duplicate",
        "Rebuild Material",
        EditorOperationPath::parse("material.rebuild.duplicate").unwrap(),
    );

    let error = registry
        .apply_contribution(
            "plugin.atomic",
            AssetTypeContribution::augment(material.clone())
                .with_context_command(duplicate.clone())
                .with_context_command(duplicate),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        AssetTypeRegistryError::DuplicateEntryOwner {
            asset_type,
            collection: "context_commands",
            first_owner,
            second_owner,
            ..
        } if asset_type == material
            && first_owner == "plugin.atomic"
            && second_owner == "plugin.atomic"
    ));
    assert_eq!(registry.generation(), before_generation);
    assert_eq!(registry.get(&material), Some(&before));
}

#[test]
fn contribution_failure_is_isolated_inside_one_generation_batch() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let material = AssetTypeId::parse("material.graph").unwrap();
    let before_generation = registry.generation();
    let duplicate = AssetContextCommandDescriptor::new(
        "material.batch.duplicate",
        "Duplicate",
        EditorOperationPath::parse("material.batch.duplicate").unwrap(),
    );

    let report = registry.apply_contributions([
        (
            "plugin.first",
            AssetTypeContribution::augment(material.clone()).with_context_command(
                AssetContextCommandDescriptor::new(
                    "material.batch.first",
                    "First",
                    EditorOperationPath::parse("material.batch.first").unwrap(),
                ),
            ),
        ),
        (
            "plugin.invalid",
            AssetTypeContribution::augment(material.clone())
                .with_toolkit(AssetToolkitDescriptor::new(
                    "editor.material.invalid",
                    EditorOperationPath::parse("material.invalid.open").unwrap(),
                ))
                .with_context_command(duplicate.clone())
                .with_context_command(duplicate),
        ),
        (
            "plugin.last",
            AssetTypeContribution::augment(material.clone())
                .with_toolkit(AssetToolkitDescriptor::new(
                    "editor.material.valid",
                    EditorOperationPath::parse("material.valid.open").unwrap(),
                ))
                .with_context_command(AssetContextCommandDescriptor::new(
                    "material.batch.last",
                    "Last",
                    EditorOperationPath::parse("material.batch.last").unwrap(),
                )),
        ),
    ]);

    assert_eq!(report.accepted_count(), 2);
    assert_eq!(report.rejected_count(), 1);
    assert_eq!(report.creation_template_sort_count(), 0);
    assert_eq!(report.creation_template_entry_count(), 0);
    assert_eq!(report.errors()[0].0, 1);
    assert!(matches!(
        &report.errors()[0].1,
        AssetTypeRegistryError::DuplicateEntryOwner {
            collection: "context_commands",
            first_owner,
            second_owner,
            ..
        } if first_owner == "plugin.invalid" && second_owner == "plugin.invalid"
    ));
    assert_eq!(registry.generation(), before_generation + 1);

    let definition = registry.get(&material).unwrap();
    assert_eq!(
        definition.toolkit().unwrap().view_id(),
        "editor.material.valid"
    );
    let command_ids = definition
        .context_commands()
        .iter()
        .map(|command| command.id())
        .collect::<Vec<_>>();
    assert_eq!(
        command_ids,
        vec!["material.batch.first", "material.batch.last"]
    );
}

#[test]
fn empty_and_all_invalid_batches_do_not_publish_a_generation() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let before_generation = registry.generation();

    let empty = registry.apply_contributions(Vec::<(String, AssetTypeContribution)>::new());
    assert_eq!(empty.accepted_count(), 0);
    assert_eq!(empty.rejected_count(), 0);
    assert_eq!(registry.generation(), before_generation);

    let first = AssetTypeId::parse("custom.invalid.first").unwrap();
    let second = AssetTypeId::parse("custom.invalid.second").unwrap();
    let invalid = registry.apply_contributions([
        (
            "plugin.invalid.first",
            AssetTypeContribution::augment(first.clone()).with_runtime_kind(ResourceKind::Data),
        ),
        (
            "plugin.invalid.second",
            AssetTypeContribution::augment(second.clone()).with_runtime_kind(ResourceKind::Data),
        ),
    ]);

    assert_eq!(invalid.accepted_count(), 0);
    assert_eq!(invalid.rejected_count(), 2);
    assert_eq!(
        invalid
            .errors()
            .iter()
            .map(|(input_index, _)| *input_index)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert!(registry.get(&first).is_none());
    assert!(registry.get(&second).is_none());
    assert_eq!(registry.generation(), before_generation);
}

#[test]
fn rejected_incomplete_new_type_does_not_block_later_definition_and_augment() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let custom = AssetTypeId::parse("custom.batch.asset").unwrap();
    let before_generation = registry.generation();

    let report = registry.apply_contributions([
        (
            "plugin.incomplete",
            AssetTypeContribution::augment(custom.clone()).with_runtime_kind(ResourceKind::Data),
        ),
        (
            "plugin.definition",
            AssetTypeContribution::define(
                custom.clone(),
                AssetTypePresentation::new("Batch Asset", "BAT", "asset-batch", "asset.batch"),
                ThumbnailProviderDescriptor::Operation(
                    EditorOperationPath::parse("custom.batch.preview").unwrap(),
                ),
            )
            .with_runtime_kind(ResourceKind::Data),
        ),
        (
            "plugin.augment",
            AssetTypeContribution::augment(custom.clone()).with_context_command(
                AssetContextCommandDescriptor::new(
                    "custom.batch.open",
                    "Open Batch Asset",
                    EditorOperationPath::parse("custom.batch.open").unwrap(),
                ),
            ),
        ),
    ]);

    assert_eq!(report.accepted_count(), 2);
    assert_eq!(report.rejected_count(), 1);
    assert_eq!(report.errors()[0].0, 0);
    assert_eq!(registry.generation(), before_generation + 1);
    let definition = registry.get(&custom).unwrap();
    assert_eq!(definition.presentation().display_name(), "Batch Asset");
    assert_eq!(definition.context_commands()[0].id(), "custom.batch.open");
}

#[test]
fn interleaved_asset_types_keep_pending_claims_and_errors_isolated() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let material = AssetTypeId::from_resource_kind(ResourceKind::Material);
    let model = AssetTypeId::from_resource_kind(ResourceKind::Model);
    let before_generation = registry.generation();

    let report = registry.apply_contributions([
        (
            "plugin.material.first",
            AssetTypeContribution::augment(material.clone()).with_toolkit(
                AssetToolkitDescriptor::new(
                    "editor.material.first",
                    EditorOperationPath::parse("material.first.open").unwrap(),
                ),
            ),
        ),
        (
            "plugin.model.first",
            AssetTypeContribution::augment(model.clone()).with_toolkit(
                AssetToolkitDescriptor::new(
                    "editor.model.first",
                    EditorOperationPath::parse("model.first.open").unwrap(),
                ),
            ),
        ),
        (
            "plugin.material.conflict",
            AssetTypeContribution::augment(material.clone()).with_toolkit(
                AssetToolkitDescriptor::new(
                    "editor.material.conflict",
                    EditorOperationPath::parse("material.conflict.open").unwrap(),
                ),
            ),
        ),
        (
            "plugin.model.command",
            AssetTypeContribution::augment(model.clone()).with_context_command(
                AssetContextCommandDescriptor::new(
                    "model.batch.inspect",
                    "Inspect Model",
                    EditorOperationPath::parse("model.batch.inspect").unwrap(),
                ),
            ),
        ),
        (
            "plugin.model.conflict",
            AssetTypeContribution::augment(model.clone()).with_toolkit(
                AssetToolkitDescriptor::new(
                    "editor.model.conflict",
                    EditorOperationPath::parse("model.conflict.open").unwrap(),
                ),
            ),
        ),
    ]);

    assert_eq!(report.accepted_count(), 3);
    assert_eq!(report.rejected_count(), 2);
    assert_eq!(report.touched_asset_type_count(), 2);
    assert_eq!(
        report
            .errors()
            .iter()
            .map(|(input_index, _)| *input_index)
            .collect::<Vec<_>>(),
        vec![2, 4]
    );
    assert_eq!(registry.generation(), before_generation + 1);
    assert_eq!(
        registry
            .get(&material)
            .unwrap()
            .toolkit()
            .unwrap()
            .view_id(),
        "editor.material.first"
    );
    assert_eq!(
        registry.get(&model).unwrap().toolkit().unwrap().view_id(),
        "editor.model.first"
    );
    assert_eq!(
        registry.get(&model).unwrap().context_commands()[0].id(),
        "model.batch.inspect"
    );
}

#[test]
fn batch_finalizes_creation_templates_once() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let material = AssetTypeId::parse("material.graph").unwrap();
    let contributions = (0..100)
        .rev()
        .map(|index| {
            let id = format!("material.template.batch_{index:03}");
            (
                "plugin.template.scale",
                AssetTypeContribution::augment(material.clone()).with_creation_template(
                    AssetCreationTemplateDescriptor::new(
                        id.clone(),
                        format!("Batch Template {index}"),
                        EditorOperationPath::parse(&id).unwrap(),
                    ),
                ),
            )
        })
        .collect::<Vec<_>>();

    let report = registry.apply_contributions(contributions);

    assert_eq!(report.accepted_count(), 100);
    assert_eq!(report.creation_template_sort_count(), 1);
    assert_eq!(report.creation_template_entry_count(), 100);
    let templates = registry.get(&material).unwrap().creation_templates();
    assert_eq!(templates.len(), 100);
    assert!(templates.windows(2).all(|pair| pair[0].id() < pair[1].id()));
}

#[test]
fn finalization_metrics_count_the_full_post_extend_collections() {
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    let material = AssetTypeId::parse("material.graph").unwrap();
    let mut initial = AssetTypeContribution::augment(material.clone());
    for index in 0..5 {
        let template_id = format!("material.metrics.template_{index:02}");
        let command_id = format!("material.metrics.command_{index:02}");
        initial = initial
            .with_creation_template(AssetCreationTemplateDescriptor::new(
                template_id.clone(),
                format!("Initial Template {index}"),
                EditorOperationPath::parse(&template_id).unwrap(),
            ))
            .with_context_command(AssetContextCommandDescriptor::new(
                command_id.clone(),
                format!("Initial Command {index}"),
                EditorOperationPath::parse(&command_id).unwrap(),
            ));
    }
    registry
        .apply_contribution("plugin.metrics.initial", initial)
        .unwrap();

    let additions = (5..15)
        .map(|index| {
            let template_id = format!("material.metrics.template_{index:02}");
            let command_id = format!("material.metrics.command_{index:02}");
            (
                "plugin.metrics.batch",
                AssetTypeContribution::augment(material.clone())
                    .with_creation_template(AssetCreationTemplateDescriptor::new(
                        template_id.clone(),
                        format!("Batch Template {index}"),
                        EditorOperationPath::parse(&template_id).unwrap(),
                    ))
                    .with_context_command(AssetContextCommandDescriptor::new(
                        command_id.clone(),
                        format!("Batch Command {index}"),
                        EditorOperationPath::parse(&command_id).unwrap(),
                    )),
            )
        })
        .collect::<Vec<_>>();

    let report = registry.apply_contributions(additions);

    assert_eq!(report.creation_template_sort_count(), 1);
    assert_eq!(report.creation_template_entry_count(), 15);
    assert_eq!(report.context_command_sort_count(), 1);
    assert_eq!(report.context_command_entry_count(), 15);
}

#[test]
fn catalog_scale_batches_sort_each_context_collection_once() {
    for contribution_count in [1, 100, 10_000, 100_000] {
        let mut registry = AssetTypeRegistry::with_builtins().unwrap();
        let material = AssetTypeId::parse("material.graph").unwrap();
        let before_generation = registry.generation();
        let contributions = (0..contribution_count)
            .rev()
            .map(|index| {
                let id = format!("material.scale.command_{index:06}");
                (
                    "plugin.scale",
                    AssetTypeContribution::augment(material.clone()).with_context_command(
                        AssetContextCommandDescriptor::new(
                            id.clone(),
                            format!("Scale command {index}"),
                            EditorOperationPath::parse(&id).unwrap(),
                        ),
                    ),
                )
            })
            .collect::<Vec<_>>();

        let report = registry.apply_contributions(contributions);

        assert_eq!(report.accepted_count(), contribution_count);
        assert_eq!(report.rejected_count(), 0);
        assert_eq!(report.touched_asset_type_count(), 1);
        assert_eq!(report.context_command_sort_count(), 1);
        assert_eq!(report.context_command_entry_count(), contribution_count);
        assert_eq!(registry.generation(), before_generation + 1);
        let commands = registry.get(&material).unwrap().context_commands();
        assert_eq!(commands.len(), contribution_count);
        assert!(commands.windows(2).all(|pair| pair[0].id() < pair[1].id()));
    }
}

#[test]
fn registry_delta_path_has_no_entry_clone_or_per_delta_full_sort() {
    let source = include_str!("../../core/asset/type_registry/registry.rs");
    let batch = include_str!("../../core/asset/type_registry/registry/batch.rs");
    assert!(!source.contains("merge_existing(existing.clone()"));
    assert!(!source.contains("fn merge_existing("));
    assert!(!source.contains("let asset_type = contribution.asset_type.clone();"));
    assert!(source.contains("pub(crate) fn apply_contributions"));
    assert!(source.contains("apply_contributions([(owner, contribution)])"));
    assert!(batch.contains("struct PendingEntryDelta"));
    assert!(batch.contains("fn finalize_pending_entries"));
    assert!(!batch.contains("binary_search_by"));
}

#[test]
fn host_asset_type_consumers_route_through_one_generation_cache() {
    let registration = include_str!("../../ui/host/editor_extension_registration.rs");
    let runtime_access = include_str!("../../ui/host/editor_event_runtime_access.rs");
    let asset_event = include_str!("../../ui/host/editor_event_execution/asset_event.rs");
    let shell = include_str!("../../ui/workbench/shell_state.rs");

    assert!(registration.contains("fn enabled_asset_types_for_shell("));
    assert!(registration.contains("asset_type_registry_cache.contributions_changed()"));
    assert!(!runtime_access.contains("materialize_enabled_asset_types("));
    assert!(!asset_event.contains("materialize_enabled_asset_types("));
    assert!(runtime_access.contains("enabled_asset_types_for_shell("));
    assert!(asset_event.contains("enabled_asset_types_for_shell("));
    assert!(shell.contains("struct AssetTypeRegistryGenerationCache"));
    assert!(shell.contains("enabled_capabilities.as_slice() == enabled_capabilities"));
}
