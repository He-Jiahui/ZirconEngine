#[path = "../../manifest/support/event_catalogs/entry.rs"]
mod entry;
#[path = "../../manifest/support/event_catalogs/line.rs"]
mod line;
#[path = "../../manifest/support/event_catalogs/state.rs"]
mod state;

use super::StaticEventCatalog;

pub(super) fn event_catalogs_from_plugin_toml(manifest: &str) -> Vec<StaticEventCatalog> {
    entry::event_catalogs_from_plugin_toml(manifest)
}
