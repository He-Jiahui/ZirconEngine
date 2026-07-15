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
        format!("{}::{}", self.package_id, self.node_id)
    }
}

/// Neutral call boundary implemented by the script owner and consumed by AI or other plugins.
pub trait ScriptBehaviorBridge: Send + Sync + 'static {
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
mod tests {
    use super::*;

    #[test]
    fn callback_reference_requires_provider_qualified_identity() {
        let callback = ScriptBehaviorCallbackRef::parse("combat::ai.attack").unwrap();
        assert_eq!(callback.package_id(), "combat");
        assert_eq!(callback.node_id(), "ai.attack");
        assert_eq!(callback.stable_id(), "combat::ai.attack");
        assert!(ScriptBehaviorCallbackRef::parse("ai.attack").is_err());
        assert!(ScriptBehaviorCallbackRef::parse("::ai.attack").is_err());
    }
}
