use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::plugin::{
    PluginPackageManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginCatalog, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
    UiComponentDescriptor,
};
use crate::scene::{components::NodeKind, World};
use crate::ui::component::UiComponentDescriptorRegistry;
use crate::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};
use zircon_runtime_interface::ui::component::{UiComponentCategory, UiSlotSchema, UiValue};

#[test]
fn runtime_extension_registry_collects_component_and_ui_component_contributions() {
    let mut registry = RuntimeExtensionRegistry::default();
    let component =
        ComponentTypeDescriptor::new("weather.Component.CloudLayer", "weather", "Cloud Layer")
            .with_property("coverage", "float", true)
            .with_property("tint", "vec4", true);
    let ui_component = UiComponentDescriptor::new(
        "weather.Ui.CloudLayerInspector",
        "weather",
        "asset://weather/editor/cloud_layer_inspector.zui",
    );

    registry
        .register_component(component.clone())
        .expect("component contribution");
    registry
        .register_ui_component(ui_component.clone())
        .expect("ui component contribution");

    assert_eq!(registry.components(), &[component]);
    assert_eq!(registry.ui_components(), &[ui_component]);
}

#[test]
fn runtime_extension_registry_accepts_dotted_component_plugin_ids() {
    let mut registry = RuntimeExtensionRegistry::default();
    let component = ComponentTypeDescriptor::new(
        "weather.layer.Component.CloudLayer",
        "weather.layer",
        "Cloud Layer",
    );
    let ui_component = UiComponentDescriptor::new(
        "weather.layer.Ui.CloudLayerInspector",
        "weather.layer",
        "asset://weather.layer/editor/cloud_layer_inspector.zui",
    );

    registry
        .register_component(component.clone())
        .expect("dotted component plugin id");
    registry
        .register_ui_component(ui_component.clone())
        .expect("dotted ui component plugin id");

    assert_eq!(registry.components(), &[component]);
    assert_eq!(registry.ui_components(), &[ui_component]);
}

#[test]
fn runtime_plugin_registration_report_validates_shadowed_manifest_component_declarations() {
    let plugin = ShadowedInvalidComponentRuntimePlugin {
        descriptor: RuntimePluginDescriptor::builder(
            "weather",
            "Weather",
            RuntimePluginId::Particles,
            "zircon_plugin_weather_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_capability("runtime.plugin.weather")
        .build(),
    };

    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);

    assert!(
        !registration.is_success(),
        "shadowed invalid manifest component declarations should remain diagnostic: {:?}",
        registration.diagnostics
    );
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("property `coverage` must be unique")));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains(".zui")));
    assert_eq!(registration.extensions.components(), &[valid_component()]);
    assert_eq!(
        registration.extensions.ui_components(),
        &[valid_ui_component()]
    );

    let report =
        RuntimePluginCatalog::from_registration_reports([registration], []).runtime_extensions();
    assert!(report.fatal_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("runtime plugin weather diagnostic")
            && diagnostic.contains("property `coverage` must be unique")
    }));
    assert!(report.fatal_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("runtime plugin weather diagnostic") && diagnostic.contains(".zui")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_diagnoses_duplicate_manifest_components() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_component(valid_component())
            .with_component(valid_component())
            .with_ui_component(valid_ui_component())
            .with_ui_component(valid_ui_component()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("component type `weather.Component.CloudLayer`")
            && diagnostic.contains("unique")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("ui component `weather.Ui.CloudLayerInspector`")
            && diagnostic.contains("unique")
    }));
    assert_eq!(registration.extensions.components().len(), 1);
    assert_eq!(registration.extensions.ui_components().len(), 1);
}

#[test]
fn native_runtime_plugin_registration_report_diagnoses_unowned_manifest_components() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_component(ComponentTypeDescriptor::new(
                "storm.Component.CloudLayer",
                "storm",
                "Cloud Layer",
            ))
            .with_ui_component(UiComponentDescriptor::new(
                "storm.Ui.CloudLayerInspector",
                "storm",
                "asset://storm/editor/cloud_layer_inspector.zui",
            )),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("component type `storm.Component.CloudLayer`")
            && diagnostic.contains("plugin_id `storm`")
            && diagnostic.contains("package id `weather`")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("ui component `storm.Ui.CloudLayerInspector`")
            && diagnostic.contains("plugin_id `storm`")
            && diagnostic.contains("package id `weather`")
    }));
    assert_eq!(registration.extensions.components().len(), 1);
    assert_eq!(registration.extensions.ui_components().len(), 1);
}

