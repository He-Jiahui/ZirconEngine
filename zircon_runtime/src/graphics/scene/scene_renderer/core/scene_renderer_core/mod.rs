mod advanced_plugin_readbacks;
mod advanced_plugin_resources;
mod scene_renderer_core;

pub(in crate::graphics::scene::scene_renderer::core) use advanced_plugin_readbacks::{
    merge_plugin_renderer_outputs, SceneRendererAdvancedPluginReadbacks,
};
pub(in crate::graphics::scene::scene_renderer::core) use advanced_plugin_resources::SceneRendererAdvancedPluginResources;
pub(in crate::graphics::scene::scene_renderer::core) use scene_renderer_core::SceneRendererCore;
