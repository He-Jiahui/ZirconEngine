use super::super::super::section::OptionalFeatureSection;
use super::super::{flush, OptionalFeatureParserState};

pub(super) fn start_optional_feature(state: &mut OptionalFeatureParserState) {
    flush::close_optional_feature_scope(state);
    state.current_feature = Some(Default::default());
    state.section = OptionalFeatureSection::Feature;
}
