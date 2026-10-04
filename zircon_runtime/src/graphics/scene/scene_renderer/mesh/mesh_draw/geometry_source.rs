use crate::core::framework::render::{
    GeometrySourceId, GEOMETRY_SOURCE_ID_MORPHED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MESH,
    GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MeshDrawGeometrySource {
    Prepared,
    Dynamic,
    // CPU morph deltas are already baked into the dynamic source mesh.
    DynamicCpuMorphedSource,
    // GPU morph deltas are applied by the geometry source from a payload slot.
    DynamicGpuMorphedSource,
    DynamicGpuSkinningSource,
    // GPU morph deltas are applied first, then GPU skinning uses the joint palette.
    DynamicGpuSkinnedMorphedSource,
    // CPU morph deltas are already baked into the dynamic source mesh before GPU skinning.
    DynamicCpuMorphedGpuSkinningSource,
}

impl MeshDrawGeometrySource {
    pub(crate) const fn shader_geometry_source_id(self) -> GeometrySourceId {
        match self {
            Self::Prepared | Self::Dynamic | Self::DynamicCpuMorphedSource => {
                GEOMETRY_SOURCE_ID_STATIC_MESH
            }
            Self::DynamicGpuSkinningSource | Self::DynamicCpuMorphedGpuSkinningSource => {
                GEOMETRY_SOURCE_ID_SKINNED_MESH
            }
            Self::DynamicGpuMorphedSource => GEOMETRY_SOURCE_ID_MORPHED_MESH,
            Self::DynamicGpuSkinnedMorphedSource => GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH,
        }
    }

    pub(crate) const fn uses_cpu_morphed_source(self) -> bool {
        matches!(
            self,
            Self::DynamicCpuMorphedSource | Self::DynamicCpuMorphedGpuSkinningSource
        )
    }

    pub(crate) const fn uses_cpu_morphed_gpu_skinning_source(self) -> bool {
        matches!(self, Self::DynamicCpuMorphedGpuSkinningSource)
    }

    pub(crate) const fn uses_gpu_morph_payload_source(self) -> bool {
        matches!(
            self,
            Self::DynamicGpuMorphedSource | Self::DynamicGpuSkinnedMorphedSource
        )
    }
}
