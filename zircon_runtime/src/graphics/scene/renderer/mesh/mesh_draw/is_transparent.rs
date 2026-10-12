use super::MeshDraw;

impl MeshDraw {
    pub(crate) fn is_transparent(&self) -> bool {
        self.pipeline_key.is_transparent()
    }

    pub(crate) fn is_alpha_mask(&self) -> bool {
        self.pipeline_key.is_alpha_mask()
    }
}
