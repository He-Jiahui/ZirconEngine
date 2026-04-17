use super::MeshDraw;

impl MeshDraw {
    pub(crate) fn is_transparent(&self) -> bool {
        self.pipeline_key.alpha_blend
    }
}
