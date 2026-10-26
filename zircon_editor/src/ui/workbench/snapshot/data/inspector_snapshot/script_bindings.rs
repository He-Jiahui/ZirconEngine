//! Typed script authoring data; never part of the protected plugin JSON projection.

use std::collections::BTreeMap;
use zircon_runtime::asset::SceneScriptBindingAsset;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorScriptBindingSnapshot {
    pub package: String,
    pub module: String,
    pub enabled: bool,
    pub update: bool,
    pub fixed_update: bool,
    pub properties: BTreeMap<String, serde_json::Value>,
}

impl From<SceneScriptBindingAsset> for InspectorScriptBindingSnapshot {
    fn from(value: SceneScriptBindingAsset) -> Self {
        Self {
            package: value.package,
            module: value.module,
            enabled: value.enabled,
            update: value.update,
            fixed_update: value.fixed_update,
            properties: value.properties,
        }
    }
}

impl From<&InspectorScriptBindingSnapshot> for SceneScriptBindingAsset {
    fn from(value: &InspectorScriptBindingSnapshot) -> Self {
        Self {
            package: value.package.clone(),
            module: value.module.clone(),
            enabled: value.enabled,
            update: value.update,
            fixed_update: value.fixed_update,
            properties: value.properties.clone(),
        }
    }
}
