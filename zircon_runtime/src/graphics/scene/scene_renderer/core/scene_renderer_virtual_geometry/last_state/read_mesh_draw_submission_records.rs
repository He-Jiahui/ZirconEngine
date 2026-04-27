#[cfg(test)]
use std::collections::HashMap;

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_mesh_draw_submission_records_with_instances(
        &self,
    ) -> Result<Vec<(Option<u32>, u64, u32, u32, u32)>, crate::graphics::types::GraphicsError> {
        if !self
            .advanced_plugin_outputs
            .virtual_geometry_mesh_draw_submission_token_records
            .is_empty()
        {
            return Ok(self
                .advanced_plugin_outputs
                .virtual_geometry_mesh_draw_submission_token_records
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
                                .virtual_geometry_mesh_draw_submission_order
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

        let execution_segments =
            self.read_last_virtual_geometry_indirect_execution_segments_with_entities()?;
        let execution_records = self.read_last_virtual_geometry_indirect_execution_records()?;
        if !execution_records.is_empty() {
            return Ok(execution_records
                .into_iter()
                .enumerate()
                .map(
                    |(
                        record_index,
                        (_draw_ref_index, entity, page_id, submission_index, draw_ref_rank),
                    )| {
                        (
                            execution_segments
                                .get(record_index)
                                .and_then(|segment| segment.instance_index),
                            entity,
                            page_id,
                            submission_index,
                            draw_ref_rank,
                        )
                    },
                )
                .collect());
        }

        let submission_tokens = self.read_last_virtual_geometry_indirect_submission_tokens()?;
        let indirect_args_with_instances = if submission_tokens.is_empty() {
            self.read_last_virtual_geometry_indirect_args_with_instances()?
        } else {
            Vec::new()
        };
        let authority_records = self.read_last_virtual_geometry_indirect_authority_records()?;
        let authority_by_draw_ref_index = authority_records
            .into_iter()
            .map(|record| {
                (
                    record.draw_ref_index as usize,
                    (
                        record.instance_index,
                        record.entity,
                        record.page_id,
                        (record.submission_index << 16) | record.draw_ref_rank,
                    ),
                )
            })
            .collect::<HashMap<_, _>>();
        if submission_tokens.is_empty()
            && indirect_args_with_instances.is_empty()
            && authority_by_draw_ref_index.is_empty()
        {
            return Ok(Vec::new());
        }

        let mesh_draw_submission_records = if self
            .advanced_plugin_outputs
            .virtual_geometry_mesh_draw_submission_records
            .is_empty()
        {
            let indirect_draw_refs = self.read_last_virtual_geometry_indirect_draw_refs()?;
            let submitted_draw_ref_indices =
                self.read_last_virtual_geometry_indirect_execution_draw_ref_indices()?;
            if !submitted_draw_ref_indices.is_empty() && !authority_by_draw_ref_index.is_empty() {
                submitted_draw_ref_indices
                    .into_iter()
                    .enumerate()
                    .filter_map(|(draw_index, draw_ref_index)| {
                        let (instance_index, entity, page_id, _submission_token) =
                            authority_by_draw_ref_index
                                .get(&(draw_ref_index as usize))
                                .copied()?;
                        Some((instance_index, entity, page_id, draw_ref_index, draw_index))
                    })
                    .collect::<Vec<_>>()
            } else {
                let indirect_segments =
                    self.read_last_virtual_geometry_indirect_segments_with_instances()?;
                if submitted_draw_ref_indices.is_empty() {
                    indirect_draw_refs
                        .into_iter()
                        .enumerate()
                        .filter_map(|(draw_ref_index, (_mesh_index_count, segment_index))| {
                            let segment = indirect_segments.get(segment_index as usize).copied()?;
                            Some((
                                segment.0,
                                segment.1,
                                segment.5,
                                draw_ref_index as u32,
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
                                execution_segments
                                    .get(draw_index)
                                    .and_then(|execution_segment| execution_segment.instance_index)
                                    .or(segment.0),
                                segment.1,
                                segment.5,
                                draw_ref_index,
                                draw_index,
                            ))
                        })
                        .collect::<Vec<_>>()
                }
            }
        } else {
            self.advanced_plugin_outputs
                .virtual_geometry_mesh_draw_submission_records
                .clone()
                .into_iter()
                .enumerate()
                .map(
                    |(record_index, (entity, page_id, draw_ref_index, original_index))| {
                        (
                            self.advanced_plugin_outputs
                                .virtual_geometry_mesh_draw_submission_order
                                .get(record_index)
                                .map(|(instance_index, _entity, _page_id)| *instance_index)
                                .flatten(),
                            entity,
                            page_id,
                            draw_ref_index,
                            original_index,
                        )
                    },
                )
                .collect::<Vec<_>>()
        };

        Ok(mesh_draw_submission_records
            .into_iter()
            .map(
                |(instance_index, entity, page_id, draw_ref_index, _original_index)| {
                    let draw_ref_index = draw_ref_index as usize;
                    let submission_token = submission_tokens
                        .get(draw_ref_index)
                        .copied()
                        .or_else(|| {
                            indirect_args_with_instances
                                .get(draw_ref_index)
                                .map(|(_first_index, _index_count, first_instance)| *first_instance)
                        })
                        .or_else(|| {
                            authority_by_draw_ref_index.get(&draw_ref_index).map(
                                |(_instance_index, _entity, _page_id, submission_token)| {
                                    *submission_token
                                },
                            )
                        })
                        .unwrap_or(u32::MAX);
                    (
                        instance_index,
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
            .virtual_geometry_indirect_submission_buffer = None;
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_authority_buffer_for_test(&mut self) {
        self.advanced_plugin_outputs
            .virtual_geometry_indirect_authority_buffer = None;
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_mesh_draw_submission_token_records_for_test(
        &mut self,
    ) {
        self.advanced_plugin_outputs
            .virtual_geometry_mesh_draw_submission_token_records
            .clear();
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_mesh_draw_submission_records_for_test(&mut self) {
        self.advanced_plugin_outputs
            .virtual_geometry_mesh_draw_submission_records
            .clear();
        self.advanced_plugin_outputs
            .virtual_geometry_mesh_draw_submission_order
            .clear();
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_args_buffer_for_test(&mut self) {
        self.advanced_plugin_outputs
            .virtual_geometry_indirect_args_buffer = None;
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_draw_refs_buffer_for_test(&mut self) {
        self.advanced_plugin_outputs
            .virtual_geometry_indirect_draw_refs_buffer = None;
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_segments_buffer_for_test(&mut self) {
        self.advanced_plugin_outputs
            .virtual_geometry_indirect_segments_buffer = None;
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
            .virtual_geometry_indirect_execution_submission_buffer = None;
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_execution_args_buffer_for_test(&mut self) {
        self.advanced_plugin_outputs
            .virtual_geometry_indirect_execution_args_buffer = None;
    }

    #[cfg(test)]
    pub(crate) fn drop_last_virtual_geometry_indirect_execution_authority_buffer_for_test(
        &mut self,
    ) {
        self.advanced_plugin_outputs
            .virtual_geometry_indirect_execution_authority_buffer = None;
    }
}
