#[test]
fn persistent_animation_mutations_prepare_a_snapshot_before_opening_history() {
    let source = include_str!("../editing.rs");
    let body = source
        .split("fn apply_animation_document_mutation")
        .nth(1)
        .expect("persistent animation mutation helper")
        .split("fn animation_document_for_instance")
        .next()
        .expect("persistent animation mutation helper body");

    assert!(body.contains("prepare_mutation"));
    assert!(body.contains("HistoryContextId::Document"));
    assert!(body.contains("commit_after_apply"));
    assert!(!body.contains("ensure_document_external_effect"));
}
