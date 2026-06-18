use crate::core::framework::render::RenderFrameExtract;
use crate::core::math::UVec2;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use super::extract_cache::RuntimeFrameExtractCacheStatus;
use super::extract_stats::record_frame_extract_stats;
use super::hud::runtime_session_hud_extract;
use super::menu::runtime_session_menu_extract;
use super::RuntimeDynamicSession;

impl RuntimeDynamicSession {
    pub(super) fn current_extract(&mut self) -> RenderFrameExtract {
        crate::profile_scope!("runtime", "frame", "runtime_frame_extract");
        let viewport_size = self.camera_controller.viewport_size();
        let (extract, status) = self
            .extract_cache
            .current_extract(&self.level, viewport_size);
        record_frame_extract_stats(&self.runtime, &extract, status);
        extract
    }

    pub(super) fn current_ui_extract(&self) -> Option<UiRenderExtract> {
        let viewport_size = self.camera_controller.viewport_size();
        self.level.with_world(|world| {
            runtime_session_menu_extract(world, viewport_size)
                .or_else(|| runtime_session_hud_extract(world, viewport_size))
        })
    }

    pub(super) fn resize_viewport(&mut self, size: UVec2) {
        let previous = self.camera_controller.viewport_size();
        self.camera_controller.resize(size);
        if self.camera_controller.viewport_size() != previous {
            self.extract_cache.invalidate();
        }
    }
}
