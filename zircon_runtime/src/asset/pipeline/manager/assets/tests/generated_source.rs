#[test]
fn generated_source_publication_uses_project_transaction_and_resource_owner() {
    let source = include_str!("../generated_source.rs");

    assert!(source.contains("prepare_generated_source_generation("));
    assert!(source.contains("prepare_targeted_project_resource_sync("));
    assert!(source.contains("commit_targeted_project_resource_sync("));
    assert!(source.contains("prepared_generation.commit()"));
    assert!(source.contains("register_transaction_watch_echoes"));
    assert!(source.contains("publish_project_generation("));
    assert!(!source.contains("source_bytes.clone()"));
    assert!(!source.contains("fs::write("));
    assert!(!source.contains("fs::read("));
}
