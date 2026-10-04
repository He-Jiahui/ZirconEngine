use super::CapabilitySet;

#[test]
fn with_keeps_capabilities_sorted_and_unique() {
    let capabilities = CapabilitySet::default()
        .with("runtime.script.extension.rpc_handler")
        .with("runtime.script.extension.bt_node")
        .with("runtime.script.extension.rpc_handler");

    assert_eq!(
        capabilities.capabilities,
        vec![
            "runtime.script.extension.bt_node",
            "runtime.script.extension.rpc_handler",
        ]
    );
}

#[test]
fn with_repairs_externally_populated_capabilities() {
    let capabilities = CapabilitySet {
        capabilities: vec![
            "runtime.script.extension.rpc_handler".to_string(),
            "runtime.script.extension.bt_node".to_string(),
            "runtime.script.extension.bt_node".to_string(),
        ],
    }
    .with("runtime.script.extension.editor_operation");

    assert_eq!(
        capabilities.capabilities,
        vec![
            "runtime.script.extension.bt_node",
            "runtime.script.extension.editor_operation",
            "runtime.script.extension.rpc_handler",
        ]
    );
}

#[test]
fn contains_accepts_manifest_order_without_sorted_storage() {
    let capabilities = CapabilitySet {
        capabilities: vec![
            "runtime.script.extension.system".to_string(),
            "runtime.script.extension.bt_node".to_string(),
            "runtime.script.extension.rpc_handler".to_string(),
            "runtime.script.extension.editor_operation".to_string(),
        ],
    };

    for capability in &capabilities.capabilities {
        assert!(capabilities.contains(capability));
    }
}
