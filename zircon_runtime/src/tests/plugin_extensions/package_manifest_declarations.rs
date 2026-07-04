use crate::asset::AssetImporterDescriptor;
use crate::builtin::RuntimeTargetMode;
use crate::core::framework::render::{
    GBufferChannelMask, GeometrySourceBindingKind, GeometrySourceBindingRequirement,
    GeometrySourceDescriptor, GeometrySourceId, GeometrySourceVertexAttribute,
    RenderShaderDefinitionValue, ShadingModelDescriptor, ShadingModelId,
    GEOMETRY_SOURCE_PLUGIN_ID_START, SHADING_MODEL_PLUGIN_ID_START,
};
use crate::core::framework::script::{ScriptHostParameterDescriptor, ScriptHostValueKind};
use crate::core::{InitLevel, ModuleDependencySpec};
use crate::plugin::{
    ComponentTypeDescriptor, ExportPackagingStrategy, ExportTargetPlatform,
    PluginDependencyManifest, PluginFeatureBundleManifest, PluginFeatureDependency,
    PluginInterfaceManifest, PluginInterfaceMethodManifest, PluginModuleKind, PluginModuleManifest,
    PluginPackageKind, PluginPackageManifest, PluginShaderModuleManifest, UiComponentDescriptor,
};

#[test]
fn plugin_package_manifest_declares_runtime_and_editor_contributions() {
    let manifest = PluginPackageManifest::new("weather", "Weather")
        .with_category("environment")
        .with_runtime_crate("zircon_plugin_weather_runtime")
        .with_editor_crate("zircon_plugin_weather_editor")
        .with_component(ComponentTypeDescriptor::new(
            "weather.Component.CloudLayer",
            "weather",
            "Cloud Layer",
        ))
        .with_ui_component(UiComponentDescriptor::new(
            "weather.Ui.CloudLayerInspector",
            "weather",
            "asset://weather/editor/cloud_layer_inspector.zui",
        ));

    assert_eq!(manifest.components.len(), 1);
    assert_eq!(manifest.category, "environment");
    assert_eq!(
        manifest.components[0].type_id,
        "weather.Component.CloudLayer"
    );
    assert_eq!(manifest.ui_components.len(), 1);
    assert_eq!(
        manifest.ui_components[0].ui_document,
        "asset://weather/editor/cloud_layer_inspector.zui"
    );

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded, manifest);
}

#[test]
fn plugin_package_manifest_declares_public_package_metadata() {
    let manifest = PluginPackageManifest::new("weather", "Weather")
        .with_sdk_api_version("0.2.0")
        .with_category("simulation")
        .with_supported_targets([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_supported_platforms([ExportTargetPlatform::Windows, ExportTargetPlatform::Linux])
        .with_capabilities([
            "runtime.plugin.weather",
            "runtime.capability.weather.forecast",
        ])
        .with_asset_root("assets")
        .with_content_root("content")
        .with_runtime_module(
            PluginModuleManifest::runtime("weather.runtime", "zircon_plugin_weather_runtime")
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ])
                .with_capabilities(["runtime.plugin.weather"]),
        )
        .with_native_module(
            PluginModuleManifest::native("weather.native", "zircon_plugin_weather_native")
                .with_target_modes([RuntimeTargetMode::ClientRuntime])
                .with_capabilities(["runtime.native.weather"]),
        )
        .with_vm_module(
            PluginModuleManifest::vm("weather.vm", "weather_vm_module")
                .with_target_modes([RuntimeTargetMode::ClientRuntime])
                .with_capabilities(["runtime.vm.weather"]),
        );

    assert_eq!(manifest.sdk_api_version, "0.2.0");
    assert_eq!(manifest.category, "simulation");
    assert_eq!(
        manifest.supported_targets,
        vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost
        ]
    );
    assert_eq!(
        manifest.supported_platforms,
        vec![ExportTargetPlatform::Windows, ExportTargetPlatform::Linux]
    );
    assert_eq!(
        manifest.capabilities,
        vec![
            "runtime.plugin.weather".to_string(),
            "runtime.capability.weather.forecast".to_string()
        ]
    );
    assert_eq!(manifest.asset_roots, vec!["assets".to_string()]);
    assert_eq!(manifest.package_id(), "com.zircon.weather");
    assert_eq!(
        manifest.asset_roots_or_default(),
        vec!["assets".to_string()]
    );
    assert_eq!(manifest.content_roots, vec!["content".to_string()]);
    assert!(manifest
        .modules
        .iter()
        .any(|module| module.kind == PluginModuleKind::Native));
    assert!(manifest
        .modules
        .iter()
        .any(|module| module.kind == PluginModuleKind::Vm));

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    assert!(encoded.contains("sdk_api_version = \"0.2.0\""));
    assert!(encoded.contains("kind = \"native\""));
    assert!(encoded.contains("kind = \"vm\""));

    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded, manifest);
}

