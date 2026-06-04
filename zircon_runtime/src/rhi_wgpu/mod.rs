//! `wgpu` capability mapping and backend wrappers for [`zircon_rhi`].

mod bind_group_validation;
mod capabilities;
mod command_validation;
mod device;
mod pipeline_validation;
mod render_pass_validation;
mod resource_validation;
mod texture_copy;
mod ui_surface;

pub use capabilities::wgpu_backend_caps;
pub use device::{WgpuCommandList, WgpuRenderDevice};
pub use ui_surface::WgpuUiSurfacePresenter;

#[cfg(test)]
mod tests;
