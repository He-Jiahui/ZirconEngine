//! 清单中的内存限额先验证配置关系，再随包传给后端；限额执行需要后端配合。
use serde::{Deserialize, Serialize};

use super::{VmPluginManagementPolicyError, VmPluginManagementPolicyResult};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
// TODO: [CR-SCRIPT-AUDIT-0005] 确认各后端是否执行清单的内存软硬限额；当前只见配置校验与解析测试，未见运行期读取限额或超限测试，需明确支持契约。
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

    pub fn validate(&self) -> VmPluginManagementPolicyResult<()> {
        if self.soft_limit_bytes == Some(0) {
            return Err(VmPluginManagementPolicyError::MemorySoftLimitBytesZero);
        }
        if self.hard_limit_bytes == Some(0) {
            return Err(VmPluginManagementPolicyError::MemoryHardLimitBytesZero);
        }
        if let (Some(soft), Some(hard)) = (self.soft_limit_bytes, self.hard_limit_bytes) {
            if soft > hard {
                return Err(
                    VmPluginManagementPolicyError::MemorySoftLimitExceedsHardLimit {
                        soft_limit_bytes: soft,
                        hard_limit_bytes: hard,
                    },
                );
            }
        }
        Ok(())
    }
}
