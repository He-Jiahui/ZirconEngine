use crate::asset::{
    AssetImportContext, AssetImportOutcome, AssetImporterDescriptor, FunctionAssetImporter,
    ImportedAsset,
};
use crate::core::framework::render::{
    RenderFrameExtract, RenderPipelineHandle, RenderViewportDescriptor, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::core::{ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode};
use crate::engine_module::{factory, qualified_name};
use crate::graphics::{
    HybridGiRuntimeFeedback, HybridGiRuntimePrepareInput, HybridGiRuntimePrepareOutput,
    HybridGiRuntimeProvider, HybridGiRuntimeProviderRegistration, HybridGiRuntimeState,
    HybridGiRuntimeUpdate, RenderFeatureDescriptor, RenderPassExecutionContext,
    RenderPassExecutorId, RenderPassExecutorRegistration, RenderPassStage, RenderPipelineAsset,
    VirtualGeometryRuntimeFeedback, VirtualGeometryRuntimePrepareInput,
    VirtualGeometryRuntimePrepareOutput, VirtualGeometryRuntimeProvider,
    VirtualGeometryRuntimeProviderRegistration, VirtualGeometryRuntimeState,
    VirtualGeometryRuntimeUpdate,
};
use crate::plugin::{
    ComponentTypeDescriptor, PluginEventCatalogManifest, PluginEventManifest,
    PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest,
    PluginOptionManifest, PluginPackageManifest, ProjectPluginFeatureSelection,
    RuntimeExtensionRegistry, RuntimePlugin, RuntimePluginCatalog, RuntimePluginDescriptor,
    RuntimePluginFeature, RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
    SceneRuntimeHook, SceneRuntimeHookContext, SceneRuntimeHookDescriptor,
    SceneRuntimeHookRegistration, UiComponentDescriptor,
};
use crate::scene::{components::NodeKind, components::SystemStage, World};
use crate::ui::component::UiComponentDescriptorRegistry;
use crate::RenderFeaturePassDescriptor;
use crate::{asset, core::manager::RenderFrameworkHandle, render_graph::QueueLane};
use crate::{
    plugin::ExportPackagingStrategy, plugin::ProjectPluginManifest, plugin::ProjectPluginSelection,
    RuntimePluginId, RuntimeTargetMode,
};
use zircon_runtime_interface::ui::component::{UiComponentCategory, UiSlotSchema, UiValue};

#[test]
fn runtime_extension_registry_collects_plugin_manager_and_component_contributions() {
    let mut registry = RuntimeExtensionRegistry::default();
    let manager = ManagerDescriptor::new(
        qualified_name("WeatherPlugin", ServiceKind::Manager, "WeatherManager"),
        StartupMode::Lazy,
        Vec::new(),
        factory(|_| Ok(std::sync::Arc::new(()) as ServiceObject)),
    );
    let component =
        ComponentTypeDescriptor::new("weather.Component.CloudLayer", "weather", "Cloud Layer")
            .with_property("coverage", "float", true)
            .with_property("tint", "vec4", true);
    let ui_component = UiComponentDescriptor::new(
        "weather.Ui.CloudLayerInspector",
        "weather",
        "asset://weather/editor/cloud_layer_inspector.ui.toml",
    );

    registry
        .register_manager("weather", manager.clone())
        .expect("manager contribution");
    registry
        .register_component(component.clone())
        .expect("component contribution");
    registry
        .register_ui_component(ui_component.clone())
        .expect("ui component contribution");

    assert_eq!(registry.managers().len(), 1);
    assert_eq!(registry.components(), &[component]);
    assert_eq!(registry.ui_components(), &[ui_component]);

    let module = ModuleDescriptor::new("WeatherPlugin", "Weather plugin").with_manager(manager);
    let merged = registry.apply_to_module(module);
    assert_eq!(merged.managers.len(), 2);
}

#[test]
fn runtime_extension_registry_collects_plugin_module_and_render_feature_contributions() {
    let mut registry = RuntimeExtensionRegistry::default();
    let module = ModuleDescriptor::new("WeatherPlugin", "Weather simulation plugin");
    let render_feature = RenderFeatureDescriptor {
        name: "weather.volumetric_clouds".to_string(),
        required_extract_sections: vec!["weather.cloud_volume".to_string()],
        capability_requirements: Vec::new(),
        history_bindings: Vec::new(),
        stage_passes: Vec::new(),
    };

    registry
        .register_module(module.clone())
        .expect("module contribution");
    registry
        .register_render_feature(render_feature.clone())
        .expect("render feature contribution");

    assert_eq!(registry.modules().len(), 1);
    assert_eq!(registry.modules()[0].name, module.name);
    assert_eq!(registry.render_features(), &[render_feature]);
}

#[test]
fn runtime_extension_registry_collects_scene_hook_contributions_in_stage_order() {
    let mut registry = RuntimeExtensionRegistry::default();

    registry
        .register_scene_hook(scene_hook_registration(
            "weather.scene.update-late",
            SystemStage::Update,
            20,
        ))
        .expect("late update hook contribution");
    registry
        .register_scene_hook(scene_hook_registration(
            "weather.scene.fixed",
            SystemStage::FixedUpdate,
            0,
        ))
        .expect("fixed hook contribution");
    registry
        .register_scene_hook(scene_hook_registration(
            "weather.scene.update-early",
            SystemStage::Update,
            -10,
        ))
        .expect("early update hook contribution");

    let hook_ids = registry
        .scene_hooks()
        .iter()
        .map(|hook| hook.descriptor().id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        hook_ids,
        vec![
            "weather.scene.fixed",
            "weather.scene.update-early",
            "weather.scene.update-late",
        ]
    );
}

#[test]
fn runtime_extension_registry_rejects_duplicate_and_invalid_scene_hooks() {
    let mut registry = RuntimeExtensionRegistry::default();

    registry
        .register_scene_hook(scene_hook_registration(
            "weather.scene.update",
            SystemStage::Update,
            0,
        ))
        .expect("first hook contribution");
    let duplicate = registry
        .register_scene_hook(scene_hook_registration(
            "weather.scene.update",
            SystemStage::Update,
            1,
        ))
        .unwrap_err();
    assert!(duplicate
        .to_string()
        .contains("scene hook weather.scene.update already registered"));

    let invalid = registry
        .register_scene_hook(SceneRuntimeHookRegistration::new(
            SceneRuntimeHookDescriptor::new("cloud.scene.update", "weather", SystemStage::Update),
            NoopSceneHook,
        ))
        .unwrap_err();
    assert!(invalid
        .to_string()
        .contains("scene hook cloud.scene.update must be prefixed by plugin id weather"));
}

#[test]
fn level_tick_dispatches_installed_scene_hooks_in_schedule_order() {
    let runtime = crate::core::CoreRuntime::new();
    runtime
        .register_module(crate::scene::module_descriptor())
        .unwrap();
    runtime
        .activate_module(crate::scene::SCENE_MODULE_NAME)
        .unwrap();

    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_scene_hook(recording_scene_hook_registration(
            "weather.scene.update",
            SystemStage::Update,
            0,
            "update",
        ))
        .expect("update hook contribution");
    registry
        .register_scene_hook(recording_scene_hook_registration(
            "weather.scene.pre-update",
            SystemStage::PreUpdate,
            0,
            "pre-update",
        ))
        .expect("pre-update hook contribution");
    runtime
        .install_scene_runtime_hooks(&registry)
        .expect("install scene hooks into core runtime");

    let level = crate::scene::create_default_level(&runtime.handle()).unwrap();
    level.tick(&runtime.handle(), 1.0 / 60.0).unwrap();

    assert_eq!(
        level.registered_subsystems(),
        vec!["pre-update".to_string(), "update".to_string()]
    );
}

#[test]
fn runtime_extension_registry_collects_asset_importer_contributions() {
    let mut registry = RuntimeExtensionRegistry::default();
    let importer = FunctionAssetImporter::new(
        AssetImporterDescriptor::new("weather.data", "weather", crate::asset::AssetKind::Data, 7)
            .with_source_extensions(["weather"])
            .with_required_capabilities(["runtime.asset.importer.data"]),
        weather_data_importer,
    );

    registry
        .register_asset_importer(importer)
        .expect("asset importer contribution");

    assert_eq!(registry.asset_importers().descriptors().len(), 1);
    assert_eq!(
        registry.asset_importers().descriptors()[0].id,
        "weather.data"
    );
    assert_eq!(
        registry.asset_importers().descriptors()[0].importer_version,
        7
    );
}

#[test]
fn runtime_plugin_registration_collects_package_manifest_declared_runtime_contributions() {
    let plugin = ManifestDeclaredRuntimePlugin {
        descriptor: RuntimePluginDescriptor::new(
            "weather",
            "Weather",
            RuntimePluginId::Particles,
            "zircon_plugin_weather_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_capability("runtime.plugin.weather"),
    };
    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert_eq!(registration.extensions.plugin_options().len(), 1);
    assert_eq!(
        registration.extensions.plugin_options()[0].key,
        "weather.precipitation"
    );
    assert_eq!(registration.extensions.plugin_event_catalogs().len(), 1);
    assert_eq!(
        registration.extensions.plugin_event_catalogs()[0].namespace,
        "weather.events"
    );
    assert_eq!(registration.extensions.components().len(), 1);
    assert_eq!(
        registration.extensions.components()[0].type_id,
        "weather.Component.CloudLayer"
    );
    assert_eq!(registration.extensions.ui_components().len(), 1);
    assert_eq!(
        registration.extensions.ui_components()[0].component_id,
        "weather.Ui.CloudLayerInspector"
    );
    assert_eq!(
        registration.extensions.asset_importers().descriptors()[0].id,
        "weather.data"
    );

    let catalog = RuntimePluginCatalog::from_registration_reports([registration], []);
    let report = catalog.runtime_extensions();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.registry.plugin_options().len(), 1);
    assert_eq!(report.registry.plugin_event_catalogs().len(), 1);
    assert_eq!(report.registry.components().len(), 1);
    assert_eq!(report.registry.ui_components().len(), 1);
    assert_eq!(report.registry.asset_importers().descriptors().len(), 1);
}

#[test]
fn runtime_extension_registry_rejects_duplicate_asset_importer_matchers() {
    let mut registry = RuntimeExtensionRegistry::default();
    let first = FunctionAssetImporter::new(
        AssetImporterDescriptor::new("weather.first", "weather", crate::asset::AssetKind::Data, 1)
            .with_source_extensions(["weather"]),
        weather_data_importer,
    );
    let second = FunctionAssetImporter::new(
        AssetImporterDescriptor::new(
            "weather.second",
            "weather",
            crate::asset::AssetKind::Data,
            1,
        )
        .with_source_extensions(["weather"]),
        weather_data_importer,
    );

    registry
        .register_asset_importer(first)
        .expect("first asset importer");
    let error = registry.register_asset_importer(second).unwrap_err();

    assert!(error.to_string().contains("duplicate importer matcher"));
}

#[test]
fn runtime_extension_registry_collects_render_pass_executor_contributions() {
    let mut registry = RuntimeExtensionRegistry::default();
    let registration =
        RenderPassExecutorRegistration::new("weather.volumetric-clouds", weather_render_executor);

    registry
        .register_render_pass_executor(registration)
        .expect("executor contribution");

    assert_eq!(registry.render_pass_executors().len(), 1);
    assert_eq!(
        registry.render_pass_executors()[0].executor_id(),
        &RenderPassExecutorId::new("weather.volumetric-clouds")
    );
}

#[test]
fn runtime_extension_registry_collects_virtual_geometry_runtime_provider_contributions() {
    let mut registry = RuntimeExtensionRegistry::default();
    let provider = VirtualGeometryRuntimeProviderRegistration::new(
        "weather.virtual_geometry",
        std::sync::Arc::new(NoopVirtualGeometryRuntimeProvider),
    );

    registry
        .register_virtual_geometry_runtime_provider(provider.clone())
        .expect("provider contribution");

    assert_eq!(
        registry.virtual_geometry_runtime_providers()[0].provider_id(),
        provider.provider_id()
    );
}

#[test]
fn runtime_extension_registry_collects_hybrid_gi_runtime_provider_contributions() {
    let mut registry = RuntimeExtensionRegistry::default();
    let provider = HybridGiRuntimeProviderRegistration::new(
        "weather.hybrid_gi",
        std::sync::Arc::new(NoopHybridGiRuntimeProvider),
    );

    registry
        .register_hybrid_gi_runtime_provider(provider.clone())
        .expect("provider contribution");

    assert_eq!(
        registry.hybrid_gi_runtime_providers()[0].provider_id(),
        provider.provider_id()
    );
}

#[test]
fn runtime_extension_registry_rejects_duplicate_module_and_render_feature_names() {
    let mut registry = RuntimeExtensionRegistry::default();
    let manager = ManagerDescriptor::new(
        qualified_name("WeatherPlugin", ServiceKind::Manager, "WeatherManager"),
        StartupMode::Lazy,
        Vec::new(),
        factory(|_| Ok(std::sync::Arc::new(()) as ServiceObject)),
    );
    let render_feature = RenderFeatureDescriptor {
        name: "weather.volumetric_clouds".to_string(),
        required_extract_sections: Vec::new(),
        capability_requirements: Vec::new(),
        history_bindings: Vec::new(),
        stage_passes: Vec::new(),
    };

    registry
        .register_manager("weather", manager.clone())
        .expect("first manager");
    let duplicate_manager = registry.register_manager("weather", manager).unwrap_err();
    assert!(duplicate_manager
        .to_string()
        .contains("manager WeatherPlugin.Manager.WeatherManager already registered"));

    registry
        .register_module(ModuleDescriptor::new("WeatherPlugin", "Weather plugin"))
        .expect("first module");
    let duplicate_module = registry
        .register_module(ModuleDescriptor::new(
            "WeatherPlugin",
            "Duplicate weather plugin",
        ))
        .unwrap_err();
    assert!(duplicate_module
        .to_string()
        .contains("module WeatherPlugin already registered"));

    registry
        .register_render_feature(render_feature.clone())
        .expect("first render feature");
    let duplicate_render_feature = registry
        .register_render_feature(render_feature)
        .unwrap_err();
    assert!(duplicate_render_feature
        .to_string()
        .contains("render feature weather.volumetric_clouds already registered"));

    let executor =
        RenderPassExecutorRegistration::new("weather.volumetric-clouds", weather_render_executor);
    registry
        .register_render_pass_executor(executor.clone())
        .expect("first executor");
    let duplicate_executor = registry
        .register_render_pass_executor(executor)
        .unwrap_err();
    assert!(duplicate_executor
        .to_string()
        .contains("render pass executor weather.volumetric-clouds already registered"));

    let provider = VirtualGeometryRuntimeProviderRegistration::new(
        "weather.virtual_geometry",
        std::sync::Arc::new(NoopVirtualGeometryRuntimeProvider),
    );
    registry
        .register_virtual_geometry_runtime_provider(provider.clone())
        .expect("first provider");
    let duplicate_provider = registry
        .register_virtual_geometry_runtime_provider(provider)
        .unwrap_err();
    assert!(duplicate_provider
        .to_string()
        .contains("virtual geometry runtime provider weather.virtual_geometry already registered"));

    let hybrid_gi_provider = HybridGiRuntimeProviderRegistration::new(
        "weather.hybrid_gi",
        std::sync::Arc::new(NoopHybridGiRuntimeProvider),
    );
    registry
        .register_hybrid_gi_runtime_provider(hybrid_gi_provider.clone())
        .expect("first hybrid GI provider");
    let duplicate_hybrid_gi_provider = registry
        .register_hybrid_gi_runtime_provider(hybrid_gi_provider)
        .unwrap_err();
    assert!(duplicate_hybrid_gi_provider
        .to_string()
        .contains("hybrid GI runtime provider weather.hybrid_gi already registered"));
}

#[test]
fn runtime_plugin_catalog_merges_module_and_render_feature_contributions() {
    let plugin = WeatherRuntimePlugin {
        descriptor: RuntimePluginDescriptor::new(
            "weather",
            "Weather",
            RuntimePluginId::Particles,
            "zircon_plugin_weather_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime]),
    };
    let catalog = RuntimePluginCatalog::from_plugins([&plugin as &dyn RuntimePlugin]);
    let report = catalog.runtime_extensions();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.registry.modules().len(), 1);
    assert_eq!(report.registry.modules()[0].name, "WeatherPlugin");
    assert_eq!(report.registry.render_features().len(), 1);
    assert_eq!(
        report.registry.render_features()[0].name,
        "weather.volumetric_clouds"
    );
    assert_eq!(report.registry.render_pass_executors().len(), 1);
    assert_eq!(
        report.registry.render_pass_executors()[0]
            .executor_id()
            .as_str(),
        "weather.volumetric-clouds"
    );
    assert_eq!(
        report.registry.virtual_geometry_runtime_providers()[0].provider_id(),
        "weather.virtual_geometry"
    );
    assert_eq!(
        report.registry.hybrid_gi_runtime_providers()[0].provider_id(),
        "weather.hybrid_gi"
    );
    assert_eq!(report.registry.scene_hooks().len(), 1);
    assert_eq!(
        report.registry.scene_hooks()[0].descriptor().id.as_str(),
        "weather.scene.update"
    );
}

#[test]
fn runtime_plugin_catalog_merges_available_feature_extensions_after_base_plugins() {
    let feature = SoundTimelineFeaturePlugin;
    let mut catalog = RuntimePluginCatalog::from_descriptors([
        RuntimePluginDescriptor::new(
            "sound",
            "Sound",
            RuntimePluginId::Sound,
            "zircon_plugin_sound_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capability("runtime.plugin.sound")
        .with_optional_feature(feature.manifest()),
        RuntimePluginDescriptor::new(
            "animation",
            "Animation",
            RuntimePluginId::Animation,
            "zircon_plugin_animation_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capability("runtime.feature.animation.timeline_event_track"),
    ]);
    catalog.register_feature(&feature);
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                        .enabled(true),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };

    let report =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.registry.modules().len(), 1);
    assert_eq!(
        report.registry.modules()[0].name,
        "SoundTimelineAnimationFeatureModule"
    );
}

#[test]
fn runtime_plugin_catalog_reports_duplicate_feature_runtime_registrations() {
    let feature = SoundTimelineFeaturePlugin;
    let mut catalog = RuntimePluginCatalog::from_descriptors([
        RuntimePluginDescriptor::new(
            "sound",
            "Sound",
            RuntimePluginId::Sound,
            "zircon_plugin_sound_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capability("runtime.plugin.sound")
        .with_optional_feature(feature.manifest()),
        RuntimePluginDescriptor::new(
            "animation",
            "Animation",
            RuntimePluginId::Animation,
            "zircon_plugin_animation_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capability("runtime.feature.animation.timeline_event_track"),
    ]);
    catalog.register_feature(&feature);
    catalog.register_feature(&feature);
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                        .enabled(true),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };

    let report =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(!report.is_success());
    assert!(report.has_fatal_diagnostics());
    assert!(report
        .fatal_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains(
            "duplicate optional feature id sound.timeline_animation_track registered at runtime"
        )));
}

#[test]
fn runtime_plugin_catalog_reports_conflicting_feature_defaults_between_package_and_runtime() {
    let feature = SoundTimelineFeaturePlugin;
    let declared_feature = feature
        .manifest()
        .with_default_packaging([ExportPackagingStrategy::NativeDynamic])
        .enabled_by_default(true);
    let mut catalog = RuntimePluginCatalog::from_descriptors([RuntimePluginDescriptor::new(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_optional_feature(declared_feature)]);
    catalog.register_feature(&feature);

    let report = catalog.feature_dependency_report(
        &ProjectPluginManifest {
            selections: Vec::new(),
        },
        RuntimeTargetMode::ClientRuntime,
    );

    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains(
        "optional feature id sound.timeline_animation_track has conflicting package manifest and runtime registration"
    )));
}

