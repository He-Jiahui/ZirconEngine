// 源码结构守卫：固定可选功能解析器的字段或值的归属；运行清单语义由 parity 测试验证。
use super::super::super::sources::*;

#[test]
fn optional_feature_parser_line_field_quoted_owner_strips_quote_suffix() {
    assert!(
        PARSER_LINE_FIELD_QUOTED.contains("super::raw::raw_value(line, prefix)")
            && PARSER_LINE_FIELD_QUOTED.contains("strip_suffix('\"')"),
        "quoted field child should own quote suffix stripping over the raw child"
    );
}
