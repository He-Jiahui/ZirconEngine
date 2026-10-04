// 源码结构守卫：固定可选功能解析器的声明归属；运行清单语义由 parity 测试验证。
use super::super::super::sources::*;

#[test]
fn optional_feature_parser_section_table_header_owner_routes_static_tables() {
    assert!(
        PARSER_SECTION_TABLE_HEADER.contains("impl OptionalFeatureSection")
            && PARSER_SECTION_TABLE_HEADER.contains("fn from_table_header")
            && PARSER_SECTION_TABLE_HEADER.contains("\"[[optional_features]]\"")
            && PARSER_SECTION_TABLE_HEADER.contains("\"[[optional_features.dependencies]]\"")
            && PARSER_SECTION_TABLE_HEADER.contains("\"[[optional_features.modules]]\"")
            && PARSER_SECTION_TABLE_HEADER.contains("_ if line.starts_with(\"[[\")"),
        "parser section table-header child should own static TOML table routing"
    );
}
