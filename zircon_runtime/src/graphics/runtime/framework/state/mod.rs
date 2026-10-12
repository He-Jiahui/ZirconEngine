#[path = "../../state/environment_capture_residency.rs"]
mod environment_capture_residency;
#[path = "../../state/environment_ibl_hydration_cache.rs"]
mod environment_ibl_hydration_cache;
#[path = "state.rs"]
mod render_framework_state;
#[path = "viewport_hit_proxy_table.rs"]
mod viewport_hit_proxy_table;
#[path = "../../state/viewport_pick_frame_registry.rs"]
mod viewport_pick_frame_registry;
#[path = "viewport_pick_store.rs"]
mod viewport_pick_store;
#[path = "viewport_product_registry.rs"]
mod viewport_product_registry;

#[cfg(test)]
#[path = "../../state/tests/viewport_pick_frame_registry.rs"]
mod viewport_pick_frame_registry_tests;
#[cfg(test)]
#[path = "tests/viewport_pick_store.rs"]
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
