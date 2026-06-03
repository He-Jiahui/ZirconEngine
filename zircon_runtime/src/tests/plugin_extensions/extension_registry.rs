use crate::asset::{
    AssetImportContext, AssetImportOutcome, AssetImporterDescriptor, FunctionAssetImporter,
    ImportedAsset,
};
use crate::core::framework::render::{
    RenderFrameExtract, RenderPipelineHandle, RenderViewportDescriptor, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::core::ModuleDescriptor;
use crate::graphics::{
    HybridGiRuntimeFeedback, HybridGiRuntimePrepareInput, HybridGiRuntimePrepareOutput,
    HybridGiRuntimeProvider, HybridGiRuntimeProviderRegistration, HybridGiRuntimeState,
    HybridGiRuntimeUpdate, RenderFeatureDescriptor, RenderPassExecutionContext,
    RenderPassExecutorId, RenderPassExecutorRegistration, RenderPassStage, RenderPipelineAsset,
    RuntimePrepareCollectorContext, RuntimePrepareCollectorRegistration,
    VirtualGeometryRuntimeFeedback, VirtualGeometryRuntimePrepareInput,
    VirtualGeometryRuntimePrepareOutput, VirtualGeometryRuntimeProvider,
    VirtualGeometryRuntimeProviderRegistration, VirtualGeometryRuntimeState,
    VirtualGeometryRuntimeUpdate,
};
use crate::plugin::{
    RuntimeExtensionRegistry, RuntimePlugin, RuntimePluginCatalog, RuntimePluginDescriptor,
    SceneRuntimeHook, SceneRuntimeHookContext, SceneRuntimeHookDescriptor,
    SceneRuntimeHookRegistration,
};
use crate::scene::{SystemStage, World};
use crate::RenderFeaturePassDescriptor;
use crate::{asset, core::manager::RenderFrameworkHandle, render_graph::QueueLane};
use crate::{RuntimePluginId, RuntimeTargetMode};

#[test]
fn runtime_extension_registry_collects_render_feature_contributions() {
    let mut registry = RuntimeExtensionRegistry::default();
    let render_feature = RenderFeatureDescriptor {
        name: "weather.volumetric_clouds".to_string(),
        required_extract_sections: vec!["weather.cloud_volume".to_string()],
        capability_requirements: Vec::new(),
        history_bindings: Vec::new(),
        stage_passes: Vec::new(),
    };

    registry
        .register_render_feature(render_feature.clone())
        .expect("render feature contribution");

    assert_eq!(registry.render_features(), &[render_feature]);
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
fn runtime_extension_registry_collects_runtime_prepare_collector_contributions() {
    let mut registry = RuntimeExtensionRegistry::default();
    let registration = RuntimePrepareCollectorRegistration::new(
        "weather.runtime-prepare",
        weather_runtime_prepare_collector,
    );

    registry
        .register_runtime_prepare_collector(registration)
        .expect("runtime prepare collector contribution");

    assert_eq!(registry.runtime_prepare_collectors().len(), 1);
    assert_eq!(
        registry.runtime_prepare_collectors()[0].collector_id(),
        "weather.runtime-prepare"
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
fn runtime_extension_registry_rejects_duplicate_render_feature_and_provider_names() {
    let mut registry = RuntimeExtensionRegistry::default();
    let render_feature = RenderFeatureDescriptor {
        name: "weather.volumetric_clouds".to_string(),
        required_extract_sections: Vec::new(),
        capability_requirements: Vec::new(),
        history_bindings: Vec::new(),
        stage_passes: Vec::new(),
    };

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

    let collector = RuntimePrepareCollectorRegistration::new(
        "weather.runtime-prepare",
        weather_runtime_prepare_collector,
    );
    registry
        .register_runtime_prepare_collector(collector.clone())
        .expect("first runtime prepare collector");
    let duplicate_collector = registry
        .register_runtime_prepare_collector(collector)
        .unwrap_err();
    assert!(duplicate_collector
        .to_string()
        .contains("runtime prepare collector weather.runtime-prepare already registered"));

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
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_capability("runtime.plugin.weather"),
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
        report.registry.runtime_prepare_collectors()[0].collector_id(),
        "weather.runtime-prepare"
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
fn runtime_modules_propagate_reported_executor_registrations_into_render_framework() {
    let plugin = WeatherRuntimePlugin {
        descriptor: RuntimePluginDescriptor::new(
            "weather",
            "Weather",
            RuntimePluginId::Particles,
            "zircon_plugin_weather_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_capability("runtime.plugin.weather"),
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
        registry.register_runtime_prepare_collector(RuntimePrepareCollectorRegistration::new(
            "weather.runtime-prepare",
            weather_runtime_prepare_collector,
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

fn weather_render_executor(_context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    Err("weather executor reached graph execution".to_string())
}

fn weather_runtime_prepare_collector(
    _context: &mut RuntimePrepareCollectorContext<'_>,
) -> Result<crate::core::framework::render::RenderPluginRendererOutputs, crate::GraphicsError> {
    Ok(crate::core::framework::render::RenderPluginRendererOutputs::default())
}

fn weather_data_importer(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, crate::asset::AssetImportError> {
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(crate::asset::DataAsset {
            uri: context.uri.clone(),
            format: crate::asset::DataAssetFormat::Json,
            text: String::from_utf8_lossy(&context.source_bytes).into_owned(),
            canonical_json: serde_json::json!({ "kind": "weather" }),
        }),
    ))
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
