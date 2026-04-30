use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(feature = "target-editor-host")]
use zircon_editor::ui::host::EditorManager;
#[cfg(feature = "target-editor-host")]
use zircon_editor::EDITOR_MANAGER_NAME;
use zircon_runtime::core::framework::render::{RenderQualityProfile, RenderViewportDescriptor};
use zircon_runtime::core::manager::ManagerResolver;
#[cfg(feature = "target-editor-host")]
use zircon_runtime::core::manager::{
    resolve_config_manager, resolve_event_manager, resolve_input_manager, resolve_rendering_manager,
};
use zircon_runtime::core::math::UVec2;
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::graphics::{
    BuiltinRenderFeature, RenderFeatureCapabilityRequirement, RenderFeatureDescriptor,
    RenderFeaturePassDescriptor, RenderPassStage, RenderPipelineAsset, RendererFeatureAsset,
};
use zircon_runtime::render_graph::QueueLane;
use zircon_runtime::scene::World;
#[cfg(feature = "target-editor-host")]
use zircon_runtime::{
    asset::pipeline::manager::resolve_asset_manager,
    input::{InputButton, InputEvent},
    scene::create_default_level,
};
use zircon_runtime::{
    RuntimeExtensionRegistry, RuntimePlugin, RuntimePluginDescriptor,
    RuntimePluginRegistrationReport,
};
use zircon_runtime::{RuntimePluginId, RuntimeTargetMode};

use super::super::{BuiltinEngineEntry, EngineEntry, EntryConfig, EntryProfile, EntryRunner};

const EDITOR_MODULE_NAME: &str = "EditorModule";

#[cfg(feature = "target-editor-host")]
#[test]
fn editor_bootstrap_registers_editor_and_primary_managers() {
    let core = EntryRunner::bootstrap(EntryConfig::new(EntryProfile::Editor)).unwrap();
    let asset_manager = resolve_asset_manager(&core).unwrap();
    let rendering_manager = resolve_rendering_manager(&core).unwrap();
    let input_manager = resolve_input_manager(&core).unwrap();
    let config_manager = resolve_config_manager(&core).unwrap();
    let event_manager = resolve_event_manager(&core).unwrap();
    let level = create_default_level(&core).unwrap();
    let _editor_manager = core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    assert!(asset_manager.pipeline_info().default_worker_count > 0);
    assert!(level.snapshot().nodes().len() >= 3);
    assert_eq!(rendering_manager.backend_info().backend_name, "wgpu");
    input_manager.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    assert_eq!(
        input_manager.snapshot().pressed_buttons,
        vec![InputButton::MouseLeft]
    );
    config_manager
        .set_value("editor.mode", serde_json::json!("docked"))
        .unwrap();
    assert_eq!(
        config_manager.get_value("editor.mode"),
        Some(serde_json::json!("docked"))
    );
    let receiver = event_manager.subscribe("editor.ready");
    event_manager.publish("editor.ready", serde_json::json!({ "booted": true }));
    assert_eq!(receiver.recv().unwrap().payload["booted"], true);
}

#[test]
fn runtime_bootstrap_excludes_editor_module() {
    let entry = BuiltinEngineEntry::for_profile(EntryProfile::Runtime).unwrap();
    assert!(entry
        .module_descriptors()
        .iter()
        .all(|descriptor| descriptor.name != EDITOR_MODULE_NAME));

    let core = EntryRunner::bootstrap(EntryConfig::new(EntryProfile::Runtime)).unwrap();
    #[cfg(feature = "target-editor-host")]
    assert!(core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .is_err());
    assert!(ManagerResolver::new(core).rendering().is_ok());
}

#[test]
fn bootstrap_fails_fast_when_required_runtime_plugin_is_unavailable() {
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_required_runtime_plugins([RuntimePluginId::VirtualGeometry]);

    let error = EntryRunner::bootstrap(config).unwrap_err();

    assert!(error
        .to_string()
        .contains("required runtime plugin VirtualGeometry is unavailable"));
}

