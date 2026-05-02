use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::graphics::runtime::WgpuRenderFramework;
use crate::graphics::{
    FrameHistoryBinding, FrameHistorySlot, RenderFeatureCapabilityRequirement,
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage,
};
use crate::render_graph::QueueLane;

pub(super) fn pluginized_wgpu_render_framework() -> WgpuRenderFramework {
    pluginized_wgpu_render_framework_with_asset_manager(Arc::new(ProjectAssetManager::default()))
}

pub(super) fn pluginized_wgpu_render_framework_with_asset_manager(
    asset_manager: Arc<ProjectAssetManager>,
) -> WgpuRenderFramework {
    WgpuRenderFramework::new_with_plugin_render_features(
        asset_manager,
        [
            virtual_geometry_render_feature_descriptor(),
            hybrid_gi_render_feature_descriptor(),
        ],
        Vec::new(),
        Vec::new(),
    )
    .unwrap()
}

pub(super) fn virtual_geometry_render_feature_descriptor() -> RenderFeatureDescriptor {
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

pub(super) fn hybrid_gi_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "hybrid_gi",
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "visibility".to_string(),
        ],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::GlobalIllumination,
        )],
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-scene-prepare",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.scene-prepare")
            .read_texture("scene-depth")
            .write_buffer("hybrid-gi-scene"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-trace-schedule",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("hybrid-gi.trace-schedule")
            .read_buffer("hybrid-gi-scene")
            .write_buffer("hybrid-gi-trace"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-resolve",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.resolve")
            .read_buffer("hybrid-gi-trace")
            .write_texture("hybrid-gi-lighting"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "hybrid-gi-history",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.history")
            .read_texture("scene-color")
            .write_external("history-global-illumination"),
        ],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::HybridGlobalIllumination)
}

pub(super) fn default_rendering_feature_descriptors() -> Vec<RenderFeatureDescriptor> {
    vec![
        rendering_ssao_descriptor(),
        rendering_reflection_probes_descriptor(),
        rendering_baked_lighting_descriptor(),
        rendering_post_process_descriptor(),
    ]
}

pub(super) fn particle_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "particle",
        vec![
            "view".to_string(),
            "particles".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Transparent,
            "particle-render",
            QueueLane::Graphics,
        )
        .with_executor_id("particle.transparent")
        .read_texture("scene-depth")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn rendering_ssao_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "screen_space_ambient_occlusion",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::AmbientOcclusion,
        )],
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::AmbientOcclusion,
            "ssao-evaluate",
            QueueLane::AsyncCompute,
        )
        .with_executor_id("ao.ssao-evaluate")
        .read_texture("scene-depth")
        .write_texture("ambient-occlusion")],
    )
}

fn rendering_reflection_probes_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "reflection_probes",
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "reflection-probe-composite",
            QueueLane::Graphics,
        )
        .with_executor_id("lighting.reflection-probes")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn rendering_baked_lighting_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "baked_lighting",
        vec!["lighting".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "baked-lighting-composite",
            QueueLane::Graphics,
        )
        .with_executor_id("lighting.baked-composite")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn rendering_post_process_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "post_process",
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "post-process",
            QueueLane::Graphics,
        )
        .with_executor_id("post.stack")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}
