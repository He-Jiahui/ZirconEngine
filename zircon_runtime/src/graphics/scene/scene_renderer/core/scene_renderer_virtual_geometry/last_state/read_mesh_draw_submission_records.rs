use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_mesh_draw_submission_records_with_instances(
        &self,
    ) -> Result<Vec<(Option<u32>, u64, u32, u32, u32)>, crate::graphics::types::GraphicsError> {
        if !self
            .advanced_plugin_outputs
            .virtual_geometry_mesh_draw_submission_token_records()
            .is_empty()
        {
            return Ok(self
                .advanced_plugin_outputs
                .virtual_geometry_mesh_draw_submission_token_records()
                .clone()
                .into_iter()
                .enumerate()
                .map(
                    |(
                        record_index,
                        (entity, page_id, submission_index, draw_ref_rank, _original_index),
                    )| {
                        (
                            self.advanced_plugin_outputs
                                .virtual_geometry_mesh_draw_submission_order()
                                .get(record_index)
                                .map(|(instance_index, _entity, _page_id)| *instance_index)
                                .flatten(),
                            entity,
                            page_id,
                            submission_index,
                            draw_ref_rank,
                        )
                    },
                )
                .collect());
        }

        let mesh_draw_submission_records = if self
            .advanced_plugin_outputs
            .virtual_geometry_mesh_draw_submission_records()
            .is_empty()
        {
            Vec::new()
        } else {
            self.advanced_plugin_outputs
                .virtual_geometry_mesh_draw_submission_records()
                .clone()
                .into_iter()
                .enumerate()
                .map(
                    |(record_index, (entity, page_id, draw_ref_index, original_index))| {
                        (
                            self.advanced_plugin_outputs
                                .virtual_geometry_mesh_draw_submission_order()
                                .get(record_index)
                                .map(|(instance_index, _entity, _page_id)| *instance_index)
                                .flatten(),
                            entity,
                            page_id,
                            draw_ref_index,
                            original_index as u32,
                        )
                    },
                )
                .collect::<Vec<_>>()
        };

        Ok(mesh_draw_submission_records)
    }

    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_mesh_draw_submission_records_with_tokens(
        &self,
    ) -> Result<Vec<(u64, u32, u32, u32)>, crate::graphics::types::GraphicsError> {
        self.read_last_virtual_geometry_mesh_draw_submission_records_with_instances()
            .map(|records| {
                records
                    .into_iter()
                    .map(
                        |(_instance_index, entity, page_id, submission_index, draw_ref_rank)| {
                            (entity, page_id, submission_index, draw_ref_rank)
                        },
                    )
                    .collect()
            })
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_submission_buffer_for_test(&mut self) {
        self.advanced_plugin_outputs
            .clear_virtual_geometry_indirect_submission_buffer();
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_authority_buffer_for_test(&mut self) {
        self.advanced_plugin_outputs
            .clear_virtual_geometry_indirect_authority_buffer();
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_mesh_draw_submission_token_records_for_test(
        &mut self,
    ) {
        self.advanced_plugin_outputs
            .clear_virtual_geometry_mesh_draw_submission_token_records();
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_mesh_draw_submission_records_for_test(&mut self) {
        self.advanced_plugin_outputs
            .clear_virtual_geometry_mesh_draw_submission_records();
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_args_buffer_for_test(&mut self) {
        self.advanced_plugin_outputs
            .clear_virtual_geometry_indirect_args_buffer();
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_draw_refs_buffer_for_test(&mut self) {
        self.advanced_plugin_outputs
            .clear_virtual_geometry_indirect_draw_refs_buffer();
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_segments_buffer_for_test(&mut self) {
        self.advanced_plugin_outputs
            .clear_virtual_geometry_indirect_segments_buffer();
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_execution_buffer_for_test(&mut self) {}

    #[cfg(test)]
    pub(crate) fn has_last_virtual_geometry_indirect_execution_buffer_for_test(&self) -> bool {
        false
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_execution_records_buffer_for_test(&mut self) {
    }

    #[cfg(test)]
    pub(crate) fn has_last_virtual_geometry_indirect_execution_records_buffer_for_test(
        &self,
    ) -> bool {
        false
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_execution_submission_buffer_for_test(
        &mut self,
    ) {
        self.advanced_plugin_outputs
            .clear_virtual_geometry_indirect_execution_submission_buffer();
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_execution_args_buffer_for_test(&mut self) {
        self.advanced_plugin_outputs
            .clear_virtual_geometry_indirect_execution_args_buffer();
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_execution_authority_buffer_for_test(
        &mut self,
    ) {
        self.advanced_plugin_outputs
            .clear_virtual_geometry_indirect_execution_authority_buffer();
    }
}
