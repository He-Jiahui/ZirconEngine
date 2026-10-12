#[path = "parser/entry.rs"]
mod entry;
#[path = "parser/line.rs"]
mod line;
#[path = "parser/pending.rs"]
mod pending;
#[path = "parser/section.rs"]
mod section;
#[path = "parser/state.rs"]
mod state;

pub(super) use entry::optional_features_from_plugin_toml;
