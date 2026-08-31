use serde::{Deserialize, Serialize};

fn insert_sorted_unique(values: &mut Vec<String>, value: String) {
    if values.windows(2).all(|pair| pair[0] < pair[1]) {
        if let Err(index) = values.binary_search(&value) {
            values.insert(index, value);
        }
        return;
    }

    values.push(value);
    values.sort();
    values.dedup();
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub capabilities: Vec<String>,
}

impl CapabilitySet {
    pub fn with(mut self, capability: impl Into<String>) -> Self {
        insert_sorted_unique(&mut self.capabilities, capability.into());
        self
    }

    /// Returns whether the capability is present, independent of manifest ordering.
    pub fn contains(&self, capability: &str) -> bool {
        self.capabilities
            .iter()
            .any(|candidate| candidate == capability)
    }
}

#[cfg(test)]
mod tests {
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
}
