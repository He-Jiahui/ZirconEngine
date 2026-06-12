use super::super::super::section::OptionalFeatureSection;
use super::super::OptionalFeatureParserState;

pub(in super::super) fn parse_section_line(state: &mut OptionalFeatureParserState, line: &str) {
    match state.section {
        OptionalFeatureSection::Feature => super::feature::parse_feature_section_line(state, line),
        OptionalFeatureSection::Dependency => {
            super::dependency::parse_dependency_section_line(state, line)
        }
        OptionalFeatureSection::Module => super::module::parse_module_section_line(state, line),
        OptionalFeatureSection::None => {}
    }
}
