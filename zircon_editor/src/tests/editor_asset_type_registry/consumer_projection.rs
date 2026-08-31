use std::sync::Arc;

use crate::core::asset::{
    builtin_asset_type_definition, AssetContextCommandDescriptor, AssetCreationTemplateDescriptor,
    AssetSourceKind, AssetSourceWritePolicy, AssetToolkitDescriptor, AssetTypeContribution,
    AssetTypeId, AssetTypeRegistryError, AssetWriteAccess,
};
use crate::core::commands::EditorCommandDescriptor;
use crate::core::editor_event::{EditorAssetEvent, EditorEvent};
use crate::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError, ViewDescriptor,
};
use crate::core::editor_operation::{
    EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
    EditorAssetFolderRecord,
};
use crate::ui::host::{EditorAssetOperationInvokeError, EditorOperationDispatchError};
use zircon_runtime::asset::project::PreviewState;
use zircon_runtime_interface::resource::ResourceKind;

#[test]
fn browser_projection_reads_the_builtin_definition_owner() {
    let material = builtin_asset_type_definition(ResourceKind::Material).unwrap();
    assert_eq!(material.id(), &AssetTypeId::parse("material").unwrap());
    assert_eq!(material.presentation().display_name(), "Material");
    assert_eq!(material.presentation().badge(), "MAT");
    assert_eq!(material.presentation().icon_name(), "asset-material");
}

#[test]
fn editor_parallel_sidecar_and_ui_kind_dispatch_are_retired() {
    let manager_root = include_str!("../../ui/host/editor_asset_manager/mod.rs");
    let catalog = include_str!("../../ui/host/editor_asset_manager/catalog.rs");
    let sync = include_str!(
        "../../ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs"
    );
    for retired in [
        "mod editor_meta;",
        "EditorAssetMetaDocument",
        "editor_meta_path_for_source",
        "editor_meta_path",
        "editor_meta:",
        ".editor.meta.toml",
    ] {
        assert!(!manager_root.contains(retired));
        assert!(!catalog.contains(retired));
        assert!(!sync.contains(retired));
    }

    let labels = include_str!("../../ui/layouts/views/asset_browser/labels.rs");
    let thumbnails = include_str!("../../ui/layouts/views/asset_browser/thumbnail_nodes.rs");
    let activity_content = include_str!("../../ui/layouts/views/assets_activity/content_nodes.rs");
    let browser = include_str!("../../ui/layouts/views/asset_browser.rs");
    let preview = include_str!(
        "../../ui/host/editor_asset_manager/manager/preview_refresh/generate_preview_artifact.rs"
    );
    for retired in [
        "fn resource_kind_badge_code",
        "fn compact_resource_kind_label",
        "fn summary_resource_kind_label",
        "fn asset_thumbnail_icon_name",
        "match record.kind",
        "preview_palette(record.kind)",
    ] {
        assert!(!labels.contains(retired));
        assert!(!thumbnails.contains(retired));
        assert!(!preview.contains(retired));
    }
    assert!(!labels.contains("builtin_asset_type_definition"));
    assert!(!activity_content.contains("asset_type_badge(asset.kind)"));
    assert!(!browser.contains("asset_type_label("));

    let selection = include_str!("../../ui/workbench/snapshot/asset/asset_selection_snapshot.rs");
    for retired in ["adapter_key", "No adapter", "AssetBrowserAdapter"] {
        assert!(!selection.contains(retired));
        assert!(!browser.contains(retired));
    }
}

#[test]
fn preview_generation_retains_resolved_source_path_views() {
    let preview = include_str!(
        "../../ui/host/editor_asset_manager/manager/preview_refresh/generate_preview_artifact.rs"
    );

    assert!(preview.contains("resolve_source_path_for_uri(&record.locator)?"));
    assert!(preview.contains("image::open(source_path.operation_path())"));
    assert!(preview.contains("source_path.display_path().display()"));
    assert!(!preview.contains("project.source_path_for_uri(&record.locator)?"));
}

#[test]
fn asset_catalog_view_projects_internal_paths_through_the_resolver_display_view() {
    let record_projection =
        include_str!("../../ui/host/editor_asset_manager/manager/catalog_generation/record.rs");

    assert!(record_projection.contains("use zircon_runtime::asset::project::ProjectPaths;"));
    assert!(record_projection.contains("ProjectPaths::display_path(&record.meta_path)"));
    assert!(record_projection.contains("ProjectPaths::display_path(&record.preview_artifact_path)"));
    assert!(
        !record_projection.contains("meta_path: record.meta_path.to_string_lossy().into_owned()")
    );
    assert!(!record_projection.contains(
        "preview_artifact_path: record.preview_artifact_path.to_string_lossy().into_owned()"
    ));
}

