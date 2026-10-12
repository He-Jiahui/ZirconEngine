#[path = "../target/asset.rs"]
mod asset;
#[path = "../target/entry.rs"]
mod entry;
#[path = "../target/native.rs"]
mod native;
#[path = "../target/passive.rs"]
mod passive;
#[path = "../target/template.rs"]
mod template;
#[path = "../target/viewport.rs"]
mod viewport;

pub(in crate::ui::retained_host::host_contract) use self::entry::dispatch_pane_button;
