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
}
