use super::super::super::section::OptionalFeatureSection;
use super::super::{flush, OptionalFeatureParserState};

pub(in super::super) fn enter_section(
    state: &mut OptionalFeatureParserState,
    section: OptionalFeatureSection,
) {
    match section {
        OptionalFeatureSection::Feature => super::feature::start_optional_feature(state),
        OptionalFeatureSection::Dependency => super::dependency::start_dependency(state),
        OptionalFeatureSection::Module => super::module::start_module(state),
        OptionalFeatureSection::None => flush::close_optional_feature_scope(state),
    }
}
