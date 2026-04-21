#[test]
fn editor_template_runtime_splits_builtin_data_from_runtime_pipeline() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("template_runtime");

    for relative in [
        "runtime/mod.rs",
        "runtime/runtime_host.rs",
        "runtime/build_session.rs",
        "runtime/projection.rs",
        "builtin/mod.rs",
        "builtin/template_documents.rs",
        "builtin/template_bindings.rs",
        "builtin/component_descriptors.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected template runtime module {relative} under {:?}",
            root
        );
    }
}
