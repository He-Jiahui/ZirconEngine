// 源码结构守卫：固定可选功能解析器的子层交接；运行清单语义由 parity 测试验证。
use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_dependency_line_entry_owner_forwards_to_dispatch() {
    assert!(
        PARSER_LINE_DEPENDENCY_ENTRY
            .contains("pub(in super::super::super) fn parse_optional_feature_dependency_line")
            && PARSER_LINE_DEPENDENCY_ENTRY.contains("super::dispatch::parse_dependency_line"),
        "parser dependency-line entry child should own the facade-visible dispatch handoff"
    );
}