#[test]
fn bootstrap_accepts_required_external_runtime_plugin_when_linked_report_contributes_module() {
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_required_runtime_plugins([RuntimePluginId::VirtualGeometry]);
    let report = RuntimePluginRegistrationReport::from_plugin(&LinkedVirtualGeometryPlugin {
        descriptor: RuntimePluginDescriptor::new(
            "virtual_geometry",
            "Virtual Geometry",
            RuntimePluginId::VirtualGeometry,
            "zircon_plugin_virtual_geometry_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime]),
    });

    let entry = BuiltinEngineEntry::for_config_with_runtime_plugin_registrations(&config, [report])
        .expect("linked required plugin should satisfy runtime startup selection");
    let descriptors = entry.module_descriptors();

    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == "VirtualGeometryPlugin"));
}

#[test]
fn runtime_bootstrap_ignores_linked_plugin_registration_for_other_target_modes() {
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_required_runtime_plugins([RuntimePluginId::VirtualGeometry]);
    let report = RuntimePluginRegistrationReport::from_plugin(&LinkedVirtualGeometryPlugin {
        descriptor: RuntimePluginDescriptor::new(
            "virtual_geometry",
            "Virtual Geometry",
            RuntimePluginId::VirtualGeometry,
            "zircon_plugin_virtual_geometry_runtime",
        )
        .with_target_modes([RuntimeTargetMode::EditorHost]),
    });

    let error = EntryRunner::bootstrap_with_runtime_plugin_registrations(config, [report])
        .expect_err("EditorHost-only plugin registration must not satisfy ClientRuntime startup");

    assert!(error
        .to_string()
        .contains("required runtime plugin VirtualGeometry is unavailable"));
}

#[test]
fn runtime_bootstrap_without_linked_virtual_geometry_keeps_base_pipeline_lightweight() {
    let core = EntryRunner::bootstrap(EntryConfig::new(EntryProfile::Runtime)).unwrap();
    let render_framework = ManagerResolver::new(core)
        .render_framework()
        .expect("runtime bootstrap should expose render framework");
    let viewport = render_framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(32, 32)))
        .expect("test viewport should be created");

    render_framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("base-vg-request")
                .with_virtual_geometry(true)
                .with_hybrid_global_illumination(true)
                .with_screen_space_ambient_occlusion(false)
                .with_clustered_lighting(false)
                .with_history_resolve(false),
        )
        .expect("base renderer should accept the profile without activating an absent plugin");
    render_framework
        .submit_frame_extract(viewport, World::new().to_render_frame_extract())
        .expect("base virtual geometry request should degrade to the base pipeline");
    let stats = render_framework.query_stats().unwrap();

    assert!(!stats
        .last_effective_features
        .iter()
        .any(|feature| feature == "virtual_geometry"));
    assert!(!stats
        .last_effective_features
        .iter()
        .any(|feature| feature == "global_illumination" || feature == "hybrid_gi"));
    assert!(!stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| pass.starts_with("virtual-geometry-")));
    assert!(!stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| pass.starts_with("hybrid-gi-")));
}

#[test]
fn quality_profile_capability_gates_do_not_reopen_legacy_builtin_render_features() {
    let core = EntryRunner::bootstrap(EntryConfig::new(EntryProfile::Runtime)).unwrap();
    let render_framework = ManagerResolver::new(core)
        .render_framework()
        .expect("runtime bootstrap should expose render framework");
    let viewport = render_framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(32, 32)))
        .expect("test viewport should be created");
    let legacy_pipeline = render_framework
        .register_pipeline_asset(legacy_advanced_builtin_pipeline())
        .expect("legacy built-in fixture pipeline should register");

    render_framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("legacy-builtins-with-capability-flags")
                .with_pipeline_asset(legacy_pipeline)
                .with_virtual_geometry(true)
                .with_hybrid_global_illumination(true)
                .with_screen_space_ambient_occlusion(false)
                .with_clustered_lighting(false)
                .with_history_resolve(false),
        )
        .expect("base renderer should accept advanced capability profile");
    render_framework
        .submit_frame_extract(viewport, World::new().to_render_frame_extract())
        .expect("capability-gated profile should not reopen legacy built-ins");
    let stats = render_framework.query_stats().unwrap();

    assert!(!stats
        .last_effective_features
        .iter()
        .any(|feature| feature == "virtual_geometry"));
    assert!(!stats
        .last_effective_features
        .iter()
        .any(|feature| feature == "global_illumination"));
    assert!(!stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| pass.starts_with("virtual-geometry-")));
    assert!(!stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| pass.starts_with("hybrid-gi-")));
}