#[test]
fn runtime_extension_catalog_treats_blocked_optional_features_as_warnings() {
    let mut catalog = RuntimePluginCatalog::from_descriptors([RuntimePluginDescriptor::new(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.sound")]);
    let feature = SoundTimelineFeaturePlugin;
    catalog.register_feature(&feature);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track").enabled(true),
        )],
    };

    let report =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(report.is_success(), "{:?}", report.fatal_diagnostics);
    assert!(report.fatal_diagnostics.is_empty());
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("optional feature sound.timeline_animation_track is blocked")));
    assert!(report.registry.modules().is_empty());
}

#[test]
fn runtime_extension_catalog_treats_blocked_required_features_as_fatal() {
    let mut catalog = RuntimePluginCatalog::from_descriptors([RuntimePluginDescriptor::new(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.sound")]);
    let feature = SoundTimelineFeaturePlugin;
    catalog.register_feature(&feature);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                .enabled(true)
                .required(true),
        )],
    };

    let report =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(!report.is_success());
    assert!(report.has_fatal_diagnostics());
    assert!(report.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("required feature sound.timeline_animation_track is blocked")));
    assert!(report.registry.modules().is_empty());
}

#[test]
fn runtime_module_load_reports_blocked_optional_features_as_warnings() {
    let sound = RuntimePluginDescriptor::new(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.sound");
    let feature = SoundTimelineFeaturePlugin;
    let sound_registration = RuntimePluginRegistrationReport::from_plugin(&sound);
    let feature_registration = RuntimePluginFeatureRegistrationReport::from_feature(&feature);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track").enabled(true),
        )],
    };

    let report = crate::runtime_modules_for_target_with_plugin_and_feature_registration_reports(
        RuntimeTargetMode::ClientRuntime,
        Some(&manifest),
        [&sound_registration],
        [&feature_registration],
    );

    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning
            .contains("optional feature sound.timeline_animation_track is blocked")));
}

