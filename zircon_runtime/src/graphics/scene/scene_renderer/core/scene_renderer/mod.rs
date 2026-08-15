mod advanced_plugin_outputs;
mod scene_renderer;

pub(in crate::graphics::scene::scene_renderer::core) use advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;
pub(in crate::graphics::scene::scene_renderer::core) use scene_renderer::{
    ScenePostProcessStartupMode, SceneRendererCaptureTarget,
};
pub use scene_renderer::{
    SceneRenderer, SceneRendererCoreStartupReport, SceneRendererDeferredLightingProfile,
    SceneRendererFrameTimingReport, SceneRendererGpuPassTiming, SceneRendererGpuTimingReport,
    SceneRendererStartupOptions, SceneRendererStartupReport,
};
