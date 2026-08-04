//! Dynamic runtime session composition and owner routing.

mod construction;
mod diagnostics;
mod error;
mod event_mirror;
mod events;
mod extract;
mod extract_cache;
mod extract_stats;
mod ffi;
mod hooks;
mod host_requests;
mod highlight_set;
mod hud;
mod input_events;
mod linked_plugins;
mod linked_session;
mod menu;
mod operation;
mod preview;
mod profile;
mod project;
mod registry;
mod scene_asset_reload_diagnostics;
mod state;
mod status;
#[cfg(test)]
mod tests;

use zircon_runtime_interface::{
    ZrRuntimeViewportHandle, ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
};

const DEFAULT_VIEWPORT: ZrRuntimeViewportHandle = ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1;

pub use error::{RuntimeDynamicSessionError, RuntimeProjectError};
pub use linked_session::create_linked_runtime_session;

use error::RuntimeDynamicSessionResult;
pub(in crate::dynamic_api) use event_mirror::{
    RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES, RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES,
};
pub(super) use ffi::{
    bind_viewport_surface, capture_accessibility_tree, capture_frame, create_session,
    destroy_session, drain_host_requests, drain_plugin_events, handle_event, present_viewport,
    profile_control, submit_highlight_set, subscribe_plugin_event, tick_frame, unbind_viewport_surface,
    unsubscribe_plugin_event,
};
use hooks::install_builtin_scene_runtime_hooks;
pub(super) use host_requests::{
    runtime_cursor_host_request, runtime_gamepad_rumble_request, runtime_ime_host_request,
};
pub(super) use operation::{harvest_operation, poll_operation, submit_operation};
use profile::RuntimeDynamicSessionProfile;
use project::RuntimeProjectConfig;
use state::RuntimeDynamicSession;