#[test]
fn runtime_module_load_reports_blocked_required_features_as_errors() {
    let sound = RuntimePluginDescriptor::new(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.sound");
    let feature = SoundTimelineFeaturePlugin;
    let sound_registration = RuntimePluginRegistrationReport::from_plugin(&sound);
    let feature_registration = RuntimePluginFeatureRegistrationReport::from_feature(&feature);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                .enabled(true)
                .required(true),
        )],
    };

    let report = crate::runtime_modules_for_target_with_plugin_and_feature_registration_reports(
        RuntimeTargetMode::ClientRuntime,
        Some(&manifest),
        [&sound_registration],
        [&feature_registration],
    );

    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    assert!(report.errors.iter().any(|error| {
        error.contains("required feature sound.timeline_animation_track is blocked")
    }));
}

#[test]
fn runtime_modules_propagate_reported_executor_registrations_into_render_framework() {
    let plugin = WeatherRuntimePlugin {
        descriptor: RuntimePluginDescriptor::new(
            "weather",
            "Weather",
            RuntimePluginId::Particles,
            "zircon_plugin_weather_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime]),
    };
    let registration = crate::plugin::RuntimePluginRegistrationReport::from_plugin(&plugin);
    assert!(registration.is_success(), "{:?}", registration.diagnostics);

    let modules = crate::runtime_modules_for_target_with_plugin_registration_reports(
        RuntimeTargetMode::ClientRuntime,
        None,
        [&registration],
    );
    let runtime = crate::core::CoreRuntime::new();
    for module in modules.modules {
        runtime.register_module(module.descriptor()).unwrap();
    }
    runtime.activate_module(asset::ASSET_MODULE_NAME).unwrap();
    runtime
        .activate_module(crate::graphics::GRAPHICS_MODULE_NAME)
        .unwrap();
    let framework = runtime
        .resolve_manager::<RenderFrameworkHandle>(crate::core::manager::RENDER_FRAMEWORK_NAME)
        .unwrap()
        .shared();

    let mut pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([weather_render_feature_descriptor()]);
    pipeline.handle = RenderPipelineHandle::new(91);
    pipeline.name = "weather-executor-propagation".to_string();
    let pipeline = framework
        .register_pipeline_asset(pipeline)
        .expect("reported executor should validate through the render framework");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 64)))
        .expect("viewport");
    framework
        .set_pipeline_asset(viewport, pipeline)
        .expect("plugin pipeline should be selectable after registration");

    let error = framework
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(1),
                World::new().to_render_snapshot(),
            ),
        )
        .expect_err("reported executor should replace descriptor no-op and run during submission");

    assert!(
        error
            .to_string()
            .contains("weather executor reached graph execution"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn runtime_plugin_catalog_reports_duplicate_manager_contributions() {
    let left = ManagerRuntimePlugin {
        descriptor: RuntimePluginDescriptor::new(
            "weather_left",
            "Weather Left",
            RuntimePluginId::Particles,
            "zircon_plugin_weather_left_runtime",
        ),
    };
    let right = ManagerRuntimePlugin {
        descriptor: RuntimePluginDescriptor::new(
            "weather_right",
            "Weather Right",
            RuntimePluginId::HybridGi,
            "zircon_plugin_weather_right_runtime",
        ),
    };
    let catalog = RuntimePluginCatalog::from_plugins([
        &left as &dyn RuntimePlugin,
        &right as &dyn RuntimePlugin,
    ]);
    let report = catalog.runtime_extensions();

    assert!(!report.is_success());
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("manager WeatherPlugin.Manager.WeatherManager already registered")));
    assert_eq!(report.registry.managers().len(), 1);
}

