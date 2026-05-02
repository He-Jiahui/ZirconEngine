use std::cell::RefCell;

use zircon_runtime::plugin::PluginPackageManifest;

use crate::core::editor_extension::{
    AssetEditorDescriptor, AssetImporterDescriptor, EditorExtensionRegistry,
};
use crate::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};
use crate::core::editor_plugin::{
    EditorPlugin, EditorPluginCatalog, EditorPluginDescriptor, EditorPluginRegistrationReport,
};
use crate::core::editor_plugin_sdk::examples::{
    ExampleAssetInspectorPlugin, ExampleWindowEditorPlugin,
};
use crate::core::editor_plugin_sdk::lifecycle::{
    EditorPluginLifecycleError, EditorPluginLifecycleEvent, EditorPluginLifecycleStage,
};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;

use super::editor_event::support::{env_lock, EventRuntimeHarness};

#[test]
fn editor_plugin_sdk_examples_publish_window_and_asset_contributions() {
    let window = ExampleWindowEditorPlugin::default();
    let asset = ExampleAssetInspectorPlugin::default();
    let plugins: Vec<(&dyn EditorPlugin, PluginPackageManifest)> = vec![
        (
            &window,
            PluginPackageManifest::new("sdk_example_window", "SDK Example Window"),
        ),
        (
            &asset,
            PluginPackageManifest::new("sdk_example_asset", "SDK Example Asset Tools"),
        ),
    ];

    let catalog = crate::core::editor_plugin::EditorPluginCatalog::from_plugins(plugins);
    let extension_report = catalog.editor_extensions();

    assert!(
        extension_report.is_success(),
        "example plugins should aggregate without diagnostics: {:?}",
        extension_report.diagnostics
    );
    let registry = extension_report.registry;
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
        .asset_editors()
        .iter()
        .any(|editor| editor.asset_kind() == "Model"
            && editor.view_id() == "sdk.example.asset_inspector"));
    assert!(registry
        .component_drawers()
        .iter()
        .any(|drawer| drawer.component_type() == "sdk.example.ModelImportSettings"));

    let stages = catalog
        .registrations()
        .iter()
        .flat_map(|registration| {
            registration
                .lifecycle
                .records()
                .iter()
                .map(|record| record.event().stage().clone())
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
            let operation_path = EditorOperationPath::parse("Sdk.Failure.Open")
                .map_err(crate::core::editor_extension::EditorExtensionRegistryError::Operation)?;
            registry.register_operation(EditorOperationDescriptor::new(
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

    let plugin = FailingLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new(
            "sdk_failure",
            "SDK Failure",
            "zircon_editor_sdk_failure",
        ),
    };

    let report = EditorPluginRegistrationReport::from_plugin(
        &plugin,
        PluginPackageManifest::new("sdk_failure", "SDK Failure"),
    );

    assert!(!report.is_success());
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("simulated enable failure")));
    assert!(report
        .extensions
        .operations()
        .descriptor(&EditorOperationPath::parse("Sdk.Failure.Open").unwrap())
        .is_some());
    assert_eq!(report.lifecycle.records().len(), 2);
}

#[test]
fn editor_plugin_sdk_dispatches_post_registration_lifecycle_events() {
    struct RecordingLifecyclePlugin {
        descriptor: EditorPluginDescriptor,
        events: RefCell<Vec<(EditorPluginLifecycleStage, Option<String>)>>,
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
                .borrow_mut()
                .push((event.stage().clone(), event.subject().map(str::to_string)));
            Ok(())
        }
    }

    let plugin = RecordingLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new(
            "sdk_lifecycle",
            "SDK Lifecycle",
            "zircon_editor_sdk_lifecycle",
        ),
        events: RefCell::default(),
    };
    let mut report = EditorPluginRegistrationReport::from_plugin(
        &plugin,
        PluginPackageManifest::new("sdk_lifecycle", "SDK Lifecycle"),
    );

    let hot_reload_report = report.record_lifecycle_event(
        &plugin,
        EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::HotReloaded)
            .with_subject("zircon_editor_sdk_lifecycle.dll"),
    );

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
    assert_eq!(report.lifecycle.records().len(), 3);
    assert!(plugin.events.borrow().iter().any(|(stage, subject)| {
        stage == &EditorPluginLifecycleStage::HotReloaded
            && subject.as_deref() == Some("zircon_editor_sdk_lifecycle.dll")
    }));
}

