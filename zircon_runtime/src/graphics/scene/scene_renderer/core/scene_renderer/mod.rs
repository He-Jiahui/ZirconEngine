mod advanced_plugin_outputs;
mod scene_renderer;

pub(in crate::graphics::scene::scene_renderer::core) use advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;
pub use scene_renderer::{
    SceneRenderer, SceneRendererCoreStartupReport, SceneRendererDeferredLightingProfile,
    SceneRendererFrameTimingReport, SceneRendererStartupOptions, SceneRendererStartupReport,
};
pub(in crate::graphics::scene::scene_renderer::core) use scene_renderer::{
    ScenePostProcessStartupMode, SceneRendererCaptureTarget,
};
