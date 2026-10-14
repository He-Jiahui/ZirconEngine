//! 面向 `ui::template` 公共选择器 API 的契约测试。
//!
//! 此处验证解析、Serde 往返、非法尾随组合器和 specificity；不执行 Runtime 节点匹配或样式级联。

use crate::ui::template::{
    UiAssetError, UiSelector, UiSelectorCombinator, UiSelectorSegment, UiSelectorSpecificity,
    UiSelectorToken,
};

/// 解析后的段/token 结构应可序列化往返，且尾随子组合器必须被拒绝。
#[test]
fn ui_selector_contracts_parse_reject_trailing_child_and_serialize() {
    let selector = UiSelector::parse("Button.primary > Label:part(text)").unwrap();
    let serialized = serde_json::to_string(&selector).unwrap();
    let round_trip: UiSelector = serde_json::from_str(&serialized).unwrap();

    assert_eq!(round_trip, selector);
    assert_eq!(selector.segments.len(), 2);
    assert_eq!(
        selector.segments[0],
        UiSelectorSegment {
            combinator: None,
            tokens: vec![
                UiSelectorToken::Type("Button".to_string()),
                UiSelectorToken::Class("primary".to_string())
            ],
        }
    );
    assert_eq!(
        selector.segments[1].combinator,
        Some(UiSelectorCombinator::Child)
    );
    assert!(selector.segments[0]
        .tokens
        .contains(&UiSelectorToken::Class("primary".to_string())));
    assert!(matches!(
        UiSelector::parse("Button >"),
        Err(UiAssetError::InvalidSelector(_))
    ));
}

/// specificity 作为公开三元组比较，ID 优先级高于任意数量的 class-like token，并检查旧显示分值。
#[test]
fn ui_selector_specificity_contract_uses_public_template_api() {
    let selector = UiSelector::parse("Button.primary#confirm:part(label) > Label").unwrap();
    let specificity = selector.specificity();

    assert_eq!(specificity, UiSelectorSpecificity::new(1, 2, 2));
    assert_eq!(specificity.legacy_display_score(), 122);
    assert!(UiSelectorSpecificity::new(1, 0, 0) > UiSelectorSpecificity::new(0, 99, 99));
}
