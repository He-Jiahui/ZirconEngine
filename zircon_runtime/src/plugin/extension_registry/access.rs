use crate::core::ManagerDescriptor;
use crate::core::ModuleDescriptor;
use crate::graphics::RenderFeatureDescriptor;
use crate::{ComponentTypeDescriptor, UiComponentDescriptor};

use super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn managers(&self) -> &[ManagerDescriptor] {
        &self.managers
    }

    pub fn modules(&self) -> &[ModuleDescriptor] {
        &self.modules
    }

    pub fn render_features(&self) -> &[RenderFeatureDescriptor] {
        &self.render_features
    }

    pub fn components(&self) -> &[ComponentTypeDescriptor] {
        &self.components
    }

    pub fn ui_components(&self) -> &[UiComponentDescriptor] {
        &self.ui_components
    }
}
