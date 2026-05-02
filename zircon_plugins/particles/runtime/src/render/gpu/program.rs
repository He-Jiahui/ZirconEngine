use crate::{ParticleEmitterHandle, ParticleSimulationBackend, ParticleSystemAsset};
use zircon_runtime::render_graph::QueueLane;

use super::layout::{ParticleGpuLayout, PARTICLE_GPU_MAX_PARTICLES};
use super::{
    compile_particle_gpu_layout, generate_particle_gpu_transparent_wgsl, generate_particle_gpu_wgsl,
};

pub const PARTICLE_GPU_WORKGROUP_SIZE: u32 = 64;
pub const PARTICLE_GPU_COUNTER_WORDS_BASE: u32 = 4;
pub const PARTICLE_GPU_INDIRECT_DRAW_WORDS: u64 = 4;
pub const PARTICLE_GPU_TRANSPARENT_RENDER_PARAMS_BYTES: u64 = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleGpuFallbackReason {
    BackendUnavailable,
    ShaderCompilationUnavailable,
    CapacityExceeded,
    UnsupportedModule,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleGpuFallbackDiagnostic {
    pub handle: ParticleEmitterHandle,
    pub reason: ParticleGpuFallbackReason,
    pub message: String,
}

impl ParticleGpuFallbackDiagnostic {
    pub fn new(
        handle: ParticleEmitterHandle,
        reason: ParticleGpuFallbackReason,
        message: impl Into<String>,
    ) -> Self {
        Self {
            handle,
            reason,
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleGpuCompileDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleGpuCompileDiagnostic {
    pub severity: ParticleGpuCompileDiagnosticSeverity,
    pub reason: ParticleGpuFallbackReason,
    pub message: String,
}

impl ParticleGpuCompileDiagnostic {
    pub fn warning(reason: ParticleGpuFallbackReason, message: impl Into<String>) -> Self {
        Self {
            severity: ParticleGpuCompileDiagnosticSeverity::Warning,
            reason,
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleGpuPassKind {
    SpawnUpdate,
    CompactAlive,
    BuildIndirectArgs,
    TransparentRender,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleGpuPassPlan {
    pub kind: ParticleGpuPassKind,
    pub pass_name: &'static str,
    pub executor_id: &'static str,
    pub queue: QueueLane,
    pub reads: Vec<&'static str>,
    pub writes: Vec<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleGpuResourcePlan {
    pub particle_buffer_bytes: u64,
    pub double_buffered_particle_buffers: u32,
    pub emitter_params_bytes: u64,
    pub counter_bytes: u64,
    pub alive_indices_bytes: u64,
    pub indirect_draw_args_bytes: u64,
    pub debug_readback_bytes: u64,
    pub transparent_render_params_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleGpuShaderEntries {
    pub spawn_update: &'static str,
    pub compact_alive: &'static str,
    pub build_indirect_args: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleGpuTransparentShaderEntries {
    pub vertex: &'static str,
    pub fragment: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleGpuShaderProgram {
    pub wgsl: String,
    pub entries: ParticleGpuShaderEntries,
    pub transparent_wgsl: String,
    pub transparent_entries: ParticleGpuTransparentShaderEntries,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleGpuProgram {
    pub layout: ParticleGpuLayout,
    pub resources: ParticleGpuResourcePlan,
    pub passes: Vec<ParticleGpuPassPlan>,
    pub shader: ParticleGpuShaderProgram,
    pub diagnostics: Vec<ParticleGpuCompileDiagnostic>,
}

impl ParticleGpuProgram {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == ParticleGpuCompileDiagnosticSeverity::Error)
    }
}

pub fn compile_particle_gpu_program(asset: &ParticleSystemAsset) -> ParticleGpuProgram {
    let layout = compile_particle_gpu_layout(asset);
    let mut diagnostics = Vec::new();

    if asset.backend != ParticleSimulationBackend::Gpu {
        diagnostics.push(ParticleGpuCompileDiagnostic::warning(
            ParticleGpuFallbackReason::BackendUnavailable,
            "particle asset is configured for CPU simulation; GPU program was compiled for inspection only",
        ));
    }

    if layout.clamped {
        diagnostics.push(ParticleGpuCompileDiagnostic::warning(
            ParticleGpuFallbackReason::CapacityExceeded,
            format!(
                "particle GPU capacity requested {} particles and was clamped to {}",
                layout.requested_capacity, PARTICLE_GPU_MAX_PARTICLES
            ),
        ));
    }

    for emitter in &asset.emitters {
        if emitter.color_over_lifetime.len() > 2 || emitter.size_over_lifetime.len() > 2 {
            diagnostics.push(ParticleGpuCompileDiagnostic::warning(
                ParticleGpuFallbackReason::UnsupportedModule,
                format!(
                    "emitter `{}` uses multi-key curves; GPU baseline evaluates first-to-last linear keys",
                    emitter.id
                ),
            ));
        }
    }

    let resources = resource_plan_for(&layout);
    let passes = default_particle_gpu_pass_plan();
    let shader = ParticleGpuShaderProgram {
        wgsl: generate_particle_gpu_wgsl(&layout),
        entries: ParticleGpuShaderEntries {
            spawn_update: "particle_spawn_update",
            compact_alive: "particle_compact_alive",
            build_indirect_args: "particle_build_indirect_args",
        },
        transparent_wgsl: generate_particle_gpu_transparent_wgsl(&layout),
        transparent_entries: ParticleGpuTransparentShaderEntries {
            vertex: "particle_gpu_transparent_vs",
            fragment: "particle_gpu_transparent_fs",
        },
    };

    ParticleGpuProgram {
        layout,
        resources,
        passes,
        shader,
        diagnostics,
    }
}

pub fn default_particle_gpu_pass_plan() -> Vec<ParticleGpuPassPlan> {
    vec![
        ParticleGpuPassPlan {
            kind: ParticleGpuPassKind::SpawnUpdate,
            pass_name: "particle-gpu-spawn-update",
            executor_id: "particle.gpu.spawn-update",
            queue: QueueLane::AsyncCompute,
            reads: vec!["particles.gpu.particles-a", "particles.gpu.emitter-params"],
            writes: vec!["particles.gpu.particles-b", "particles.gpu.counters"],
        },
        ParticleGpuPassPlan {
            kind: ParticleGpuPassKind::CompactAlive,
            pass_name: "particle-gpu-compact-alive",
            executor_id: "particle.gpu.compact-alive",
            queue: QueueLane::AsyncCompute,
            reads: vec!["particles.gpu.particles-b"],
            writes: vec!["particles.gpu.alive-indices", "particles.gpu.counters"],
        },
        ParticleGpuPassPlan {
            kind: ParticleGpuPassKind::BuildIndirectArgs,
            pass_name: "particle-gpu-build-indirect-args",
            executor_id: "particle.gpu.indirect-args",
            queue: QueueLane::AsyncCompute,
            reads: vec!["particles.gpu.counters"],
            writes: vec![
                "particles.gpu.indirect-draw-args",
                "particles.gpu.debug-readback",
            ],
        },
        ParticleGpuPassPlan {
            kind: ParticleGpuPassKind::TransparentRender,
            pass_name: "particle-render",
            executor_id: "particle.transparent",
            queue: QueueLane::Graphics,
            reads: vec![
                "particles.gpu.particles-b",
                "particles.gpu.alive-indices",
                "particles.gpu.indirect-draw-args",
            ],
            writes: Vec::new(),
        },
    ]
}

fn resource_plan_for(layout: &ParticleGpuLayout) -> ParticleGpuResourcePlan {
    let particle_buffer_bytes = layout
        .storage_bytes()
        .max(std::mem::size_of::<u32>() as u64);
    let counter_words = PARTICLE_GPU_COUNTER_WORDS_BASE as u64 + layout.emitter_count as u64;
    ParticleGpuResourcePlan {
        particle_buffer_bytes,
        double_buffered_particle_buffers: 2,
        emitter_params_bytes: (layout.emitter_count as u64).max(1) * 256,
        counter_bytes: counter_words * std::mem::size_of::<u32>() as u64,
        alive_indices_bytes: (layout.capacity as u64).max(1) * std::mem::size_of::<u32>() as u64,
        indirect_draw_args_bytes: PARTICLE_GPU_INDIRECT_DRAW_WORDS
            * std::mem::size_of::<u32>() as u64,
        debug_readback_bytes: counter_words * std::mem::size_of::<u32>() as u64
            + PARTICLE_GPU_INDIRECT_DRAW_WORDS * std::mem::size_of::<u32>() as u64,
        transparent_render_params_bytes: PARTICLE_GPU_TRANSPARENT_RENDER_PARAMS_BYTES,
    }
}
