use super::super::super::pending::push_optional_feature_module;
use super::super::OptionalFeatureParserState;

pub(in super::super) fn flush_pending_module(state: &mut OptionalFeatureParserState) {
    push_optional_feature_module(
        &mut state.current_feature,
        &mut state.current_module_name,
        &mut state.current_module_kind,
        &mut state.current_module_crate_name,
        &mut state.current_module_target_modes,
        &mut state.current_module_capabilities,
    );
}
