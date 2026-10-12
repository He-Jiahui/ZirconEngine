use super::super::super::pending::push_optional_feature;
use super::super::OptionalFeatureParserState;

pub(in super::super) fn flush_pending_feature(state: &mut OptionalFeatureParserState) {
    push_optional_feature(&mut state.features, &mut state.current_feature);
}
