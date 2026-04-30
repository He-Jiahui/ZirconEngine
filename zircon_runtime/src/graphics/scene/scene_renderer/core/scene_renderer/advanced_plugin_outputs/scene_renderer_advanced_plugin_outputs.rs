use super::{
    HybridGiReadbackOutputs, VirtualGeometryCullOutputs, VirtualGeometryIndirectOutputs,
    VirtualGeometryReadbackOutputs, VirtualGeometryRenderPathOutputs,
};

#[derive(Default)]
pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginOutputs {
    hybrid_gi_readback: HybridGiReadbackOutputs,
    virtual_geometry_readback: VirtualGeometryReadbackOutputs,
    virtual_geometry_cull: VirtualGeometryCullOutputs,
    virtual_geometry_render_path: VirtualGeometryRenderPathOutputs,
    virtual_geometry_indirect: VirtualGeometryIndirectOutputs,
}

impl SceneRendererAdvancedPluginOutputs {
    pub(super) fn hybrid_gi_readback_mut(&mut self) -> &mut HybridGiReadbackOutputs {
        &mut self.hybrid_gi_readback
    }

    pub(super) fn virtual_geometry_readback(&self) -> &VirtualGeometryReadbackOutputs {
        &self.virtual_geometry_readback
    }

    pub(super) fn virtual_geometry_readback_mut(&mut self) -> &mut VirtualGeometryReadbackOutputs {
        &mut self.virtual_geometry_readback
    }

    pub(super) fn virtual_geometry_cull(&self) -> &VirtualGeometryCullOutputs {
        &self.virtual_geometry_cull
    }

    pub(super) fn virtual_geometry_cull_mut(&mut self) -> &mut VirtualGeometryCullOutputs {
        &mut self.virtual_geometry_cull
    }

    pub(super) fn virtual_geometry_render_path(&self) -> &VirtualGeometryRenderPathOutputs {
        &self.virtual_geometry_render_path
    }

    pub(super) fn virtual_geometry_render_path_mut(
        &mut self,
    ) -> &mut VirtualGeometryRenderPathOutputs {
        &mut self.virtual_geometry_render_path
    }

    pub(super) fn virtual_geometry_indirect(&self) -> &VirtualGeometryIndirectOutputs {
        &self.virtual_geometry_indirect
    }

    pub(super) fn virtual_geometry_indirect_mut(&mut self) -> &mut VirtualGeometryIndirectOutputs {
        &mut self.virtual_geometry_indirect
    }
}
