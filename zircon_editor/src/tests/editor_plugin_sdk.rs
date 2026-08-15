use crate::core::asset::{
    AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId, AssetTypeRegistry,
};
use crate::core::commands::EditorCommandDescriptor;
use crate::core::editor_authoring_extension::SceneModeDescriptor;
use std::sync::{Arc, Mutex};

use zircon_runtime::plugin::PluginPackageManifest;
use zircon_runtime_interface::resource::ResourceKind;

use crate::core::editor_extension::{
    AssetImporterDescriptor, EditorExtensionRegistry, ViewDescriptor,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::plugin::sdk::examples::{ExampleAssetInspectorPlugin, ExampleWindowEditorPlugin};
use crate::core::plugin::sdk::lifecycle::{
    EditorPluginLifecycleError, EditorPluginLifecycleEvent, EditorPluginLifecycleStage,
};
use crate::core::plugin::{
    EditorPlugin, EditorPluginCatalog, EditorPluginDescriptor, EditorPluginManager,
    EditorPluginTransitionError,
};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;

use super::editor_event::support::{env_lock, EventRuntimeHarness};

#[test]
fn editor_plugin_sdk_examples_publish_window_and_asset_contributions() {
    let window = Arc::new(ExampleWindowEditorPlugin::default());
    let asset = Arc::new(ExampleAssetInspectorPlugin::default());
    let plugins: Vec<(Arc<dyn EditorPlugin + Send + Sync>, PluginPackageManifest)> = vec![
        (
            Arc::clone(&window),
            PluginPackageManifest::new("sdk_example_window", "SDK Example Window"),
        ),
        (
            Arc::clone(&asset),
            PluginPackageManifest::new("sdk_example_asset", "SDK Example Asset Tools"),
        ),
    ];

    let catalog = crate::core::plugin::EditorPluginCatalog::from_plugins(plugins);
    let manager = EditorPluginManager::new(catalog).expect("the example catalog is admissible");
    let snapshot = manager
        .advance_loading_phase(crate::core::plugin::EditorPluginLoadingPhase::Default)
        .expect("the default phase should activate example plugins");
    let extension_report = snapshot.active_extensions();

    assert!(
        extension_report.is_success(),
        "example plugins should aggregate without diagnostics: {:?}",
        extension_report.diagnostics
    );
    let model_type = AssetTypeId::from_resource_kind(ResourceKind::Model);
    let model_definition = extension_report
        .asset_types
        .get(&model_type)
        .expect("model asset type definition");
    assert_eq!(
        model_definition.toolkit().unwrap().view_id(),
        "sdk.example.asset_inspector"
    );
    let registry = &extension_report.registry;
    assert!(registry
        .views()
        .iter()
        .any(|view| view.id() == "sdk.example.weather_window"));
    assert!(registry
        .asset_importers()
        .iter()
        .any(
            |importer| importer.id() == "sdk.example.asset.model_importer"
                && importer.source_extensions() == ["glb".to_string(), "gltf".to_string()]
        ));
    assert!(registry
        .inspector_customizations()
        .iter()
        .any(|customization| { customization.target_type() == "sdk.example.ModelImportSettings" }));

    let stages = ["sdk_example_window", "sdk_example_asset"]
        .into_iter()
        .flat_map(|package_id| {
            snapshot
                .catalog_snapshot()
                .registration(package_id)
                .expect("the activated example plugin should retain its lifecycle report")
                .lifecycle
                .records()
                .iter()
                .map(|record| record.event().stage().clone())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        stages,
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
        ]
    );
}

#[test]
fn editor_plugin_catalog_reuses_one_materialization_per_generation() {
    let catalog = EditorPluginCatalog::from_descriptors(
        EditorPluginDescriptor::builtin_catalog(),
        Vec::<PluginPackageManifest>::new(),
    );

    let manager = EditorPluginManager::new(catalog).expect("the builtin catalog is admissible");
    let snapshot = manager
        .advance_loading_phase(crate::core::plugin::EditorPluginLoadingPhase::Default)
        .expect("the default phase should materialize builtin extensions");
    let first = Arc::clone(snapshot.active_extensions());
    let second = Arc::clone(manager.state_snapshot().active_extensions());

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.catalog_generation, snapshot.catalog_generation());
}

