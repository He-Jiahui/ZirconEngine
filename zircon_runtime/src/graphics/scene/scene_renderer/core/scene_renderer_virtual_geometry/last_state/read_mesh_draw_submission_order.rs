use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_mesh_draw_submission_order_with_instances(
        &self,
    ) -> Vec<(Option<u32>, u64, u32)> {
        self.read_last_virtual_geometry_mesh_draw_submission_records_with_instances()
            .map(|records| {
                records
                    .into_iter()
                    .map(
                        |(instance_index, entity, page_id, _submission_index, _draw_ref_rank)| {
                            (instance_index, entity, page_id)
                        },
                    )
                    .collect()
            })
            .unwrap_or_else(|_| {
                self.advanced_plugin_outputs
                    .virtual_geometry_mesh_draw_submission_order
                    .clone()
            })
    }
}
