use super::super::super::section::OptionalFeatureSection;
use super::super::OptionalFeatureParserState;
use super::{dependency, feature, module};

pub(in super::super) fn close_optional_feature_scope(state: &mut OptionalFeatureParserState) {
    dependency::flush_pending_dependency(state);
    module::flush_pending_module(state);
    feature::flush_pending_feature(state);
    state.section = OptionalFeatureSection::None;
}
