use serde::{Deserialize, Serialize};

use super::super::{CapabilitySet, PluginSlotId};

/// Stable dense reference to a VM export.
///
/// `module` and `function` are registration-time slots. `generation` is refreshed
/// when the owning package is hot reloaded, while the symbolic target remains in
/// the registry for re-resolution.
/// 重新解析由 VM registry 完成，调用方不能把旧 generation 当作永久有效。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VmCallbackHandle {
    /// Stable owner slot allocated by the VM plugin coordinator.
    pub slot: PluginSlotId,
    /// Dense module slot resolved during registration.
    pub module: u32,
    /// Dense function slot within `module`.
    pub function: u32,
    /// Owner generation used by the last successful resolution.
    pub generation: u32,
}

/// Authenticated identity supplied to a host-interface registration call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmInterfaceCaller {
    /// Package slot that owns the registration.
    pub slot: PluginSlotId,
    /// Package generation performing the registration.
    pub generation: u32,
    /// Capabilities granted by the package manifest.
    pub capabilities: CapabilitySet,
}

impl VmInterfaceCaller {
    /// Creates an authenticated caller from coordinator-owned package state.
    /// 保存注册调用携带的槽位、代际与能力集；构造器本身不认证身份，后续由注册表校验。
    pub fn new(slot: PluginSlotId, generation: u32, capabilities: CapabilitySet) -> Self {
        Self {
            slot,
            generation,
            capabilities,
        }
    }
}
