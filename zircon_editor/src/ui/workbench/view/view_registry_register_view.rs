//! 视图种类的唯一注册入口；重复身份拒绝覆盖，活实例另由打开和恢复链管理。
use std::collections::hash_map::Entry;

use super::{ViewDescriptor, ViewRegistry};

impl ViewRegistry {
    /// 只保证种类键唯一；模板资源、能力与布局宿主是否合法仍由后续消费者检查。
    pub fn register_view(&mut self, descriptor: ViewDescriptor) -> Result<(), String> {
        match self.descriptors.entry(descriptor.descriptor_id.clone()) {
            Entry::Occupied(_) => Err(format!(
                "view descriptor {} already registered",
                descriptor.descriptor_id.0
            )),
            Entry::Vacant(entry) => {
                entry.insert(descriptor);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
#[path = "view_registry_register_view/tests/entry_tests.rs"]
mod entry_tests;
