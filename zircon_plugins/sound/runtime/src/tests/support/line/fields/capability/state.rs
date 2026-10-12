use super::super::super::super::state::DependencyParserState;

pub(super) fn set_dependency_capability(parser: &mut DependencyParserState, capability: String) {
    parser.set_capability(capability);
}
