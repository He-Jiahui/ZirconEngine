use super::super::super::super::state::EventCatalogParserState;

pub(super) fn set_event_catalog_version(parser: &mut EventCatalogParserState, version: u32) {
    parser.set_version(version);
}
