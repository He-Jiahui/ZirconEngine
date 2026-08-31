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
mod tests {
    #[test]
    fn render_timing_overlays_cached_scene_content_without_entering_cache_identity() {
        let session_extract = include_str!("extract.rs");
        let extract_cache = include_str!("extract_cache.rs");
        let cache_lookup = session_extract
            .find(".current_extract(&self.level, viewport_size)")
            .expect("session must resolve the scene extract through its cache");
        let timing_overlay = session_extract
            .find("cached.extract.set_timing(self.last_render_frame_timing)")
            .expect("session must overlay authoritative timing after cache lookup");

        assert!(cache_lookup < timing_overlay);
        assert!(!extract_cache.contains("RenderFrameTiming"));
        assert!(!extract_cache.contains("last_render_frame_timing"));
    }
}
