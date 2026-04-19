use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_mesh_draw_submission_order(&self) -> Vec<(u64, u32)> {
        self.read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()
            .map(|records| {
                records
                    .into_iter()
                    .map(|(entity, page_id, _submission_index, _draw_ref_rank)| (entity, page_id))
                    .collect()
            })
            .unwrap_or_else(|_| {
                self.last_virtual_geometry_mesh_draw_submission_order
                    .clone()
            })
    }
}