#[test]
fn editor_plugin_catalog_records_registered_lifecycle_events_and_rejects_unknown_plugins() {
    struct RecordingLifecyclePlugin {
        descriptor: EditorPluginDescriptor,
        events: RefCell<Vec<EditorPluginLifecycleStage>>,
    }

    impl EditorPlugin for RecordingLifecyclePlugin {
        fn descriptor(&self) -> &EditorPluginDescriptor {
            &self.descriptor
        }

        fn on_lifecycle_event(
            &self,
            event: &EditorPluginLifecycleEvent,
        ) -> Result<(), EditorPluginLifecycleError> {
            self.events.borrow_mut().push(event.stage().clone());
            Ok(())
        }
    }

    let plugin = RecordingLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new(
            "sdk_catalog_lifecycle",
            "SDK Catalog Lifecycle",
            "zircon_editor_sdk_catalog_lifecycle",
        ),
        events: RefCell::default(),
    };
    let unknown_plugin = RecordingLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new(
            "sdk_unknown_lifecycle",
            "SDK Unknown Lifecycle",
            "zircon_editor_sdk_unknown_lifecycle",
        ),
        events: RefCell::default(),
    };
    let mut catalog = EditorPluginCatalog::from_plugins([(
        &plugin as &dyn EditorPlugin,
        PluginPackageManifest::new("sdk_catalog_lifecycle", "SDK Catalog Lifecycle"),
    )]);

    let disabled_report = catalog.record_lifecycle_event(
        &plugin,
        EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::Disabled),
    );
    assert!(disabled_report.is_success());
    assert_eq!(
        catalog.registrations()[0].lifecycle.records()[2]
            .event()
            .stage(),
        &EditorPluginLifecycleStage::Disabled
    );

    let unknown_report = catalog.record_lifecycle_event(
        &unknown_plugin,
        EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::Unloaded),
    );
    assert!(!unknown_report.is_success());
    assert!(unknown_report
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.contains("sdk_unknown_lifecycle")));
    assert!(unknown_plugin.events.borrow().is_empty());
}

#[test]
fn asset_contribution_descriptors_normalize_extensions_and_capability_gates() {
    let import_operation = EditorOperationPath::parse("Sdk.Asset.ImportModel").unwrap();
    let open_operation = EditorOperationPath::parse("Sdk.Asset.OpenModelInspector").unwrap();

    let mut registry = EditorExtensionRegistry::default();
    registry
        .register_operation(EditorOperationDescriptor::new(
            import_operation.clone(),
            "Import Model",
        ))
        .unwrap();
    registry
        .register_operation(EditorOperationDescriptor::new(
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
            .with_output_kind("Model")
            .with_required_capabilities([
                "editor.extension.asset_authoring",
                "editor.extension.asset_authoring",
            ]),
        )
        .unwrap();
    registry
        .register_asset_editor(
            AssetEditorDescriptor::new(
                "Model",
                "sdk.asset.model_inspector",
                "SDK Model Inspector",
                open_operation,
            )
            .with_required_capabilities(["editor.extension.asset_authoring"]),
        )
        .unwrap();

    let importer = registry.asset_importers()[0];
    assert_eq!(
        importer.source_extensions(),
        &["glb".to_string(), "gltf".to_string()]
    );
    assert_eq!(importer.output_kind(), Some("Model"));
    assert_eq!(
        importer.required_capabilities(),
        &["editor.extension.asset_authoring".to_string()]
    );
    assert_eq!(registry.asset_editors()[0].asset_kind(), "Model");
}

#[test]
fn editor_runtime_gates_asset_authoring_contributions_by_plugin_capability() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_plugin_sdk_asset_authoring_gate",
        &[],
    );
    let capability = "editor.extension.asset_authoring".to_string();
    let import_operation = EditorOperationPath::parse("Sdk.Asset.ImportModel").unwrap();
    let open_operation = EditorOperationPath::parse("Sdk.Asset.OpenModelInspector").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(EditorOperationDescriptor::new(
            import_operation.clone(),
            "Import Model",
        ))
        .unwrap();
    extension
        .register_operation(EditorOperationDescriptor::new(
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
            .with_output_kind("Model"),
        )
        .unwrap();
    extension
        .register_asset_editor(AssetEditorDescriptor::new(
            "Model",
            "sdk.asset.model_inspector",
            "SDK Model Inspector",
            open_operation,
        ))
        .unwrap();

    runtime
        .runtime
        .register_editor_extension_with_required_capabilities(extension, vec![capability.clone()])
        .expect("register asset authoring extension");
    assert!(runtime
        .runtime
        .asset_importers_for_extension(".glb")
        .is_empty());
    assert!(runtime.runtime.asset_editor_descriptor("Model").is_none());

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
    let editor = runtime
        .runtime
        .asset_editor_descriptor("Model")
        .expect("asset editor should be visible after capability is enabled");
    assert_eq!(editor.view_id(), "sdk.asset.model_inspector");
}
