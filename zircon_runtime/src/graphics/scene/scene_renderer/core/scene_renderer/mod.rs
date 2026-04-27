mod advanced_plugin_outputs;
mod scene_renderer;

pub(in crate::graphics::scene::scene_renderer::core) use advanced_plugin_outputs::{
    SceneRendererAdvancedPluginOutputs, VirtualGeometryCullOutputUpdate,
    VirtualGeometryIndirectOutputUpdate, VirtualGeometryLastOutputUpdate,
    VirtualGeometryRenderPathOutputUpdate,
};
pub use scene_renderer::SceneRenderer;
