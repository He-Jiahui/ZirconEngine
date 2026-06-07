use super::super::super::super::types::StaticOptionalFeatureManifest;
use super::super::{flush, OptionalFeatureParserState};

impl OptionalFeatureParserState {
    pub(in super::super::super) fn finish(mut self) -> Vec<StaticOptionalFeatureManifest> {
        flush::close_optional_feature_scope(&mut self);
        self.features
    }
}
