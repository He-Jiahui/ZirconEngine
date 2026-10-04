#[test]
fn publication_commits_only_the_copy_already_bound_to_the_frame_identity() {
    let source = include_str!("../publish_viewport_product.rs");

    assert!(source.contains("frame.take_viewport_product_copy()"));
    assert!(source.contains("frame.submission_receipt()"));
    assert!(source.contains("viewport_products.publish("));
    assert!(!source.contains("replace_submission_receipt"));
    assert!(!source.contains("retain_viewport_product_submission_receipt"));
}
