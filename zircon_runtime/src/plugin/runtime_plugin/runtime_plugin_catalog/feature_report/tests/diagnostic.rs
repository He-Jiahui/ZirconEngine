use super::*;

#[test]
fn streaming_feature_block_diagnostic_preserves_contract() {
    let block = RuntimePluginFeatureBlock {
        feature_id: "rendering.shadow".to_string(),
        required: true,
        missing_plugins: vec!["rendering".to_string(), "lighting".to_string()],
        missing_capabilities: vec!["render.shadow".to_string(), "gpu.compute".to_string()],
        target_unsupported: true,
        cycle: true,
        invalid_owner_dependency: true,
        provider_missing: true,
        unknown_feature: true,
        ..RuntimePluginFeatureBlock::default()
    };

    let diagnostic = block.to_diagnostic();
    assert_eq!(
        diagnostic,
        "required feature rendering.shadow is blocked: feature is not declared by the plugin catalog; owner dependency is missing, not marked primary, or not the only primary dependency; concrete runtime feature provider registration is missing; target mode is not supported; missing plugins: rendering, lighting; missing capabilities: render.shadow, gpu.compute; feature capability dependencies form a cycle"
    );
    assert_eq!(diagnostic.len(), diagnostic.capacity());

    assert_eq!(
        RuntimePluginFeatureBlock {
            feature_id: "audio.spatial".to_string(),
            ..RuntimePluginFeatureBlock::default()
        }
        .to_diagnostic(),
        "optional feature audio.spatial is blocked: dependency status is unresolved"
    );
}
