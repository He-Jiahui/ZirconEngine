//! `wgpu` capability mapping and native UI presentation support.
//!
//! The product device/queue/resource owner lives in `graphics::backend` and uses real `wgpu`
//! objects. The deterministic host-mirror device below exists only for RHI contract tests.

#[cfg(test)]
mod bind_group_validation;
mod capabilities;
#[cfg(test)]
mod command_validation;
#[cfg(test)]
mod device;
#[cfg(test)]
mod pipeline_validation;
#[cfg(test)]
mod render_pass_validation;
#[cfg(test)]
mod resource_validation;
#[cfg(test)]
mod texture_copy;
mod ui_surface;

pub use capabilities::wgpu_backend_caps;
pub use ui_surface::WgpuUiSurfacePresenter;

#[cfg(test)]
pub(crate) use device::{DeterministicRhiContractCommandList, DeterministicRhiContractDevice};

#[cfg(test)]
mod tests;
