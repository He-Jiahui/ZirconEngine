#[path = "../../../../../../contract/pointer/buttons/target/entry/sequence/entry.rs"]
mod entry;
#[path = "../../../../../../contract/pointer/buttons/target/entry/sequence/run.rs"]
mod run;

pub(in crate::ui::retained_host::host_contract) use self::entry::dispatch_pane_button;
