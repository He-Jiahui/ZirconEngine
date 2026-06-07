use super::super::state::CapabilityStatusParserState;

pub(super) fn capability_status_table_transition_consumed(
    line: &str,
    parser: &mut CapabilityStatusParserState,
) -> bool {
    if line == "[[capability_statuses]]" {
        parser.begin_status_table();
        return true;
    }
    if line.starts_with("[[") {
        parser.leave_status_table();
    }
    !parser.is_inside_status()
}
