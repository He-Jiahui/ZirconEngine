mod fields;
mod table;

use super::state::EventCatalogParserState;

pub(super) fn parse_event_catalog_line(line: &str, parser: &mut EventCatalogParserState) {
    if table::event_catalog_table_transition_consumed(line, parser) {
        return;
    }
    fields::parse_event_catalog_fields(line, parser);
}
