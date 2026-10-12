#[path = "../../../../core/core/readbacks/collect_into_outputs.rs"]
mod collect_into_outputs;
#[path = "../../../../../core/core/readbacks/merge_plugin_renderer_outputs.rs"]
mod merge_plugin_renderer_outputs;
#[path = "../../../../../core/core/readbacks/advanced_plugin_readbacks.rs"]
mod scene_renderer_advanced_plugin_readbacks;

pub(in crate::graphics::scene::scene_renderer::core) use merge_plugin_renderer_outputs::merge_plugin_renderer_outputs;
pub(in crate::graphics::scene::scene_renderer::core) use scene_renderer_advanced_plugin_readbacks::SceneRendererAdvancedPluginReadbacks;
