use std::collections::{HashMap, HashSet};

use crate::core::framework::render::{
    RenderBloomSettings, RenderColorGradingSettings, RenderExposureSettings,
    RenderPostProcessEffectStackSettings,
};

use super::resolved_stack::RenderResolvedPostProcessSettings;
use super::volume_component::{
    VolumeComponentApplyError, VolumeComponentDescriptor, BUILTIN_POST_PROCESS_VOLUME_COMPONENTS,
};

/// 体积组件注册表以稳定 component_id 建立描述符索引，并在注册时校验组件 ID 与参数名；评估器随后按该表应用内建或插件组件。
#[derive(Clone, Debug, Default)]
pub struct VolumeComponentRegistry {
    descriptors: Vec<VolumeComponentDescriptor>,
    descriptor_indices: HashMap<&'static str, usize>,
}

impl VolumeComponentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_builtin_post_process_components() -> Self {
        let mut registry = Self::new();
        registry
            .register_builtin_post_process_components()
            .expect("built-in post-process volume component ids and params must be unique");
        registry
    }

    pub fn register_builtin_post_process_components(&mut self) -> Result<(), VolumeRegistryError> {
        for descriptor in BUILTIN_POST_PROCESS_VOLUME_COMPONENTS {
            self.register(*descriptor)?;
        }
        Ok(())
    }

    pub fn register(
        &mut self,
        descriptor: VolumeComponentDescriptor,
    ) -> Result<(), VolumeRegistryError> {
        validate_descriptor(descriptor)?;
        if self
            .descriptor_indices
            .contains_key(descriptor.component_id)
        {
            return Err(VolumeRegistryError::DuplicateComponentId {
                component_id: descriptor.component_id.to_string(),
            });
        }

        let descriptor_index = self.descriptors.len();
        self.descriptor_indices
            .insert(descriptor.component_id, descriptor_index);
        self.descriptors.push(descriptor);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &VolumeComponentDescriptor> {
        self.descriptors.iter()
    }

    pub fn contains(&self, component_id: &str) -> bool {
        self.descriptor_indices.contains_key(component_id)
    }

    pub fn get(&self, component_id: &str) -> Option<&VolumeComponentDescriptor> {
        self.descriptor_indices
            .get(component_id)
            .and_then(|descriptor_index| self.descriptors.get(*descriptor_index))
    }

    pub fn default_resolved_post_process_settings(
        &self,
    ) -> Result<RenderResolvedPostProcessSettings, VolumeComponentApplyError> {
        let mut settings = RenderResolvedPostProcessSettings::new(
            RenderBloomSettings::default(),
            RenderExposureSettings::default(),
            RenderColorGradingSettings::default(),
            RenderPostProcessEffectStackSettings::default(),
        );
        for descriptor in &self.descriptors {
            descriptor.apply_defaults(&mut settings)?;
        }
        Ok(settings)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VolumeRegistryError {
    EmptyComponentId,
    DuplicateComponentId {
        component_id: String,
    },
    EmptyParamName {
        component_id: String,
    },
    DuplicateParamName {
        component_id: String,
        param_name: String,
    },
}

fn validate_descriptor(descriptor: VolumeComponentDescriptor) -> Result<(), VolumeRegistryError> {
    if descriptor.component_id.trim().is_empty() {
        return Err(VolumeRegistryError::EmptyComponentId);
    }

    let mut param_names = HashSet::with_capacity(descriptor.params.len());
    for param in descriptor.params {
        if param.name.trim().is_empty() {
            return Err(VolumeRegistryError::EmptyParamName {
                component_id: descriptor.component_id.to_string(),
            });
        }
        if !param_names.insert(param.name) {
            return Err(VolumeRegistryError::DuplicateParamName {
                component_id: descriptor.component_id.to_string(),
                param_name: param.name.to_string(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/volume_registry.rs"]
mod tests;

#[cfg(test)]
#[path = "volume_registry/tests/optimization_batch_in_runtime624_tests.rs"]
mod optimization_batch_in_runtime624_tests;