#[derive(Debug)]
struct ManagerRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl RuntimePlugin for ManagerRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), crate::plugin::RuntimeExtensionRegistryError> {
        registry.register_manager(
            self.descriptor().package_id.clone(),
            ManagerDescriptor::new(
                qualified_name("WeatherPlugin", ServiceKind::Manager, "WeatherManager"),
                StartupMode::Lazy,
                Vec::new(),
                factory(|_| Ok(std::sync::Arc::new(()) as ServiceObject)),
            ),
        )
    }
}

#[derive(Debug)]
struct ManifestDeclaredRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl RuntimePlugin for ManifestDeclaredRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        self.descriptor
            .package_manifest()
            .with_option(
                PluginOptionManifest::new("weather.precipitation", "Precipitation", "bool", "true")
                    .with_required_capability("runtime.plugin.weather"),
            )
            .with_event_catalog(PluginEventCatalogManifest {
                namespace: "weather.events".to_string(),
                version: 1,
                events: vec![PluginEventManifest {
                    id: "weather.events.StormFrontArrived".to_string(),
                    display_name: "Storm Front Arrived".to_string(),
                    payload_schema: "weather.schemas.StormFrontPayload.v1".to_string(),
                }],
            })
            .with_component(ComponentTypeDescriptor::new(
                "weather.Component.CloudLayer",
                "weather",
                "Cloud Layer",
            ))
            .with_ui_component(UiComponentDescriptor::new(
                "weather.Ui.CloudLayerInspector",
                "weather",
                "asset://weather/editor/cloud_layer_inspector.ui.toml",
            ))
            .with_asset_importer(
                AssetImporterDescriptor::new(
                    "weather.data",
                    "weather",
                    crate::asset::AssetKind::Data,
                    7,
                )
                .with_source_extensions(["weather"]),
            )
    }
}

