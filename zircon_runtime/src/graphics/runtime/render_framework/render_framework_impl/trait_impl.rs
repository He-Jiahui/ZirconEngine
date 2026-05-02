use crate::core::framework::render::{
    CapturedFrame, RenderFrameExtract, RenderFramework, RenderFrameworkError, RenderPipelineHandle,
    RenderQualityProfile, RenderStats, RenderViewportDescriptor, RenderViewportHandle,
    RenderVirtualGeometryDebugSnapshot,
};
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use super::super::capture_frame::capture_frame;
use super::super::create_viewport::create_viewport;
use super::super::destroy_viewport::destroy_viewport;
use super::super::query_stats::query_stats;
use super::super::query_virtual_geometry_debug_snapshot::query_virtual_geometry_debug_snapshot;
use super::super::register_pipeline_asset::register_pipeline_asset;
use super::super::reload_pipeline::reload_pipeline;
use super::super::set_pipeline_asset::set_pipeline_asset;
use super::super::set_quality_profile::set_quality_profile;
use super::super::submit_frame_extract::{submit_frame_extract, submit_frame_extract_with_ui};
use super::super::wgpu_render_framework::WgpuRenderFramework;
use crate::RenderPipelineAsset;

impl RenderFramework for WgpuRenderFramework {
    fn create_viewport(
        &self,
        descriptor: RenderViewportDescriptor,
    ) -> Result<RenderViewportHandle, RenderFrameworkError> {
        create_viewport(self, descriptor)
    }

    fn destroy_viewport(&self, viewport: RenderViewportHandle) -> Result<(), RenderFrameworkError> {
        destroy_viewport(self, viewport)
    }

    fn submit_frame_extract(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
    ) -> Result<(), RenderFrameworkError> {
        submit_frame_extract(self, viewport, extract)
    }

    fn submit_frame_extract_with_ui(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        ui: Option<UiRenderExtract>,
    ) -> Result<(), RenderFrameworkError> {
        submit_frame_extract_with_ui(self, viewport, extract, ui)
    }

    fn set_pipeline_asset(
        &self,
        viewport: RenderViewportHandle,
        pipeline: RenderPipelineHandle,
    ) -> Result<(), RenderFrameworkError> {
        set_pipeline_asset(self, viewport, pipeline)
    }

    fn register_pipeline_asset(
        &self,
        pipeline: RenderPipelineAsset,
    ) -> Result<RenderPipelineHandle, RenderFrameworkError> {
        register_pipeline_asset(self, pipeline)
    }

    fn reload_pipeline(&self, pipeline: RenderPipelineHandle) -> Result<(), RenderFrameworkError> {
        reload_pipeline(self, pipeline)
    }

    fn query_stats(&self) -> Result<RenderStats, RenderFrameworkError> {
        query_stats(self)
    }

    fn query_virtual_geometry_debug_snapshot(
        &self,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        query_virtual_geometry_debug_snapshot(self)
    }

    fn capture_frame(
        &self,
        viewport: RenderViewportHandle,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        capture_frame(self, viewport)
    }

    fn set_quality_profile(
        &self,
        viewport: RenderViewportHandle,
        profile: RenderQualityProfile,
    ) -> Result<(), RenderFrameworkError> {
        set_quality_profile(self, viewport, profile)
    }
}
