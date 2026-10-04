use super::scene_gizmo_pass::SceneGizmoPass;

impl SceneGizmoPass {
    /// 由帧成功收尾路径确认图标上传债务；帧失败时保留准备态以便下帧重放。
    pub(crate) fn commit_pending_icon_uploads(&mut self) -> u32 {
        self.icon_atlas.commit_pending_uploads()
    }
}
