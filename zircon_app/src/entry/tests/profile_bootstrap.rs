use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
#[cfg(feature = "target-editor-host")]
use zircon_editor::ui::host::EditorManager;
#[cfg(feature = "target-editor-host")]
use zircon_editor::EDITOR_MANAGER_NAME;
use zircon_runtime::core::framework::bridge::{BridgeInterfaceStatus, PluginInterface};
#[cfg(feature = "target-editor-host")]
use zircon_runtime::core::framework::events::EngineEventDeliveryPolicy;
use zircon_runtime::core::framework::render::{
    RenderProductFeature, RenderProductProfile, RenderProfileBundle, RenderQualityProfile,
    RenderViewportDescriptor, RENDER_PROFILE_CONFIG_KEY,
};
use zircon_runtime::core::framework::window::{
    WindowDescriptor, WindowResolution, PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY,
};
use zircon_runtime::core::manager::ManagerResolver;
use zircon_runtime::core::math::UVec2;
use zircon_runtime::core::ModuleDescriptor;
#[cfg(feature = "graphics")]
use zircon_runtime::graphics::{
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderPassExecutionContext, RenderPassExecutorRegistration, RenderPassStage,
    VirtualGeometryRuntimeFeedback, VirtualGeometryRuntimePrepareInput,
    VirtualGeometryRuntimePrepareOutput, VirtualGeometryRuntimeProvider,
    VirtualGeometryRuntimeProviderRegistration, VirtualGeometryRuntimeState,
    VirtualGeometryRuntimeUpdate,
};
use zircon_runtime::platform::{
    PlatformConfig, PlatformFeatureSelection, PlatformTarget, PLATFORM_CONFIG_KEY,
};
#[cfg(feature = "graphics")]
use zircon_runtime::render_graph::QueueLane;
use zircon_runtime::scene::World;
#[cfg(feature = "target-editor-host")]
use zircon_runtime::{
    asset::asset_manager_handle,
    input::{InputButton, InputEvent},
    scene::create_default_level,
};
use zircon_runtime::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};
use zircon_runtime::{
    core::framework::project::RuntimeProfileId, plugin::PluginModuleKind,
    plugin::RuntimePluginBridgeLifecycleEvent,
};
use zircon_runtime::{
    plugin::RuntimeExtensionRegistry, plugin::RuntimePlugin,
    plugin::RuntimePluginAvailabilityCategory, plugin::RuntimePluginDescriptor,
    plugin::RuntimePluginRegistrationReport,
};

use super::super::{
    BuiltinEngineEntry, EngineEntry, EntryConfig, EntryProfile, ProductCompositionRequest,
};

mod first_party_runtime_plugins;

const EDITOR_MODULE_NAME: &str = "EditorModule";

struct IsolatedConfigFile {
    path: PathBuf,
}

impl IsolatedConfigFile {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            path: std::env::temp_dir().join(format!(
                "zircon-app-profile-config-{}-{stamp}-{id}.json",
                std::process::id()
            )),
        }
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl Drop for IsolatedConfigFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(feature = "target-editor-host")]
#[test]
fn editor_bootstrap_registers_editor_and_primary_managers() {
    let config_file = IsolatedConfigFile::new();
    let composition = ProductCompositionRequest::new(EntryConfig::new(EntryProfile::Editor))
        .with_config_file_path(config_file.path())
        .compose()
        .unwrap();
    {
        let core = composition.core().clone();
        let resolver = ManagerResolver::new(core.clone());
        let asset_manager = resolver
            .resolve(asset_manager_handle(&core).unwrap())
            .unwrap();
        let rendering_manager = resolver
            .resolve(resolver.rendering_handle().unwrap())
            .unwrap();
        let input_manager = resolver.resolve(resolver.input_handle().unwrap()).unwrap();
        let config_manager = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
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
        let receiver = core
            .subscribe_events(
                "editor.ready".to_owned(),
                EngineEventDeliveryPolicy::Reliable {
                    limits: zircon_runtime::core::framework::events::EventRetentionLimits::default(
                    ),
                },
            )
            .expect("bounded subscription");
        core.try_publish_event(
            "editor.ready".to_owned(),
            serde_json::json!({ "booted": true }),
        )
        .expect("event admission");
        assert_eq!(
            receiver.recv().unwrap().decode_payload().unwrap()["booted"],
            true
        );
    }
    super::product_composition::close_owner::close_composition(composition);
}

