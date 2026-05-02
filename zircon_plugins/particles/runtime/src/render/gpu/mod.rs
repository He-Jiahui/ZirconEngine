mod backend;
mod layout;
mod planner;
mod program;
mod readback;
mod shaders;
mod transparent;

pub use backend::{
    ParticleGpuBackend, ParticleGpuBackendError, ParticleGpuBuffers, ParticleGpuReadbackRequest,
};
pub use layout::{
    compile_particle_gpu_layout, ParticleGpuAttribute, ParticleGpuEmitterLayout, ParticleGpuLayout,
    ParticleGpuValueType, PARTICLE_GPU_MAX_PARTICLES,
};
pub use planner::{ParticleGpuEmitterFrameParams, ParticleGpuFrameParams, ParticleGpuFramePlanner};
pub use program::{
    compile_particle_gpu_program, ParticleGpuCompileDiagnostic,
    ParticleGpuCompileDiagnosticSeverity, ParticleGpuFallbackDiagnostic, ParticleGpuFallbackReason,
    ParticleGpuPassKind, ParticleGpuPassPlan, ParticleGpuProgram, ParticleGpuResourcePlan,
    ParticleGpuShaderEntries, ParticleGpuShaderProgram, ParticleGpuTransparentShaderEntries,
};
pub use readback::{
    ParticleGpuCounterReadback, ParticleGpuCpuParityReport, ParticleGpuReadbackDecodeError,
};
pub use transparent::{ParticleGpuTransparentRenderConfig, ParticleGpuTransparentRenderParams};

pub(crate) use shaders::{generate_particle_gpu_transparent_wgsl, generate_particle_gpu_wgsl};
