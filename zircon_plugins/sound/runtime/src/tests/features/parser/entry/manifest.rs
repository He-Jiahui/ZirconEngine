// 源码结构守卫：固定可选功能解析器的子层交接；运行清单语义由 parity 测试验证。
use super::super::super::sources::*;

#[test]
fn optional_feature_parser_entry_manifest_owner_scans_lines() {
    assert!(
        PARSER_ENTRY.contains("OptionalFeatureParserState::default()")
            && PARSER_ENTRY.contains("manifest.lines().map(str::trim)")
            && PARSER_ENTRY.contains("parser.parse_manifest_line(line)")
            && PARSER_ENTRY.contains("parser.finish()"),
        "parser entry child should own scanner state lifecycle and manifest line iteration"
    );
}
