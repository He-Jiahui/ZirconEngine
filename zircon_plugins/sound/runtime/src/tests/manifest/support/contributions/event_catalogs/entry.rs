use super::{line, state};

pub(super) fn event_catalogs_from_plugin_toml(
    manifest: &str,
) -> Vec<super::super::StaticEventCatalog> {
    let mut parser = state::EventCatalogParserState::default();

    for line in manifest.lines().map(str::trim) {
        line::parse_event_catalog_line(line, &mut parser);
    }
    parser.finish()
}