#[test]
fn catalog_asset_batch_preserves_diagnostics_between_other_extension_failures() {
    struct OrderedCatalogPlugin {
        descriptor: EditorPluginDescriptor,
        toolkit_view: &'static str,
        toolkit_operation: &'static str,
    }

    impl EditorPlugin for OrderedCatalogPlugin {
        fn descriptor(&self) -> &EditorPluginDescriptor {
            &self.descriptor
        }

        fn register_editor_extensions(
            &self,
            registry: &mut EditorExtensionRegistry,
        ) -> Result<(), crate::core::editor_extension::EditorExtensionRegistryError> {
            registry.register_view(ViewDescriptor::new(
                "sdk.batch.shared_view",
                "Shared Batch View",
                "Testing",
            ))?;
            registry.register_asset_type_contribution(
                AssetTypeContribution::augment(AssetTypeId::from_resource_kind(
                    ResourceKind::Model,
                ))
                .with_toolkit(AssetToolkitDescriptor::new(
                    self.toolkit_view,
                    EditorOperationPath::parse(self.toolkit_operation).unwrap(),
                )),
            )?;
            registry.register_scene_mode(
                crate::tests::support::pass_through_scene_mode_registration(
                    SceneModeDescriptor::new(
                        "sdk.batch.shared_tool",
                        "Shared Batch Tool",
                        "sdk.batch.shared_view",
                        EditorOperationPath::parse("sdk.batch.tool.activate").unwrap(),
                    ),
                ),
            )
        }
    }

    let first = Arc::new(OrderedCatalogPlugin {
        descriptor: EditorPluginDescriptor::new(
            "sdk_batch_first",
            "SDK Batch First",
            "zircon_editor_sdk_batch_first",
        ),
        toolkit_view: "sdk.batch.first_toolkit",
        toolkit_operation: "sdk.batch.first.open",
    });
    let second = Arc::new(OrderedCatalogPlugin {
        descriptor: EditorPluginDescriptor::new(
            "sdk_batch_second",
            "SDK Batch Second",
            "zircon_editor_sdk_batch_second",
        ),
        toolkit_view: "sdk.batch.second_toolkit",
        toolkit_operation: "sdk.batch.second.open",
    });
    let catalog = EditorPluginCatalog::from_plugins([
        (
            Arc::clone(&first) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new("sdk_batch_first", "SDK Batch First"),
        ),
        (
            Arc::clone(&second) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new("sdk_batch_second", "SDK Batch Second"),
        ),
    ]);

    let manager = EditorPluginManager::new(catalog).expect("the batch catalog is admissible");
    let snapshot = manager
        .advance_loading_phase(crate::core::plugin::EditorPluginLoadingPhase::Default)
        .expect("the default phase should materialize batch contributions");
    let report = snapshot.active_extensions();

    assert_eq!(report.diagnostics.len(), 3);
    assert!(report.diagnostics[0].contains("editor view sdk.batch.shared_view already registered"));
    assert!(report.diagnostics[1].contains(
        "asset type `model` field `toolkit` is owned by both `sdk_batch_first` and `sdk_batch_second`"
    ));
    assert!(report.diagnostics[2]
        .contains("editor scene mode sdk.batch.shared_tool already registered"));
    assert_eq!(
        report.asset_types.generation(),
        AssetTypeRegistry::with_builtins().unwrap().generation() + 1
    );
}

#[test]
fn editor_host_reuses_asset_type_registry_for_unchanged_extension_generation() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("asset_type_registry_generation_cache");
    let model_type = AssetTypeId::from_resource_kind(ResourceKind::Model);

    let _ = runtime.runtime.asset_type_definition(&model_type).unwrap();
    let after_first = runtime.runtime.asset_type_registry_cache_counts();
    let _ = runtime.runtime.asset_type_definition(&model_type).unwrap();
    let after_second = runtime.runtime.asset_type_registry_cache_counts();

    assert_eq!(after_second.1, after_first.1);
    assert_eq!(after_second.0, after_first.0 + 1);
}

