use super::super::super::super::state::EventCatalogParserState;

pub(super) fn set_event_catalog_namespace(parser: &mut EventCatalogParserState, namespace: String) {
    parser.set_namespace(namespace);
}
