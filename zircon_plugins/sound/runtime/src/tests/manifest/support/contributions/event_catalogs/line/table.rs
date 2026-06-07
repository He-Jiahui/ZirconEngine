use super::super::state::EventCatalogParserState;

pub(super) fn event_catalog_table_transition_consumed(
    line: &str,
    parser: &mut EventCatalogParserState,
) -> bool {
    if line == "[[event_catalogs]]" {
        parser.begin_event_catalog_table();
        return true;
    }
    if line.starts_with("[[") {
        parser.leave_event_catalog_table();
    }
    !parser.is_inside_event_catalog()
}
