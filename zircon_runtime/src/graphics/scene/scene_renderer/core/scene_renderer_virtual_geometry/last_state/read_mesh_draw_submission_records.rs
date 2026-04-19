use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_mesh_draw_submission_records_with_tokens(
        &self,
    ) -> Result<Vec<(u64, u32, u32, u32)>, crate::graphics::types::GraphicsError> {
        if !self
            .last_virtual_geometry_mesh_draw_submission_token_records
            .is_empty()
        {
            return Ok(self
                .last_virtual_geometry_mesh_draw_submission_token_records
                .clone()
                .into_iter()
                .map(
                    |(entity, page_id, submission_index, draw_ref_rank, _original_index)| {
                        (entity, page_id, submission_index, draw_ref_rank)
                    },
                )
                .collect());
        }

        let execution_records = self.read_last_virtual_geometry_indirect_execution_records()?;
        if !execution_records.is_empty() {
            return Ok(execution_records
                .into_iter()
                .map(
                    |(_draw_ref_index, entity, page_id, submission_index, draw_ref_rank)| {
                        (entity, page_id, submission_index, draw_ref_rank)
                    },
                )
                .collect());
        }

        const INDIRECT_ARGS_STRIDE_BYTES: u64 = (std::mem::size_of::<u32>() as u64) * 5;

        let submission_tokens = self.read_last_virtual_geometry_indirect_submission_tokens()?;
        let indirect_args_with_instances = if submission_tokens.is_empty() {
            self.read_last_virtual_geometry_indirect_args_with_instances()?
        } else {
            Vec::new()
        };
        if submission_tokens.is_empty() && indirect_args_with_instances.is_empty() {
            return Ok(Vec::new());
        }

        let mesh_draw_submission_records = if self
            .last_virtual_geometry_mesh_draw_submission_records
            .is_empty()
        {
            let indirect_segments =
                self.read_last_virtual_geometry_indirect_segments_with_entities()?;
            let indirect_draw_refs = self.read_last_virtual_geometry_indirect_draw_refs()?;
            let submitted_draw_ref_indices =
                self.read_last_virtual_geometry_indirect_execution_draw_ref_indices()?;
            if submitted_draw_ref_indices.is_empty() {
                indirect_draw_refs
                    .into_iter()
                    .enumerate()
                    .filter_map(|(draw_ref_index, (_mesh_index_count, segment_index))| {
                        let segment = indirect_segments.get(segment_index as usize).copied()?;
                        Some((
                            segment.0,
                            segment.4,
                            (draw_ref_index as u64) * INDIRECT_ARGS_STRIDE_BYTES,
                            draw_ref_index,
                        ))
                    })
                    .collect::<Vec<_>>()
            } else {
                submitted_draw_ref_indices
                    .into_iter()
                    .enumerate()
                    .filter_map(|(draw_index, draw_ref_index)| {
                        let (_mesh_index_count, segment_index) =
                            indirect_draw_refs.get(draw_ref_index as usize).copied()?;
                        let segment = indirect_segments.get(segment_index as usize).copied()?;
                        Some((
                            segment.0,
                            segment.4,
                            (draw_ref_index as u64) * INDIRECT_ARGS_STRIDE_BYTES,
                            draw_index,
                        ))
                    })
                    .collect::<Vec<_>>()
            }
        } else {
            self.last_virtual_geometry_mesh_draw_submission_records
                .clone()
        };

        Ok(mesh_draw_submission_records
            .into_iter()
            .map(|(entity, page_id, indirect_args_offset, original_index)| {
                let draw_ref_index = (indirect_args_offset / INDIRECT_ARGS_STRIDE_BYTES) as usize;
                let submission_token = submission_tokens
                    .get(draw_ref_index)
                    .copied()
                    .or_else(|| {
                        indirect_args_with_instances
                            .get(draw_ref_index)
                            .map(|(_first_index, _index_count, first_instance)| *first_instance)
                    })
                    .unwrap_or(u32::MAX);
                (
                    submission_token,
                    indirect_args_offset,
                    original_index,
                    entity,
                    page_id,
                )
            })
            .map(
                |(submission_token, _indirect_args_offset, _original_index, entity, page_id)| {
                    (
                        entity,
                        page_id,
                        submission_token >> 16,
                        submission_token & 0xffff,
                    )
                },
            )
            .collect())
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_submission_buffer_for_test(&mut self) {
        self.last_virtual_geometry_indirect_submission_buffer = None;
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_mesh_draw_submission_token_records_for_test(
        &mut self,
    ) {
        self.last_virtual_geometry_mesh_draw_submission_token_records
            .clear();
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_mesh_draw_submission_records_for_test(&mut self) {
        self.last_virtual_geometry_mesh_draw_submission_records
            .clear();
        self.last_virtual_geometry_mesh_draw_submission_order
            .clear();
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_args_buffer_for_test(&mut self) {
        self.last_virtual_geometry_indirect_args_buffer = None;
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_draw_refs_buffer_for_test(&mut self) {
        self.last_virtual_geometry_indirect_draw_refs_buffer = None;
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_segments_buffer_for_test(&mut self) {
        self.last_virtual_geometry_indirect_segments_buffer = None;
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_execution_buffer_for_test(
        &mut self,
    ) {
        self.last_virtual_geometry_indirect_execution_buffer = None;
    }
}
