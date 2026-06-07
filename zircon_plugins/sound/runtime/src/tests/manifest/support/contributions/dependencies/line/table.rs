use super::super::state::DependencyParserState;

pub(super) fn dependency_table_transition_consumed(
    line: &str,
    parser: &mut DependencyParserState,
) -> bool {
    if line == "[[dependencies]]" {
        parser.begin_dependency_table();
        return true;
    }
    if line.starts_with("[[") {
        parser.leave_dependency_table();
    }
    !parser.is_inside_dependency()
}
