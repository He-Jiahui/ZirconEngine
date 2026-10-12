#[path = "../../../../retained/contract/pointer/buttons/native_panes/hierarchy.rs"]
mod hierarchy;
#[path = "../../../../retained/contract/pointer/buttons/native_panes/welcome.rs"]
mod welcome;

pub(in crate::ui::retained_host::host_contract) use self::hierarchy::dispatch_hierarchy_button;
pub(in crate::ui::retained_host::host_contract) use self::welcome::dispatch_welcome_button;
