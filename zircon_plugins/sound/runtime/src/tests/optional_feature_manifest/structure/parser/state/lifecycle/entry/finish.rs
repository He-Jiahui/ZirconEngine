// 源码结构守卫：固定可选功能解析器的子层交接；运行清单语义由 parity 测试验证。
use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_entry_finish_child_owns_final_flush_and_output() {
    assert!(
        PARSER_STATE_ENTRY_FINISH.contains("impl OptionalFeatureParserState")
            && PARSER_STATE_ENTRY_FINISH.contains("fn finish")
            && PARSER_STATE_ENTRY_FINISH.contains("flush::close_optional_feature_scope")
            && PARSER_STATE_ENTRY_FINISH.contains("self.features"),
        "parser state finish child should own final flush and output handoff"
    );
}