#[derive(Debug)]
struct WeatherRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl RuntimePlugin for WeatherRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), crate::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(ModuleDescriptor::new(
            "WeatherPlugin",
            "Weather simulation plugin",
        ))?;
        registry.register_render_feature(RenderFeatureDescriptor {
            name: "weather.volumetric_clouds".to_string(),
            required_extract_sections: vec!["weather.cloud_volume".to_string()],
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            stage_passes: Vec::new(),
        })?;
        registry.register_render_pass_executor(RenderPassExecutorRegistration::new(
            "weather.volumetric-clouds",
            weather_render_executor,
        ))?;
        registry.register_virtual_geometry_runtime_provider(
            VirtualGeometryRuntimeProviderRegistration::new(
                "weather.virtual_geometry",
                std::sync::Arc::new(NoopVirtualGeometryRuntimeProvider),
            ),
        )?;
        registry.register_hybrid_gi_runtime_provider(HybridGiRuntimeProviderRegistration::new(
            "weather.hybrid_gi",
            std::sync::Arc::new(NoopHybridGiRuntimeProvider),
        ))?;
        registry.register_scene_hook(scene_hook_registration(
            "weather.scene.update",
            SystemStage::Update,
            0,
        ))?;
        Ok(())
    }
}

#[derive(Debug)]
struct NoopSceneHook;