#[test]
fn asset_catalog_snapshot_projects_roots_through_the_resolver_display_view() {
    let catalog_build =
        include_str!("../../ui/host/editor_asset_manager/manager/catalog_generation/build.rs");

    assert!(catalog_build
        .contains("use zircon_runtime::asset::project::{ProjectManager, ProjectPaths};"));
    assert!(catalog_build.contains("ProjectPaths::display_path(project.paths().root())"));
    assert!(catalog_build.contains("ProjectPaths::display_path(assets_root)"));
    assert!(catalog_build.contains("ProjectPaths::display_path(project.paths().cache_root())"));
    assert!(!catalog_build.contains("project.paths().root().to_string_lossy().into_owned()"));
    assert!(!catalog_build.contains("assets_root.to_string_lossy().into_owned()"));
    assert!(!catalog_build.contains("project.paths().cache_root().to_string_lossy().into_owned()"));
}

#[test]
fn startup_path_presentation_uses_the_resolver_without_a_second_windows_prefix_rule() {
    let display_path = include_str!("../../ui/workbench/startup/display_project_path.rs");

    assert!(display_path.contains("ProjectPaths::display_path(Path::new(path.as_ref()))"));
    assert!(!display_path.contains("fn display_project_text"));
    assert!(!display_path.contains("replace(\"\\\\\\\\?\\\\UNC\\\""));
    assert!(!display_path.contains("replace(\"\\\\\\\\?\\\""));
}

