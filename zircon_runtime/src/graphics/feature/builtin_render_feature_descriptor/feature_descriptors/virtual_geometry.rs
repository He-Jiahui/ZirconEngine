use crate::render_graph::QueueLane;

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;
use crate::RenderFeatureCapabilityRequirement;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "virtual_geometry",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-prepare",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.prepare")
            .write_buffer("virtual-geometry-page-requests"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-node-cluster-cull",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("virtual-geometry.node-cluster-cull")
            .read_buffer("virtual-geometry-page-requests")
            .write_buffer("virtual-geometry-visible-clusters"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-page-feedback",
                QueueLane::AsyncCopy,
            )
            .with_executor_id("virtual-geometry.page-feedback")
            .read_buffer("virtual-geometry-visible-clusters")
            .write_external("virtual-geometry-feedback"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-visbuffer",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.visbuffer")
            .read_buffer("virtual-geometry-visible-clusters")
            .write_texture("scene-depth"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Overlay,
                "virtual-geometry-debug-overlay",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.debug-overlay")
            .read_buffer("virtual-geometry-visible-clusters")
            .read_texture("scene-color")
            .write_texture("scene-color"),
        ],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry)
}
