//! 插件清单携带能力集合，宿主导出与扩展通道在每次调用或注册时检查它；构建时去重排序，但反序列化后的集合仍须按语义查询。
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
#[path = "tests/capability_set.rs"]
mod tests;
