//! 工作台视图注册表保存声明与具体pane实例，并以单例索引和序号支持打开/恢复。
//! 能力集合来自当前宿主快照；描述符存在和实例可打开是两个独立判断。
use std::collections::{HashMap, HashSet};

use super::{ViewDescriptor, ViewDescriptorId, ViewInstance, ViewInstanceId};

#[derive(Clone, Debug, Default)]
/// 宿主拥有的视图身份表；各写入入口必须保持实例表、单例索引和计数器一致。
pub struct ViewRegistry {
    pub(super) descriptors: HashMap<ViewDescriptorId, ViewDescriptor>,
    pub(super) instances: HashMap<ViewInstanceId, ViewInstance>,
    pub(super) single_instance_index: HashMap<ViewDescriptorId, ViewInstanceId>,
    pub(super) counters: HashMap<ViewDescriptorId, usize>,
    pub(super) available_capabilities: HashSet<String>,
}

impl ViewRegistry {
    /// 用当前宿主能力快照替换可用集合；后续列举、打开与恢复据此重新判断描述符。
    pub fn set_available_capabilities<I, S>(&mut self, capabilities: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.available_capabilities = capabilities.into_iter().map(Into::into).collect();
    }

    /// 报告声明中尚未启用的能力，保持声明顺序；这是展示和打开门槛，不是注册状态证明。
    pub fn descriptor_capability_error(&self, descriptor: &ViewDescriptor) -> Option<String> {
        const PREFIX: &str = "view descriptor ";
        const DISABLED_CAPABILITIES: &str = " requires disabled capabilities: ";

        let mut required_capabilities = descriptor.required_capabilities.iter();
        let first_missing = required_capabilities
            .find(|capability| !self.available_capabilities.contains(capability.as_str()))?;
        let remaining_capacity = required_capabilities
            .clone()
            .map(|capability| capability.len().saturating_add(2))
            .sum::<usize>();
        let mut error = String::with_capacity(
            PREFIX
                .len()
                .saturating_add(descriptor.descriptor_id.0.len())
                .saturating_add(DISABLED_CAPABILITIES.len())
                .saturating_add(first_missing.len())
                .saturating_add(remaining_capacity),
        );
        error.push_str(PREFIX);
        error.push_str(&descriptor.descriptor_id.0);
        error.push_str(DISABLED_CAPABILITIES);
        error.push_str(first_missing);
        for capability in required_capabilities {
            if self.available_capabilities.contains(capability) {
                continue;
            }
            error.push_str(", ");
            error.push_str(capability);
        }
        Some(error)
    }
}

#[cfg(test)]
#[path = "tests/view_registry.rs"]
mod tests;
