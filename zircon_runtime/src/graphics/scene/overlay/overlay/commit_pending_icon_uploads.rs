use super::viewport_overlay_renderer::ViewportOverlayRenderer;

impl ViewportOverlayRenderer {
    /// 只供帧提交成功后的收尾路径调用；提前确认会使失败帧丢失后续重试所需的图标上传。
    pub(crate) fn commit_pending_icon_uploads(&mut self) -> u32 {
        self.interaction_overlays
            .as_mut()
            .map(|overlays| overlays.scene_gizmo.commit_pending_icon_uploads())
            .unwrap_or(0)
    }
}
