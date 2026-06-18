use std::collections::BTreeMap;

use crate::core::framework::render::{
    GBufferChannelMask, RenderMaterialLightingModel, ShadingModelDescriptor, ShadingModelId,
    ShadingModelRegistrationError, SHADING_MODEL_PLUGIN_ID_START,
};

#[derive(Clone, Debug)]
pub(crate) struct ShadingModelRegistry {
    supported_channels: GBufferChannelMask,
    descriptors: BTreeMap<ShadingModelId, ShadingModelDescriptor>,
    tokens: BTreeMap<String, ShadingModelId>,
}

impl ShadingModelRegistry {
    pub(crate) fn new(supported_channels: GBufferChannelMask) -> Self {
        Self {
            supported_channels,
            descriptors: BTreeMap::new(),
            tokens: BTreeMap::new(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn supported_channels(&self) -> GBufferChannelMask {
        self.supported_channels
    }

    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        self.descriptors.len()
    }

    pub(crate) fn get(&self, id: ShadingModelId) -> Option<&ShadingModelDescriptor> {
        self.descriptors.get(&id)
    }

    #[allow(dead_code)]
    pub(crate) fn resolve_token(&self, token: &str) -> Option<&ShadingModelDescriptor> {
        self.tokens
            .get(&token.trim().to_ascii_lowercase())
            .and_then(|id| self.get(*id))
    }

    pub(crate) fn resolve_lighting_model(
        &self,
        model: &RenderMaterialLightingModel,
    ) -> Option<&ShadingModelDescriptor> {
        let id = ShadingModelId::from_lighting_model(model)?;
        self.get(id)
    }

    pub(crate) fn register_builtin(
        &mut self,
        descriptor: ShadingModelDescriptor,
    ) -> Result<(), ShadingModelRegistrationError> {
        self.register(descriptor, false)
    }

    #[allow(dead_code)]
    pub(crate) fn register_plugin(
        &mut self,
        descriptor: ShadingModelDescriptor,
    ) -> Result<(), ShadingModelRegistrationError> {
        self.register(descriptor, true)
    }

    fn register(
        &mut self,
        mut descriptor: ShadingModelDescriptor,
        plugin: bool,
    ) -> Result<(), ShadingModelRegistrationError> {
        descriptor.token = descriptor.token.trim().to_ascii_lowercase();
        if plugin && descriptor.id.value() < SHADING_MODEL_PLUGIN_ID_START {
            return Err(ShadingModelRegistrationError::PluginIdBelowReservedRange {
                id: descriptor.id,
            });
        }
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
mod tests {
    use crate::core::framework::render::{
        GBufferChannelMask, ShadingModelDescriptor, ShadingModelRegistrationError,
        SHADING_MODEL_ID_STANDARD_PBR, SHADING_MODEL_PLUGIN_ID_START,
    };

    use super::ShadingModelRegistry;

    fn descriptor(id: u8, token: &str) -> ShadingModelDescriptor {
        ShadingModelDescriptor::new(
            crate::core::framework::render::ShadingModelId::new(id),
            token,
            "forward",
            "gbuffer",
            "deferred",
            GBufferChannelMask::standard_lit(),
        )
    }

    #[test]
    fn shading_model_registry_rejects_duplicate_id() {
        let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());
        registry
            .register_builtin(descriptor(SHADING_MODEL_ID_STANDARD_PBR.value(), "pbr"))
            .unwrap();

        let error = registry
            .register_builtin(descriptor(
                SHADING_MODEL_ID_STANDARD_PBR.value(),
                "standard_pbr",
            ))
            .unwrap_err();
        assert!(matches!(
            error,
            ShadingModelRegistrationError::DuplicateId { .. }
        ));
    }

    #[test]
    fn shading_model_registry_rejects_duplicate_token() {
        let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());
        registry.register_builtin(descriptor(2, "pbr")).unwrap();

        let error = registry.register_builtin(descriptor(3, "PBR")).unwrap_err();
        assert!(matches!(
            error,
            ShadingModelRegistrationError::DuplicateToken { .. }
        ));
    }

    #[test]
    fn shading_model_registry_rejects_plugin_ids_below_reserved_range() {
        let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());
        let error = registry
            .register_plugin(descriptor(3, "custom"))
            .unwrap_err();
        assert!(matches!(
            error,
            ShadingModelRegistrationError::PluginIdBelowReservedRange { .. }
        ));
    }

    #[test]
    fn shading_model_registry_rejects_unsupported_required_channels() {
        let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());
        let descriptor = ShadingModelDescriptor::new(
            crate::core::framework::render::ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START),
            "subsurface",
            "forward",
            "gbuffer",
            "deferred",
            GBufferChannelMask::standard_lit().union(GBufferChannelMask::CUSTOM0),
        );
        let error = registry.register_plugin(descriptor).unwrap_err();
        assert!(matches!(
            error,
            ShadingModelRegistrationError::RequiredChannelsUnsupported { .. }
        ));
    }
}
