#[path = "../../../../core/core/resources/build_mesh_draws.rs"]
mod build_mesh_draws;
#[path = "../../../../core/core/resources/runtime_prepare.rs"]
mod runtime_prepare;
#[path = "../../../../../core/core/resources/advanced_plugin_resources.rs"]
mod scene_renderer_advanced_plugin_resources;

pub(in crate::graphics::scene::scene_renderer::core) use scene_renderer_advanced_plugin_resources::SceneRendererAdvancedPluginResources;
