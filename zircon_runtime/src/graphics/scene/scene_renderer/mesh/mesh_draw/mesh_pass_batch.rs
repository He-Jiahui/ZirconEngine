use std::sync::Arc;

use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshBatchRef, MeshBindHandle, MeshDrawArgs, MeshGeometryHandle,
};

use super::MeshDraw;

impl MeshDraw {
    pub(crate) fn mesh_pass_batch_ref(
        &self,
        _sort_key: u64,
        source_draw_index: usize,
    ) -> MeshBatchRef {
        let batch = MeshBatchRef::new(
            self.queue_profile(),
            self.pipeline_key.clone(),
            self.command_sort_input.components(),
            MeshGeometryHandle::new(arc_id(&self.mesh), self.mesh.clone()),
            self.mesh_draw_args(),
        )
        .with_source_draw_index(source_draw_index)
        .with_cache_identity(
            self.source_entity(),
            self.stable_instance_key(),
            self.source_draw_ordinal(),
        )
        .with_static_state(self.static_state())
        .with_casts_shadow(self.casts_shadow())
        .with_disabled_passes(self.disabled_passes)
        .with_taa_reactive_mask_strength(self.taa_reactive_mask_strength)
        .with_visibility(
            self.primitive_relevance,
            self.main_view_visible,
            self.shadow_view_visible,
        )
        .with_previous_velocity_transform(self.has_previous_velocity_transform)
        .with_material(MeshBindHandle::new(
            ref_id(&self.material_bind_group),
            self.material_bind_group.clone(),
        ))
        .with_standard_material(MeshBindHandle::new(
            ref_id(&self.standard_material_bind_group),
            self.standard_material_bind_group.clone(),
        ));

        let batch = if let Some(gpu_scene_bind_group) = &self.gpu_scene_bind_group {
            batch.with_gpu_scene_bind_group(MeshBindHandle::new(
                ref_id(gpu_scene_bind_group),
                gpu_scene_bind_group.clone(),
            ))
        } else {
            batch
        };

        if let Some((first_instance_index, instance_count)) = self.gpu_scene_instance_span {
            let batch = batch.with_gpu_scene_instance_span(first_instance_index, instance_count);
            if let Some(previous_source) = &self.previous_skinned_gpu_source {
                batch.with_previous_velocity_geometry(MeshGeometryHandle::new(
                    arc_id(previous_source),
                    previous_source.clone(),
                ))
            } else {
                batch
            }
        } else {
            batch
        }
    }

    fn mesh_draw_args(&self) -> MeshDrawArgs {
        if let Some(indirect_args_buffer) = &self.indirect_args_buffer {
            MeshDrawArgs::indexed_indirect(
                arc_id(indirect_args_buffer),
                indirect_args_buffer.clone(),
                self.indirect_args_offset,
            )
        } else {
            MeshDrawArgs::direct_indexed(self.first_index, self.draw_index_count)
        }
    }
}

fn ref_id<T>(value: &T) -> u64 {
    value as *const T as usize as u64
}

fn arc_id<T>(value: &Arc<T>) -> u64 {
    Arc::as_ptr(value) as usize as u64
}
