mod asset;
mod capability;
mod component;
mod interop;
mod module;
mod package;
mod plugin;
mod render;
mod service;
mod simulation;

pub use asset::{
    ParticleBurst, ParticleColorKey, ParticleCoordinateSpace, ParticleEmitterAsset,
    ParticleScalarKey, ParticleScalarRange, ParticleShape, ParticleSimulationBackend,
    ParticleSystemAsset, ParticleVec3Range,
};
pub use capability::{PARTICLES_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES};
pub use component::{
    particle_component_descriptors, ParticleEmitterHandle, ParticleSystemComponent,
    PARTICLE_SYSTEM_COMPONENT_TYPE,
};
pub use interop::{
    ParticleAnimationBinding, ParticleAnimationEvent, ParticleAnimationEventKind,
    ParticleOptionalFeatureStatus, ParticlePhysicsOptions,
};
pub use module::{
    module_descriptor, module_descriptor_with_manager, ParticlesModule, PARTICLES_MANAGER_NAME,
    PARTICLES_MODULE_NAME,
};
pub use package::{
    attach_particles_manifest_contributions, particle_dependencies, particle_event_catalogs,
    particle_options, PARTICLES_DYNAMIC_EVENT_NAMESPACE,
};
pub use plugin::{
    package_manifest, particle_animation_feature_manifest, particle_gpu_feature_manifest,
    particle_physics_feature_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, ParticlesRuntimePlugin,
    PARTICLES_DIST_CRATE_NAME, PARTICLES_DIST_RUNTIME_ENTRY, PARTICLES_FEATURE_NAME, PLUGIN_ID,
};
pub use render::{
    build_particle_extract, compile_particle_gpu_layout, compile_particle_gpu_program,
    particle_render_pass_executor_registrations,
    particle_render_pass_executor_registrations_with_gpu_owner,
    particle_runtime_prepare_collector_registration,
    particle_runtime_prepare_collector_registration_with_manager,
    particle_runtime_prepare_collector_registration_with_manager_and_owner,
    render_feature_descriptor, ParticleGpuAttribute, ParticleGpuBackend, ParticleGpuBackendError,
    ParticleGpuBuffers, ParticleGpuCompileDiagnostic, ParticleGpuCompileDiagnosticSeverity,
    ParticleGpuCounterReadback, ParticleGpuCpuParityReport, ParticleGpuEmitterFrameParams,
    ParticleGpuEmitterLayout, ParticleGpuFallbackDiagnostic, ParticleGpuFallbackReason,
    ParticleGpuFrameParams, ParticleGpuFramePlanner, ParticleGpuLayout, ParticleGpuPassKind,
    ParticleGpuPassPlan, ParticleGpuProgram, ParticleGpuReadbackDecodeError,
    ParticleGpuReadbackRequest, ParticleGpuResourcePlan, ParticleGpuRuntimeBufferBindings,
    ParticleGpuRuntimeFrame, ParticleGpuRuntimeOwner, ParticleGpuRuntimeOwnerError,
    ParticleGpuRuntimeOwnerHandle, ParticleGpuShaderEntries, ParticleGpuShaderProgram,
    ParticleGpuTransparentRenderConfig, ParticleGpuTransparentRenderParams,
    ParticleGpuTransparentShaderEntries, ParticleGpuValueType, PARTICLE_GPU_MAX_PARTICLES,
};
pub use service::{
    ParticleEmitterState, ParticleGpuRuntimeInstance, ParticleRuntimeDiagnostic,
    ParticleRuntimeDiagnosticSeverity, ParticleRuntimeSnapshot, ParticlesManager,
};
pub use simulation::{ParticleSimulationError, ParticleSpriteSnapshot};

#[cfg(test)]
mod tests;
