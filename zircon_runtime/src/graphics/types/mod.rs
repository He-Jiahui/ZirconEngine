#[path = "gpu_resource_handle.rs"]
mod gpu_resource_handle;
#[path = "graphics_error.rs"]
mod graphics_error;
#[path = "viewport_camera_stack_attachment_policy.rs"]
mod viewport_camera_stack_attachment_policy;
#[path = "viewport_camera_stack_output_policy.rs"]
mod viewport_camera_stack_output_policy;
#[path = "viewport_frame.rs"]
mod viewport_frame;
#[path = "viewport_frame_texture_handle.rs"]
mod viewport_frame_texture_handle;
#[path = "viewport_render_frame.rs"]
mod viewport_render_frame;
#[path = "viewport_render_frame_from_extract.rs"]
mod viewport_render_frame_from_extract;
#[path = "viewport_render_frame_from_snapshot.rs"]
mod viewport_render_frame_from_snapshot;
#[path = "../../viewport_render_frame_with_camera_stack_output_policy.rs"]
mod viewport_render_frame_with_camera_stack_output_policy;
#[path = "viewport_render_frame_with_frame_visibility.rs"]
mod viewport_render_frame_with_frame_visibility;
#[path = "viewport_render_frame_with_output_target.rs"]
mod viewport_render_frame_with_output_target;
#[path = "../../viewport_render_frame_with_prepared_runtime_sidebands.rs"]
mod viewport_render_frame_with_prepared_runtime_sidebands;
#[path = "../../viewport_render_frame_with_previous_motion_vector_camera.rs"]
mod viewport_render_frame_with_previous_motion_vector_camera;
#[path = "viewport_render_frame_with_runtime_overlays.rs"]
mod viewport_render_frame_with_runtime_overlays;
#[path = "viewport_render_frame_with_shader_quality.rs"]
mod viewport_render_frame_with_shader_quality;
#[path = "../viewport_render_frame_with_texture_max_anisotropy.rs"]
mod viewport_render_frame_with_texture_max_anisotropy;
#[path = "viewport_render_frame_with_texture_mip_bias.rs"]
mod viewport_render_frame_with_texture_mip_bias;
#[path = "viewport_render_frame_with_ui.rs"]
mod viewport_render_frame_with_ui;
#[path = "../../viewport_render_frame_with_virtual_geometry_debug_snapshot.rs"]
mod viewport_render_frame_with_virtual_geometry_debug_snapshot;
#[path = "viewport_render_output_target.rs"]
mod viewport_render_output_target;
#[path = "viewport_render_region.rs"]
mod viewport_render_region;

pub use gpu_resource_handle::GpuResourceHandle;
pub use graphics_error::GraphicsError;
pub(crate) use viewport_camera_stack_attachment_policy::{
    ViewportCameraStackAttachmentPolicy, ViewportSceneClearPlan, ViewportSceneColorClear,
};
pub(crate) use viewport_camera_stack_output_policy::ViewportCameraStackOutputPolicy;
pub use viewport_frame::ViewportFrame;
pub use viewport_frame_texture_handle::ViewportFrameTextureHandle;
pub(crate) use viewport_render_frame::{RendererPostProcessSnapshot, ViewportRenderFrame};
pub(crate) use viewport_render_output_target::{
    ViewportRenderOutputTarget, ViewportTextureGraphImportPlan, ViewportTextureGraphImportStatus,
    ViewportTextureWritebackPlan, ViewportTextureWritebackStatus,
};
#[cfg(test)]
pub(crate) use viewport_render_output_target::{
    FRAMEWORK_OUTPUT_FORMAT_LABEL, LINEAR_OUTPUT_FORMAT_LABEL,
};
pub use viewport_render_region::ViewportRenderRegion;
