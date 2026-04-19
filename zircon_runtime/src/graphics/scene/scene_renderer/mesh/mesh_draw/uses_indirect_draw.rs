use super::MeshDraw;

impl MeshDraw {
    pub(crate) fn uses_indirect_draw(&self) -> bool {
        self.indirect_args_buffer.is_some()
    }
}