#[test]
fn plugin_module_manifest_declares_system_sets_and_anchors() {
    let manifest = PluginPackageManifest::new("physics", "Physics").with_runtime_module(
        PluginModuleManifest::runtime("physics.runtime", "zircon_plugin_physics_runtime")
            .with_target_modes([
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ])
            .with_capabilities(["runtime.plugin.physics"])
            .with_system_sets(["physics.main", "physics.simulation"])
            .with_system_anchors(["physics.step", "physics.sync_to_scene"]),
    );

    assert_eq!(
        manifest.modules[0].system_sets,
        vec!["physics.main".to_string(), "physics.simulation".to_string()]
    );
    assert_eq!(
        manifest.modules[0].system_anchors,
        vec![
            "physics.step".to_string(),
            "physics.sync_to_scene".to_string()
        ]
    );

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    assert!(encoded.contains("system_sets = ["));
    assert!(encoded.contains("\"physics.main\""));
    assert!(encoded.contains("system_anchors = ["));
    assert!(encoded.contains("\"physics.step\""));

    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded, manifest);
}

#[test]
fn plugin_module_manifest_projects_module_descriptor_fields() {
    let module = PluginModuleManifest::runtime("weather.runtime", "zircon_plugin_weather_runtime")
        .with_description("Weather runtime module")
        .with_init_level(InitLevel::Scene)
        .with_module_dependency(ModuleDependencySpec::named("scene.runtime"))
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_capabilities(["runtime.plugin.weather"]);
    let descriptor = module.module_descriptor();

    assert_eq!(descriptor.name, "weather.runtime");
    assert_eq!(descriptor.description, "Weather runtime module");
    assert_eq!(descriptor.init_level, InitLevel::Scene);
    assert_eq!(
        descriptor.module_dependencies,
        vec![ModuleDependencySpec::named("scene.runtime")]
    );

    let manifest = PluginPackageManifest::new("weather", "Weather").with_runtime_module(module);
    let encoded = toml::to_string(&manifest).expect("manifest toml");
    assert!(encoded.contains("description = \"Weather runtime module\""));
    assert!(encoded.contains("init_level = \"scene\""));
    assert!(encoded.contains("module_dependencies"));

    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(
        decoded.modules[0].module_descriptor().init_level,
        InitLevel::Scene
    );
    assert_eq!(
        decoded.modules[0].module_descriptor().module_dependencies,
        vec![ModuleDependencySpec::named("scene.runtime")]
    );
}

#[test]
fn plugin_module_manifest_defaults_descriptor_description_for_manifest_rows() {
    let module: PluginModuleManifest = toml::from_str(
        r#"
name = "weather.runtime"
kind = "runtime"
crate_name = "zircon_plugin_weather_runtime"
"#,
    )
    .expect("manifest module row");

    assert!(module.description.is_empty());
    assert_eq!(
        module.module_descriptor().description,
        "Runtime plugin module weather.runtime"
    );
}

