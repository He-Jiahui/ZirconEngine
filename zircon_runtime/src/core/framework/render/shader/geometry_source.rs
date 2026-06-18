use serde::{Deserialize, Serialize};

pub const GEOMETRY_SOURCE_ID_STATIC_MESH: GeometrySourceId = GeometrySourceId::new(0);
pub const GEOMETRY_SOURCE_ID_SKINNED_MESH: GeometrySourceId = GeometrySourceId::new(1);
pub const GEOMETRY_SOURCE_ID_MORPHED_MESH: GeometrySourceId = GeometrySourceId::new(2);
pub const GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH: GeometrySourceId = GeometrySourceId::new(3);
pub const GEOMETRY_SOURCE_PLUGIN_ID_START: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GeometrySourceId(u8);

impl GeometrySourceId {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u8 {
        self.0
    }

    pub const fn is_plugin_range(self) -> bool {
        self.0 >= GEOMETRY_SOURCE_PLUGIN_ID_START
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GeometrySourceId, GEOMETRY_SOURCE_ID_MORPHED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MESH,
        GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
        GEOMETRY_SOURCE_PLUGIN_ID_START,
    };

    #[test]
    fn render_shader_geometry_source_ids_reserve_builtin_segment() {
        assert_eq!(GEOMETRY_SOURCE_ID_STATIC_MESH.value(), 0);
        assert_eq!(GEOMETRY_SOURCE_ID_SKINNED_MESH.value(), 1);
        assert_eq!(GEOMETRY_SOURCE_ID_MORPHED_MESH.value(), 2);
        assert_eq!(GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH.value(), 3);
        assert!(!GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH.is_plugin_range());
        assert!(GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START).is_plugin_range());
    }
}
