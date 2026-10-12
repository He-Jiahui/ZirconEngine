#[path = "compiled_scene_outputs.rs"]
mod compiled_scene_outputs;
#[path = "history/mod.rs"]
mod history;
#[path = "post/mod.rs"]
mod post_process;
#[path = "render/mod.rs"]
mod render;
#[path = "runtime_prepare/mod.rs"]
mod runtime_prepare;
#[path = "scene_passes/mod.rs"]
mod scene_passes;

pub(in crate::graphics::scene::scene_renderer::core) use compiled_scene_outputs::SceneRendererCompiledSceneOutputs;
