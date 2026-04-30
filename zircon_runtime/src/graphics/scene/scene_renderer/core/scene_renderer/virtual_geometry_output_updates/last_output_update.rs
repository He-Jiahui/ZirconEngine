use super::{
    VirtualGeometryCullOutputUpdate, VirtualGeometryIndirectOutputUpdate,
    VirtualGeometryRenderPathOutputUpdate,
};

pub(in crate::graphics::scene::scene_renderer::core) struct VirtualGeometryLastOutputUpdate {
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull:
        VirtualGeometryCullOutputUpdate,
    pub(in crate::graphics::scene::scene_renderer::core) render_path:
        VirtualGeometryRenderPathOutputUpdate,
    pub(in crate::graphics::scene::scene_renderer::core) indirect:
        VirtualGeometryIndirectOutputUpdate,
}
