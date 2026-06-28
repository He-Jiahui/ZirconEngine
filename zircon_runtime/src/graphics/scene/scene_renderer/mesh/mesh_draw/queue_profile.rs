use std::sync::Arc;

use crate::core::framework::render::{GeometrySourceId, GEOMETRY_SOURCE_ID_SKINNED_MESH};
use crate::core::framework::scene::Mobility;
use crate::graphics::scene::resources::PipelineKey;

use super::{MeshDraw, MeshDrawGeometrySource};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MeshDrawQueuePhase {
    Opaque,
    AlphaMask,
    Transparent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MeshDrawQueueProfile {
    phase: MeshDrawQueuePhase,
    geometry_source: MeshDrawGeometrySource,
    mobility: Mobility,
    uses_indirect_draw: bool,
    uses_skinned_gpu_skinning: bool,
    uses_mesh_lod: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MeshDrawBatchKey {
    geometry_source: MeshDrawGeometrySource,
    mesh: usize,
    base_color_texture: usize,
    normal_texture: usize,
    metallic_roughness_texture: usize,
    occlusion_texture: usize,
    emissive_texture: usize,
    material_uniform: usize,
    standard_material_uniform: usize,
    pipeline_key: PipelineKey,
    first_index: u32,
    draw_index_count: u32,
}

impl MeshDrawQueuePhase {
    pub(crate) fn from_pipeline_flags(is_transparent: bool, is_alpha_mask: bool) -> Self {
        if is_transparent {
            Self::Transparent
        } else if is_alpha_mask {
            Self::AlphaMask
        } else {
            Self::Opaque
        }
    }

    pub(crate) fn casts_shadow(self) -> bool {
        matches!(self, Self::Opaque | Self::AlphaMask)
    }
}

impl MeshDrawQueueProfile {
    pub(crate) fn new(
        phase: MeshDrawQueuePhase,
        geometry_source: MeshDrawGeometrySource,
        mobility: Mobility,
        uses_indirect_draw: bool,
        uses_skinned_gpu_skinning: bool,
        uses_mesh_lod: bool,
    ) -> Self {
        Self {
            phase,
            geometry_source,
            mobility,
            uses_indirect_draw,
            uses_skinned_gpu_skinning,
            uses_mesh_lod,
        }
    }

    pub(crate) fn phase(self) -> MeshDrawQueuePhase {
        self.phase
    }

    pub(crate) fn geometry_source(self) -> MeshDrawGeometrySource {
        self.geometry_source
    }

    pub(crate) fn shader_geometry_source_id(self) -> GeometrySourceId {
        if self.uses_skinned_gpu_skinning {
            match self.geometry_source {
                MeshDrawGeometrySource::Prepared
                | MeshDrawGeometrySource::DynamicGpuSkinningSource => {
                    return GEOMETRY_SOURCE_ID_SKINNED_MESH;
                }
                MeshDrawGeometrySource::Dynamic => {}
            }
        }
        self.geometry_source.shader_geometry_source_id()
    }

    pub(crate) fn early_z_eligible(self) -> bool {
        self.phase.casts_shadow()
    }

    pub(crate) fn static_batch_eligible(self) -> bool {
        self.direct_prepared_non_transparent() && self.mobility == Mobility::Static
    }

    pub(crate) fn dynamic_batch_eligible(self) -> bool {
        self.direct_prepared_non_transparent() && self.mobility == Mobility::Dynamic
    }

    pub(crate) fn gpu_instancing_eligible(self) -> bool {
        self.direct_prepared_non_transparent()
    }

    pub(crate) fn uses_indirect_draw(self) -> bool {
        self.uses_indirect_draw
    }

    pub(crate) fn uses_mesh_lod(self) -> bool {
        self.uses_mesh_lod
    }

    pub(crate) fn velocity_history_eligible(self) -> bool {
        self.mobility == Mobility::Dynamic
    }

    fn direct_prepared_non_transparent(self) -> bool {
        self.geometry_source == MeshDrawGeometrySource::Prepared
            && !self.uses_indirect_draw
            && !self.uses_skinned_gpu_skinning
            && self.early_z_eligible()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        GEOMETRY_SOURCE_ID_SKINNED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
    };
    use crate::core::framework::scene::Mobility;

    use super::{MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile};

    #[test]
    fn queue_profile_maps_prepared_gpu_skinning_to_skinned_shader_geometry() {
        let profile = MeshDrawQueueProfile::new(
            MeshDrawQueuePhase::Opaque,
            MeshDrawGeometrySource::Prepared,
            Mobility::Dynamic,
            false,
            true,
            false,
        );

        assert_eq!(profile.geometry_source(), MeshDrawGeometrySource::Prepared);
        assert_eq!(
            profile.shader_geometry_source_id(),
            GEOMETRY_SOURCE_ID_SKINNED_MESH
        );
    }

    #[test]
    fn queue_profile_keeps_cpu_fallback_dynamic_on_static_shader_geometry() {
        let profile = MeshDrawQueueProfile::new(
            MeshDrawQueuePhase::Opaque,
            MeshDrawGeometrySource::Dynamic,
            Mobility::Dynamic,
            false,
            false,
            false,
        );

        assert_eq!(
            profile.shader_geometry_source_id(),
            GEOMETRY_SOURCE_ID_STATIC_MESH
        );
    }
}

impl MeshDraw {
    pub(crate) fn queue_profile(&self) -> MeshDrawQueueProfile {
        MeshDrawQueueProfile::new(
            MeshDrawQueuePhase::from_pipeline_flags(self.is_transparent(), self.is_alpha_mask()),
            self.geometry_source,
            self.mobility,
            self.uses_indirect_draw(),
            self.uses_skinned_gpu_skinning(),
            self.mesh_lod.is_some(),
        )
    }

    pub(crate) fn casts_shadow(&self) -> bool {
        self.cast_shadows && self.queue_profile().phase().casts_shadow()
    }

    pub(crate) fn batch_key(&self) -> MeshDrawBatchKey {
        MeshDrawBatchKey {
            geometry_source: self.geometry_source,
            mesh: Arc::as_ptr(&self.mesh) as usize,
            base_color_texture: self.material_textures.base_color.identity(),
            normal_texture: self.material_textures.normal.identity(),
            metallic_roughness_texture: self.material_textures.metallic_roughness.identity(),
            occlusion_texture: self.material_textures.occlusion.identity(),
            emissive_texture: self.material_textures.emissive.identity(),
            material_uniform: Arc::as_ptr(&self.material_uniform) as usize,
            standard_material_uniform: Arc::as_ptr(&self.standard_material_uniform) as usize,
            pipeline_key: self.pipeline_key.clone(),
            first_index: self.first_index,
            draw_index_count: self.draw_index_count,
        }
    }
}
