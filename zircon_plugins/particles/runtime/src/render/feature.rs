use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage,
};
use zircon_runtime::render_graph::QueueLane;

use crate::PARTICLES_FEATURE_NAME;

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
            .with_side_effects()
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
            .with_side_effects()
            .read_external_buffer("particles.gpu.particles-b")
            .write_external_buffer("particles.gpu.alive-indices")
            .write_external_buffer("particles.gpu.counters"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                "particle-gpu-build-indirect-args",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("particle.gpu.indirect-args")
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
