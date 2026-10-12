#[path = "parser/entry.rs"]
mod entry;
#[path = "parser/line.rs"]
mod line;
#[path = "parser/state.rs"]
mod state;

pub(in crate::tests::manifest::support) use entry::option_manifests_from_plugin_toml;
