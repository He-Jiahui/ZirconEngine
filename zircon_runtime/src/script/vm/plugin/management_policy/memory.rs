use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmPluginMemoryPolicy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soft_limit_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hard_limit_bytes: Option<u64>,
}

impl VmPluginMemoryPolicy {
    pub fn with_limits(soft_limit_bytes: Option<u64>, hard_limit_bytes: Option<u64>) -> Self {
        Self {
            soft_limit_bytes,
            hard_limit_bytes,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.soft_limit_bytes == Some(0) {
            return Err("memory soft_limit_bytes must be greater than zero".to_string());
        }
        if self.hard_limit_bytes == Some(0) {
            return Err("memory hard_limit_bytes must be greater than zero".to_string());
        }
        if let (Some(soft), Some(hard)) = (self.soft_limit_bytes, self.hard_limit_bytes) {
            if soft > hard {
                return Err(format!(
                    "memory soft_limit_bytes {soft} exceeds hard_limit_bytes {hard}"
                ));
            }
        }
        Ok(())
    }
}
