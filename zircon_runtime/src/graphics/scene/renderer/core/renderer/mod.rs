#[path = "../../../core/renderer/advanced_plugin_outputs/mod.rs"]
mod advanced_plugin_outputs;
#[path = "renderer.rs"]
mod scene_renderer;
#[path = "startup_report.rs"]
mod startup_report;

pub(in crate::graphics::scene::scene_renderer::core) use advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;
pub(in crate::graphics::scene::scene_renderer::core) use scene_renderer::{
    ScenePostProcessStartupMode, SceneRendererCaptureTarget,
};
pub use scene_renderer::{
    SceneRenderer, SceneRendererDeferredLightingProfile, SceneRendererFrameTimingReport,
    SceneRendererGpuPassTiming, SceneRendererGpuTimingReport, SceneRendererStartupOptions,
};
pub use startup_report::{
    SceneRendererCoreStartupReport, SceneRendererEnvironmentOnlyPbrBasePrewarmReport,
    SceneRendererStartupReport,
};