#[test]
fn runtime_bootstrap_excludes_editor_module() {
    let config_file = IsolatedConfigFile::new();
    let entry = BuiltinEngineEntry::for_profile(EntryProfile::Runtime).unwrap();
    assert!(entry
        .module_descriptors()
        .iter()
        .all(|descriptor| descriptor.name != EDITOR_MODULE_NAME));

    let composition = ProductCompositionRequest::new(EntryConfig::new(EntryProfile::Runtime))
        .with_config_file_path(config_file.path())
        .compose()
        .unwrap();
    {
        let core = composition.core().clone();
        #[cfg(feature = "target-editor-host")]
        assert!(core
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .is_err());
        let resolver = ManagerResolver::new(core.clone());
        assert!(resolver
            .rendering_handle()
            .and_then(|handle| resolver.resolve(handle))
            .is_ok());
    }
    super::product_composition::close_owner::close_composition(composition);
}

#[test]
fn product_compositions_isolate_concurrent_foundation_config_files() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-app-foundation-config-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir(&root).unwrap();
    let start = Arc::new(std::sync::Barrier::new(2));
    let mut workers = Vec::new();
    for owner in ["left", "right"] {
        let path = root.join(format!("{owner}.json"));
        fs::write(
            &path,
            serde_json::to_vec(&serde_json::json!({"fixture.owner": owner})).unwrap(),
        )
        .unwrap();
        let start = Arc::clone(&start);
        workers.push(std::thread::spawn(move || {
            start.wait();
            let composition = ProductCompositionRequest::new(EntryConfig::for_runtime_profile(
                RuntimeProfileId::Minimal,
            ))
            .with_config_file_path(path.clone())
            .compose()
            .expect("each product host should use its own Foundation config file");
            {
                let resolver = ManagerResolver::new(composition.core().clone());
                let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
                assert_eq!(
                    config.get_value("fixture.owner"),
                    Some(serde_json::json!(owner))
                );
                config
                    .set_value("fixture.written", serde_json::json!(owner))
                    .unwrap();
                config.flush(Duration::from_secs(5)).unwrap();
            }
            super::product_composition::close_owner::close_composition(composition);
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }
    for owner in ["left", "right"] {
        let path = root.join(format!("{owner}.json"));
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved["fixture.owner"], serde_json::json!(owner));
        assert_eq!(saved["fixture.written"], serde_json::json!(owner));
        fs::remove_file(path).unwrap();
    }
    fs::remove_dir(root).unwrap();
}

#[test]
fn product_composition_rejects_non_absolute_foundation_config_file_paths() {
    for path in [PathBuf::new(), PathBuf::from("relative-config.json")] {
        let error = ProductCompositionRequest::new(EntryConfig::for_runtime_profile(
            RuntimeProfileId::Minimal,
        ))
        .with_config_file_path(path)
        .module_selection_report()
        .expect_err("invalid host-owned config path must fail during preparation");
        assert!(error.to_string().contains("nonempty and absolute"));
    }
}

#[test]
fn bootstrap_fails_fast_when_required_runtime_plugin_is_unavailable() {
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_required_runtime_plugins([RuntimePluginId::VirtualGeometry]);

    let error = ProductCompositionRequest::new(config)
        .with_runtime_plugin_registrations(std::iter::empty::<RuntimePluginRegistrationReport>())
        .compose()
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("required runtime plugin VirtualGeometry is unavailable"));
}

#[test]
fn entry_config_projects_runtime_profile_to_entry_target_and_manifest() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Server)
        .resolve()
        .expect("server runtime profile should resolve");
    let manifest = config
        .project_plugin_manifest()
        .expect("runtime profile should project a project plugin manifest");

    assert_eq!(config.profile(), EntryProfile::Headless);
    assert_eq!(config.runtime_profile(), Some(RuntimeProfileId::Server));
    assert_eq!(config.target_mode(), RuntimeTargetMode::ServerRuntime);
    assert!(manifest
        .selections
        .iter()
        .any(|selection| selection.id == RuntimePluginId::Net.key()));
    assert!(manifest
        .selections
        .iter()
        .all(|selection| selection.id != RuntimePluginId::Ui.key()));
}

