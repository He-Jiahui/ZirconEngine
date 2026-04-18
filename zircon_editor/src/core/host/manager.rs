//! Editor manager service and window host bookkeeping.

mod builtin_layout;
mod builtin_views;
mod editor_error;
mod editor_manager;
mod editor_session_state;
mod layout_commands;
mod layout_hosts;
mod layout_persistence;
mod project_access;
mod startup;
mod ui_asset_promotion;
mod ui_asset_sessions;
mod view_registry;
mod window_host_manager;
mod workspace_state;

pub(crate) use builtin_layout::builtin_hybrid_layout;
pub use editor_error::EditorError;
pub use editor_manager::EditorManager;
pub use window_host_manager::NativeWindowHostState;
