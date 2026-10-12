// 源码结构守卫：固定可选功能解析器的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_pending_feature_facade_stays_structural() {
    assert!(
        PARSER_PENDING_FEATURE.contains("mod entry;")
            && PARSER_PENDING_FEATURE.contains("mod normalize;")
            && PARSER_PENDING_FEATURE.contains("mod output;")
            && PARSER_PENDING_FEATURE.contains("mod static_manifest;"),
        "pending feature parent must remain a structural child-module owner"
    );
    assert!(
        !PARSER_PENDING_FEATURE.contains("fn push_optional_feature")
            && !PARSER_PENDING_FEATURE.contains("feature.take()")
            && !PARSER_PENDING_FEATURE.contains("normalize_optional_feature")
            && !PARSER_PENDING_FEATURE.contains("static_optional_feature_manifest"),
        "pending feature parent must not own finalizer composition or feature state handoff"
    );
    assert!(
        PARSER_PENDING_FEATURE.contains("use entry::push_optional_feature"),
        "pending feature parent should expose the entry child through a re-export"
    );
}
