use super::super::super::line::parse_optional_feature_module_line;
use super::super::OptionalFeatureParserState;

pub(super) fn parse_module_section_line(state: &mut OptionalFeatureParserState, line: &str) {
    parse_optional_feature_module_line(
        line,
        &mut state.current_module_name,
        &mut state.current_module_kind,
        &mut state.current_module_crate_name,
        &mut state.current_module_target_modes,
        &mut state.current_module_capabilities,
    );
}
