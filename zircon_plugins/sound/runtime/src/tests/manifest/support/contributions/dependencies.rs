mod entry;
mod line;
mod state;

use super::StaticDependency;

pub(super) fn dependencies_from_plugin_toml(manifest: &str) -> Vec<StaticDependency> {
    entry::dependencies_from_plugin_toml(manifest)
}
