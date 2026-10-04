use std::sync::Arc;

use crate::core::framework::render::{RenderFrameExtract, UiRenderSubmission};
use crate::core::math::UVec2;

use super::extract_cache::RuntimeFrameExtractCacheStatus;
use super::extract_stats::record_frame_extract_stats;
use super::{RuntimeDynamicSession, RuntimeDynamicSessionError, RuntimeDynamicSessionResult};

impl RuntimeDynamicSession {
    pub(super) fn current_extract(&mut self) -> RenderFrameExtract {
        crate::profile_scope!("runtime", "frame", "runtime_frame_extract");
        let viewport_size = self.camera_controller.viewport_size();
        // 场景快照命中缓存后再叠加 session 专属相机与帧计时，避免它们污染场景缓存键。
        let mut cached = self
            .extract_cache
            .current_extract(&self.level, viewport_size);
        record_frame_extract_stats(&self.runtime, cached.diagnostics_summary, cached.status);
        self.camera_controller
            .apply_editor_camera_to_extract(&mut cached.extract);
        cached.extract.set_timing(self.last_render_frame_timing);
        cached.extract
    }

    pub(super) fn current_ui_submission(
        &mut self,
    ) -> RuntimeDynamicSessionResult<Option<Arc<UiRenderSubmission>>> {
        let viewport_size = self.camera_controller.viewport_size();
        if let Some(project_ui) = self
            .runtime_ui
            .render_submission(viewport_size)
            .map_err(|source| RuntimeDynamicSessionError::RuntimeUiLayout { source })?
        {
            return Ok(Some(project_ui));
        }
        let level = &self.level;
        let ui_extract_cache = &mut self.ui_extract_cache;
        Ok(level.with_world(|world| {
            ui_extract_cache
                .current_extract(world, viewport_size)
                .map(UiRenderSubmission::single)
        }))
    }

    pub(super) fn resize_viewport(&mut self, size: UVec2) {
        let previous = self.camera_controller.viewport_size();
        self.camera_controller.resize(size);
        if self.camera_controller.viewport_size() != previous {
            self.extract_cache.invalidate();
        }
    }
}

#[cfg(test)]
#[path = "tests/extract.rs"]
mod tests;
