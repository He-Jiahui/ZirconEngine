//! Backend-neutral workbench host events and stable editor UI bindings.

mod constants;
mod core_event_conversion;
mod dispatch_workbench_binding;
mod menu_action_binding;
mod menu_action_from_id;
mod menu_action_id;
mod node_kind_from_id;
mod node_kind_id;
mod workbench_host_event;
mod workbench_host_event_error;

pub(crate) use core_event_conversion::{core_layout_command_from_ui, ui_layout_command_from_core};
pub use dispatch_workbench_binding::dispatch_workbench_binding;
pub use menu_action_binding::menu_action_binding;
pub use workbench_host_event::WorkbenchHostEvent;
pub use workbench_host_event_error::WorkbenchHostEventError;