#[test]
fn editor_plugin_sdk_reports_lifecycle_failures_without_discarding_extensions() {
    struct FailingLifecyclePlugin {
        descriptor: EditorPluginDescriptor,
    }

    impl EditorPlugin for FailingLifecyclePlugin {
        fn descriptor(&self) -> &EditorPluginDescriptor {
            &self.descriptor
        }

        fn register_editor_extensions(
            &self,
            registry: &mut EditorExtensionRegistry,
        ) -> Result<(), crate::core::editor_extension::EditorExtensionRegistryError> {
            let operation_path = EditorOperationPath::parse("sdk.failure.open").map_err(
                crate::core::editor_extension::EditorExtensionRegistryError::OperationPath,
            )?;
            registry.register_command(EditorCommandDescriptor::operation(
                operation_path,
                "Open Failure Panel",
            ))
        }

        fn on_lifecycle_event(
            &self,
            event: &EditorPluginLifecycleEvent,
        ) -> Result<(), EditorPluginLifecycleError> {
            if event.stage() == &EditorPluginLifecycleStage::Enabled {
                return Err(EditorPluginLifecycleError::new(
                    event.stage().clone(),
                    "simulated enable failure",
                ));
            }
            Ok(())
        }
    }

    let plugin = Arc::new(FailingLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new(
            "sdk_failure",
            "SDK Failure",
            "zircon_editor_sdk_failure",
        ),
    });
    let catalog = EditorPluginCatalog::from_plugins([(
        Arc::clone(&plugin) as Arc<dyn EditorPlugin + Send + Sync>,
        PluginPackageManifest::new("sdk_failure", "SDK Failure"),
    )]);
    let manager = EditorPluginManager::new(catalog).expect("the fixture catalog is admissible");
    let snapshot = manager
        .advance_loading_phase(crate::core::plugin::EditorPluginLoadingPhase::Default)
        .expect("the manager should dispatch the enabled lifecycle callback");
    let report = snapshot
        .catalog_snapshot()
        .registration("sdk_failure")
        .expect("the failed plugin registration should remain inspectable");

    assert!(!report.is_success());
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("simulated enable failure")));
    assert!(report
        .extensions
        .pending_command(&EditorOperationPath::parse("sdk.failure.open").unwrap())
        .is_some());
    assert_eq!(report.lifecycle.records().len(), 2);
}

#[test]
fn editor_plugin_sdk_dispatches_post_registration_lifecycle_events() {
    struct RecordingLifecyclePlugin {
        descriptor: EditorPluginDescriptor,
        events: Mutex<Vec<(EditorPluginLifecycleStage, Option<String>)>>,
    }

    impl EditorPlugin for RecordingLifecyclePlugin {
        fn descriptor(&self) -> &EditorPluginDescriptor {
            &self.descriptor
        }

        fn on_lifecycle_event(
            &self,
            event: &EditorPluginLifecycleEvent,
        ) -> Result<(), EditorPluginLifecycleError> {
            self.events
                .lock()
                .expect("lifecycle event fixture lock should not be poisoned")
                .push((event.stage().clone(), event.subject().map(str::to_string)));
            Ok(())
        }
    }

    let plugin = Arc::new(RecordingLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new(
            "sdk_lifecycle",
            "SDK Lifecycle",
            "zircon_editor_sdk_lifecycle",
        ),
        events: Mutex::default(),
    });
    let catalog = EditorPluginCatalog::from_plugins([(
        Arc::clone(&plugin) as Arc<dyn EditorPlugin + Send + Sync>,
        PluginPackageManifest::new("sdk_lifecycle", "SDK Lifecycle"),
    )]);
    let manager = EditorPluginManager::new(catalog).expect("the fixture catalog is admissible");
    manager
        .advance_loading_phase(crate::core::plugin::EditorPluginLoadingPhase::Default)
        .expect("the manager should schedule the default plugin");

    let hot_reload_report = manager
        .dispatch_lifecycle_event(
            "sdk_lifecycle",
            EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::HotReloaded)
                .with_subject("zircon_editor_sdk_lifecycle.dll"),
        )
        .expect("the manager should dispatch post-registration lifecycle events");

    assert!(hot_reload_report.is_success());
    assert_eq!(hot_reload_report.records().len(), 1);
    assert_eq!(
        hot_reload_report.records()[0].event().stage(),
        &EditorPluginLifecycleStage::HotReloaded
    );
    assert_eq!(
        hot_reload_report.records()[0].event().subject(),
        Some("zircon_editor_sdk_lifecycle.dll")
    );
    assert_eq!(
        manager
            .catalog_snapshot()
            .registration("sdk_lifecycle")
            .expect("the lifecycle fixture registration should remain published")
            .lifecycle
            .records()
            .len(),
        3
    );
    assert!(plugin
        .events
        .lock()
        .expect("lifecycle event fixture lock should not be poisoned")
        .iter()
        .any(|(stage, subject)| {
            stage == &EditorPluginLifecycleStage::HotReloaded
                && subject.as_deref() == Some("zircon_editor_sdk_lifecycle.dll")
        }));
}

