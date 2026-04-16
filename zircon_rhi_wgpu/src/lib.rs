//! `wgpu` capability mapping and backend wrappers for [`zircon_rhi`].

mod capabilities;
mod device;

pub use capabilities::wgpu_backend_caps;
pub use device::{WgpuCommandList, WgpuRenderDevice};

#[cfg(test)]
mod tests;
