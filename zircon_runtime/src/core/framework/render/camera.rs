mod camera_snapshot;
mod clear_color;
mod defaults;
mod display_mode;
mod dynamic_resolution;
mod extract_request;
mod fallback_skybox;
mod layer;
mod layer_set;
mod projection_mode;
mod target;
mod target_kind;
mod viewport_rect;
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
mod tests;
