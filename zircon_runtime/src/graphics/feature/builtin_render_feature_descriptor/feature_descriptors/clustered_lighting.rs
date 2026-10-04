use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::QueueLane;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;
use super::compute_workload::clustered_lighting_dispatch_plan;

// 光照网格在异步计算队列生成共享缓冲，后续 mesh/deferred lighting pass 通过资源名消费。
pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    let light_grid_dispatch = clustered_lighting_dispatch_plan();

    RenderFeatureDescriptor::new(
        "clustered_lighting",
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Lighting,
            "light-grid-build",
            QueueLane::AsyncCompute,
        )
        .with_executor_id("lighting.light-grid")
        .with_compute_dispatch_plan(light_grid_dispatch)],
    )
}

#[cfg(test)]
#[path = "tests/clustered_lighting.rs"]
mod tests;
