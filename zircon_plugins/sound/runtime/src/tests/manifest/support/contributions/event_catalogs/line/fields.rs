mod namespace;
mod version;

use super::super::state::EventCatalogParserState;

pub(super) fn parse_event_catalog_fields(line: &str, parser: &mut EventCatalogParserState) {
    if namespace::parse_event_catalog_namespace_field(line, parser) {
        return;
    }
    version::parse_event_catalog_version_field(line, parser);
}
