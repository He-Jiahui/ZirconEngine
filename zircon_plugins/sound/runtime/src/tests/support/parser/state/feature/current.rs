use super::super::super::super::super::types::PendingOptionalFeatureManifest;
use super::super::super::OptionalFeatureParserState;

pub(super) fn required_current_feature(
    state: &mut OptionalFeatureParserState,
) -> &mut PendingOptionalFeatureManifest {
    state
        .current_feature
        .as_mut()
        .expect("optional feature table should have a current feature")
}
