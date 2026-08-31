use crate::core::framework::scene::EntityId;
use crate::core::math::Transform;
use crate::core::resource::ResourceId;

use super::super::mesh::render_mesh_stable_instance_key;

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryInstance {
    pub entity: EntityId,
    /// Stable render-instance identity shared with the mesh draw pipeline.
    pub stable_instance_key: u64,
    pub source_model: Option<ResourceId>,
    pub transform: Transform,
    pub cluster_offset: u32,
    pub cluster_count: u32,
    pub page_offset: u32,
    pub page_count: u32,
    pub mesh_name: Option<String>,
    pub source_hint: Option<String>,
}

impl Default for RenderVirtualGeometryInstance {
    fn default() -> Self {
        Self {
            entity: 0,
            stable_instance_key: 0,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 0,
            page_offset: 0,
            page_count: 0,
            mesh_name: None,
            source_hint: None,
        }
    }
}

impl RenderVirtualGeometryInstance {
    /// Preserves authored extracts produced before virtual geometry carried the render key.
    pub fn stable_instance_key_or_legacy(&self) -> u64 {
        if self.stable_instance_key == 0 {
            render_mesh_stable_instance_key(self.entity, 0)
        } else {
            self.stable_instance_key
        }
    }
}
