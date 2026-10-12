//! 相机配置由场景抽取为视口快照，再交给渲染框架解析目标、层和输出尺寸。
//! 此模块承载跨场景与渲染器的中立契约；具体 GPU 资源由后端持有。
#[path = "camera/camera_snapshot.rs"]
mod camera_snapshot;
#[path = "camera/clear_color.rs"]
mod clear_color;
#[path = "camera/defaults.rs"]
mod defaults;
#[path = "camera/display_mode.rs"]
mod display_mode;
#[path = "camera/dynamic_resolution.rs"]
mod dynamic_resolution;
#[path = "camera/extract_request.rs"]
mod extract_request;
#[path = "camera/fallback_skybox.rs"]
mod fallback_skybox;
#[path = "camera/layer.rs"]
mod layer;
#[path = "camera/layer_set.rs"]
mod layer_set;
#[path = "camera/projection_mode.rs"]
mod projection_mode;
#[path = "camera/target.rs"]
mod target;
#[path = "camera/target_kind.rs"]
mod target_kind;
#[path = "camera/viewport_rect.rs"]
mod viewport_rect;
#[path = "camera/viewport_settings.rs"]
mod viewport_settings;

pub use camera_snapshot::ViewportCameraSnapshot;
pub use clear_color::RenderCameraClearColor;
pub use defaults::{
    aspect_ratio_from_viewport_size, default_viewport_aspect_ratio, DEFAULT_CAMERA_EXPOSURE_EV100,
    DEFAULT_CAMERA_MSAA_SAMPLES,
};
pub use display_mode::DisplayMode;
pub use dynamic_resolution::{
    RenderDynamicResolutionSettings, DEFAULT_DYNAMIC_RESOLUTION_SCALE,
    MAX_DYNAMIC_RESOLUTION_SCALE, MIN_DYNAMIC_RESOLUTION_SCALE,
};
pub use extract_request::SceneViewportExtractRequest;
pub use fallback_skybox::FallbackSkyboxKind;
pub use layer::{RenderLayer, DEFAULT_RENDER_LAYER, DEFAULT_RENDER_LAYER_MASK};
pub use layer_set::RenderLayerSet;
pub use projection_mode::ProjectionMode;
pub use target::RenderCameraTarget;
pub use target_kind::RenderCameraTargetKind;
pub use viewport_rect::RenderViewportRect;
pub use viewport_settings::ViewportRenderSettings;

#[cfg(test)]
#[path = "camera/tests/cases.rs"]
mod tests;