#[test]
fn editor_plugin_manager_records_lifecycle_events_and_rejects_unknown_plugins() {
    struct RecordingLifecyclePlugin {
        descriptor: EditorPluginDescriptor,
        events: Mutex<Vec<EditorPluginLifecycleStage>>,
    }

    impl EditorPlugin for RecordingLifecyclePlugin {
        fn descriptor(&self) -> &EditorPluginDescriptor {
            &self.descriptor
        }

        fn on_lifecycle_event(
            &self,
            event: &EditorPluginLifecycleEvent,
        ) -> Result<(), EditorPluginLifecycleError> {
            self.events
                .lock()
                .expect("lifecycle event fixture lock should not be poisoned")
                .push(event.stage().clone());
            Ok(())
        }
    }

    let plugin = Arc::new(RecordingLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new(
            "sdk_catalog_lifecycle",
            "SDK Catalog Lifecycle",
            "zircon_editor_sdk_catalog_lifecycle",
        ),
        events: Mutex::default(),
    });
    let catalog = EditorPluginCatalog::from_plugins([(
        Arc::clone(&plugin) as Arc<dyn EditorPlugin + Send + Sync>,
        PluginPackageManifest::new("sdk_catalog_lifecycle", "SDK Catalog Lifecycle"),
    )]);
    let manager = EditorPluginManager::new(catalog).expect("the fixture catalog is admissible");

    let hot_reload_report = manager
        .dispatch_lifecycle_event(
            "sdk_catalog_lifecycle",
            EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::HotReloaded),
        )
        .expect("the manager should own plugin lifecycle dispatch");
    assert!(hot_reload_report.is_success());
    assert_eq!(
        manager
            .catalog_snapshot()
            .registration("sdk_catalog_lifecycle")
            .expect("the lifecycle fixture registration should remain published")
            .lifecycle
            .records()[0]
            .event()
            .stage(),
        &EditorPluginLifecycleStage::HotReloaded
    );

    let unknown_error = manager
        .dispatch_lifecycle_event(
            "sdk_unknown_lifecycle",
            EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::Unloaded),
        )
        .expect_err("unknown plugins must not receive a lifecycle callback");
    assert!(matches!(
        unknown_error,
        EditorPluginTransitionError::UnknownPlugin { .. }
    ));
    assert_eq!(
        plugin
            .events
            .lock()
            .expect("lifecycle event fixture lock should not be poisoned")
            .as_slice(),
        [EditorPluginLifecycleStage::HotReloaded]
    );
}

#[test]
fn asset_contribution_descriptors_normalize_extensions_and_capability_gates() {
    let import_operation = EditorOperationPath::parse("sdk.asset.import_model").unwrap();
    let open_operation = EditorOperationPath::parse("sdk.asset.open_model_inspector").unwrap();

    let mut registry = EditorExtensionRegistry::default();
    registry
        .register_command(EditorCommandDescriptor::operation(
            import_operation.clone(),
            "Import Model",
        ))
        .unwrap();
    registry
        .register_command(EditorCommandDescriptor::operation(
            open_operation.clone(),
            "Open Model Inspector",
        ))
        .unwrap();
    registry
        .register_asset_importer(
            AssetImporterDescriptor::new(
                "sdk.asset.model_importer",
                "SDK Model Importer",
                import_operation,
            )
            .with_source_extension(".GLB")
            .with_source_extension("gltf")
            .with_source_extension("glb")
            .with_output_type(AssetTypeId::from_resource_kind(ResourceKind::Model))
            .with_required_capabilities([
                "editor.extension.asset_authoring",
                "editor.extension.asset_authoring",
            ]),
        )
        .unwrap();
    registry
        .register_asset_type_contribution(
            AssetTypeContribution::augment(AssetTypeId::from_resource_kind(ResourceKind::Model))
                .with_toolkit(
                    AssetToolkitDescriptor::new("sdk.asset.model_inspector", open_operation)
                        .with_required_capabilities(["editor.extension.asset_authoring"]),
                ),
        )
        .unwrap();

    let importer = registry.asset_importers()[0];
    assert_eq!(
        importer.source_extensions(),
        &["glb".to_string(), "gltf".to_string()]
    );
    assert_eq!(
        importer.output_type().map(AssetTypeId::as_str),
        Some("model")
    );
    assert_eq!(
        importer.required_capabilities(),
        &["editor.extension.asset_authoring".to_string()]
    );
    assert_eq!(
        registry.asset_type_contributions()[0].asset_type(),
        &AssetTypeId::from_resource_kind(ResourceKind::Model)
    );
}