#[test]
fn linked_runtime_render_feature_descriptors_rebuild_default_pipelines() {
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_required_runtime_plugins([RuntimePluginId::VirtualGeometry]);
    let report = RuntimePluginRegistrationReport::from_plugin(&LinkedVirtualGeometryPlugin {
        descriptor: RuntimePluginDescriptor::new(
            "virtual_geometry",
            "Virtual Geometry",
            RuntimePluginId::VirtualGeometry,
            "zircon_plugin_virtual_geometry_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime]),
    });
    let core = EntryRunner::bootstrap_with_runtime_plugin_registrations(config, [report])
        .expect("linked render feature plugin should bootstrap");
    let render_framework = ManagerResolver::new(core)
        .render_framework()
        .expect("runtime bootstrap should expose render framework");
    let viewport = render_framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(32, 32)))
        .expect("test viewport should be created");

    render_framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("linked-vg")
                .with_virtual_geometry(true)
                .with_screen_space_ambient_occlusion(false)
                .with_clustered_lighting(false)
                .with_history_resolve(false),
        )
        .expect("linked virtual geometry profile should be supported by headless renderer");
    render_framework
        .submit_frame_extract(viewport, World::new().to_render_frame_extract())
        .expect("linked virtual geometry frame should submit");
    let stats = render_framework.query_stats().unwrap();

    assert!(stats
        .last_effective_features
        .iter()
        .any(|feature| feature == "virtual_geometry"));
    assert!(stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| pass == "linked-virtual-geometry-pass"));
    assert!(!stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| pass == "virtual-geometry-prepare"));
}

#[test]
fn bootstrap_accepts_required_native_dynamic_plugin_from_export_load_manifest() {
    let export_root = unique_export_root("zircon_app_native_dynamic_bootstrap");
    fs::create_dir_all(export_root.join("plugins/virtual_geometry")).unwrap();
    fs::write(
        export_root.join("plugins/native_plugins.toml"),
        r#"
[[plugins]]
id = "virtual_geometry"
path = "plugins/virtual_geometry"
manifest = "plugins/virtual_geometry/plugin.toml"
"#,
    )
    .unwrap();
    fs::write(
        export_root.join("plugins/virtual_geometry/plugin.toml"),
        r#"
id = "virtual_geometry"
version = "0.1.0"
display_name = "Virtual Geometry"

[[modules]]
name = "virtual_geometry.runtime"
kind = "runtime"
crate_name = "zircon_plugin_virtual_geometry_runtime"
target_modes = ["client_runtime"]
"#,
    )
    .unwrap();
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_required_runtime_plugins([RuntimePluginId::VirtualGeometry]);

    let entry = BuiltinEngineEntry::for_config_with_runtime_plugin_registrations(
        &config,
        zircon_runtime::NativePluginLoader
            .load_runtime_from_load_manifest(&export_root)
            .runtime_plugin_registration_reports(),
    )
    .expect("native dynamic load manifest should satisfy required plugin availability");
    assert!(entry
        .module_descriptors()
        .iter()
        .any(|descriptor| descriptor.name == "virtual_geometry.runtime"));

    let core = EntryRunner::bootstrap_with_native_plugins_from_export_root(config, &export_root)
        .expect("native dynamic load manifest should satisfy required plugin availability");

    assert!(ManagerResolver::new(core).rendering().is_ok());

    let _ = fs::remove_dir_all(export_root);
}

#[derive(Debug)]
struct LinkedVirtualGeometryPlugin {
    descriptor: RuntimePluginDescriptor,
}

impl RuntimePlugin for LinkedVirtualGeometryPlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::RuntimeExtensionRegistryError> {
        registry.register_module(ModuleDescriptor::new(
            "VirtualGeometryPlugin",
            "Linked virtual geometry plugin module",
        ))?;
        registry.register_render_feature(
            RenderFeatureDescriptor::new(
                "virtual_geometry",
                Vec::new(),
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::DepthPrepass,
                    "linked-virtual-geometry-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("virtual-geometry.prepare")
                .with_side_effects()],
            )
            .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry),
        )
    }
}

fn unique_export_root(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{stamp}"))
}

fn legacy_advanced_builtin_pipeline() -> RenderPipelineAsset {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(
            BuiltinRenderFeature::VirtualGeometry,
        ));
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(
            BuiltinRenderFeature::GlobalIllumination,
        ));
    pipeline
}
