//! Editor UI host ownership and orchestration surfaces.

pub(crate) mod animation_editor_sessions;
pub(crate) mod asset_editor_sessions;
mod builtin_layout;
mod builtin_views;
pub(crate) mod editor_asset_manager;
mod editor_error;
mod editor_event_control_requests;
mod editor_event_dispatch;
mod editor_event_execution;
mod editor_event_runtime_access;
mod editor_event_runtime_bootstrap;
mod editor_event_runtime_reflection;
mod editor_manager;
mod editor_manager_animation_editor;
mod editor_manager_asset_editor;
mod editor_manager_layout;
mod editor_manager_project;
mod editor_manager_startup;
mod editor_manager_workspace;
mod editor_session_state;
mod editor_ui_host;
mod layout_commands;
mod layout_hosts;
mod layout_persistence;
pub(crate) mod module;
mod project_access;
pub(crate) mod resource_access;
mod startup;
mod ui_asset_promotion;
mod view_registry;
mod window_host_manager;
mod workspace_state;

pub(crate) use builtin_layout::builtin_hybrid_layout;
pub use editor_error::EditorError;
pub use editor_manager::EditorManager;
pub use window_host_manager::NativeWindowHostState;
