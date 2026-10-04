use super::PluginFeatureBundleBuilder;

#[test]
fn capability_batch_appends_after_existing_in_input_order() {
    let manifest = PluginFeatureBundleBuilder::new("feature", "Feature", "owner")
        .with_capability("runtime.existing")
        .with_capabilities(["runtime.batch.first", "runtime.batch.second"])
        .build();

    assert_eq!(
        manifest.capabilities,
        [
            "runtime.existing",
            "runtime.batch.first",
            "runtime.batch.second",
        ]
    );
}
