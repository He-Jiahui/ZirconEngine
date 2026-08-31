use crate::capability::{
    NAVIGATION_DECLARATION, NAVIGATION_RECAST_CAPABILITY, NAVIGATION_RUNTIME_CAPABILITY,
    RUNTIME_CAPABILITIES, RUNTIME_CRATE_NAME,
};
use crate::{
    module_descriptor, navigation_component_descriptors, navigation_event_catalog,
    navigation_plugin_options, DefaultNavigationManager, NavigationOverlayFrame,
    DEFAULT_NAVIGATION_RUNTIME_DRIVER_NAME, NAVIGATION_MODULE_NAME,
    NAVIGATION_OVERLAY_FRAME_EVENT_ID, PLUGIN_ID,
};
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NavMeshBakeReport, NavPathResult, NavigationDebugCapture, NavigationError,
    OffMeshTraverseEvent,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::navigation::NavRepathBudget;
use zircon_runtime::plugin::{
    CapabilityStatus, CapabilityStatusManifest, PluginDistributionManifest, PluginModuleManifest,
    PluginPackageManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginDescriptor,
};

pub const NAVIGATION_DIST_CRATE_NAME: &str = "zircon_plugin_navigation_dist";
pub const NAVIGATION_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_navigation_runtime_entry_v3";
pub const NAVIGATION_MAIN_SYSTEM_SET: &str = "navigation.main";
const NAVIGATION_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;
pub const NAVIGATION_AGENT_TICK_SYSTEM: &str = "navigation.agent_tick";

#[derive(Clone, Debug)]
pub struct NavigationRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl NavigationRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for NavigationRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for NavigationRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest = self.descriptor().package_manifest();
        for descriptor in navigation_component_descriptors() {
            manifest = manifest.with_component(descriptor);
        }
        for option in navigation_plugin_options() {
            manifest = manifest.with_option(option);
        }
        manifest = manifest.with_event_catalog(navigation_event_catalog());
        manifest = manifest.with_native_module(
            PluginModuleManifest::native("navigation.dist", NAVIGATION_DIST_CRATE_NAME)
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::ServerRuntime,
                    RuntimeTargetMode::EditorHost,
                ])
                .with_capabilities(RUNTIME_CAPABILITIES.iter().copied()),
        );
        manifest.with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: NAVIGATION_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: NAVIGATION_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: NAVIGATION_DIST_RUNTIME_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        for descriptor in navigation_component_descriptors() {
            registry.register_component(descriptor)?;
        }
        for option in navigation_plugin_options() {
            registry.register_plugin_option(option)?;
        }
        let owner = registry.intern_plugin_module(NAVIGATION_MODULE_NAME)?;
        let event_catalog = navigation_event_catalog();
        let event = |id: &str| {
            event_catalog
                .events
                .iter()
                .find(|event| event.id == id)
                .cloned()
                .expect("navigation event manifest exists")
        };
        registry
            .register_event::<NavMeshBakeReport>(owner, event("navigation.events.navmesh_baked"))?;
        registry.register_event::<NavPathResult>(
            owner,
            event("navigation.events.path_query_completed"),
        )?;
        registry.register_event::<NavigationError>(
            owner,
            event("navigation.events.path_query_failed"),
        )?;
        registry.register_event::<NavAgentTickReport>(
            owner,
            event("navigation.events.agent_tick_completed"),
        )?;
        registry.register_mirrored_event::<NavigationOverlayFrame>(
            owner,
            event(NAVIGATION_OVERLAY_FRAME_EVENT_ID),
            |world, reader_count| {
                let capture = world
                    .get_resource_mut::<NavigationDebugCapture>()
                    .ok_or_else(|| {
                        zircon_runtime::scene::SceneError::Message(
                            "navigation debug capture resource is not registered".to_string(),
                        )
                    })?;
                capture.enabled = reader_count > 0;
                Ok(())
            },
        )?;
        registry.register_event::<OffMeshTraverseEvent>(
            owner,
            event("navigation.events.off_mesh_traverse"),
        )?;
        registry.register_resource(owner, NavRepathBudget::default)?;
        registry.register_resource(owner, NavigationDebugCapture::default)?;
        let navigation_main_system_set = registry.intern_system_set(NAVIGATION_MAIN_SYSTEM_SET)?;
        registry
            .register_runtime_scene_system(
                owner,
                NAVIGATION_AGENT_TICK_SYSTEM,
                zircon_runtime::scene::SystemStage::Update,
                || {
                    |context| {
                        let manager = context.core.resolve_driver::<DefaultNavigationManager>(
                            DEFAULT_NAVIGATION_RUNTIME_DRIVER_NAME,
                        )?;
                        context
                            .level
                            .with_world_mut(
                                |world| -> Result<NavAgentTickReport, NavigationError> {
                                    let report = manager
                                        .tick_world_agents(world, context.tick().delta_seconds())?;
                                    let overlay_frame = navigation_overlay_frame_if_enabled(
                                        &manager, world, &report,
                                    );
                                    world.send_event(report.clone());
                                    if let Some(overlay_frame) = overlay_frame {
                                        world.send_event(overlay_frame);
                                    }
                                    Ok(report)
                                },
                            )
                            .map(|_| ())
                            .map_err(|error| {
                                zircon_runtime::core::CoreError::Initialization(
                                    "navigation.agent_tick".to_string(),
                                    error.to_string(),
                                )
                            })
                    }
                },
            )
            .in_set(navigation_main_system_set)
            .after(zircon_runtime::scene::ecs::SystemRef::System(
                "ai.behavior_tick".to_string(),
            ))
            .register()?;
        Ok(())
    }
}

pub(crate) fn navigation_overlay_frame_if_enabled(
    manager: &DefaultNavigationManager,
    world: &zircon_runtime::scene::World,
    report: &NavAgentTickReport,
) -> Option<NavigationOverlayFrame> {
    world
        .get_resource::<NavigationDebugCapture>()
        .is_some_and(|capture| capture.enabled)
        .then(|| manager.navigation_overlay_frame(report.clone()))
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    NAVIGATION_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .with_system_sets([NAVIGATION_MAIN_SYSTEM_SET])
        .with_system_anchors([NAVIGATION_AGENT_TICK_SYSTEM])
        .with_capability_status(
            CapabilityStatusManifest::new(NAVIGATION_RUNTIME_CAPABILITY, CapabilityStatus::Partial)
                .with_note(
                    "Gameplay navmesh/pathfinding is optional; UI navigation parity is separate.",
                ),
        )
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(NavigationRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}
