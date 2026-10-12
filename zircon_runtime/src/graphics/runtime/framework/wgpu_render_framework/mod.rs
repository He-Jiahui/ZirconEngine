#[path = "../../wgpu_render_framework/submission_metrics.rs"]
mod submission_metrics;
#[path = "../../../wgpu_render_framework/wgpu_render_framework.rs"]
mod wgpu_render_framework;

pub use wgpu_render_framework::WgpuRenderFramework;
pub(in crate::graphics::runtime::render_framework) use wgpu_render_framework::{
    WgpuRenderFrameworkAccess, WgpuRenderFrameworkCore,
};
