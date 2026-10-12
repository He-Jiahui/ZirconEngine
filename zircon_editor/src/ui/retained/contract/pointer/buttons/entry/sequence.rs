#[path = "sequence/entry.rs"]
mod entry;
#[path = "sequence/steps.rs"]
mod steps;

pub(in crate::ui::retained_host::host_contract) use self::entry::dispatch_native_pointer_button;
