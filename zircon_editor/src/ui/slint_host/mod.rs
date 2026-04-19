pub(crate) mod activity_rail_pointer;
mod app;
pub(crate) mod asset_pointer;
pub(crate) mod callback_dispatch;
pub(crate) mod detail_pointer;
pub(crate) mod document_tab_pointer;
pub(crate) mod drawer_header_pointer;
pub(crate) mod drawer_resize;
pub(crate) mod event_bridge;
pub(crate) mod floating_window_projection;
pub(crate) mod hierarchy_pointer;
pub(crate) mod host_page_pointer;
pub(crate) mod menu_pointer;
pub(crate) mod root_shell_projection;
pub(crate) mod scroll_surface_host;
pub(crate) mod shell_pointer;
pub mod tab_drag;
mod ui;
mod viewport;
pub(crate) mod viewport_toolbar_pointer;
pub(crate) mod welcome_recent_pointer;

mod generated {
    #![allow(dead_code)]

    slint::include_modules!();
}

pub(crate) use generated::*;

#[cfg(test)]
pub(crate) use app::backend_refresh::{plan_asset_backend_refresh, AssetBackendRefreshPlan};
pub use app::run_editor;
#[cfg(test)]
pub(crate) use app::{
    collect_native_floating_window_targets, configure_native_floating_window_presentation,
    NativeFloatingWindowTarget, NativeWindowPresenterStore,
};
