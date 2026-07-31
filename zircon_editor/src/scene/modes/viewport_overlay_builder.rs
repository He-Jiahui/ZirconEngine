use zircon_runtime::core::framework::render::SceneGizmoOverlayExtract;

#[derive(Debug, Default)]
pub struct ViewportOverlayBuilder {
    scene_gizmos: Vec<SceneGizmoOverlayExtract>,
}

impl ViewportOverlayBuilder {
    pub fn push_scene_gizmo(&mut self, gizmo: SceneGizmoOverlayExtract) {
        self.scene_gizmos.push(gizmo);
    }

    pub fn extend_scene_gizmos(
        &mut self,
        gizmos: impl IntoIterator<Item = SceneGizmoOverlayExtract>,
    ) {
        self.scene_gizmos.extend(gizmos);
    }

    pub fn scene_gizmos(&self) -> &[SceneGizmoOverlayExtract] {
        &self.scene_gizmos
    }

    pub fn finish(self) -> Vec<SceneGizmoOverlayExtract> {
        self.scene_gizmos
    }

    pub(crate) fn checkpoint(&self) -> usize {
        self.scene_gizmos.len()
    }

    pub(crate) fn restore(&mut self, checkpoint: usize) {
        self.scene_gizmos.truncate(checkpoint);
    }
}
