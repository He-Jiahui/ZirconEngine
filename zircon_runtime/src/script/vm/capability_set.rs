use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub capabilities: Vec<String>,
}

impl CapabilitySet {
    pub fn with(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self.capabilities.sort();
        self.capabilities.dedup();
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
