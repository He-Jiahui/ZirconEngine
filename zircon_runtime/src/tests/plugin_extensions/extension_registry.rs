use crate::core::{ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode};
use crate::engine_module::{factory, qualified_name};
use crate::graphics::{
    RenderFeatureDescriptor, RenderPassExecutionContext, RenderPassExecutorId,
    RenderPassExecutorRegistration, VirtualGeometryRuntimeFeedback,
    VirtualGeometryRuntimePrepareInput, VirtualGeometryRuntimePrepareOutput,
    VirtualGeometryRuntimeProvider, VirtualGeometryRuntimeProviderRegistration,
    VirtualGeometryRuntimeState, VirtualGeometryRuntimeUpdate,
};
use crate::plugin::{
    ComponentTypeDescriptor, RuntimeExtensionRegistry, RuntimePlugin, RuntimePluginCatalog,
    RuntimePluginDescriptor, UiComponentDescriptor,
};
use crate::scene::{components::NodeKind, World};
use crate::ui::component::UiComponentDescriptorRegistry;
use crate::{RuntimePluginId, RuntimeTargetMode};
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
    ) -> Result<(), crate::RuntimeExtensionRegistryError> {
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
    ) -> Result<(), crate::RuntimeExtensionRegistryError> {
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
        Ok(())
    }
}

fn weather_render_executor(_context: &RenderPassExecutionContext) -> Result<(), String> {
    Ok(())
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
