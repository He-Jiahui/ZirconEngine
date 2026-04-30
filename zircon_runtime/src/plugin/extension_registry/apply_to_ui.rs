use crate::ui::component::UiComponentDescriptorRegistry;
use crate::RuntimeExtensionRegistryError;

use super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn apply_ui_components_to_registry(
        &self,
        registry: &mut UiComponentDescriptorRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        for component in self.ui_components() {
            if registry.descriptor(&component.component_id).is_some() {
                return Err(RuntimeExtensionRegistryError::DuplicateUiComponent(
                    component.component_id.clone(),
                ));
            }
            registry.register(component.to_runtime_component_descriptor());
        }
        Ok(())
    }
}
