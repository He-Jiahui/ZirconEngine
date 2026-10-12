use super::super::super::super::state::DependencyParserState;

pub(super) fn set_dependency_id(parser: &mut DependencyParserState, id: String) {
    parser.set_id(id);
}
