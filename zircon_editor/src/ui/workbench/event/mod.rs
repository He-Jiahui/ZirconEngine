//! Backend-neutral workbench host events and stable editor UI bindings.

mod constants;
mod core_event_conversion;
mod dispatch_editor_host_binding;
mod editor_host_event;
mod editor_host_event_error;
mod editor_operation_binding;
mod menu_action_binding;
mod menu_action_from_id;
mod menu_action_id;
mod node_kind_from_id;
mod node_kind_id;

pub(crate) use core_event_conversion::{core_layout_command_from_ui, ui_layout_command_from_core};
pub use dispatch_editor_host_binding::dispatch_editor_host_binding;
pub use editor_host_event::EditorHostEvent;
pub use editor_host_event_error::EditorHostEventError;
pub use editor_operation_binding::editor_operation_binding;
pub use menu_action_binding::menu_action_binding;
