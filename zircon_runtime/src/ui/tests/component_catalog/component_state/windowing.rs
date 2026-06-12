use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventKind, UiComponentState, UiValue,
};

#[test]
fn material_pagination_events_update_retained_page_window() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let pagination = registry
        .descriptor("Pagination")
        .expect("Pagination descriptor");
    assert!(pagination.supports_event(UiComponentEventKind::SetPage));

    let mut state = UiComponentState::new().with_value("total_count", UiValue::Int(25));
    state
        .apply_event(
            pagination,
            UiComponentEvent::SetPage {
                page_index: 8,
                page_size: 10,
            },
        )
        .unwrap();

    assert_eq!(state.value("page_size"), Some(&UiValue::Int(10)));
    assert_eq!(state.value("page_count"), Some(&UiValue::Int(3)));
    assert_eq!(state.value("page_index"), Some(&UiValue::Int(2)));
    assert_eq!(state.value("page_start"), Some(&UiValue::Int(20)));
    assert_eq!(state.value("page_end"), Some(&UiValue::Int(25)));
    assert_eq!(state.value("empty"), Some(&UiValue::Bool(false)));
}

#[test]
fn material_pagination_window_handles_empty_and_invalid_page_size() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let table_pagination = registry
        .descriptor("TablePagination")
        .expect("TablePagination descriptor");
    assert!(table_pagination.supports_event(UiComponentEventKind::SetPage));

    let mut state = UiComponentState::new().with_value("total_count", UiValue::Int(0));
    state
        .apply_event(
            table_pagination,
            UiComponentEvent::SetPage {
                page_index: 4,
                page_size: 0,
            },
        )
        .unwrap();

    assert_eq!(state.value("page_size"), Some(&UiValue::Int(1)));
    assert_eq!(state.value("page_count"), Some(&UiValue::Int(0)));
    assert_eq!(state.value("page_index"), Some(&UiValue::Int(0)));
    assert_eq!(state.value("page_start"), Some(&UiValue::Int(0)));
    assert_eq!(state.value("page_end"), Some(&UiValue::Int(0)));
    assert_eq!(state.value("empty"), Some(&UiValue::Bool(true)));
}
