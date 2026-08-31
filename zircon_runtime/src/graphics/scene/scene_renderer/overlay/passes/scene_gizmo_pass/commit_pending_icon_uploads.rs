use super::scene_gizmo_pass::SceneGizmoPass;

impl SceneGizmoPass {
    pub(crate) fn commit_pending_icon_uploads(&mut self) -> u32 {
        self.icon_atlas.commit_pending_uploads()
    }
}
