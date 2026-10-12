// 源码结构守卫：固定可选功能解析器的字段或值的归属；运行清单语义由 parity 测试验证。
use super::super::super::sources::*;

#[test]
fn optional_feature_parser_line_field_raw_owner_strips_prefix() {
    assert!(
        PARSER_LINE_FIELD_RAW.contains("line.strip_prefix(prefix)"),
        "raw field child should own prefix stripping"
    );
}
