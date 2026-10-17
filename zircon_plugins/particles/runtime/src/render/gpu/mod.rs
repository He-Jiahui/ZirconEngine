#[path = "backend.rs"]
mod backend;
#[path = "layout.rs"]
mod layout;
#[path = "neutral_buffers.rs"]
mod neutral_buffers;
#[path = "planner.rs"]
mod planner;
#[path = "program.rs"]
mod program;
#[path = "readback.rs"]
mod readback;
#[path = "runtime_owner.rs"]
mod runtime_owner;
#[path = "shaders.rs"]
mod shaders;
#[path = "transparent.rs"]
mod transparent;

pub use backend::{
    ParticleGpuBackend, ParticleGpuBackendError, ParticleGpuBuffers, ParticleGpuFragmentMaterial,
    ParticleGpuReadbackRequest,
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
pub use runtime_owner::{
    ParticleGpuRuntimeBufferBindings, ParticleGpuRuntimeFrame, ParticleGpuRuntimeOwner,
    ParticleGpuRuntimeOwnerError, ParticleGpuRuntimeOwnerHandle,
};
pub use transparent::{ParticleGpuTransparentRenderConfig, ParticleGpuTransparentRenderParams};

pub(crate) use neutral_buffers::PARTICLE_GPU_NEUTRAL_MAX_EMITTERS;
pub(crate) use program::{
    PARTICLE_GPU_COUNTER_WORDS_BASE, PARTICLE_GPU_INDIRECT_DRAW_WORDS, PARTICLE_GPU_WORKGROUP_SIZE,
};
pub(crate) use shaders::{generate_particle_gpu_transparent_wgsl, generate_particle_gpu_wgsl};