#[cfg(feature = "graphics")]
#[test]
fn bootstrap_accepts_required_external_runtime_plugin_when_linked_report_contributes_module() {
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_required_runtime_plugins([RuntimePluginId::VirtualGeometry]);
    let report = RuntimePluginRegistrationReport::from_plugin(&LinkedVirtualGeometryPlugin {
        descriptor: RuntimePluginDescriptor::builder(
            "virtual_geometry",
            "Virtual Geometry",
            RuntimePluginId::VirtualGeometry,
            "zircon_plugin_virtual_geometry_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_capability("runtime.plugin.virtual_geometry")
        .with_capability("runtime.render.advanced.virtual_geometry")
        .build(),
    });

    let entry = BuiltinEngineEntry::for_config_with_runtime_plugin_registrations(&config, [report])
        .expect("linked required plugin should satisfy runtime startup selection");
    let descriptors = entry.module_descriptors();

    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == "VirtualGeometryPlugin"));
}

#[test]
fn runtime_plugin_bootstrap_installs_neutral_module_lifecycle_observer() {
    let config_file = IsolatedConfigFile::new();
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_required_runtime_plugins([RuntimePluginId::Physics]);
    let report = RuntimePluginRegistrationReport::from_plugin(&LinkedPhysicsBridgePlugin {
        descriptor: RuntimePluginDescriptor::builder(
            "physics",
            "Physics",
            RuntimePluginId::Physics,
            "zircon_plugin_physics_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_capability("runtime.plugin.physics")
        .build(),
    });

    let entry = BuiltinEngineEntry::for_config_with_runtime_plugin_registrations(&config, [report])
        .expect("linked bridge provider should resolve")
        .with_config_file_path(config_file.path())
        .expect("the host config path should bind to Foundation");
    let state = entry
        .runtime_plugin_bridge_lifecycle_state()
        .cloned()
        .expect("entry should retain its plugin-owned bridge lifecycle state");
    let plan = entry
        .compiled_project_plugin_plan()
        .expect("entry should retain the compiled plugin plan used for bootstrap");
    assert!(std::ptr::eq(
        plan.runtime_extensions(),
        state.extension_report()
    ));
    assert_eq!(
        state.catalog().project_plan_metrics().project_plan_builds,
        1
    );
    let runtime = zircon_runtime::core::CoreRuntime::new();
    entry
        .bootstrap(&runtime)
        .expect("linked bridge provider should bootstrap");
    let _core = runtime.handle();
    {
        assert!(state
            .bridge_table()
            .resolve_slot(<dyn EntryBootstrapPhysicsBridge as PluginInterface>::INTERFACE_ID)
            .is_some());
        let outcome = state.apply_provider_lifecycle_event(
            RuntimePluginBridgeLifecycleEvent::disable_provider("physics"),
        );

        assert!(outcome.is_applied());
        assert_eq!(
            state.bridge_table().interface_status(
                <dyn EntryBootstrapPhysicsBridge as PluginInterface>::INTERFACE_ID
            ),
            BridgeInterfaceStatus::Disabled
        );
    }
    drop(_core);
    super::product_composition::close_owner::close_runtime(&runtime);
}

#[cfg(feature = "graphics")]
#[test]
fn runtime_bootstrap_ignores_linked_plugin_registration_for_other_target_modes() {
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_required_runtime_plugins([RuntimePluginId::VirtualGeometry]);
    let report = RuntimePluginRegistrationReport::from_plugin(&LinkedVirtualGeometryPlugin {
        descriptor: RuntimePluginDescriptor::builder(
            "virtual_geometry",
            "Virtual Geometry",
            RuntimePluginId::VirtualGeometry,
            "zircon_plugin_virtual_geometry_runtime",
        )
        .with_target_modes([RuntimeTargetMode::EditorHost])
        .with_capability("runtime.plugin.virtual_geometry")
        .with_capability("runtime.render.advanced.virtual_geometry")
        .build(),
    });

    let error = ProductCompositionRequest::new(config)
        .with_runtime_plugin_registrations([report])
        .compose()
        .expect_err("EditorHost-only plugin registration must not satisfy ClientRuntime startup");

    assert!(error
        .to_string()
        .contains("required runtime plugin VirtualGeometry is unavailable"));
}

#[cfg(feature = "graphics")]
#[test]
fn runtime_bootstrap_without_linked_virtual_geometry_keeps_base_pipeline_lightweight() {
    let config_file = IsolatedConfigFile::new();
    let composition = ProductCompositionRequest::new(EntryConfig::new(EntryProfile::Runtime))
        .with_config_file_path(config_file.path())
        .compose()
        .unwrap();
    {
        let core = composition.core().clone();
        let resolver = ManagerResolver::new(core.clone());
        let render_framework = resolver
            .render_framework_handle()
            .and_then(|handle| resolver.resolve(handle))
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
                    .with_temporal_history(false),
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
    super::product_composition::close_owner::close_composition(composition);
}

#[test]
fn runtime_bootstrap_stores_default_render_profile_bundle() {
    let config_file = IsolatedConfigFile::new();
    let composition = ProductCompositionRequest::new(EntryConfig::new(EntryProfile::Runtime))
        .with_config_file_path(config_file.path())
        .compose()
        .unwrap();
    {
        let core = composition.core();
        let bundle = core
            .load_config::<RenderProfileBundle>(RENDER_PROFILE_CONFIG_KEY)
            .expect("runtime bootstrap should store the selected render profile bundle");

        assert_eq!(bundle.profile(), RenderProductProfile::DefaultRender);
        assert!(bundle.enables(RenderProductProfile::Render2d));
        assert!(bundle.enables(RenderProductProfile::Render3d));
        assert!(bundle.enables(RenderProductProfile::Ui));
        assert!(!bundle.enables(RenderProductProfile::SolariExperimental));
        assert!(!bundle.has_feature(RenderProductFeature::VirtualGeometry));
        assert!(!bundle.has_feature(RenderProductFeature::HybridGlobalIllumination));
    }
    super::product_composition::close_owner::close_composition(composition);
}

#[test]
fn entry_config_can_select_headless_render_profile_bundle() {
    let config_file = IsolatedConfigFile::new();
    let config = EntryConfig::new(EntryProfile::Headless)
        .with_render_profile(RenderProfileBundle::headless());
    let composition = ProductCompositionRequest::new(config)
        .with_config_file_path(config_file.path())
        .compose()
        .unwrap();
    {
        let core = composition.core();
        let bundle = core
            .load_config::<RenderProfileBundle>(RENDER_PROFILE_CONFIG_KEY)
            .expect("headless bootstrap should store the selected render profile bundle");

        assert_eq!(bundle.profile(), RenderProductProfile::Headless);
        assert!(!bundle.enables(RenderProductProfile::Render2d));
        assert!(!bundle.enables(RenderProductProfile::Render3d));
        assert!(!bundle.enables(RenderProductProfile::Ui));
    }
    super::product_composition::close_owner::close_composition(composition);
}

#[test]
fn runtime_bootstrap_stores_primary_window_descriptor() {
    let config_file = IsolatedConfigFile::new();
    let descriptor = WindowDescriptor::default()
        .with_title("Runtime Config Window")
        .with_resolution(WindowResolution::new(1600, 900));
    let composition = ProductCompositionRequest::new(
        EntryConfig::new(EntryProfile::Runtime).with_window_descriptor(descriptor.clone()),
    )
    .with_config_file_path(config_file.path())
    .compose()
    .unwrap();
    {
        let core = composition.core();
        let stored = core
            .load_config::<WindowDescriptor>(PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY)
            .expect("runtime bootstrap should store the selected primary window descriptor");

        assert_eq!(stored, descriptor);
        assert_eq!(stored.title, "Runtime Config Window");
        assert_eq!(stored.resolution.physical_size(), UVec2::new(1600, 900));
    }
    super::product_composition::close_owner::close_composition(composition);
}

#[test]
fn headless_bootstrap_stores_absent_primary_window_descriptor() {
    let config_file = IsolatedConfigFile::new();
    let composition = ProductCompositionRequest::new(EntryConfig::new(EntryProfile::Headless))
        .with_config_file_path(config_file.path())
        .compose()
        .unwrap();
    {
        let core = composition.core();
        let descriptor = core
            .load_config::<WindowDescriptor>(PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY)
            .expect("headless bootstrap should store a diagnostic window descriptor");

        assert_eq!(descriptor.primary_window, None);
        assert!(!descriptor.visible);
        assert!(descriptor
            .format_diagnostics()
            .contains("window.primary_window=none"));
    }
    super::product_composition::close_owner::close_composition(composition);
}

#[test]
fn headless_bootstrap_stores_headless_platform_config() {
    let config_file = IsolatedConfigFile::new();
    let composition = ProductCompositionRequest::new(EntryConfig::new(EntryProfile::Headless))
        .with_config_file_path(config_file.path())
        .compose()
        .unwrap();
    {
        let core = composition.core();
        let platform_config = core
            .load_config::<PlatformConfig>(PLATFORM_CONFIG_KEY)
            .expect("headless bootstrap should store the selected platform config");

        assert!(platform_config.enabled);
        assert_eq!(platform_config.target, PlatformTarget::Headless);
        assert_eq!(
            platform_config.target_mode,
            RuntimeTargetMode::ServerRuntime
        );
        assert_eq!(
            platform_config.features,
            PlatformFeatureSelection::headless()
        );
    }
    super::product_composition::close_owner::close_composition(composition);
}

#[test]
fn minimal_runtime_profile_stores_disabled_platform_config() {
    let config_file = IsolatedConfigFile::new();
    let entry = BuiltinEngineEntry::for_runtime_profile(RuntimeProfileId::Minimal)
        .unwrap()
        .with_config_file_path(config_file.path())
        .unwrap();
    let runtime = zircon_runtime::core::CoreRuntime::new();
    entry.bootstrap(&runtime).unwrap();
    let core = runtime.handle();
    {
        let platform_config = core
            .load_config::<PlatformConfig>(PLATFORM_CONFIG_KEY)
            .expect("minimal runtime profile should store a diagnostic platform config");

        assert!(!platform_config.enabled);
        assert_eq!(
            platform_config.target_mode,
            RuntimeTargetMode::ClientRuntime
        );
    }
    drop(core);
    super::product_composition::close_owner::close_runtime(&runtime);
}

#[cfg(feature = "graphics")]
#[test]
fn linked_runtime_render_feature_descriptors_rebuild_default_pipelines() {
    let config_file = IsolatedConfigFile::new();
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_required_runtime_plugins([RuntimePluginId::VirtualGeometry]);
    let report = RuntimePluginRegistrationReport::from_plugin(&LinkedVirtualGeometryPlugin {
        descriptor: RuntimePluginDescriptor::builder(
            "virtual_geometry",
            "Virtual Geometry",
            RuntimePluginId::VirtualGeometry,
            "zircon_plugin_virtual_geometry_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_capability("runtime.plugin.virtual_geometry")
        .with_capability("runtime.render.advanced.virtual_geometry")
        .build(),
    });
    let composition = ProductCompositionRequest::new(config)
        .with_runtime_plugin_registrations([report])
        .with_config_file_path(config_file.path())
        .compose()
        .expect("linked render feature plugin should bootstrap");
    {
        let core = composition.core().clone();
        let resolver = ManagerResolver::new(core.clone());
        let render_framework = resolver
            .render_framework_handle()
            .and_then(|handle| resolver.resolve(handle))
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
                    .with_temporal_history(false),
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
        assert_eq!(
            stats
                .advanced_provider_availability
                .virtual_geometry_provider_id
                .as_deref(),
            Some("virtual_geometry")
        );
        assert!(stats
            .last_graph_executed_passes
            .iter()
            .any(|pass| pass == "linked-virtual-geometry-pass"));
        assert!(!stats
            .last_graph_executed_passes
            .iter()
            .any(|pass| pass == "virtual-geometry-prepare"));
    }
    super::product_composition::close_owner::close_composition(composition);
}

#[cfg(windows)]
#[test]
#[ignore = "requires a coordinator-built dist DLL in the same managed target"]
fn bootstrap_accepts_required_native_dynamic_plugin_from_export_load_manifest() {
    use zircon_runtime::core::framework::project::{
        ExportPackagingStrategy, ExportTargetPlatform, ProjectPluginManifest,
        ProjectPluginSelection,
    };
    use zircon_runtime::plugin::native::{
        NativePluginArtifactAuthority, NativePluginArtifactDigest, NativePluginArtifactTarget,
        NativePluginBehaviorHealth,
    };

    let managed_target = PathBuf::from(
        std::env::var_os("CARGO_TARGET_DIR").expect("managed CARGO_TARGET_DIR is required"),
    )
    .canonicalize()
    .expect("managed Cargo target directory must exist");
    let dist_dll = managed_target
        .join("debug/zircon_plugin_virtual_geometry_dist.dll")
        .canonicalize()
        .expect("build the VirtualGeometry dist DLL in this managed target before this test");
    assert_eq!(
        dist_dll.file_name().and_then(|name| name.to_str()),
        Some("zircon_plugin_virtual_geometry_dist.dll")
    );
    let approved_roots = ["D:/cargo-targets", "E:/cargo-targets", "F:/cargo-targets"]
        .into_iter()
        .filter_map(|root| PathBuf::from(root).canonicalize().ok())
        .collect::<Vec<_>>();
    assert!(
        approved_roots
            .iter()
            .any(|root| managed_target.starts_with(root)),
        "managed target must physically reside under an approved Cargo target root"
    );
    assert!(
        approved_roots.iter().any(|root| dist_dll.starts_with(root)),
        "dist DLL must physically reside under an approved Cargo target root"
    );
    let dist_sha256 = NativePluginArtifactDigest::capture(&dist_dll)
        .expect("hash the managed dist DLL")
        .sha256;
    eprintln!("APP08_NATIVE_DIST_SHA256={dist_sha256}");
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after Unix epoch")
        .as_nanos();
    let export_root = managed_target.join(format!(
        "zircon_app_native_dynamic_bootstrap_{}_{stamp}",
        std::process::id()
    ));
    let package_root = export_root.join("plugins/virtual_geometry");
    fs::create_dir_all(package_root.join("native")).unwrap();
    fs::write(
        export_root.join("plugins/native_plugins.toml"),
        "[[plugins]]\nid = \"virtual_geometry\"\npath = \"plugins/virtual_geometry\"\nmanifest = \"plugins/virtual_geometry/plugin.toml\"\n",
    )
    .unwrap();
    let manifest = package_root.join("plugin.toml");
    fs::write(
        &manifest,
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../zircon_plugins/virtual_geometry/plugin.toml"
        )),
    )
    .expect("stage the source-pinned VirtualGeometry manifest");
    let staged_dll = package_root.join("native/zircon_plugin_virtual_geometry_dist.dll");
    fs::copy(&dist_dll, &staged_dll).expect("stage the managed-built dist DLL");
    assert_eq!(
        NativePluginArtifactDigest::capture(&staged_dll)
            .expect("hash the staged dist DLL")
            .sha256,
        dist_sha256,
        "staged bytes must match the managed target dist DLL"
    );
    let expectations = NativePluginArtifactAuthority::capture_trusted_local_package(
        "virtual_geometry",
        &manifest,
        "app08-native-dynamic-product-test",
        NativePluginArtifactTarget::new(
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        ),
        [PluginModuleKind::Runtime],
    )
    .expect("capture the selected first-party package bytes");
    let authority = NativePluginArtifactAuthority::from_expectations(expectations)
        .expect("create exact artifact authority");
    let config =
        EntryConfig::new(EntryProfile::Runtime).with_project_plugins(ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                "virtual_geometry",
                true,
                true,
            )
            .with_packaging(ExportPackagingStrategy::NativeDynamic)
            .with_target_modes([RuntimeTargetMode::ClientRuntime])],
        });

    let config_file = IsolatedConfigFile::new();
    let bootstrap = ProductCompositionRequest::new(config)
        .with_native_plugin_artifact_authority(authority)
        .with_native_plugins_from_export_root(&export_root)
        .with_config_file_path(config_file.path())
        .compose()
        .expect("the admitted native dist package must bootstrap");
    {
        let resolver = ManagerResolver::new(bootstrap.core().clone());
        assert!(resolver
            .rendering_handle()
            .and_then(|handle| resolver.resolve(handle))
            .is_ok());
        assert!(bootstrap
            .module_selection_report()
            .runtime_plugin_availability
            .contains(
                RuntimePluginAvailabilityCategory::NativeDynamic,
                RuntimePluginId::VirtualGeometry
            ));
        assert!(!bootstrap
            .module_selection_report()
            .runtime_plugin_availability
            .has_missing_required());
        assert!(bootstrap
            .module_selection_report()
            .module_keys()
            .contains(&"virtual_geometry.runtime"));
        assert!(bootstrap
            .native_plugin_host()
            .expect("native composition should retain its host owner")
            .loaded_plugin_ids(PluginModuleKind::Runtime)
            .expect("bootstrap host should expose loaded native runtime ids")
            .contains(&"virtual_geometry".to_owned()));
        let behavior = bootstrap
            .runtime_behavior_descriptor("virtual_geometry")
            .expect("the required plugin must expose a live Runtime entry");
        assert!(
            matches!(
                behavior
                    .validation_report
                    .as_ref()
                    .expect("Runtime entry must have a validation report")
                    .health,
                NativePluginBehaviorHealth::Clean | NativePluginBehaviorHealth::Degraded
            ),
            "required native entry cannot have invalid behavior"
        );
        assert_eq!(behavior.is_stateless, Some(true));
        assert!(behavior
            .registration_manifest
            .as_deref()
            .is_some_and(|manifest| !manifest.trim().is_empty()));
        assert!(!bootstrap.diagnostics().iter().any(|diagnostic| {
            diagnostic.contains("native plugin virtual_geometry skipped because library is missing")
        }));

        drop(resolver);
    }
    super::product_composition::close_owner::close_composition(bootstrap);
    assert!(export_root.starts_with(&managed_target));
    fs::remove_dir_all(export_root).expect("remove only the isolated managed fixture");
}

