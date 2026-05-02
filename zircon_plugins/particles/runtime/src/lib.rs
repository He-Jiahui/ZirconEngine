pub const PLUGIN_ID: &str = "particles";
pub const PARTICLES_FEATURE_NAME: &str = "particle";
pub const PARTICLES_RUNTIME_CAPABILITY: &str = "runtime.plugin.particles";

mod asset;
mod component;
mod interop;
mod module;
mod package;
mod render;
mod service;
mod simulation;

pub use asset::{
    ParticleBurst, ParticleColorKey, ParticleCoordinateSpace, ParticleEmitterAsset,
    ParticleScalarKey, ParticleScalarRange, ParticleShape, ParticleSimulationBackend,
    ParticleSystemAsset, ParticleVec3Range,
};
pub use component::{
    particle_component_descriptors, ParticleEmitterHandle, ParticleSystemComponent,
    PARTICLE_SYSTEM_COMPONENT_TYPE,
};
pub use interop::{
    ParticleAnimationBinding, ParticleAnimationEvent, ParticleAnimationEventKind,
    ParticleOptionalFeatureStatus, ParticlePhysicsOptions,
};
pub use module::{
    module_descriptor, ParticlesModule, PARTICLES_MANAGER_NAME, PARTICLES_MODULE_NAME,
};
pub use package::{
    attach_particles_manifest_contributions, particle_dependencies, particle_event_catalogs,
    particle_options, PARTICLES_DYNAMIC_EVENT_NAMESPACE,
};
pub use render::{
    build_particle_extract, compile_particle_gpu_layout, compile_particle_gpu_program,
    particle_render_pass_executor_registrations, render_feature_descriptor, ParticleGpuAttribute,
    ParticleGpuBackend, ParticleGpuBackendError, ParticleGpuBuffers, ParticleGpuCompileDiagnostic,
    ParticleGpuCompileDiagnosticSeverity, ParticleGpuCounterReadback, ParticleGpuCpuParityReport,
    ParticleGpuEmitterFrameParams, ParticleGpuEmitterLayout, ParticleGpuFallbackDiagnostic,
    ParticleGpuFallbackReason, ParticleGpuFrameParams, ParticleGpuFramePlanner, ParticleGpuLayout,
    ParticleGpuPassKind, ParticleGpuPassPlan, ParticleGpuProgram, ParticleGpuReadbackDecodeError,
    ParticleGpuReadbackRequest, ParticleGpuResourcePlan, ParticleGpuShaderEntries,
    ParticleGpuShaderProgram, ParticleGpuTransparentRenderConfig,
    ParticleGpuTransparentRenderParams, ParticleGpuTransparentShaderEntries, ParticleGpuValueType,
    PARTICLE_GPU_MAX_PARTICLES,
};
pub use service::{
    ParticleEmitterState, ParticleRuntimeDiagnostic, ParticleRuntimeDiagnosticSeverity,
    ParticleRuntimeSnapshot, ParticlesManager,
};
pub use simulation::{ParticleSimulationError, ParticleSpriteSnapshot};

#[derive(Clone, Debug)]
pub struct ParticlesRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl ParticlesRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for ParticlesRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_runtime::plugin::RuntimePlugin for ParticlesRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> zircon_runtime::plugin::PluginPackageManifest {
        attach_particles_manifest_contributions(self.descriptor.package_manifest())
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())?;
        registry.register_render_feature(render_feature_descriptor())?;
        for registration in particle_render_pass_executor_registrations() {
            registry.register_render_pass_executor(registration)?;
        }
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

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Particles",
        zircon_runtime::RuntimePluginId::Particles,
        "zircon_plugin_particles_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability(PARTICLES_RUNTIME_CAPABILITY)
    .with_optional_feature(particle_physics_feature_manifest())
    .with_optional_feature(particle_animation_feature_manifest())
    .with_optional_feature(particle_gpu_feature_manifest())
}

pub fn runtime_plugin() -> ParticlesRuntimePlugin {
    ParticlesRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection {
    zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
    zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[PARTICLES_RUNTIME_CAPABILITY]
}

pub fn particle_physics_feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_runtime::plugin::PluginFeatureBundleManifest::new(
        "particles.physics",
        "Physical Particles",
        PLUGIN_ID,
    )
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        PLUGIN_ID,
        PARTICLES_RUNTIME_CAPABILITY,
    ))
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
        "physics",
        "runtime.plugin.physics",
    ))
    .with_capability("runtime.feature.particles.physics")
}

pub fn particle_animation_feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_runtime::plugin::PluginFeatureBundleManifest::new(
        "particles.animation_control",
        "Animation Controlled Particles",
        PLUGIN_ID,
    )
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        PLUGIN_ID,
        PARTICLES_RUNTIME_CAPABILITY,
    ))
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
        "animation",
        "runtime.plugin.animation",
    ))
    .with_capability("runtime.feature.particles.animation_control")
}

pub fn particle_gpu_feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_runtime::plugin::PluginFeatureBundleManifest::new(
        "particles.gpu_simulation",
        "GPU Particle Simulation",
        PLUGIN_ID,
    )
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        PLUGIN_ID,
        PARTICLES_RUNTIME_CAPABILITY,
    ))
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
        "render_graph",
        "runtime.module.render_graph",
    ))
    .with_capability("runtime.feature.particles.gpu_simulation")
}

#[cfg(test)]
mod tests;
