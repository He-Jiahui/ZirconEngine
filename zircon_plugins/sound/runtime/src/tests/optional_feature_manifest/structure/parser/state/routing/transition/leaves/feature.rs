// 源码结构守卫：固定可选功能解析器的子层交接；运行清单语义由 parity 测试验证。
use super::super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_transition_feature_leaf_starts_pending_feature() {
    assert!(
        PARSER_STATE_TRANSITION_FEATURE
            .contains("state.current_feature = Some(Default::default())"),
        "parser state transition feature leaf should own pending-feature start"
    );
}
