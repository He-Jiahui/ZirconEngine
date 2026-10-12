use crate::plugin::{
    PluginFeatureBundleManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
};

/// 可选特性的具体提供者契约；清单负责选择信息，register 负责运行时扩展登记。
pub trait RuntimePluginFeature {
    fn manifest(&self) -> PluginFeatureBundleManifest;

    fn register(
        &self,
        _registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        Ok(())
    }
}
