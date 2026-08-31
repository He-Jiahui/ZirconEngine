use super::viewport_overlay_renderer::ViewportOverlayRenderer;

impl ViewportOverlayRenderer {
    pub(crate) fn commit_pending_icon_uploads(&mut self) -> u32 {
        self.interaction_overlays
            .as_mut()
            .map(|overlays| overlays.scene_gizmo.commit_pending_icon_uploads())
            .unwrap_or(0)
    }
}
