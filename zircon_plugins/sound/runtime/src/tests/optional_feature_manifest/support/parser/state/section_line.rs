mod dependency;
mod feature;
mod module;

use super::super::section::OptionalFeatureSection;
use super::OptionalFeatureParserState;

pub(super) fn parse_section_line(state: &mut OptionalFeatureParserState, line: &str) {
    match state.section {
        OptionalFeatureSection::Feature => feature::parse_feature_section_line(state, line),
        OptionalFeatureSection::Dependency => {
            dependency::parse_dependency_section_line(state, line)
        }
        OptionalFeatureSection::Module => module::parse_module_section_line(state, line),
        OptionalFeatureSection::None => {}
    }
}