impl SceneRuntimeHook for NoopSceneHook {
    fn run(&self, _context: SceneRuntimeHookContext<'_>) -> Result<(), crate::core::CoreError> {
        Ok(())
    }
}

#[derive(Debug)]
struct RecordingSceneHook {
    label: &'static str,
}

impl SceneRuntimeHook for RecordingSceneHook {
    fn run(&self, context: SceneRuntimeHookContext<'_>) -> Result<(), crate::core::CoreError> {
        context.level.register_subsystem(self.label);
        Ok(())
    }
}

fn scene_hook_registration(
    id: &str,
    stage: SystemStage,
    order: i32,
) -> SceneRuntimeHookRegistration {
    SceneRuntimeHookRegistration::new(
        SceneRuntimeHookDescriptor::new(id, "weather", stage).with_order(order),
        NoopSceneHook,
    )
}

fn recording_scene_hook_registration(
    id: &str,
    stage: SystemStage,
    order: i32,
    label: &'static str,
) -> SceneRuntimeHookRegistration {
    SceneRuntimeHookRegistration::new(
        SceneRuntimeHookDescriptor::new(id, "weather", stage).with_order(order),
        RecordingSceneHook { label },
    )
}

#[derive(Debug)]
struct SoundTimelineFeaturePlugin;

