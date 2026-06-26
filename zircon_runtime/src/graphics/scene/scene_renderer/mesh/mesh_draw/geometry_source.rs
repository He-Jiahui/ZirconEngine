use crate::core::framework::render::{
    GeometrySourceId, GEOMETRY_SOURCE_ID_SKINNED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MeshDrawGeometrySource {
    Prepared,
    Dynamic,
    DynamicGpuSkinningSource,
}

impl MeshDrawGeometrySource {
    pub(crate) const fn shader_geometry_source_id(self) -> GeometrySourceId {
        match self {
            Self::Prepared | Self::Dynamic => GEOMETRY_SOURCE_ID_STATIC_MESH,
            Self::DynamicGpuSkinningSource => GEOMETRY_SOURCE_ID_SKINNED_MESH,
        }
    }
}
