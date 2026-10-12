//! Backend-neutral workbench host events and stable editor UI bindings.

#[path = "constants.rs"]
mod constants;
#[path = "core_event_conversion.rs"]
mod core_event_conversion;
#[path = "dispatch_editor_host_binding.rs"]
mod dispatch_editor_host_binding;
#[path = "editor_host_event.rs"]
mod editor_host_event;
#[path = "editor_host_event_error.rs"]
mod editor_host_event_error;
#[path = "editor_operation_binding.rs"]
mod editor_operation_binding;
#[path = "menu_action_binding.rs"]
mod menu_action_binding;
#[path = "menu_action_from_id.rs"]
mod menu_action_from_id;
#[path = "menu_action_id.rs"]
mod menu_action_id;
#[path = "menu_item_binding.rs"]
mod menu_item_binding;
#[path = "node_kind_from_id.rs"]
mod node_kind_from_id;
#[path = "node_kind_id.rs"]
mod node_kind_id;

pub(crate) use core_event_conversion::{core_layout_command_from_ui, ui_layout_command_from_core};
pub use dispatch_editor_host_binding::dispatch_editor_host_binding;
pub use editor_host_event::EditorHostEvent;
pub use editor_host_event_error::EditorHostEventError;
pub use editor_operation_binding::editor_operation_binding;
pub use menu_action_binding::menu_action_binding;
pub use menu_item_binding::menu_item_binding;
pub(crate) use menu_item_binding::operation_path_for_menu_action;
