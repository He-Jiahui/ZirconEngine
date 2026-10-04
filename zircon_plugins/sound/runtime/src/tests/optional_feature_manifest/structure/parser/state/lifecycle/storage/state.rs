// 源码结构守卫：固定可选功能解析器的状态与提交边界；运行清单语义由 parity 测试验证。
use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_storage_state_child_owns_scanner_fields() {
    assert!(
        PARSER_STATE_STORAGE.contains("struct OptionalFeatureParserState")
            && PARSER_STATE_STORAGE.contains("features: Vec<StaticOptionalFeatureManifest>")
            && PARSER_STATE_STORAGE
                .contains("current_feature: Option<PendingOptionalFeatureManifest>")
            && PARSER_STATE_STORAGE.contains("section: OptionalFeatureSection"),
        "parser state storage child should own scanner state fields"
    );
}
