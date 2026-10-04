use crate::plugin::RuntimeExtensionRegistryError;
use crate::ui::component::UiComponentDescriptorRegistry;

use super::super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    /// 在 UI 注册表建立前安装插件组件；失败时已登记的前序组件仍留在目标表中。
    pub fn apply_ui_components_to_registry(
        &mut self,
        registry: &mut UiComponentDescriptorRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.finalize();
        for component in self.ui_components() {
            if registry.descriptor(&component.component_id).is_some() {
                return Err(RuntimeExtensionRegistryError::DuplicateUiComponent(
                    component.component_id.clone(),
                ));
            }
            if let Err(error) = registry.register(component.to_runtime_component_descriptor()) {
                return Err(RuntimeExtensionRegistryError::InvalidUiComponent(
                    error.to_string(),
                ));
            }
        }
        Ok(())
    }
}
