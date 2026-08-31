use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    AdvancedProviderReport, AdvancedProviderStatus, AdvancedRenderDegradationReason,
    AdvancedRenderFeature, COLOR_LUT_SIZE_DEFAULT, CameraRenderDescriptor, CameraRenderType,
    FrameHistoryHandle, FrameHistoryInvalidationReason, PostProcessVolumeExtract,
    RenderBloomSettings, RenderCameraOrderAmbiguity, RenderCameraOrderReport,
    RenderCameraTargetOrderKey, RenderCapabilityKind, RenderCapabilityMismatchDetail,
    RenderCapabilitySummary, RenderChromaticAberrationSettings, RenderColorGradingSettings,
    RenderColorLookupSettings, RenderDepthOfFieldSettings, RenderDitherSettings,
    RenderDynamicResolutionSettings, RenderFilmGrainSettings, RenderFogSettings,
    RenderFrameExtract, RenderFramework, RenderFrameworkError, RenderHybridGiExtract,
    RenderHybridGiPayloadSource, RenderPipelineHandle, RenderPostProcessEffectStackSettings,
    RenderPostProcessVolumeProfile, RenderQualityProfile, RenderScreenSpaceReflectionSettings,
    RenderStats, RenderViewportDescriptor, RenderViewportHandle, RenderVignetteSettings,
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract, RenderVirtualGeometryPage,
    RenderVirtualGeometryPayloadSource, RenderWorldSnapshotHandle, SortedRenderCamera,
    UiRenderSubmission, ViewportCameraSnapshot, VolumeComponentOverride,
};
use crate::core::math::{Transform, UVec2, Vec3};
use crate::scene::world::World;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
    UiTextAlign, UiTextRenderMode, UiTextWrap,
};

use crate::graphics::{
    BuiltinRenderFeature, RenderFeatureCapabilityRequirement, RenderFeatureDescriptor,
    RenderFeaturePassDescriptor, RenderPassExecutionContext, RenderPassExecutorRegistration,
    RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions,
    runtime::WgpuRenderFramework,
};
use crate::render_graph::{QueueLane, RenderGraphComputeWorkload};

use super::plugin_render_feature_fixtures::{
    pluginized_wgpu_render_framework, pluginized_wgpu_render_framework_with_advanced_providers,
    virtual_geometry_render_feature_descriptor,
};

mod advanced_providers;
mod history;
mod hybrid_gi_visual_export;
mod neural_compute;
mod pipeline_profiles;
mod stats;

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}

fn test_ui_extract(text: &str) -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("test.ui"),
        list: UiRenderList {
            commands: vec![UiRenderCommand {
                node_id: UiNodeId::new(1),
                kind: UiRenderCommandKind::Quad,
                frame: UiFrame::new(8.0, 8.0, 180.0, 28.0),
                clip_frame: None,
                z_index: 0,
                style: UiResolvedStyle {
                    background_color: Some("#1b2330cc".to_string()),
                    foreground_color: Some("#f5f7fb".to_string()),
                    border_color: Some("#63b0ff88".to_string()),
                    border_width: 1.0,
                    font: Some("res://fonts/default.font.toml".to_string()),
                    font_size: 14.0,
                    line_height: 18.0,
                    text_align: UiTextAlign::Center,
                    wrap: UiTextWrap::None,
                    text_render_mode: UiTextRenderMode::Auto,
                    ..UiResolvedStyle::default()
                },
                text_layout: None,
                text: Some(text.to_string()),
                image: None,
                opacity: 1.0,
            }],
        },
        raster_scale: 1.0,
    }
}

fn missing_capabilities(
    capabilities: &[RenderCapabilityKind],
) -> Vec<RenderCapabilityMismatchDetail> {
    capabilities
        .iter()
        .copied()
        .map(RenderCapabilityMismatchDetail::new)
        .collect()
}

fn capability_test_summary() -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        backend_name: "capability-test".to_string(),
        supports_offscreen: true,
        supports_fxaa: true,
        supports_storage_buffers: true,
        supports_indirect_draw: true,
        supports_buffer_readback: true,
        ..Default::default()
    }
}

fn neural_compute_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "plugin.neural_compute.activation",
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "plugin-neural-inference",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("plugin.neural.inference")
            .with_compute_workload(RenderGraphComputeWorkload::per_pixel(
                "zircon-neural-inference",
                [8, 8, 1],
                "scene-color",
                [8, 8],
            ))
            .read_texture("scene-color")
            .write_storage_external("neural-inference-output")
            .with_side_effects(),
        ],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::NeuralCompute)
}

fn neural_compute_render_pass_executor(
    _context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    Ok(())
}

fn advanced_provider_report(
    stats: &RenderStats,
    feature: AdvancedRenderFeature,
) -> &AdvancedProviderReport {
    stats
        .last_advanced_provider_reports
        .iter()
        .find(|report| report.feature == feature)
        .expect("advanced provider report should be recorded")
}

fn empty_flagship_extract() -> RenderFrameExtract {
    let world = World::new();
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(UVec2::new(320, 240));
    extract
}

fn flagship_extract() -> RenderFrameExtract {
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(UVec2::new(320, 240));
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 1,
        clusters: vec![
            virtual_geometry_cluster(mesh, 15, 150, 1, None, Vec3::new(100.0, 0.0, 0.0), 9.0),
            virtual_geometry_cluster(mesh, 30, 300, 0, None, Vec3::ZERO, 8.0),
            virtual_geometry_cluster(mesh, 20, 200, 1, None, Vec3::new(0.1, 0.0, 0.0), 5.0),
            virtual_geometry_cluster(mesh, 10, 100, 2, None, Vec3::new(0.2, 0.0, 0.0), 2.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_geometry_page(100, false),
            virtual_geometry_page(150, false),
            virtual_geometry_page(200, true),
            virtual_geometry_page(300, false),
            virtual_geometry_page(500, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        mode: Default::default(),
        profile: Default::default(),
        quality: Default::default(),
        trace_budget: 2,
        card_budget: 1,
        voxel_budget: 2,
        debug_view: Default::default(),
    });
    extract
}

fn virtual_geometry_cluster(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    lod_level: u8,
    parent_cluster_id: Option<u32>,
    bounds_center: Vec3,
    screen_space_error: f32,
) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        hierarchy_node_id: None,
        page_id,
        lod_level,
        parent_cluster_id,
        bounds_center,
        bounds_radius: 0.5,
        screen_space_error,
    }
}

fn virtual_geometry_page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: 4096,
    }
}

fn average_region_channel(
    rgba: &[u8],
    width: u32,
    height: u32,
    channel: usize,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }

    let start_x = ((width as f32) * min_x).floor() as u32;
    let end_x = ((width as f32) * max_x).ceil() as u32;
    let start_y = ((height as f32) * min_y).floor() as u32;
    let end_y = ((height as f32) * max_y).ceil() as u32;
    let mut total = 0.0f32;
    let mut count = 0.0f32;

    for y in start_y.min(height)..end_y.min(height) {
        for x in start_x.min(width)..end_x.min(width) {
            let index = ((y * width + x) as usize) * 4;
            total += rgba[index + channel] as f32;
            count += 1.0;
        }
    }

    if count <= 0.0 { 0.0 } else { total / count }
}
