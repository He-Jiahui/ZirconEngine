use std::sync::Arc;

use crate::core::framework::render::{
    CapturedFrame, CapturedHdrFrame, GraphicsDebuggerStatus, RenderFrameExtract, RenderFramework,
    RenderFrameworkError, RenderPipelineHandle, RenderQualityProfile, RenderStats,
    RenderSubmissionConfig, RenderViewportDescriptor, RenderViewportHandle, RenderViewportProduct,
    RenderViewportSurfaceDescriptor, RenderVirtualGeometryDebugSnapshot,
    RenderVisibleSpatialQuerySnapshot,
};
use zircon_runtime_interface::ui::surface::UiRenderExtract;
use zr_rhi::{UiSurfaceDescriptor, UiSurfacePresenter};

use super::super::capture_frame::{
    capture_frame, capture_frame_if_newer, capture_scene_color_hdr, poll_captured_frame_if_newer,
};
use super::super::create_viewport::create_viewport;
use super::super::destroy_viewport::destroy_viewport;
use super::super::graphics_debugger_capture::{
    query_graphics_debugger_status, request_graphics_debugger_capture,
};
use super::super::query_stats::query_stats;
use super::super::query_virtual_geometry_debug_snapshot::query_virtual_geometry_debug_snapshot;
use super::super::query_visible_spatial_snapshot::query_visible_spatial_snapshot;
use super::super::reload_pipeline::reload_pipeline;
use super::super::render_framework_state::WgpuViewportProductProvider;
use super::super::set_pipeline_asset::set_pipeline_asset;
use super::super::set_quality_profile::set_quality_profile;
use super::super::submit_frame_extract::{present_frame_extract, present_frame_extract_with_ui};
use super::super::submit_frame_extract::{submit_frame_extract, submit_frame_extract_with_ui};
use super::super::viewport_surface::{bind_viewport_surface, unbind_viewport_surface};
use super::super::wgpu_render_framework::WgpuRenderFramework;

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

    fn set_submission_config(
        &self,
        config: RenderSubmissionConfig,
    ) -> Result<(), RenderFrameworkError> {
        WgpuRenderFramework::set_submission_config(self, config)
    }

    fn submission_config(&self) -> RenderSubmissionConfig {
        WgpuRenderFramework::submission_config(self)
    }

    fn bind_viewport_surface(
        &self,
        viewport: RenderViewportHandle,
        descriptor: RenderViewportSurfaceDescriptor,
    ) -> Result<(), RenderFrameworkError> {
        bind_viewport_surface(self, viewport, descriptor)
    }

    fn unbind_viewport_surface(
        &self,
        viewport: RenderViewportHandle,
    ) -> Result<(), RenderFrameworkError> {
        unbind_viewport_surface(self, viewport)
    }

    fn present_frame_extract(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
    ) -> Result<(), RenderFrameworkError> {
        present_frame_extract(self, viewport, extract)
    }

    fn present_frame_extract_with_ui(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        ui: Option<UiRenderExtract>,
    ) -> Result<(), RenderFrameworkError> {
        present_frame_extract_with_ui(self, viewport, extract, ui)
    }

    fn set_pipeline_asset(
        &self,
        viewport: RenderViewportHandle,
        pipeline: RenderPipelineHandle,
    ) -> Result<(), RenderFrameworkError> {
        set_pipeline_asset(self, viewport, pipeline)
    }

    fn reload_pipeline(&self, pipeline: RenderPipelineHandle) -> Result<(), RenderFrameworkError> {
        reload_pipeline(self, pipeline)
    }

    fn query_stats(&self) -> Result<RenderStats, RenderFrameworkError> {
        query_stats(self)
    }

    fn query_visible_spatial_snapshot(
        &self,
        viewport: RenderViewportHandle,
    ) -> Result<Option<RenderVisibleSpatialQuerySnapshot>, RenderFrameworkError> {
        query_visible_spatial_snapshot(self, viewport)
    }

    fn query_virtual_geometry_debug_snapshot(
        &self,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        query_virtual_geometry_debug_snapshot(self)
    }

    fn request_graphics_debugger_capture(
        &self,
        viewport: RenderViewportHandle,
    ) -> Result<(), RenderFrameworkError> {
        request_graphics_debugger_capture(self, viewport)
    }

    fn query_graphics_debugger_status(
        &self,
    ) -> Result<GraphicsDebuggerStatus, RenderFrameworkError> {
        query_graphics_debugger_status(self)
    }

    fn capture_frame(
        &self,
        viewport: RenderViewportHandle,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        capture_frame(self, viewport)
    }

    fn capture_scene_color_hdr(
        &self,
        viewport: RenderViewportHandle,
    ) -> Result<Option<CapturedHdrFrame>, RenderFrameworkError> {
        capture_scene_color_hdr(self, viewport)
    }

    fn capture_frame_if_newer(
        &self,
        viewport: RenderViewportHandle,
        last_generation: Option<u64>,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        capture_frame_if_newer(self, viewport, last_generation)
    }

    fn poll_captured_frame_if_newer(
        &self,
        viewport: RenderViewportHandle,
        last_generation: Option<u64>,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        poll_captured_frame_if_newer(self, viewport, last_generation)
    }

    fn poll_viewport_product_if_newer(
        &self,
        viewport: RenderViewportHandle,
        last_generation: Option<u64>,
    ) -> Result<Option<RenderViewportProduct>, RenderFrameworkError> {
        let products = Arc::clone(&self.lock_state().viewport_products);
        Ok(products.poll_if_newer(viewport, last_generation))
    }

    fn create_ui_surface_presenter(
        &self,
        descriptor: UiSurfaceDescriptor,
    ) -> Result<Box<dyn UiSurfacePresenter>, RenderFrameworkError> {
        let _operation_guard = self.lock_operation();
        let state = self.lock_state();
        let context = state.renderer.ui_surface_context();
        let provider = Arc::new(WgpuViewportProductProvider::new(Arc::clone(
            &state.viewport_products,
        )));
        drop(state);
        zr_rhi_wgpu::WgpuUiSurfacePresenter::new_with_context_and_external_images(
            descriptor,
            context,
            Some(provider),
        )
        .map(|presenter| Box::new(presenter) as Box<dyn UiSurfacePresenter>)
        .map_err(|error| RenderFrameworkError::Backend(error.to_string()))
    }

    fn set_quality_profile(
        &self,
        viewport: RenderViewportHandle,
        profile: RenderQualityProfile,
    ) -> Result<(), RenderFrameworkError> {
        set_quality_profile(self, viewport, profile)
    }
}