#[test]
fn create_and_context_dispatch_resolve_operations_from_the_materialized_registry() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("editor09_registry_create_context_projection");
    let asset_type = AssetTypeId::from_resource_kind(ResourceKind::UiLayout);
    let create = EditorOperationPath::parse("ui_asset.layout.create").unwrap();
    let validate = EditorOperationPath::parse("ui_asset.layout.validate").unwrap();
    let open = EditorOperationPath::parse("view.editor.ui_asset.test.open").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(
            "editor.ui_asset.test",
            "UI Asset Test",
            "Assets",
        ))
        .unwrap();
    for operation in [create.clone(), validate.clone()] {
        extension
            .register_command(
                EditorCommandDescriptor::operation(operation)
                    .with_event(EditorEvent::Asset(EditorAssetEvent::OpenAssetBrowser)),
            )
            .unwrap();
    }
    extension
        .register_asset_type_contribution(
            AssetTypeContribution::augment(asset_type.clone())
                .with_toolkit(AssetToolkitDescriptor::new(
                    "editor.ui_asset.test",
                    open.clone(),
                ))
                .with_creation_template(AssetCreationTemplateDescriptor::new(
                    "ui_asset.layout",
                    "UI Layout",
                    create.clone(),
                ))
                .with_context_command(
                    AssetContextCommandDescriptor::new(
                        "ui_asset.validate",
                        "Validate UI Layout",
                        validate.clone(),
                    )
                    .with_mutation_access(),
                ),
        )
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .unwrap();
    runtime.runtime.sync_asset_catalog(Arc::new(
        EditorAssetCatalogGeneration::from_snapshot_record(
            EditorAssetCatalogSnapshotRecord {
                project_name: "Registry Projection".to_string(),
                project_root: "E:/RegistryProjection".to_string(),
                assets_root: "E:/RegistryProjection/assets".to_string(),
                cache_root: "E:/RegistryProjection/.zircon/cache".to_string(),
                default_scene_uri: String::new(),
                catalog_revision: 1,
                folders: vec![
                    EditorAssetFolderRecord {
                        folder_id: "res://".to_string(),
                        parent_folder_id: None,
                        locator_prefix: "res://".to_string(),
                        display_name: "Assets".to_string(),
                        child_folder_ids: vec!["res://ui".to_string()],
                        direct_asset_uuids: Vec::new(),
                        recursive_asset_count: 1,
                    },
                    EditorAssetFolderRecord {
                        folder_id: "res://ui".to_string(),
                        parent_folder_id: Some("res://".to_string()),
                        locator_prefix: "res://ui/".to_string(),
                        display_name: "ui".to_string(),
                        child_folder_ids: Vec::new(),
                        direct_asset_uuids: vec![
                            "11111111-1111-1111-1111-111111111111".to_string(),
                        ],
                        recursive_asset_count: 1,
                    },
                ],
                assets: vec![EditorAssetCatalogRecord {
                    uuid: "11111111-1111-1111-1111-111111111111".to_string(),
                    id: "22222222-2222-2222-2222-222222222222".to_string(),
                    locator: "res://ui/main.zui".to_string(),
                    kind: ResourceKind::UiLayout,
                    display_name: "main.zui".to_string(),
                    file_name: "main.zui".to_string(),
                    extension: "zui".to_string(),
                    preview_state: PreviewState::Dirty,
                    meta_path: "E:/RegistryProjection/assets/ui/main.zui.zmeta".to_string(),
                    preview_artifact_path: String::new(),
                    source_mtime_unix_ms: 0,
                    source_hash: String::new(),
                    dirty: false,
                    diagnostics: Vec::new(),
                    direct_reference_uuids: Vec::new(),
                }],
            },
            1,
        ),
    ));
    runtime
        .runtime
        .dispatch_event(
            crate::core::editor_event::EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::SelectFolder {
                folder_id: "res://ui".to_string(),
            }),
        )
        .unwrap();
    runtime
        .runtime
        .dispatch_event(
            crate::core::editor_event::EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::SelectItem {
                asset_uuid: "11111111-1111-1111-1111-111111111111".to_string(),
            }),
        )
        .unwrap();

    let creation_templates = runtime
        .runtime
        .asset_creation_templates(&asset_type)
        .unwrap();
    assert_eq!(creation_templates.len(), 1);
    assert_eq!(creation_templates[0].operation().as_str(), create.as_str());
    assert_eq!(
        runtime
            .runtime
            .asset_context_commands(&asset_type)
            .unwrap()
            .len(),
        1
    );
    let snapshot = runtime.runtime.editor_snapshot().asset_browser;
    let item = snapshot
        .visible_assets
        .iter()
        .find(|item| item.locator == "res://ui/main.zui")
        .unwrap();
    assert_eq!(item.source_authority().kind(), AssetSourceKind::Project);
    assert_eq!(
        item.source_authority().write_access(),
        AssetWriteAccess::Writable
    );
    assert_eq!(
        snapshot
            .selection
            .source_authority()
            .unwrap()
            .write_access(),
        AssetWriteAccess::Writable
    );
    assert!(snapshot.creation_menu.entries().iter().any(|entry| {
        entry.asset_type() == &asset_type && entry.template_id() == "ui_asset.layout"
    }));
    assert!(snapshot
        .selection
        .context_commands
        .iter()
        .any(|command| command.id == "ui_asset.validate"
            && command.operation_id == validate.as_str()));
    let open_record = runtime
        .runtime
        .dispatch_event(
            crate::core::editor_event::EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: "res://ui/main.zui".to_string(),
            }),
        )
        .unwrap();
    assert!(open_record
        .effects
        .contains(&crate::core::editor_event::EditorEventEffect::LayoutChanged));
    assert!(runtime
        .runtime
        .current_view_instances()
        .iter()
        .any(|instance| instance.descriptor_id
            == crate::ui::workbench::view::ViewDescriptorId::new("editor.ui_asset.test")));
    let create_record = runtime
        .runtime
        .invoke_asset_creation_template(
            EditorOperationSource::UiBinding,
            &asset_type,
            "ui_asset.layout",
            "res://ui",
        )
        .unwrap();
    assert_eq!(create_record.operation_id.as_deref(), Some(create.as_str()));
    let context_record = runtime
        .runtime
        .invoke_asset_context_command(
            EditorOperationSource::UiBinding,
            &asset_type,
            "ui_asset.validate",
            "res://ui/main.zui",
        )
        .unwrap();
    assert_eq!(
        context_record.operation_id.as_deref(),
        Some(validate.as_str())
    );

    for (target_folder, source_kind) in [
        ("package://com.zircon.ui/layouts", "package"),
        ("builtin://ui/layouts", "builtin"),
        ("lib://ui/layouts", "library"),
        ("mem://ui/layouts", "transient"),
    ] {
        let read_only = runtime
            .runtime
            .handle_operation_control_request_from_source(
                EditorOperationSource::Cli,
                crate::core::editor_operation::EditorOperationControlRequest::InvokeOperation(
                    crate::core::editor_operation::EditorOperationInvocation::new(create.clone())
                        .with_arguments(serde_json::json!({
                            "asset_type": asset_type.as_str(),
                            "target_folder": target_folder,
                        })),
                ),
            );
        assert_eq!(
            read_only.error,
            Some(format!(
                "asset operation ui_asset.layout.create cannot write to read-only {source_kind} source `{target_folder}`"
            ))
        );
    }

    let unsupported_derived = runtime
        .runtime
        .handle_operation_control_request_from_source(
            EditorOperationSource::Cli,
            crate::core::editor_operation::EditorOperationControlRequest::InvokeOperation(
                crate::core::editor_operation::EditorOperationInvocation::new(create)
                    .with_arguments(serde_json::json!({
                        "asset_type": asset_type.as_str(),
                        "target_folder": "derived://ui/layouts",
                    })),
            ),
        );
    assert_eq!(
        unsupported_derived.error.as_deref(),
        Some(
            "asset operation ui_asset.layout.create has invalid source target `derived://ui/layouts`: unsupported resource scheme: derived"
        )
    );
}

