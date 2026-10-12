#[path = "../../../../../pointer/buttons/target/entry/result_targets/entry.rs"]
mod entry;
#[path = "../../../../../pointer/buttons/target/entry/result_targets/input.rs"]
mod input;
#[path = "../../../../../pointer/buttons/target/entry/result_targets/passive.rs"]
mod passive;
#[path = "../../../../../pointer/buttons/target/entry/result_targets/sequence.rs"]
mod sequence;
#[path = "../../../../../pointer/buttons/target/entry/result_targets/template.rs"]
mod template;
#[path = "../../../../../pointer/buttons/target/entry/result_targets/viewport.rs"]
mod viewport;

pub(super) use self::entry::dispatch_result_pane_targets;
