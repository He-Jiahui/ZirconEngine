use crate::asset::{AssetReference, TextureUploadSupport};
use crate::core::framework::render::{
    RenderMaterialFallbackPolicy, RenderMaterialFallbackReason, RenderMaterialFallbackUsage,
    RenderMaterialValidationError,
};
use crate::core::resource::ResourceId;

use super::ResourceStreamer;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::graphics::scene::resources) struct ResolvedTextureReference {
    pub(in crate::graphics::scene::resources) id: Option<ResourceId>,
    pub(in crate::graphics::scene::resources) validation_error:
        Option<RenderMaterialValidationError>,
    pub(in crate::graphics::scene::resources) fallback_usage: Option<RenderMaterialFallbackUsage>,
}

impl ResolvedTextureReference {
    pub(in crate::graphics::scene::resources) fn id(&self) -> Option<ResourceId> {
        self.id
    }
}

impl ResourceStreamer {
    pub(in crate::graphics::scene::resources) fn resolve_texture_reference(
        &self,
        slot: &'static str,
        reference: Option<&AssetReference>,
    ) -> ResolvedTextureReference {
        self.resolve_texture_reference_with_support(
            slot,
            reference,
            TextureUploadSupport::uncompressed_only(),
        )
    }

    pub(in crate::graphics::scene::resources) fn resolve_texture_reference_with_support(
        &self,
        slot: &str,
        reference: Option<&AssetReference>,
        support: TextureUploadSupport,
    ) -> ResolvedTextureReference {
        let Some(reference) = reference else {
            return ResolvedTextureReference {
                id: None,
                validation_error: None,
                fallback_usage: None,
            };
        };

        let Some(id) = self
            .asset_manager
            .resource_manager()
            .registry()
            .get_by_locator(&reference.locator)
            .map(|record| record.id())
        else {
            return ResolvedTextureReference {
                id: None,
                validation_error: Some(RenderMaterialValidationError::UnresolvedTextureReference {
                    slot: slot.to_string(),
                    reference: reference.clone(),
                }),
                fallback_usage: Some(RenderMaterialFallbackUsage {
                    reason: RenderMaterialFallbackReason::Texture {
                        slot: slot.to_string(),
                        reference: reference.clone(),
                    },
                    fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
                }),
            };
        };

        let texture = match self.asset_manager.load_texture_asset(id) {
            Ok(texture) => texture,
            Err(_) => {
                return ResolvedTextureReference {
                    id: None,
                    validation_error: Some(
                        RenderMaterialValidationError::UnresolvedTextureReference {
                            slot: slot.to_string(),
                            reference: reference.clone(),
                        },
                    ),
                    fallback_usage: Some(RenderMaterialFallbackUsage {
                        reason: RenderMaterialFallbackReason::Texture {
                            slot: slot.to_string(),
                            reference: reference.clone(),
                        },
                        fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
                    }),
                };
            }
        };

        if let Some(reason) = texture.upload_readiness(support).unsupported_reason() {
            return ResolvedTextureReference {
                id: None,
                validation_error: Some(RenderMaterialValidationError::TextureNotUploadReady {
                    slot: slot.to_string(),
                    reference: reference.clone(),
                    reason: reason.to_string(),
                }),
                fallback_usage: Some(RenderMaterialFallbackUsage {
                    reason: RenderMaterialFallbackReason::Texture {
                        slot: slot.to_string(),
                        reference: reference.clone(),
                    },
                    fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
                }),
            };
        }

        ResolvedTextureReference {
            id: Some(id),
            validation_error: None,
            fallback_usage: None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn resolve_texture_id(
        &self,
        reference: Option<&AssetReference>,
    ) -> Option<ResourceId> {
        self.resolve_texture_reference("texture", reference).id()
    }
}
