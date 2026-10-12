use super::super::super::line::parse_optional_feature_dependency_line;
use super::super::OptionalFeatureParserState;

pub(super) fn parse_dependency_section_line(state: &mut OptionalFeatureParserState, line: &str) {
    parse_optional_feature_dependency_line(
        line,
        &mut state.current_dependency_plugin_id,
        &mut state.current_dependency_capability,
        &mut state.current_dependency_primary,
    );
}