#[test]
fn plugin_package_manifest_declares_bridge_interfaces() {
    let manifest = PluginPackageManifest::new("weather", "Weather")
        .with_provided_interface(
            PluginInterfaceManifest::new("weather.query.v1").with_method(
                PluginInterfaceMethodManifest::new("sample_temperature", 0)
                    .with_return_value_kind(ScriptHostValueKind::Int)
                    .with_parameter(ScriptHostParameterDescriptor::new(
                        "region",
                        ScriptHostValueKind::String,
                    ))
                    .with_required_capability("runtime.plugin.weather.query")
                    .with_documentation("Samples the weather provider temperature."),
            ),
        )
        .with_provided_interface_id("weather.forecast.v1")
        .with_dependency(
            PluginDependencyManifest::new("physics", true)
                .with_capability("runtime.plugin.physics")
                .with_interfaces(["physics.query.v1", "physics.force.v1"]),
        )
        .with_dependency(
            PluginDependencyManifest::new("sound", false)
                .with_capability("runtime.plugin.sound")
                .with_interface("sound.occlusion.v1"),
        );

    assert_eq!(
        manifest.provides_interfaces,
        vec![
            PluginInterfaceManifest::new("weather.query.v1").with_method(
                PluginInterfaceMethodManifest::new("sample_temperature", 0)
                    .with_return_value_kind(ScriptHostValueKind::Int)
                    .with_parameter(ScriptHostParameterDescriptor::new(
                        "region",
                        ScriptHostValueKind::String,
                    ))
                    .with_required_capability("runtime.plugin.weather.query")
                    .with_documentation("Samples the weather provider temperature."),
            ),
            PluginInterfaceManifest::new("weather.forecast.v1"),
        ]
    );
    assert_eq!(
        manifest
            .bridge_interface("weather.query.v1")
            .unwrap()
            .method("sample_temperature")
            .unwrap()
            .method_slot,
        0
    );
    assert_eq!(manifest.bridge_methods().count(), 1);
    assert_eq!(
        manifest.dependencies[0].interfaces,
        vec![
            "physics.query.v1".to_string(),
            "physics.force.v1".to_string()
        ]
    );
    assert_eq!(
        manifest.dependencies[1].interfaces,
        vec!["sound.occlusion.v1".to_string()]
    );

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    assert!(encoded.contains("[[provides_interfaces]]"));
    assert!(encoded.contains("id = \"weather.query.v1\""));
    assert!(encoded.contains("[[provides_interfaces.methods]]"));
    assert!(encoded.contains("name = \"sample_temperature\""));
    assert!(encoded.contains("method_slot = 0"));
    assert!(encoded.contains("interfaces = ["));
    assert!(encoded.contains("\"physics.query.v1\""));

    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded, manifest);
}

#[test]
fn plugin_package_manifest_declares_asset_importer_descriptors() {
    let importer =
        AssetImporterDescriptor::new("weather.data", "weather", crate::asset::AssetKind::Data, 3)
            .with_source_extensions(["weather"])
            .with_full_suffixes([".weather.toml"])
            .with_required_capabilities(["runtime.asset.importer.data"]);
    let manifest = PluginPackageManifest::new("weather", "Weather")
        .with_runtime_crate("zircon_plugin_weather_runtime")
        .with_asset_importer(importer.clone());

    assert_eq!(manifest.asset_importers, vec![importer]);

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded, manifest);
}

#[test]
fn plugin_package_manifest_declares_custom_shading_model_descriptors() {
    let descriptor = ShadingModelDescriptor::new(
        ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START),
        "custom:toon",
        "zr_shading_toon",
        "zr_gbuffer_encode_toon",
        "zr_shade_deferred_toon",
        GBufferChannelMask::standard_lit(),
    );
    let manifest = PluginPackageManifest::new("toon", "Toon")
        .with_shading_model_descriptor(descriptor.clone())
        .with_shader_shading_model_id("custom:toon", SHADING_MODEL_PLUGIN_ID_START);

    assert_eq!(manifest.shading_models, vec![descriptor]);
    assert_eq!(
        manifest.shader_permutation.shading_model_ids,
        vec![crate::plugin::PluginShaderPermutationIdManifest::new(
            "custom:toon",
            SHADING_MODEL_PLUGIN_ID_START,
        )]
    );

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    assert!(encoded.contains("[[shading_models]]"));
    assert!(encoded.contains("token = \"custom:toon\""));
    assert!(encoded.contains("required_channels = 7"));

    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded, manifest);
}

#[test]
fn plugin_package_manifest_declares_custom_geometry_source_descriptors() {
    let descriptor = virtual_geometry_source_descriptor();
    let manifest = PluginPackageManifest::new("virtual_geometry", "Virtual Geometry")
        .with_geometry_source_descriptor(descriptor.clone())
        .with_shader_geometry_source_id("custom:virtual_geometry", GEOMETRY_SOURCE_PLUGIN_ID_START);

    assert_eq!(manifest.geometry_sources, vec![descriptor]);
    assert_eq!(
        manifest.shader_permutation.geometry_source_ids,
        vec![crate::plugin::PluginShaderPermutationIdManifest::new(
            "custom:virtual_geometry",
            GEOMETRY_SOURCE_PLUGIN_ID_START,
        )]
    );

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    assert!(encoded.contains("[[geometry_sources]]"));
    assert!(encoded.contains("token = \"custom:virtual_geometry\""));
    assert!(encoded.contains("wgsl_include = \"zr_geometry_virtual_geometry.wgsl\""));

    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded, manifest);
}

