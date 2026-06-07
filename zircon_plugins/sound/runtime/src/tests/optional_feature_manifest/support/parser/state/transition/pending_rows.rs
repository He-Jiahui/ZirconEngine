use super::super::{flush, OptionalFeatureParserState};

pub(super) fn flush_dependency_and_module_rows(state: &mut OptionalFeatureParserState) {
    flush::flush_pending_dependency(state);
    flush::flush_pending_module(state);
}
