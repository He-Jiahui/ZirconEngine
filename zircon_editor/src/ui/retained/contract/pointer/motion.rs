#[path = "motion/clear.rs"]
mod clear;
#[path = "motion/dock_overflow.rs"]
mod dock_overflow;
#[path = "motion/entry.rs"]
mod entry;
#[path = "motion/menu.rs"]
mod menu;
#[path = "motion/page_overflow.rs"]
mod page_overflow;
#[path = "motion/pane.rs"]
mod pane;
#[path = "motion/workbench.rs"]
mod workbench;

pub(in crate::ui::retained_host::host_contract) use self::entry::dispatch_native_pointer_move;
