use super::*;

#[test]
fn registration_deduplicates_only_the_same_published_frame_arc() {
    let frame = Arc::new(UiSurfaceFrame::default());
    let equivalent_but_distinct = Arc::new(frame.as_ref().clone());
    let mut table = HostRenderSourceTable::default();

    let first = table.register(&frame).expect("first source key");
    let repeated = table.register(&frame).expect("repeated source key");
    let distinct = table
        .register(&equivalent_but_distinct)
        .expect("distinct source key");

    assert_eq!(first, repeated);
    assert_ne!(first, distinct);
    assert_eq!(table.len(), 2);
    assert!(table
        .resolve(first)
        .is_some_and(|resolved| Arc::ptr_eq(resolved, &frame)));
}
