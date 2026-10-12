use super::super::super::pending::push_optional_feature_dependency;
use super::super::OptionalFeatureParserState;

pub(in super::super) fn flush_pending_dependency(state: &mut OptionalFeatureParserState) {
    push_optional_feature_dependency(
        &mut state.current_feature,
        &mut state.current_dependency_plugin_id,
        &mut state.current_dependency_capability,
        &mut state.current_dependency_primary,
    );
}
