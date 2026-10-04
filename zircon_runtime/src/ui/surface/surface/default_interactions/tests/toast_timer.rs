use super::*;

#[test]
fn optimization_batch_20260909_runtime274_component_state_toast_id_ref_borrows_value_storage() {
    let state =
        UiComponentState::new().with_value(CURRENT_TOAST_ID, UiValue::String("save".to_string()));
    let source = match state.value(CURRENT_TOAST_ID) {
        Some(UiValue::String(source)) => source.as_str(),
        _ => unreachable!(),
    };

    let parsed =
        string_component_state_value_ref(&state, CURRENT_TOAST_ID).expect("retained toast id");

    assert_eq!(parsed, "save");
    assert!(std::ptr::eq(parsed.as_ptr(), source.as_ptr()));
}

#[test]
fn optimization_batch_20260909_runtime274_toast_timer_source_contract_keeps_owned_conversion_at_boundary(
) {
    let source = include_str!("../toast_timer.rs");
    assert!(source.contains("toast_timer_for_component_node_ref"));
    assert!(source.contains("string_retained_value_ref"));
    assert!(source.contains("map(|(toast_id, timeout_ms)| (toast_id.to_owned(), timeout_ms))"));
}
