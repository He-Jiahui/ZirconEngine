#[path = "workbench/entry.rs"]
mod entry;
#[path = "workbench/primary.rs"]
mod primary;
#[path = "workbench/release.rs"]
mod release;
#[path = "workbench/secondary.rs"]
mod secondary;

pub(super) use self::entry::dispatch_workbench_button;
