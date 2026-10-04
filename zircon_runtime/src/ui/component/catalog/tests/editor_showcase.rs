#[test]
fn editor_showcase_catalog_builds_on_small_stack() {
    std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(|| {
            let registry = super::build_editor_showcase_registry();
            assert!(registry.len() >= 40);
            assert!(registry.contains("Container"));
            assert!(registry.contains("ContextActionMenu"));
        })
        .expect("spawn small-stack showcase catalog test")
        .join()
        .expect("showcase catalog should not overflow the stack");
}
