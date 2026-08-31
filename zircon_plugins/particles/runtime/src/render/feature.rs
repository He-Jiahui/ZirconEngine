use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage,
};
use zircon_runtime::render_graph::{QueueLane, RenderGraphComputeWorkload};

use super::gpu::PARTICLE_GPU_WORKGROUP_SIZE;
use crate::PARTICLES_FEATURE_NAME;

const PARTICLE_GPU_COMPUTE_WORKGROUP_SIZE: [u32; 3] = [PARTICLE_GPU_WORKGROUP_SIZE, 1, 1];
const PARTICLE_GPU_DYNAMIC_DISPATCH_GROUPS: [u32; 3] = [1, 1, 1];

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        PARTICLES_FEATURE_NAME,
        vec![
            "view".to_string(),
            "particles".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                "particle-gpu-spawn-update",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("particle.gpu.spawn-update")
            .with_compute_workload(RenderGraphComputeWorkload::fixed(
                "zircon-particle-gpu-spawn-update",
                PARTICLE_GPU_COMPUTE_WORKGROUP_SIZE,
                PARTICLE_GPU_DYNAMIC_DISPATCH_GROUPS,
            ))
            .read_external_buffer("particles.gpu.particles-a")
            .read_external_buffer("particles.gpu.emitter-params")
            .write_external_buffer("particles.gpu.particles-b")
            .write_external_buffer("particles.gpu.counters"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                "particle-gpu-compact-alive",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("particle.gpu.compact-alive")
            .with_compute_workload(RenderGraphComputeWorkload::fixed(
                "zircon-particle-gpu-compact-alive",
                PARTICLE_GPU_COMPUTE_WORKGROUP_SIZE,
                PARTICLE_GPU_DYNAMIC_DISPATCH_GROUPS,
            ))
            .read_external_buffer("particles.gpu.particles-b")
            .write_external_buffer("particles.gpu.alive-indices")
            .write_external_buffer("particles.gpu.counters"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                "particle-gpu-build-indirect-args",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("particle.gpu.indirect-args")
            .with_compute_workload(RenderGraphComputeWorkload::fixed(
                "zircon-particle-gpu-indirect-args",
                PARTICLE_GPU_COMPUTE_WORKGROUP_SIZE,
                PARTICLE_GPU_DYNAMIC_DISPATCH_GROUPS,
            ))
            .with_side_effects()
            .read_external_buffer("particles.gpu.counters")
            .write_external_buffer("particles.gpu.indirect-draw-args")
            .write_external_buffer("particles.gpu.debug-readback"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                "particle-render",
                QueueLane::Graphics,
            )
            .with_executor_id("particle.transparent")
            .read_external_buffer("particles.gpu.particles-b")
            .read_external_buffer("particles.gpu.alive-indices")
            .read_external_buffer("particles.gpu.indirect-draw-args")
            .read_texture("scene-depth")
            .read_texture("scene-color")
            .write_texture("scene-color"),
        ],
    )
}
