#[test]
fn delivery_page_json_projection_stays_outside_the_listener_lock_scope() {
    let source = include_str!("../listener_control.rs");
    let page_body = source
        .split("EditorEventListenerControlRequest::QueryDeliveriesPage")
        .nth(1)
        .expect("delivery page control branch should remain available");
    let lock_scope_end = page_body
        .find("};\n                let page")
        .expect("listener handle must be captured before the page result");
    let projection = page_body
        .find("listener_delivery_json")
        .expect("delivery JSON projection should remain explicit");
    assert!(projection > lock_scope_end);
}
