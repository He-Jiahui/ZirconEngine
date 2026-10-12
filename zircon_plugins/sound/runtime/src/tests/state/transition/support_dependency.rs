use super::super::super::section::OptionalFeatureSection;
use super::super::OptionalFeatureParserState;
use super::pending_rows;

pub(super) fn start_dependency(state: &mut OptionalFeatureParserState) {
    pending_rows::flush_dependency_and_module_rows(state);
    state.section = OptionalFeatureSection::Dependency;
}