#[test]
fn builtin_editor_catalog_declares_authoring_plugin_capabilities() {
    let descriptors = EditorPluginDescriptor::builtin_catalog();

    for (package_id, crate_name, capability) in [
        (
            "terrain",
            "zircon_plugin_terrain_editor",
            "editor.extension.terrain_authoring",
        ),
        (
            "tilemap_2d",
            "zircon_plugin_tilemap_2d_editor",
            "editor.extension.tilemap_2d_authoring",
        ),
        (
            "material_editor",
            "zircon_plugin_material_editor_editor",
            "editor.extension.material_editor_authoring",
        ),
        (
            "prefab_tools",
            "zircon_plugin_prefab_tools_editor",
            "editor.extension.prefab_tools_authoring",
        ),
        (
            "timeline_sequence",
            "zircon_plugin_timeline_sequence_editor",
            "editor.extension.timeline_sequence_authoring",
        ),
        (
            "animation_graph",
            "zircon_plugin_animation_graph_editor",
            "editor.extension.animation_graph_authoring",
        ),
    ] {
        let descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.package_id == package_id)
            .expect("authoring editor plugin should be in builtin editor catalog");

        assert_eq!(descriptor.crate_name, crate_name);
        assert_eq!(descriptor.capabilities, vec![capability.to_string()]);
    }
}

#[test]
fn editor_runtime_gates_asset_authoring_contributions_by_plugin_capability() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_plugin_sdk_asset_authoring_gate",
        &[],
    );
    let capability = "editor.extension.asset_authoring".to_string();
    let import_operation = EditorOperationPath::parse("sdk.asset.import_model").unwrap();
    let open_operation = EditorOperationPath::parse("sdk.asset.open_model_inspector").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(EditorCommandDescriptor::operation(
            import_operation.clone(),
            "Import Model",
        ))
        .unwrap();
    extension
        .register_command(EditorCommandDescriptor::operation(
            open_operation.clone(),
            "Open Model Inspector",
        ))
        .unwrap();
    extension
        .register_asset_importer(
            AssetImporterDescriptor::new(
                "sdk.asset.model_importer",
                "SDK Model Importer",
                import_operation,
            )
            .with_source_extensions(["glb", "gltf"])
            .with_output_type(AssetTypeId::from_resource_kind(ResourceKind::Model)),
        )
        .unwrap();
    extension
        .register_asset_type_contribution(
            AssetTypeContribution::augment(AssetTypeId::from_resource_kind(ResourceKind::Model))
                .with_toolkit(AssetToolkitDescriptor::new(
                    "sdk.asset.model_inspector",
                    open_operation,
                )),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension_with_required_capabilities(
            extension.into_contribution_batch().unwrap(),
            vec![capability.clone()],
        )
        .expect("register asset authoring extension");
    assert!(runtime
        .runtime
        .asset_importers_for_extension(".glb")
        .is_empty());
    let model_type = AssetTypeId::from_resource_kind(ResourceKind::Model);
    assert!(runtime
        .runtime
        .asset_type_definition(&model_type)
        .unwrap()
        .toolkit()
        .is_none());

    let manager = runtime
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    manager
        .set_editor_capabilities_enabled(&[capability], true)
        .unwrap();
    runtime.runtime.refresh_reflection();

    let importers = runtime.runtime.asset_importers_for_extension("GLB");
    assert_eq!(importers.len(), 1);
    assert_eq!(importers[0].id(), "sdk.asset.model_importer");
    let definition = runtime
        .runtime
        .asset_type_definition(&model_type)
        .expect("asset type should be visible after capability is enabled");
    assert_eq!(
        definition.toolkit().unwrap().view_id(),
        "sdk.asset.model_inspector"
    );
}
