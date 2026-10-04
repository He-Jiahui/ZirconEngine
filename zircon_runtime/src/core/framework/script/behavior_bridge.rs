use crate::core::framework::bridge::PluginInterface;

use super::{ScriptHostError, ScriptHostValue};

pub const SCRIPT_BEHAVIOR_BRIDGE_INTERFACE_ID: &str = "script.behavior.v1";

/// Stable, provider-qualified asset reference for one script-owned behavior callback.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScriptBehaviorCallbackRef {
    package_id: String,
    node_id: String,
}

impl ScriptBehaviorCallbackRef {
    // BUG: [CR-FRAMEWORK-TEXTSCRIPT-0001] new("a::b", "c") 可成功，但 stable_id 后再 parse 会变成 ("a", "b::c")；VM 按两个字段查找回调，身份不再往返一致。
    /// 构造供 VM 桥接查找的包名与节点名；调用方应提供稳定的提供者身份。
    pub fn new(
        package_id: impl Into<String>,
        node_id: impl Into<String>,
    ) -> Result<Self, ScriptHostError> {
        let package_id = package_id.into();
        let node_id = node_id.into();
        if package_id.is_empty() || package_id.trim() != package_id {
            return Err(ScriptHostError::new(
                "script behavior callback package id must be non-empty and trimmed",
            ));
        }
        if node_id.is_empty() || node_id.trim() != node_id {
            return Err(ScriptHostError::new(
                "script behavior callback node id must be non-empty and trimmed",
            ));
        }
        Ok(Self {
            package_id,
            node_id,
        })
    }

    /// 从对外标识拆分提供者与节点；随后 invoke 会按这两个字段解析活动 VM 包和注册节点。
    pub fn parse(value: &str) -> Result<Self, ScriptHostError> {
        let Some((package_id, node_id)) = value.split_once("::") else {
            return Err(ScriptHostError::new(
                "script behavior callback must use `<package>::<node-id>`",
            ));
        };
        Self::new(package_id, node_id)
    }

    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn stable_id(&self) -> String {
        let mut stable_id = String::with_capacity(self.package_id.len() + 2 + self.node_id.len());
        stable_id.push_str(&self.package_id);
        stable_id.push_str("::");
        stable_id.push_str(&self.node_id);
        stable_id
    }
}

/// Neutral call boundary implemented by the script owner and consumed by AI or other plugins.
pub trait ScriptBehaviorBridge: Send + Sync + 'static {
    /// 对活动包的已注册节点发起同步调用；实现者负责处理热重载后的句柄代际。
    fn invoke(
        &self,
        callback: &ScriptBehaviorCallbackRef,
        arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, ScriptHostError>;
}

impl PluginInterface for dyn ScriptBehaviorBridge {
    const INTERFACE_ID: &'static str = SCRIPT_BEHAVIOR_BRIDGE_INTERFACE_ID;
}

#[cfg(test)]
#[path = "tests/behavior_bridge.rs"]
mod tests;
