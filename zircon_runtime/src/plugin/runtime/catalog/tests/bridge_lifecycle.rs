use super::*;

#[test]
fn streaming_bridge_lifecycle_block_diagnostic_preserves_contract() {
    let block = RuntimePluginBridgeLifecycleBlock {
        provider_package_id: "rendering".to_string(),
        mode: BridgeOwnerTransitionMode::Disable,
        blockers: vec![
            RuntimePluginBridgeDisableBlocker {
                provider_package_id: "rendering".to_string(),
                dependent_package_id: "editor_a".to_string(),
                interface_ids: vec!["render.api".to_string()],
            },
            RuntimePluginBridgeDisableBlocker {
                provider_package_id: "rendering".to_string(),
                dependent_package_id: "editor_b".to_string(),
                interface_ids: vec!["render.debug".to_string()],
            },
        ],
    };

    assert_eq!(
        block.diagnostic(),
        "bridge.provider_lifecycle_blocked: provider plugin `rendering` Disable blocked by 2 strong dependent(s): bridge.strong_target_disable_blocked: provider plugin `rendering` cannot be disabled while dependent plugin `editor_a` requires interfaces [`render.api`]; bridge.strong_target_disable_blocked: provider plugin `rendering` cannot be disabled while dependent plugin `editor_b` requires interfaces [`render.debug`]"
    );
}
