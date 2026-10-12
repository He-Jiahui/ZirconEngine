#[path = "../../manifest/support/dependencies/entry.rs"]
mod entry;
#[path = "../../manifest/support/dependencies/line.rs"]
mod line;
#[path = "../../manifest/support/dependencies/state.rs"]
mod state;

use super::StaticDependency;

pub(super) fn dependencies_from_plugin_toml(manifest: &str) -> Vec<StaticDependency> {
    entry::dependencies_from_plugin_toml(manifest)
}