#[cfg(feature = "graphics")]
#[derive(Debug)]
struct LinkedVirtualGeometryPlugin {
    descriptor: RuntimePluginDescriptor,
}

#[cfg(feature = "graphics")]
impl RuntimePlugin for LinkedVirtualGeometryPlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(ModuleDescriptor::new(
            "VirtualGeometryPlugin",
            "Linked virtual geometry plugin module",
        ))?;
        registry.register_render_pass_executor(RenderPassExecutorRegistration::new(
            "virtual-geometry.prepare",
            linked_virtual_geometry_prepare_executor,
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
        )?;
        registry.register_virtual_geometry_runtime_provider(
            VirtualGeometryRuntimeProviderRegistration::new(
                "virtual_geometry",
                Arc::new(LinkedVirtualGeometryRuntimeProvider),
            ),
        )
    }
}

#[cfg(feature = "graphics")]
fn linked_virtual_geometry_prepare_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if context.pass_name == "linked-virtual-geometry-pass" {
        Ok(())
    } else {
        Err(format!(
            "linked virtual geometry executor received unexpected pass `{}`",
            context.pass_name
        ))
    }
}

#[cfg(feature = "graphics")]
#[derive(Debug)]
struct LinkedVirtualGeometryRuntimeProvider;

#[cfg(feature = "graphics")]
impl VirtualGeometryRuntimeProvider for LinkedVirtualGeometryRuntimeProvider {
    fn create_state(&self) -> Box<dyn VirtualGeometryRuntimeState> {
        Box::new(LinkedVirtualGeometryRuntimeState)
    }
}

