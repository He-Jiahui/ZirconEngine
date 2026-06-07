mod fields;
mod table;

use super::state::CapabilityStatusParserState;

pub(super) fn parse_capability_status_line(line: &str, parser: &mut CapabilityStatusParserState) {
    if table::capability_status_table_transition_consumed(line, parser) {
        return;
    }
    fields::parse_capability_status_fields(line, parser);
}