#[test]
fn asset_creation_invocation_retains_extension_registry_materialization_failure() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("editor09_registry_creation_error_retention");
    let asset_type = AssetTypeId::from_resource_kind(ResourceKind::Material);
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_asset_type_contribution(
            AssetTypeContribution::augment(asset_type.clone())
                .with_source_write_policy(AssetSourceWritePolicy::ProjectOnly),
        )
        .expect("the deferred registry fixture should register its contribution");
    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("the registry fixture should defer materialization until invocation");

    let error = runtime
        .runtime
        .invoke_asset_creation_template(
            EditorOperationSource::UiBinding,
            &asset_type,
            "unreachable.template",
            "res://materials",
        )
        .expect_err("registry materialization failure must not become a missing-template error");
    assert!(matches!(
        error,
        EditorAssetOperationInvokeError::ExtensionRegistry(
            EditorExtensionRegistryError::AssetTypeRegistry(
                AssetTypeRegistryError::DuplicateFieldOwner {
                    field: "source_write_policy",
                    ..
                }
            )
        )
    ));
}

#[test]
fn asset_registry_materialization_failure_remains_visible_at_every_host_boundary() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("editor09_registry_boundary_error_retention");
    let asset_type = AssetTypeId::from_resource_kind(ResourceKind::Material);
    let operation = EditorOperationPath::parse("asset.registry.materialization_probe").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation.clone())
                .with_event(EditorEvent::Asset(EditorAssetEvent::OpenAssetBrowser))
                .with_asset_write_target_arguments("asset_type", "target_folder"),
        )
        .expect("the operation fixture should register its command");
    extension
        .register_asset_type_contribution(
            AssetTypeContribution::augment(asset_type.clone())
                .with_source_write_policy(AssetSourceWritePolicy::ProjectOnly),
        )
        .expect("the deferred registry fixture should register its contribution");
    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("the registry fixture should defer materialization until a host boundary reads it");

    let query_error = runtime
        .runtime
        .asset_type_definition(&asset_type)
        .expect_err("asset type query must retain registry materialization failure");
    assert_registry_materialization_error(query_error);
    let templates_error = runtime
        .runtime
        .asset_creation_templates(&asset_type)
        .expect_err("asset creation menu query must retain registry materialization failure");
    assert_registry_materialization_error(templates_error);
    let commands_error = runtime
        .runtime
        .asset_context_commands(&asset_type)
        .expect_err("asset context menu query must retain registry materialization failure");
    assert_registry_materialization_error(commands_error);

    let operation_error = runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::UiBinding,
            EditorOperationInvocation::new(operation).with_arguments(serde_json::json!({
                "asset_type": asset_type.as_str(),
                "target_folder": "res://materials",
            })),
        )
        .expect_err(
            "asset write operation must not misreport registry failure as unregistered type",
        );
    assert!(matches!(
        operation_error,
        EditorOperationDispatchError::ExtensionRegistry(
            EditorExtensionRegistryError::AssetTypeRegistry(
                AssetTypeRegistryError::DuplicateFieldOwner {
                    field: "source_write_policy",
                    ..
                }
            )
        )
    ));

    let snapshot = runtime.runtime.editor_snapshot();
    assert!(snapshot
        .status_line
        .contains("Asset type registry unavailable:"));
    assert!(snapshot.status_line.contains("source_write_policy"));
}

fn assert_registry_materialization_error(error: EditorExtensionRegistryError) {
    assert!(matches!(
        error,
        EditorExtensionRegistryError::AssetTypeRegistry(
            AssetTypeRegistryError::DuplicateFieldOwner {
                field: "source_write_policy",
                ..
            }
        )
    ));
}
