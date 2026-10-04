use super::{retention_class, EditorEvent, EditorEventRetentionClass, EditorViewportEvent};

#[test]
fn cancel_interaction_is_frame_local() {
    assert_eq!(
        retention_class(&EditorEvent::Viewport(
            EditorViewportEvent::CancelInteraction
        )),
        EditorEventRetentionClass::FrameLocal
    );
}

#[test]
fn retention_acknowledgement_and_pages_keep_delivery_cursor_indexed() {
    let source = include_str!("../retention.rs");
    let acknowledge_body = source
        .split("fn acknowledge_through")
        .nth(1)
        .and_then(|body| body.split("fn next_after").next())
        .expect("retention acknowledge body should remain available");
    assert!(acknowledge_body.contains("delivery_order.front"));
    assert!(acknowledge_body.contains("delivery_order.pop_front"));
    assert!(acknowledge_body.contains("delivery_cursor"));
    assert!(!acknowledge_body.contains("retained_by_event_sequence"));
    assert!(!acknowledge_body.contains(".retain("));

    let diagnostics_body = source
        .split("fn diagnostics")
        .nth(1)
        .and_then(|body| {
            body.split("pub(crate) struct EditorEventRetentionStore")
                .next()
        })
        .expect("retention diagnostics body should remain available");
    assert!(diagnostics_body.contains("retained_by_event_sequence"));
    assert!(diagnostics_body.contains("retained_by_age"));

    let page_body = source
        .split("fn records_page_after")
        .nth(1)
        .and_then(|body| body.split("pub(crate) fn acknowledge_through").next())
        .expect("retention page body should remain available");
    assert!(page_body.contains("after_delivery_cursor"));
    assert!(page_body.contains("next_after"));
    assert!(!page_body.contains("sort_unstable"));
    let counting_writer_name = ["Counting", "Writer"].concat();
    assert!(!source.contains(&counting_writer_name));
}
