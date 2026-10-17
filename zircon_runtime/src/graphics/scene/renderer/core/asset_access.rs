use crate::asset::ProjectAssetManager;
use crate::graphics::GraphicsError;

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    /// Runtime extract 借用与 renderer streamer 同源的项目资源管理器，避免渲染与提取使用不同资源世代。
    pub(crate) fn asset_manager_for_runtime_extract(
        &self,
    ) -> Result<crate::core::runtime::ServiceCallGuard<ProjectAssetManager>, GraphicsError> {
        self.streamer.asset_manager()
    }
}

impl SceneRenderer {
    pub(crate) fn admit_virtual_geometry_sources(
        &mut self,
        mesh: &crate::core::framework::render::RenderMeshSnapshot,
    ) -> Result<Vec<crate::graphics::VirtualGeometryPrimitiveAdmission>, GraphicsError> {
        let source = match mesh.mesh {
            Some(handle) => self.streamer.admit_mesh_for_virtual_geometry(handle)?,
            None => self.streamer.admit_model_for_virtual_geometry(mesh.model)?,
        };
        Ok((0..source.primitives().len())
            .map(|index| {
                source
                    .primitive(index)
                    .expect("admitted source primitive index is in range")
            })
            .collect())
    }
}
