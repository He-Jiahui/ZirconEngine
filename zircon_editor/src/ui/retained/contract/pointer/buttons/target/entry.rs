#[path = "../../../../../contract/pointer/buttons/target/entry/callback_targets.rs"]
mod callback_targets;
#[path = "entry/input.rs"]
mod input;
#[path = "../../../../../contract/pointer/buttons/target/entry/result_targets.rs"]
mod result_targets;
#[path = "entry/sequence.rs"]
mod sequence;

pub(in crate::ui::retained_host::host_contract) use self::sequence::dispatch_pane_button;
