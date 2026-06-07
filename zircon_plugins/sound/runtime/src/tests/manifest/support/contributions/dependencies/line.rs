mod fields;
mod table;

use super::state::DependencyParserState;

pub(super) fn parse_dependency_line(line: &str, parser: &mut DependencyParserState) {
    if table::dependency_table_transition_consumed(line, parser) {
        return;
    }
    fields::parse_dependency_fields(line, parser);
}
