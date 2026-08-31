mod environment_capture_residency;
mod environment_ibl_hydration_cache;
mod render_framework_state;
mod viewport_hit_proxy_table;
mod viewport_pick_frame_registry;
mod viewport_pick_store;
mod viewport_product_registry;

#[cfg(test)]
mod viewport_pick_frame_registry_tests;
#[cfg(test)]
mod viewport_pick_store_tests;

pub(in crate::graphics::runtime::render_framework) use environment_capture_residency::EnvironmentCaptureResidency;
pub(in crate::graphics::runtime::render_framework) use environment_ibl_hydration_cache::EnvironmentIblHydrationCache;
pub(in crate::graphics::runtime::render_framework) use render_framework_state::RenderFrameworkState;
pub(in crate::graphics::runtime::render_framework) use viewport_hit_proxy_table::{
    ViewportHitProxyIdentity, ViewportHitProxyTable,
};
pub(in crate::graphics::runtime::render_framework) use viewport_pick_frame_registry::{
    ViewportPickFrameRegistry, ViewportPickFrameSnapshot,
};
pub(in crate::graphics::runtime::render_framework) use viewport_pick_store::{
    ViewportPickCompletionSender, ViewportPickStore,
};
pub(in crate::graphics::runtime::render_framework) use viewport_product_registry::{
    ViewportProductRegistry, WgpuViewportProductProvider,
};
