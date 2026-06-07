mod entry;
mod line;
mod state;

use super::StaticEventCatalog;

pub(super) fn event_catalogs_from_plugin_toml(manifest: &str) -> Vec<StaticEventCatalog> {
    entry::event_catalogs_from_plugin_toml(manifest)
}
