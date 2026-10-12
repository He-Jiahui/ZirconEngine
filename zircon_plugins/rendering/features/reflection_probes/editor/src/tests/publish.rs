#[test]
fn project_publication_consumes_encoded_source_through_runtime_transaction_owner() {
    let source = include_str!("../publication.rs");

    assert!(source.contains("source.into_parts()"));
    assert!(source.contains("publish_generated_project_source(output_uri, bytes)"));
    assert!(!source.contains("std::fs"));
    assert!(!source.contains("ResourceRecord::new"));
}
