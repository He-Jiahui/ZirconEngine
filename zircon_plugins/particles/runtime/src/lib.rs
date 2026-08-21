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
pub use capability::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, PARTICLES_DECLARATION, PARTICLES_RUNTIME_CAPABILITY,
    PLUGIN_ID, RUNTIME_CAPABILITIES,
};
pub use component::{
    PARTICLE_SYSTEM_COMPONENT_TYPE, ParticleEmitterHandle, ParticleSystemComponent,
    particle_component_descriptors,
};
pub use interop::{
    ParticleAnimationBinding, ParticleAnimationEvent, ParticleAnimationEventKind,
    ParticleOptionalFeatureStatus, ParticlePhysicsOptions,
};
pub use module::{
    PARTICLES_MANAGER_NAME, PARTICLES_MODULE_NAME, ParticlesModule, module_descriptor,
    module_descriptor_with_manager,
};
pub use package::{
    PARTICLES_DYNAMIC_EVENT_NAMESPACE, attach_particles_manifest_contributions,
    particle_dependencies, particle_event_catalogs, particle_options,
};
pub use plugin::{
    PARTICLES_DIST_CRATE_NAME, PARTICLES_DIST_RUNTIME_ENTRY, PARTICLES_FEATURE_NAME,
    ParticlesRuntimePlugin, package_manifest, particle_animation_feature_manifest,
    particle_gpu_feature_manifest, particle_physics_feature_manifest, plugin_registration,
    runtime_capabilities, runtime_plugin, runtime_plugin_descriptor, runtime_selection,
};
pub use render::{
    PARTICLE_GPU_MAX_PARTICLES, ParticleGpuAttribute, ParticleGpuBackend, ParticleGpuBackendError,
    ParticleGpuBuffers, ParticleGpuCompileDiagnostic, ParticleGpuCompileDiagnosticSeverity,
    ParticleGpuCounterReadback, ParticleGpuCpuParityReport, ParticleGpuEmitterFrameParams,
    ParticleGpuEmitterLayout, ParticleGpuFallbackDiagnostic, ParticleGpuFallbackReason,
    ParticleGpuFrameParams, ParticleGpuFramePlanner, ParticleGpuLayout, ParticleGpuPassKind,
    ParticleGpuPassPlan, ParticleGpuProgram, ParticleGpuReadbackDecodeError,
    ParticleGpuReadbackRequest, ParticleGpuResourcePlan, ParticleGpuRuntimeBufferBindings,
    ParticleGpuRuntimeFrame, ParticleGpuRuntimeOwner, ParticleGpuRuntimeOwnerError,
    ParticleGpuRuntimeOwnerHandle, ParticleGpuShaderEntries, ParticleGpuShaderProgram,
    ParticleGpuTransparentRenderConfig, ParticleGpuTransparentRenderParams,
    ParticleGpuTransparentShaderEntries, ParticleGpuValueType, build_particle_extract,
    compile_particle_gpu_layout, compile_particle_gpu_program,
    particle_render_pass_executor_registrations,
    particle_render_pass_executor_registrations_with_gpu_owner,
    particle_runtime_prepare_collector_registration,
    particle_runtime_prepare_collector_registration_with_manager,
    particle_runtime_prepare_collector_registration_with_manager_and_owner,
    render_feature_descriptor,
};
pub use service::{
    ParticleEmitterState, ParticleGpuRuntimeInstance, ParticleRuntimeDiagnostic,
    ParticleRuntimeDiagnosticEntry, ParticleRuntimeDiagnosticPage,
    ParticleRuntimeDiagnosticSeverity, ParticleRuntimeSnapshot, ParticlesManager,
};
pub use simulation::{ParticleSimulationError, ParticleSpriteSnapshot};

#[cfg(test)]
mod tests;
