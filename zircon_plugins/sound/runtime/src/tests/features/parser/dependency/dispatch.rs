// 源码结构守卫：固定可选功能解析器的子层交接；运行清单语义由 parity 测试验证。
use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_dependency_line_dispatch_owner_routes_fields() {
    assert!(
        PARSER_LINE_DEPENDENCY_DISPATCH.contains("identity::parse_dependency_identity_line")
            && PARSER_LINE_DEPENDENCY_DISPATCH.contains("primary::parse_dependency_primary_line"),
        "parser dependency-line dispatch child should own ordered dependency-field routing"
    );
}
