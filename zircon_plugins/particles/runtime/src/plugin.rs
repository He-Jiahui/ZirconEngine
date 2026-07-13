use crate::capability::{PARTICLES_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES};
use crate::component::particle_component_descriptors;
use crate::module::module_descriptor_with_manager;
use crate::package::{
    attach_particles_manifest_contributions, particle_event_catalogs, particle_options,
};
use crate::render::{
    particle_render_pass_executor_registrations_with_gpu_owner,
    particle_runtime_prepare_collector_registration_with_manager_and_owner,
    render_feature_descriptor, ParticleGpuRuntimeOwnerHandle,
};
use crate::service::ParticlesManager;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::plugin::{
    CapabilityStatus, CapabilityStatusManifest, PluginDistributionManifest,
    PluginFeatureBundleManifest, PluginFeatureDependency, PluginMaturity, PluginModuleManifest,
    PluginPackageManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginDescriptor,
};
use zircon_runtime::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

pub const PLUGIN_ID: &str = "particles";
pub const PARTICLES_FEATURE_NAME: &str = "particle";
pub const PARTICLES_DIST_CRATE_NAME: &str = "zircon_plugin_particles_dist";
pub const PARTICLES_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_particles_runtime_entry_v3";

const PARTICLES_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct ParticlesRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
    manager: ParticlesManager,
}

impl ParticlesRuntimePlugin {
    pub fn new() -> Self {
        let manager = ParticlesManager::default();
        Self {
            descriptor: runtime_plugin_descriptor_with_manager(manager.clone()),
            manager,
        }
    }

    pub fn manager(&self) -> ParticlesManager {
        self.manager.clone()
    }
}

impl Default for ParticlesRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for ParticlesRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest =
            attach_particles_manifest_contributions(self.descriptor.package_manifest());
        manifest
            .default_packaging
            .push(ExportPackagingStrategy::NativeDynamic);
        manifest.modules.push(
            PluginModuleManifest::native("particles.dist", PARTICLES_DIST_CRATE_NAME)
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ])
                .with_capabilities(RUNTIME_CAPABILITIES.iter().copied()),
        );
        manifest.with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: PARTICLES_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: PARTICLES_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: PARTICLES_DIST_RUNTIME_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let runtime_owner = ParticleGpuRuntimeOwnerHandle::default();
        registry.register_render_feature(render_feature_descriptor())?;
        for registration in
            particle_render_pass_executor_registrations_with_gpu_owner(runtime_owner.clone())
        {
            registry.register_render_pass_executor(registration)?;
        }
        registry.register_runtime_prepare_collector(
            particle_runtime_prepare_collector_registration_with_manager_and_owner(
                self.manager.clone(),
                runtime_owner,
            ),
        )?;
        for component in particle_component_descriptors() {
            registry.register_component(component)?;
        }
        for option in particle_options() {
            registry.register_plugin_option(option)?;
        }
        for event_catalog in particle_event_catalogs() {
            registry.register_plugin_event_catalog(event_catalog)?;
        }
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    runtime_plugin_descriptor_with_manager(ParticlesManager::default())
}

fn runtime_plugin_descriptor_with_manager(manager: ParticlesManager) -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Particles",
        RuntimePluginId::Particles,
        "zircon_plugin_particles_runtime",
    )
    .with_module_descriptor(module_descriptor_with_manager(manager))
    .with_category("runtime")
    .with_maturity(PluginMaturity::Experimental)
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability(PARTICLES_RUNTIME_CAPABILITY)
    .with_capability_status(
        CapabilityStatusManifest::new(PARTICLES_RUNTIME_CAPABILITY, CapabilityStatus::Partial)
            .with_note("Advanced optional VFX capability; not a Bevy default parity blocker."),
    )
    .with_optional_feature(particle_physics_feature_manifest())
    .with_optional_feature(particle_animation_feature_manifest())
    .with_optional_feature(particle_gpu_feature_manifest())
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(ParticlesRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}

pub fn particle_physics_feature_manifest() -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new("particles.physics", "Physical Particles", PLUGIN_ID)
        .with_dependency(PluginFeatureDependency::primary(
            PLUGIN_ID,
            PARTICLES_RUNTIME_CAPABILITY,
        ))
        .with_dependency(PluginFeatureDependency::required(
            "physics",
            "runtime.plugin.physics",
        ))
        .with_capability("runtime.feature.particles.physics")
}

pub fn particle_animation_feature_manifest() -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new(
        "particles.animation_control",
        "Animation Controlled Particles",
        PLUGIN_ID,
    )
    .with_dependency(PluginFeatureDependency::primary(
        PLUGIN_ID,
        PARTICLES_RUNTIME_CAPABILITY,
    ))
    .with_dependency(PluginFeatureDependency::required(
        "animation",
        "runtime.plugin.animation",
    ))
    .with_capability("runtime.feature.particles.animation_control")
}

pub fn particle_gpu_feature_manifest() -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new(
        "particles.gpu_simulation",
        "GPU Particle Simulation",
        PLUGIN_ID,
    )
    .with_dependency(PluginFeatureDependency::primary(
        PLUGIN_ID,
        PARTICLES_RUNTIME_CAPABILITY,
    ))
    .with_dependency(PluginFeatureDependency::required(
        "render_graph",
        "runtime.module.render_graph",
    ))
    .with_capability("runtime.feature.particles.gpu_simulation")
}
