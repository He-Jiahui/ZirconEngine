use crate::ui::template::{
    UiAssetError, UiSelector, UiSelectorCombinator, UiSelectorSegment, UiSelectorSpecificity,
    UiSelectorToken,
};

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

#[test]
fn ui_selector_specificity_contract_uses_public_template_api() {
    let selector = UiSelector::parse("Button.primary#confirm:part(label) > Label").unwrap();
    let specificity = selector.specificity();

    assert_eq!(specificity, UiSelectorSpecificity::new(1, 2, 2));
    assert_eq!(specificity.legacy_display_score(), 122);
    assert!(UiSelectorSpecificity::new(1, 0, 0) > UiSelectorSpecificity::new(0, 99, 99));
}
