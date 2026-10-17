#[cfg(feature = "test-support")]
use crate::core::framework::render::{
    RenderFrameExtract, RenderFramework, RenderViewportHandle, RenderVirtualGeometryReadbackOutputs,
};
use crate::core::framework::render::{RenderFrameworkError, RenderVirtualGeometryDebugSnapshot};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn query_virtual_geometry_debug_snapshot(
    framework: &WgpuRenderFramework,
) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
    let state = framework.lock_state();
    let snapshot = state.last_virtual_geometry_debug_snapshot.clone();
    drop(state);
    Ok(snapshot.as_deref().cloned())
}

pub(in crate::graphics::runtime::render_framework) fn query_virtual_geometry_debug_snapshot_available(
    framework: &WgpuRenderFramework,
) -> Result<bool, RenderFrameworkError> {
    let state = framework.lock_state();
    Ok(state.last_virtual_geometry_debug_snapshot.is_some())
}

#[cfg(feature = "test-support")]
impl WgpuRenderFramework {
    pub fn test_support_virtual_geometry_native_probe_scope(
        &self,
        viewport: RenderViewportHandle,
    ) -> Result<Option<u64>, RenderFrameworkError> {
        let _operation = self.lock_operation();
        let state = self.lock_state();
        let record =
            state
                .viewports
                .get(&viewport)
                .ok_or(RenderFrameworkError::UnknownViewport {
                    viewport: viewport.raw(),
                })?;
        Ok(record.native_probe_scope_id(state.renderer.virtual_geometry_device_epoch()))
    }
    pub fn test_support_virtual_geometry_native_probe_source(
        &self,
        viewport: RenderViewportHandle,
        scope: u64,
        role: u32,
        drop_source: bool,
    ) -> Result<bool, String> {
        let _operation = self.lock_operation();
        let state = self.lock_state();
        let record = state
            .viewports
            .get(&viewport)
            .ok_or("native probe viewport destroyed")?;
        record
            .native_probe_handle(scope, state.renderer.virtual_geometry_device_epoch())?
            .native_probe_source(role, drop_source)
    }
    pub fn test_support_read_virtual_geometry_native_probe(
        &self,
        viewport: RenderViewportHandle,
        scope: u64,
        kind: u32,
    ) -> Result<Vec<u8>, String> {
        let _operation = self.lock_operation();
        let state = self.lock_state();
        let record = state
            .viewports
            .get(&viewport)
            .ok_or("native probe viewport destroyed")?;
        let handle =
            record.native_probe_handle(scope, state.renderer.virtual_geometry_device_epoch())?;
        state
            .renderer
            .read_virtual_geometry_native_probe(&handle, kind)
    }
    /// Submits one ordinary product frame through the public runtime framework contract, waits
    /// for the real submission to complete, and returns its renderer-produced debug snapshot.
    /// This is available only to tests and diagnostic support builds.
    pub fn test_support_submit_virtual_geometry_frame(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        RenderFramework::submit_frame_extract(self, viewport, extract)?;
        self.finish_submission()?;
        RenderFramework::query_virtual_geometry_debug_snapshot(self)
    }

    /// Submits one frame through the normal runtime collector/provider/feedback path while using
    /// the exact compiled graph supplied by the test. The override is one-shot and is removed
    /// even when preflight or submission fails.
    pub fn test_support_submit_compiled_virtual_geometry_frame(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        pipeline: &crate::graphics::CompiledRenderPipeline,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        self.test_support_submit_compiled_virtual_geometry_frame_with_inputs(
            viewport,
            extract,
            pipeline,
            Default::default(),
        )
    }

    pub fn test_support_submit_compiled_virtual_geometry_frame_with_inputs(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        pipeline: &crate::graphics::CompiledRenderPipeline,
        inputs: crate::core::framework::render::RenderVirtualGeometryExecutionInputs,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        self.finish_submission()?;
        let viewport_generation = self
            .lock_state()
            .viewports
            .get(&viewport)
            .ok_or(RenderFrameworkError::UnknownViewport {
                viewport: viewport.raw(),
            })?
            .generation();
        self.core.arm_test_virtual_geometry_override(
            viewport,
            viewport_generation,
            pipeline,
            inputs,
        )?;
        let submitted = RenderFramework::submit_frame_extract(self, viewport, extract);
        let finished = self.finish_submission();
        let _ = self.core.take_test_compiled_pipeline_override();
        submitted?;
        finished?;
        self.test_support_finish_virtual_geometry_readbacks(viewport)?;
        RenderFramework::query_virtual_geometry_debug_snapshot(self)
    }

    /// Waits only in explicit test support after a successful product submission.
    fn test_support_finish_virtual_geometry_readbacks(
        &self,
        viewport: RenderViewportHandle,
    ) -> Result<(), RenderFrameworkError> {
        let _operation = self.lock_operation();
        let deadline = web_time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            let mut state = self.lock_state();
            state
                .renderer
                .finish_virtual_geometry_native_readbacks_for_test()
                .map_err(
                    super::super::render_framework_backend_error::render_framework_backend_error,
                )?;
            let super::super::render_framework_state::RenderFrameworkState {
                renderer,
                viewports,
                ..
            } = &mut *state;
            let Some(record) = viewports.get_mut(&viewport) else {
                return Err(RenderFrameworkError::UnknownViewport {
                    viewport: viewport.raw(),
                });
            };
            record.observe_virtual_geometry_readbacks(renderer);
            record.apply_ready_virtual_geometry_readbacks();
            if record.virtual_geometry_pending_readback_count() == 0 {
                return Ok(());
            }
            if web_time::Instant::now() >= deadline {
                return Err(RenderFrameworkError::UnsupportedCapability {
                    capability:
                        "virtual geometry native readback did not complete before test deadline"
                            .to_string(),
                });
            }
            drop(state);
            std::thread::yield_now();
        }
    }

    /// Returns the actual renderer readback payload captured immediately before the runtime
    /// provider consumes it. Available only to tests and diagnostic support builds.
    pub fn test_support_last_virtual_geometry_readback_outputs(
        &self,
    ) -> Result<Option<RenderVirtualGeometryReadbackOutputs>, RenderFrameworkError> {
        let state = self.lock_state();
        Ok(state
            .renderer
            .test_support_last_virtual_geometry_readback_outputs())
    }

    pub fn test_support_last_virtual_geometry_gpu_completion(
        &self,
    ) -> Result<Option<crate::graphics::VirtualGeometryGpuCompletion>, RenderFrameworkError> {
        let state = self.lock_state();
        Ok(state
            .renderer
            .test_support_last_virtual_geometry_gpu_completion())
    }
}

#[cfg(test)]
#[path = "../../tests/query_virtual_geometry_debug_snapshot_snapshot.rs"]
mod tests;
