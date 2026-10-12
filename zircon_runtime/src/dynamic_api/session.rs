//! Dynamic runtime session composition and owner routing.

#[path = "session/composition_receipt.rs"]
mod composition_receipt;
#[path = "session/construction.rs"]
mod construction;
#[path = "session/diagnostics.rs"]
mod diagnostics;
#[path = "session/editor_transform.rs"]
mod editor_transform;
#[path = "session/error.rs"]
mod error;
#[path = "session/event_mirror.rs"]
mod event_mirror;
#[path = "session/events.rs"]
mod events;
#[path = "session/extract.rs"]
mod extract;
#[path = "session/extract_cache.rs"]
mod extract_cache;
#[path = "session/extract_stats.rs"]
mod extract_stats;
#[path = "session/ffi.rs"]
mod ffi;
#[path = "session/highlight_set.rs"]
mod highlight_set;
#[path = "session/host_requests.rs"]
mod host_requests;
#[path = "session/hud.rs"]
mod hud;
#[path = "session/ime_composition_configuration.rs"]
mod ime_composition_configuration;
#[path = "session/ime_composition_generation.rs"]
mod ime_composition_generation;
#[path = "session/ime_composition_route.rs"]
mod ime_composition_route;
#[path = "session/input_events.rs"]
mod input_events;
#[path = "session/linked_plugins.rs"]
mod linked_plugins;
#[path = "session/linked_session.rs"]
mod linked_session;
#[path = "session/menu.rs"]
mod menu;
#[path = "session/operation.rs"]
mod operation;
#[path = "session/preview.rs"]
mod preview;
#[path = "session/profile.rs"]
mod profile;
#[path = "session/project.rs"]
mod project;
#[path = "session/registry/mod.rs"]
mod registry;
#[path = "session/runtime_ui.rs"]
mod runtime_ui;
#[path = "session/scene_asset_reload_diagnostics.rs"]
mod scene_asset_reload_diagnostics;
#[path = "session/script_systems.rs"]
mod script_systems;
#[path = "session/shutdown.rs"]
mod shutdown;
#[path = "session/state.rs"]
mod state;
#[path = "session/status.rs"]
mod status;
#[cfg(test)]
#[path = "session/tests/mod.rs"]
mod tests;
#[path = "session/ui_extract_cache.rs"]
mod ui_extract_cache;
#[path = "session/viewport_pick.rs"]
mod viewport_pick;
#[path = "session/world_sync.rs"]
mod world_sync;

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
pub(super) use ffi::cancel_viewport_pick;
pub(super) use ffi::{
    bind_viewport_surface, capture_accessibility_tree, capture_frame, configure_app_session,
    create_session, destroy_session, drain_host_requests, drain_plugin_events,
    drain_world_invalidations, handle_event, poll_viewport_pick, present_viewport, profile_control,
    query_world, release_allocation, request_viewport_pick, submit_highlight_set,
    subscribe_plugin_event, tick_frame, unbind_viewport_surface, unsubscribe_plugin_event,
    unwatch_world, watch_world,
};
pub(super) use host_requests::{
    runtime_cursor_host_request, runtime_gamepad_rumble_request, runtime_ime_host_request,
};
pub(super) use operation::{harvest_operation, poll_operation, submit_operation};
use profile::RuntimeDynamicSessionProfile;
use project::RuntimeProjectConfig;
use script_systems::merge_builtin_script_scene_systems;
use state::RuntimeDynamicSession;
