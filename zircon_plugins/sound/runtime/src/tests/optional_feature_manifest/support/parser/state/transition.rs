mod dependency;
mod feature;
mod module;
mod pending_rows;

use super::super::section::OptionalFeatureSection;
use super::{flush, OptionalFeatureParserState};

pub(super) fn enter_section(
    state: &mut OptionalFeatureParserState,
    section: OptionalFeatureSection,
) {
    match section {
        OptionalFeatureSection::Feature => feature::start_optional_feature(state),
        OptionalFeatureSection::Dependency => dependency::start_dependency(state),
        OptionalFeatureSection::Module => module::start_module(state),
        OptionalFeatureSection::None => flush::close_optional_feature_scope(state),
    }
}
