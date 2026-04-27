//! Editor UI host ownership and orchestration surfaces.

pub(crate) mod animation_editor_sessions;
pub(crate) mod asset_editor_sessions;
mod builtin_layout;
mod builtin_views;
pub(crate) mod editor_asset_manager;
mod editor_capabilities;
mod editor_error;
mod editor_event_control_requests;
mod editor_event_dispatch;
mod editor_event_execution;
mod editor_event_listener_control;
mod editor_event_runtime_access;
mod editor_event_runtime_bootstrap;
mod editor_event_runtime_reflection;
mod editor_extension_registration;
mod editor_extension_views;
mod editor_manager;
mod editor_manager_animation_editor;
mod editor_manager_asset_editor;
mod editor_manager_layout;
mod editor_manager_minimal_host;
mod editor_manager_plugins_export;
mod editor_manager_project;
mod editor_manager_runtime_diagnostics;
mod editor_manager_startup;
mod editor_manager_workspace;
mod editor_operation_dispatch;
mod editor_session_state;
mod editor_subsystems;
mod editor_ui_host;
mod host_capability_bridge;
mod layout_commands;
mod layout_hosts;
mod layout_persistence;
pub(crate) mod minimal_host_contract;
pub(crate) mod module;
mod native_dynamic_export_preparation;
mod project_access;
pub(crate) mod resource_access;
mod startup;
mod ui_asset_promotion;
mod view_registry;
mod window_host_manager;
mod workspace_state;

pub(crate) use builtin_layout::builtin_hybrid_layout;
pub use editor_capabilities::EditorCapabilitySnapshot;
pub use editor_error::EditorError;
pub use editor_manager::EditorManager;
pub use editor_manager_plugins_export::{
    EditorExportBuildReport, EditorPluginEnableReport, EditorPluginStatus, EditorPluginStatusReport,
};
pub use editor_subsystems::{
    EditorSubsystemReport, EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
    EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY, EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
    EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING, EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
    EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING,
};
pub use host_capability_bridge::{EditorHostVmBridgeReport, EditorVmExtensionLoadReport};
pub use minimal_host_contract::{
    editor_host_minimal_contract, EditorHostMinimalContract, EditorHostMinimalReport,
};
pub use window_host_manager::NativeWindowHostState;
