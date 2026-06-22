use std::collections::BTreeMap;

use crate::core::framework::render::{
    GBufferChannelMask, RenderMaterialLightingModel, ShadingModelDescriptor, ShadingModelId,
    ShadingModelRegistrationError,
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

    pub(crate) fn get(&self, id: ShadingModelId) -> Option<&ShadingModelDescriptor> {
        self.descriptors.get(&id)
    }

    fn resolve_token(&self, token: &str) -> Option<&ShadingModelDescriptor> {
        self.tokens
            .get(&token.trim().to_ascii_lowercase())
            .and_then(|id| self.get(*id))
    }

    pub(crate) fn resolve_lighting_model(
        &self,
        model: &RenderMaterialLightingModel,
    ) -> Option<&ShadingModelDescriptor> {
        self.resolve_token(&model.as_token())
    }

    pub(crate) fn register_builtin(
        &mut self,
        descriptor: ShadingModelDescriptor,
    ) -> Result<(), ShadingModelRegistrationError> {
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
mod tests {
    use crate::core::framework::render::{
        GBufferChannelMask, RenderMaterialLightingModel, ShadingModelDescriptor, ShadingModelId,
        ShadingModelRegistrationError, SHADING_MODEL_ID_STANDARD_PBR,
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
    fn shading_model_registry_resolves_registered_lighting_model_tokens() {
        let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());
        registry
            .register_builtin(descriptor(4, "custom:subsurface"))
            .unwrap();

        let resolved = registry
            .resolve_lighting_model(&RenderMaterialLightingModel::Custom {
                name: "subsurface".to_string(),
            })
            .unwrap();
        assert_eq!(resolved.id, ShadingModelId::new(4));
    }

    #[test]
    fn shading_model_registry_rejects_unsupported_required_channels() {
        let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());
        let descriptor = ShadingModelDescriptor::new(
            crate::core::framework::render::ShadingModelId::new(4),
            "subsurface",
            "forward",
            "gbuffer",
            "deferred",
            GBufferChannelMask::standard_lit().union(GBufferChannelMask::CUSTOM0),
        );
        let error = registry.register_builtin(descriptor).unwrap_err();
        assert!(matches!(
            error,
            ShadingModelRegistrationError::RequiredChannelsUnsupported { .. }
        ));
    }
}
