use std::collections::{BTreeMap, HashMap};

use crate::core::framework::render::{
    GBufferChannelMask, RenderMaterialLightingModel, ShadingModelDescriptor, ShadingModelId,
    ShadingModelRegistrationError, SHADING_MODEL_PLUGIN_ID_START,
};

#[cfg(test)]
#[path = "registry/tests/hash_token_tests.rs"]
mod hash_token_tests;

#[derive(Clone, Debug)]
pub(crate) struct ShadingModelRegistry {
    supported_channels: GBufferChannelMask,
    descriptors: BTreeMap<ShadingModelId, ShadingModelDescriptor>,
    tokens: HashMap<String, ShadingModelId>,
}

impl ShadingModelRegistry {
    pub(crate) fn new(supported_channels: GBufferChannelMask) -> Self {
        Self {
            supported_channels,
            descriptors: BTreeMap::new(),
            tokens: HashMap::new(),
        }
    }

    pub(crate) fn get(&self, id: ShadingModelId) -> Option<&ShadingModelDescriptor> {
        self.descriptors.get(&id)
    }

    pub(crate) fn descriptors(&self) -> impl Iterator<Item = &ShadingModelDescriptor> {
        self.descriptors.values()
    }

    fn resolve_token(&self, token: &str) -> Option<&ShadingModelDescriptor> {
        let trimmed = token.trim();
        if let Some(id) = self.tokens.get(trimmed) {
            return self.get(*id);
        }
        let normalized = trimmed.to_ascii_lowercase();
        self.tokens.get(&normalized).and_then(|id| self.get(*id))
    }

    pub(crate) fn resolve_lighting_model(
        &self,
        model: &RenderMaterialLightingModel,
    ) -> Option<&ShadingModelDescriptor> {
        match model {
            RenderMaterialLightingModel::Pbr => self.resolve_token("pbr"),
            RenderMaterialLightingModel::BlinnPhong => self.resolve_token("blinn_phong"),
            RenderMaterialLightingModel::Unlit => self.resolve_token("unlit"),
            RenderMaterialLightingModel::Custom { name } => {
                let mut token = String::with_capacity("custom:".len() + name.len());
                token.push_str("custom:");
                token.push_str(name);
                self.resolve_token(&token)
            }
        }
    }

    pub(crate) fn register_builtin(
        &mut self,
        descriptor: ShadingModelDescriptor,
    ) -> Result<(), ShadingModelRegistrationError> {
        self.register(descriptor)
    }

    pub(crate) fn register_plugin_descriptor(
        &mut self,
        descriptor: ShadingModelDescriptor,
    ) -> Result<(), ShadingModelRegistrationError> {
        if !descriptor.id.is_plugin_range() {
            return Err(ShadingModelRegistrationError::PluginIdReserved {
                token: descriptor.token.trim().to_ascii_lowercase(),
                id: descriptor.id,
                minimum: SHADING_MODEL_PLUGIN_ID_START,
            });
        }
        self.register(descriptor)
    }

    fn register(
        &mut self,
        mut descriptor: ShadingModelDescriptor,
    ) -> Result<(), ShadingModelRegistrationError> {
        descriptor.token = descriptor.token.trim().to_ascii_lowercase();
        if !self
            .supported_channels
            .contains(descriptor.required_channels)
        {
            return Err(ShadingModelRegistrationError::RequiredChannelsUnsupported {
                token: descriptor.token,
                required: descriptor.required_channels,
                supported: self.supported_channels,
            });
        }
        if let Some(existing) = self.descriptors.get(&descriptor.id) {
            return Err(ShadingModelRegistrationError::DuplicateId {
                id: descriptor.id,
                existing_token: existing.token.clone(),
                new_token: descriptor.token,
            });
        }
        if let Some(existing_id) = self.tokens.get(&descriptor.token) {
            return Err(ShadingModelRegistrationError::DuplicateToken {
                token: descriptor.token,
                existing_id: *existing_id,
                new_id: descriptor.id,
            });
        }
        self.tokens.insert(descriptor.token.clone(), descriptor.id);
        self.descriptors.insert(descriptor.id, descriptor);
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/registry.rs"]
mod tests;