#[test]
fn plugin_package_manifest_declares_shader_module_registration() {
    let manifest = PluginPackageManifest::new("toon", "Toon")
        .with_shader_module("custom::toon::noise", "assets/shaders/noise.zshader");

    assert_eq!(
        manifest.shader_permutation.shader_modules,
        vec![PluginShaderModuleManifest::new(
            "custom::toon::noise",
            "assets/shaders/noise.zshader",
        )]
    );

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    assert!(encoded.contains("[[shader_permutation.shader_modules]]"));
    assert!(encoded.contains("import_path = \"custom::toon::noise\""));
    assert!(encoded.contains("source = \"assets/shaders/noise.zshader\""));

    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded, manifest);
}

#[test]
fn plugin_package_manifest_overrides_default_packaging() {
    let manifest = PluginPackageManifest::new("weather", "Weather")
        .with_default_packaging([ExportPackagingStrategy::NativeDynamic]);

    assert_eq!(
        manifest.default_packaging,
        vec![ExportPackagingStrategy::NativeDynamic]
    );

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded.default_packaging, manifest.default_packaging);
}

#[test]
fn plugin_package_manifest_declares_optional_feature_bundles() {
    let feature = sound_timeline_feature_manifest();
    let manifest = PluginPackageManifest::new("sound", "Sound")
        .with_runtime_crate("zircon_plugin_sound_runtime")
        .with_optional_feature(feature.clone());

    assert_eq!(manifest.optional_features, vec![feature]);
    assert!(!manifest.optional_features[0].enabled_by_default);
    assert_eq!(manifest.optional_features[0].owner_plugin_id, "sound");
    assert!(manifest.optional_features[0]
        .dependencies
        .iter()
        .any(|dependency| dependency.plugin_id == "sound" && dependency.primary));
    assert!(manifest.optional_features[0]
        .modules
        .iter()
        .any(|module| module.crate_name == "zircon_plugin_sound_timeline_animation_runtime"));

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded, manifest);
}

#[test]
fn plugin_package_manifest_declares_feature_extension_packages() {
    let feature = sound_timeline_feature_manifest();
    let manifest = PluginPackageManifest::new(
        "sound_timeline_animation_track",
        "Sound Timeline Animation Track Provider",
    )
    .as_feature_extension()
    .with_feature_extension(feature.clone());

    assert_eq!(manifest.package_kind, PluginPackageKind::FeatureExtension);
    assert!(manifest.optional_features.is_empty());
    assert_eq!(manifest.feature_extensions, vec![feature]);

    let encoded = toml::to_string(&manifest).expect("feature extension manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("feature extension manifest roundtrip");
    assert_eq!(decoded, manifest);
}

fn virtual_geometry_source_descriptor() -> GeometrySourceDescriptor {
    GeometrySourceDescriptor {
        id: GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START),
        token: "custom:virtual_geometry".to_string(),
        wgsl_include: "zr_geometry_virtual_geometry.wgsl".to_string(),
        vertex_attributes: vec![
            GeometrySourceVertexAttribute::Position,
            GeometrySourceVertexAttribute::Normal,
            GeometrySourceVertexAttribute::Tangent,
            GeometrySourceVertexAttribute::Uv0,
        ],
        required_bindings: vec![
            GeometrySourceBindingRequirement::new(
                GeometrySourceBindingKind::VirtualGeometryPages,
                "virtual_geometry.pages",
            ),
            GeometrySourceBindingRequirement::new(
                GeometrySourceBindingKind::VirtualGeometryClusters,
                "virtual_geometry.clusters",
            ),
        ],
        shader_defines: vec![RenderShaderDefinitionValue::bool(
            "ZR_GEOMETRY_SOURCE_VIRTUAL_GEOMETRY",
            true,
        )],
    }
}

fn sound_timeline_feature_manifest() -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new(
        "sound.timeline_animation_track",
        "Timeline Animation Track",
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
    .with_runtime_module(
        PluginModuleManifest::runtime(
            "sound.timeline_animation_track.runtime",
            "zircon_plugin_sound_timeline_animation_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities(["runtime.feature.sound.timeline_animation_track"]),
    )
    .with_editor_module(
        PluginModuleManifest::editor(
            "sound.timeline_animation_track.editor",
            "zircon_plugin_sound_timeline_animation_editor",
        )
        .with_capabilities(["editor.feature.sound.timeline_animation_track"]),
    )
}
