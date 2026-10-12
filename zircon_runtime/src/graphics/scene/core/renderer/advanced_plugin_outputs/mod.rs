#[path = "../../advanced_plugin_outputs/output_access.rs"]
mod output_access;
#[path = "../../advanced_plugin_outputs/output_storage.rs"]
mod output_storage;
#[path = "../../advanced_plugin_outputs/outputs.rs"]
mod scene_renderer_advanced_plugin_outputs;

pub(in crate::graphics::scene::scene_renderer::core) use scene_renderer_advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;
