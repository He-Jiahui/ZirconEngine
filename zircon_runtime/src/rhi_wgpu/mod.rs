//! `wgpu` capability mapping and backend wrappers for [`zircon_rhi`].

mod capabilities;
mod device;
mod ui_surface;

pub use capabilities::wgpu_backend_caps;
pub use device::{WgpuCommandList, WgpuRenderDevice};
pub use ui_surface::WgpuUiSurfacePresenter;

#[cfg(test)]
mod tests;
