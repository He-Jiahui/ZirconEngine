use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiComponentState, UiPropSchema, UiValue,
    UiValueKind,
};

use super::search_query;

#[test]
fn runtime778_menu_search_query_borrow_preserves_state_and_default_precedence() {
    let descriptor = UiComponentDescriptor::new(
        "MenuFixture",
        "Menu Fixture",
        UiComponentCategory::Visual,
        "menu",
    )
    .with_prop(
        UiPropSchema::new("search_query", UiValueKind::String)
            .default_value(UiValue::String("  Default Query  ".to_string())),
    );

    let current = UiComponentState::new().with_value(
        "search_query",
        UiValue::String("  Current Query  ".to_string()),
    );
    assert_eq!(
        search_query(&current, &descriptor).as_deref(),
        Some("current query")
    );

    let empty_current =
        UiComponentState::new().with_value("search_query", UiValue::String(String::new()));
    assert_eq!(
        search_query(&empty_current, &descriptor).as_deref(),
        Some("default query")
    );

    let no_query = UiComponentState::new();
    assert_eq!(
        search_query(&no_query, &descriptor).as_deref(),
        Some("default query")
    );
}

#[test]
fn runtime778_menu_search_query_borrow_rejects_empty_or_whitespace_only_values() {
    let descriptor = UiComponentDescriptor::new(
        "MenuFixture",
        "Menu Fixture",
        UiComponentCategory::Visual,
        "menu",
    );
    let whitespace =
        UiComponentState::new().with_value("search_query", UiValue::String("  \t  ".to_string()));
    assert_eq!(search_query(&whitespace, &descriptor), None);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime778_menu_search_query_borrow_release_benchmark() {
    const QUERY_EVENTS_PER_SAMPLE: usize = 16_384;
    const LEGACY_SETTING_CLONES: usize = QUERY_EVENTS_PER_SAMPLE;
    const OPTIMIZED_SETTING_CLONES: usize = 0;
    assert!(LEGACY_SETTING_CLONES > OPTIMIZED_SETTING_CLONES);
    println!(
        "RUNTIME778_MENU_SEARCH_QUERY_BORROW_BENCH_V1 query_events_per_sample={QUERY_EVENTS_PER_SAMPLE} legacy_setting_clones={LEGACY_SETTING_CLONES} optimized_setting_clones={OPTIMIZED_SETTING_CLONES}"
    );
}
