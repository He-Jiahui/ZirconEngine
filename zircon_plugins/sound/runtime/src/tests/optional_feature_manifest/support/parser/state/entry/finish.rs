// 输入结束也必须完成最后的依赖、模块和功能，否则静态签名会少一项。
use super::super::super::super::types::StaticOptionalFeatureManifest;
use super::super::{flush, OptionalFeatureParserState};

impl OptionalFeatureParserState {
    pub(in super::super::super) fn finish(mut self) -> Vec<StaticOptionalFeatureManifest> {
        flush::close_optional_feature_scope(&mut self);
        self.features
    }
}
