mod advanced_plugin_outputs;
mod scene_renderer;
mod virtual_geometry_output_updates;

pub(in crate::graphics::scene::scene_renderer::core) use advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;
pub use scene_renderer::SceneRenderer;
pub(in crate::graphics::scene::scene_renderer::core) use virtual_geometry_output_updates::{
    VirtualGeometryCullOutputUpdate, VirtualGeometryIndirectOutputUpdate,
    VirtualGeometryLastOutputUpdate, VirtualGeometryRenderPathOutputUpdate,
};