#[test]
fn runtime_extension_registry_rejects_duplicate_component_and_ui_component_ids() {
    let mut registry = RuntimeExtensionRegistry::default();
    let component =
        ComponentTypeDescriptor::new("weather.Component.CloudLayer", "weather", "Cloud");
    let ui_component = UiComponentDescriptor::new(
        "weather.Ui.CloudLayerInspector",
        "weather",
        "asset://weather/editor/cloud_layer_inspector.zui",
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
fn runtime_extension_registry_rejects_invalid_component_descriptor_fields() {
    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_component(ComponentTypeDescriptor::new(
            " weather.Component.CloudLayer",
            "weather",
            "Cloud Layer",
        ))
        .unwrap_err();
    assert!(error.to_string().contains("type_id"));

    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_component(ComponentTypeDescriptor::new(
            "weather.Component.CloudLayer",
            "weather",
            " ",
        ))
        .unwrap_err();
    assert!(error.to_string().contains("display_name"));
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
fn runtime_extension_registry_rejects_invalid_component_plugin_ids() {
    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_component(ComponentTypeDescriptor::new(
            "Weather.Component.CloudLayer",
            "Weather",
            "Cloud Layer",
        ))
        .unwrap_err();

    assert!(error.to_string().contains("plugin_id"));
    assert!(error.to_string().contains("lowercase ASCII"));

    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_component(ComponentTypeDescriptor::new(
            "weather..layer.Component.CloudLayer",
            "weather..layer",
            "Cloud Layer",
        ))
        .unwrap_err();

    assert!(error.to_string().contains("plugin_id"));
    assert!(error.to_string().contains("lowercase ASCII"));
}

#[test]
fn runtime_extension_registry_rejects_invalid_component_properties() {
    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_component(
            ComponentTypeDescriptor::new("weather.Component.CloudLayer", "weather", "Cloud Layer")
                .with_property("coverage", "float", true)
                .with_property("coverage", "float", false),
        )
        .unwrap_err();
    assert!(error.to_string().contains("must be unique"));

    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_component(
            ComponentTypeDescriptor::new("weather.Component.CloudLayer", "weather", "Cloud Layer")
                .with_property(" tint", "vec4", true),
        )
        .unwrap_err();
    assert!(error.to_string().contains("property name"));
}

#[test]
fn runtime_extension_registry_rejects_ui_component_ids_without_plugin_prefix() {
    let mut registry = RuntimeExtensionRegistry::default();
    let invalid_component = UiComponentDescriptor::new(
        "cloud.Ui.CloudLayerInspector",
        "weather",
        "asset://weather/editor/cloud_layer_inspector.zui",
    );

    let error = registry
        .register_ui_component(invalid_component)
        .unwrap_err();
    assert!(error.to_string().contains(
        "ui component cloud.Ui.CloudLayerInspector must be prefixed by plugin id weather"
    ));
}

#[test]
fn runtime_extension_registry_rejects_invalid_ui_component_descriptor_fields() {
    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_ui_component(UiComponentDescriptor::new(
            " weather.Ui.CloudLayerInspector",
            "weather",
            "asset://weather/editor/cloud_layer_inspector.zui",
        ))
        .unwrap_err();
    assert!(error.to_string().contains("component_id"));

    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_ui_component(UiComponentDescriptor::new(
            "weather.Ui.CloudLayerInspector",
            "weather",
            " asset://weather/editor/cloud_layer_inspector.zui",
        ))
        .unwrap_err();
    assert!(error.to_string().contains("ui_document"));
}

#[test]
fn runtime_extension_registry_rejects_invalid_ui_component_plugin_ids() {
    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_ui_component(UiComponentDescriptor::new(
            "Weather.Ui.CloudLayerInspector",
            "Weather",
            "asset://weather/editor/cloud_layer_inspector.zui",
        ))
        .unwrap_err();

    assert!(error.to_string().contains("plugin_id"));
    assert!(error.to_string().contains("lowercase ASCII"));

    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_ui_component(UiComponentDescriptor::new(
            "weather..layer.Ui.CloudLayerInspector",
            "weather..layer",
            "asset://weather/editor/cloud_layer_inspector.zui",
        ))
        .unwrap_err();

    assert!(error.to_string().contains("plugin_id"));
    assert!(error.to_string().contains("lowercase ASCII"));
}

#[test]
fn runtime_extension_registry_rejects_non_zui_ui_component_documents() {
    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_ui_component(UiComponentDescriptor::new(
            "weather.Ui.CloudLayerInspector",
            "weather",
            "asset://weather/editor/cloud_layer_inspector.toml",
        ))
        .unwrap_err();
    assert!(error.to_string().contains(".zui"));
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
        "asset://weather/editor/cloud_layer_inspector.zui",
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
        UiValue::String("asset://weather/editor/cloud_layer_inspector.zui".to_string())
    )));

    let duplicate = extensions
        .apply_ui_components_to_registry(&mut ui_registry)
        .unwrap_err();
    assert!(duplicate
        .to_string()
        .contains("ui component weather.Ui.CloudLayerInspector already registered"));
}

#[derive(Debug)]
struct ShadowedInvalidComponentRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl RuntimePlugin for ShadowedInvalidComponentRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        self.descriptor()
            .package_manifest()
            .with_component(
                ComponentTypeDescriptor::new(
                    "weather.Component.CloudLayer",
                    "weather",
                    "Cloud Layer",
                )
                .with_property("coverage", "float", true)
                .with_property("coverage", "float", false),
            )
            .with_ui_component(UiComponentDescriptor::new(
                "weather.Ui.CloudLayerInspector",
                "weather",
                "asset://weather/editor/cloud_layer_inspector.toml",
            ))
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        registry.register_component(valid_component())?;
        registry.register_ui_component(valid_ui_component())
    }
}

fn valid_component() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new("weather.Component.CloudLayer", "weather", "Cloud Layer")
}

fn valid_ui_component() -> UiComponentDescriptor {
    UiComponentDescriptor::new(
        "weather.Ui.CloudLayerInspector",
        "weather",
        "asset://weather/editor/cloud_layer_inspector.zui",
    )
}