#[cfg(feature = "graphics")]
#[derive(Debug)]
struct LinkedVirtualGeometryRuntimeState;

#[cfg(feature = "graphics")]
impl VirtualGeometryRuntimeState for LinkedVirtualGeometryRuntimeState {
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
struct LinkedPhysicsBridgePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl RuntimePlugin for LinkedPhysicsBridgePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> zircon_runtime::plugin::PluginPackageManifest {
        self.descriptor
            .package_manifest()
            .with_provided_interface_id(
                <dyn EntryBootstrapPhysicsBridge as PluginInterface>::INTERFACE_ID,
            )
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        let owner = registry.intern_plugin_module("physics.runtime")?;
        registry.export_interface::<dyn EntryBootstrapPhysicsBridge>(
            owner,
            Arc::new(EntryBootstrapPhysicsProvider),
        )
    }
}

trait EntryBootstrapPhysicsBridge: Send + Sync {}

impl PluginInterface for dyn EntryBootstrapPhysicsBridge {
    const INTERFACE_ID: &'static str = "entry.bootstrap.physics.v1";
}

#[derive(Debug)]
struct EntryBootstrapPhysicsProvider;

impl EntryBootstrapPhysicsBridge for EntryBootstrapPhysicsProvider {}

fn unique_export_root(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{stamp}"))
}