impl RuntimePluginFeature for SoundTimelineFeaturePlugin {
    fn manifest(&self) -> PluginFeatureBundleManifest {
        PluginFeatureBundleManifest::new(
            "sound.timeline_animation_track",
            "Sound Timeline Animation Track",
            "sound",
        )
        .with_dependency(PluginFeatureDependency::primary(
            "sound",
            "runtime.plugin.sound",
        ))
        .with_dependency(PluginFeatureDependency::required(
            "animation",
            "runtime.feature.animation.timeline_event_track",
        ))
        .with_capability("runtime.feature.sound.timeline_animation_track")
        .with_runtime_module(
            PluginModuleManifest::runtime(
                "sound.timeline_animation_track.runtime",
                "zircon_plugin_sound_timeline_animation_runtime",
            )
            .with_target_modes([
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ]),
        )
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), crate::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(ModuleDescriptor::new(
            "SoundTimelineAnimationFeatureModule",
            "Sound timeline animation track feature",
        ))
    }
}

fn weather_render_executor(_context: &RenderPassExecutionContext) -> Result<(), String> {
    Err("weather executor reached graph execution".to_string())
}

fn weather_data_importer(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, crate::asset::AssetImportError> {
    Ok(AssetImportOutcome::new(ImportedAsset::Data(
        crate::asset::DataAsset {
            uri: context.uri.clone(),
            format: crate::asset::DataAssetFormat::Json,
            text: String::from_utf8_lossy(&context.source_bytes).into_owned(),
            canonical_json: serde_json::json!({ "kind": "weather" }),
        },
    )))
}

fn weather_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "weather.volumetric_clouds",
        vec!["weather.cloud_volume".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "weather-volumetric-clouds",
            QueueLane::Graphics,
        )
        .with_executor_id("weather.volumetric-clouds")
        .with_side_effects()],
    )
}

#[derive(Debug)]
struct NoopVirtualGeometryRuntimeProvider;

impl VirtualGeometryRuntimeProvider for NoopVirtualGeometryRuntimeProvider {
    fn create_state(&self) -> Box<dyn VirtualGeometryRuntimeState> {
        Box::new(NoopVirtualGeometryRuntimeState)
    }
}

#[derive(Debug)]
struct NoopVirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState for NoopVirtualGeometryRuntimeState {
    fn prepare_frame(
        &mut self,
        _input: VirtualGeometryRuntimePrepareInput<'_>,
    ) -> VirtualGeometryRuntimePrepareOutput {
        VirtualGeometryRuntimePrepareOutput::default()
    }

    fn update_after_render(
        &mut self,
        _feedback: VirtualGeometryRuntimeFeedback,
    ) -> VirtualGeometryRuntimeUpdate {
        VirtualGeometryRuntimeUpdate::default()
    }
}

#[derive(Debug)]
struct NoopHybridGiRuntimeProvider;

impl HybridGiRuntimeProvider for NoopHybridGiRuntimeProvider {
    fn create_state(&self) -> Box<dyn HybridGiRuntimeState> {
        Box::new(NoopHybridGiRuntimeState)
    }
}

struct NoopHybridGiRuntimeState;

impl HybridGiRuntimeState for NoopHybridGiRuntimeState {
    fn prepare_frame(
        &mut self,
        _input: HybridGiRuntimePrepareInput<'_>,
    ) -> HybridGiRuntimePrepareOutput {
        HybridGiRuntimePrepareOutput::default()
    }

