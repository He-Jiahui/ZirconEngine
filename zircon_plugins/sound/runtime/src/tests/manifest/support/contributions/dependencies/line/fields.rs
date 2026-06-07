mod capability;
mod id;
mod required;

use super::super::state::DependencyParserState;

pub(super) fn parse_dependency_fields(line: &str, parser: &mut DependencyParserState) {
    if id::parse_dependency_id_field(line, parser) {
        return;
    }
    if required::parse_dependency_required_field(line, parser) {
        return;
    }
    capability::parse_dependency_capability_field(line, parser);
}
