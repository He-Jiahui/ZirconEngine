// 源码结构守卫：固定可选功能解析器的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::sources::*;

#[test]
fn optional_feature_parser_line_field_facade_stays_split_from_extractors() {
    assert!(
        PARSER_LINE_FIELD.contains("mod bracketed;")
            && PARSER_LINE_FIELD.contains("mod quoted;")
            && PARSER_LINE_FIELD.contains("mod raw;"),
        "parser line field parent must remain a structural child-module owner"
    );
    assert!(
        !PARSER_LINE_FIELD.contains("fn raw_value")
            && !PARSER_LINE_FIELD.contains("fn quoted_value")
            && !PARSER_LINE_FIELD.contains("fn bracketed_value")
            && !PARSER_LINE_FIELD.contains("strip_prefix")
            && !PARSER_LINE_FIELD.contains("strip_suffix"),
        "parser line field parent must not own TOML field extraction behavior"
    );
    assert!(
        PARSER_LINE_FIELD.contains("use bracketed::bracketed_value")
            && PARSER_LINE_FIELD.contains("use quoted::quoted_value")
            && PARSER_LINE_FIELD.contains("use raw::raw_value"),
        "parser line field parent should expose field extractors through child re-exports"
    );
}