    fn update_after_render(&mut self, _feedback: HybridGiRuntimeFeedback) -> HybridGiRuntimeUpdate {
        HybridGiRuntimeUpdate::default()
    }
}

#[test]
fn runtime_extension_registry_rejects_duplicate_component_and_ui_component_ids() {
    let mut registry = RuntimeExtensionRegistry::default();
    let component =
        ComponentTypeDescriptor::new("weather.Component.CloudLayer", "weather", "Cloud");
    let ui_component = UiComponentDescriptor::new(
        "weather.Ui.CloudLayerInspector",
        "weather",
        "asset://weather/editor/cloud_layer_inspector.ui.toml",
    );

    registry
        .register_component(component.clone())
        .expect("first component");
    let duplicate_component = registry.register_component(component).unwrap_err();
    assert!(duplicate_component
        .to_string()
        .contains("component type weather.Component.CloudLayer already registered"));

    registry
        .register_ui_component(ui_component.clone())
        .expect("first ui component");
    let duplicate_ui = registry.register_ui_component(ui_component).unwrap_err();
    assert!(duplicate_ui
        .to_string()
        .contains("ui component weather.Ui.CloudLayerInspector already registered"));
}

#[test]
fn runtime_extension_registry_rejects_component_ids_without_plugin_prefix() {
    let mut registry = RuntimeExtensionRegistry::default();
    let invalid_component =
        ComponentTypeDescriptor::new("cloud.Component.CloudLayer", "weather", "Cloud");

    let error = registry.register_component(invalid_component).unwrap_err();
    assert!(error.to_string().contains(
        "component type cloud.Component.CloudLayer must be prefixed by plugin id weather"
    ));
}

#[test]
fn runtime_extension_registry_rejects_ui_component_ids_without_plugin_prefix() {
    let mut registry = RuntimeExtensionRegistry::default();
    let invalid_component = UiComponentDescriptor::new(
        "cloud.Ui.CloudLayerInspector",
        "weather",
        "asset://weather/editor/cloud_layer_inspector.ui.toml",
    );

    let error = registry
        .register_ui_component(invalid_component)
        .unwrap_err();
    assert!(error.to_string().contains(
        "ui component cloud.Ui.CloudLayerInspector must be prefixed by plugin id weather"
    ));
}

#[test]
fn runtime_extension_registry_installs_component_types_into_world_registry() {
    let mut registry = RuntimeExtensionRegistry::default();
    let component =
        ComponentTypeDescriptor::new("weather.Component.CloudLayer", "weather", "Cloud");
    registry
        .register_component(component.clone())
        .expect("component contribution");

    let mut world = World::new();
    registry
        .apply_component_types_to_world(&mut world)
        .expect("world component registry install");

    assert_eq!(
        world
            .component_type_descriptor("weather.Component.CloudLayer")
            .map(|descriptor| descriptor.display_name.as_str()),
        Some("Cloud")
    );
    let entity = world.spawn_node(NodeKind::Cube);
    world
        .set_dynamic_component(
            entity,
            "weather.Component.CloudLayer",
            serde_json::json!({ "coverage": 0.5 }),
        )
        .expect("registered component can attach");

    let duplicate = registry
        .apply_component_types_to_world(&mut world)
        .unwrap_err();
    assert!(duplicate
        .to_string()
        .contains("component type weather.Component.CloudLayer already registered"));
}

#[test]
fn runtime_extension_registry_installs_ui_components_into_runtime_registry() {
    let mut extensions = RuntimeExtensionRegistry::default();
    let component = UiComponentDescriptor::new(
        "weather.Ui.CloudLayerInspector",
        "weather",
        "asset://weather/editor/cloud_layer_inspector.ui.toml",
    );
    extensions
        .register_ui_component(component)
        .expect("ui component contribution");

    let mut ui_registry = UiComponentDescriptorRegistry::editor_showcase();
    extensions
        .apply_ui_components_to_registry(&mut ui_registry)
        .expect("ui component registry install");

    let descriptor = ui_registry
        .descriptor("weather.Ui.CloudLayerInspector")
        .expect("installed plugin ui component");
    assert_eq!(descriptor.display_name, "CloudLayerInspector");
    assert_eq!(descriptor.category, UiComponentCategory::Container);
    assert_eq!(descriptor.role, "plugin-ui-component");
    assert!(descriptor
        .slot_schema
        .contains(&UiSlotSchema::new("content").multiple(true)));
    assert!(descriptor.default_props.contains(&(
        "ui_document".to_string(),
        UiValue::String("asset://weather/editor/cloud_layer_inspector.ui.toml".to_string())
    )));

    let duplicate = extensions
        .apply_ui_components_to_registry(&mut ui_registry)
        .unwrap_err();
    assert!(duplicate
        .to_string()
        .contains("ui component weather.Ui.CloudLayerInspector already registered"));
}
