use super::super::super::super::state::DependencyParserState;

pub(super) fn set_dependency_required(parser: &mut DependencyParserState, required: bool) {
    parser.set_required(required);
}
