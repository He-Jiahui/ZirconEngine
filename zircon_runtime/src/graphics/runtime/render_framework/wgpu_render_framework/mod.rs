mod submission_metrics;
mod wgpu_render_framework;

pub use wgpu_render_framework::WgpuRenderFramework;
pub(in crate::graphics::runtime::render_framework) use wgpu_render_framework::{
    WgpuRenderFrameworkAccess, WgpuRenderFrameworkCore,
};
