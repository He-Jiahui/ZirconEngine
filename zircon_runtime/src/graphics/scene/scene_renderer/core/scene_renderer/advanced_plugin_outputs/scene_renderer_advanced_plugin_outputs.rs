use super::{
    VirtualGeometryCullOutputs, VirtualGeometryIndirectOutputs, VirtualGeometryRenderPathOutputs,
};
use crate::core::framework::render::RenderPluginRendererOutputs;

#[derive(Default)]
pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginOutputs {
    virtual_geometry_cull: VirtualGeometryCullOutputs,
    virtual_geometry_render_path: VirtualGeometryRenderPathOutputs,
    virtual_geometry_indirect: VirtualGeometryIndirectOutputs,
    plugin_renderer_outputs: RenderPluginRendererOutputs,
}

impl SceneRendererAdvancedPluginOutputs {
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

    pub(super) fn plugin_renderer_outputs_ref(&self) -> &RenderPluginRendererOutputs {
        &self.plugin_renderer_outputs
    }

    pub(super) fn plugin_renderer_outputs_mut(&mut self) -> &mut RenderPluginRendererOutputs {
        &mut self.plugin_renderer_outputs
    }
}
