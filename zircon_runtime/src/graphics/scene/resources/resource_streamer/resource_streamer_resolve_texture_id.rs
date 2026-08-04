use crate::asset::{AssetReference, TextureUploadSupport};
use crate::core::framework::render::{
    RenderMaterialFallbackPolicy, RenderMaterialFallbackReason, RenderMaterialFallbackUsage,
    RenderMaterialTextureDimension, RenderMaterialTextureSlotFallback,
    RenderMaterialValidationError,
};
use crate::core::resource::ResourceId;

use super::ResourceStreamer;

const DEFAULT_MATERIAL_TEXTURE_BINDING_REASON: &str =
    "default material texture binding supports only texture_2d<f32>";

#[derive(Clone, Debug, PartialEq)]
pub(in crate::graphics::scene::resources) struct ResolvedTextureReference {
    pub(in crate::graphics::scene::resources) id: Option<ResourceId>,
    pub(in crate::graphics::scene::resources) validation_error:
        Option<RenderMaterialValidationError>,
    pub(in crate::graphics::scene::resources) fallback_usage: Option<RenderMaterialFallbackUsage>,
    pub(in crate::graphics::scene::resources) slot_fallback:
        Option<RenderMaterialTextureSlotFallback>,
    pub(in crate::graphics::scene::resources) expected_dimension: RenderMaterialTextureDimension,
    pub(in crate::graphics::scene::resources) actual_dimension:
        Option<RenderMaterialTextureDimension>,
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
        self.resolve_texture_reference_with_dimension_support(
            slot,
            reference,
            RenderMaterialTextureDimension::D2,
            support,
        )
    }

    pub(in crate::graphics::scene::resources) fn resolve_texture_reference_with_dimension_support(
        &self,
        slot: &str,
        reference: Option<&AssetReference>,
        expected_dimension: RenderMaterialTextureDimension,
        support: TextureUploadSupport,
    ) -> ResolvedTextureReference {
        let Some(reference) = reference else {
            return ResolvedTextureReference {
                id: None,
                validation_error: None,
                fallback_usage: None,
                slot_fallback: None,
                expected_dimension,
                actual_dimension: None,
            };
        };

        let Ok(asset_manager) = self.asset_manager() else {
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
                slot_fallback: Some(RenderMaterialTextureSlotFallback::unresolved_reference(
                    reference.clone(),
                )),
                expected_dimension,
                actual_dimension: None,
            };
        };

        let Some(id) = asset_manager
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
                slot_fallback: Some(RenderMaterialTextureSlotFallback::unresolved_reference(
                    reference.clone(),
                )),
                expected_dimension,
                actual_dimension: None,
            };
        };

        let texture = match asset_manager.load_texture_asset(id) {
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
                    slot_fallback: Some(RenderMaterialTextureSlotFallback::unresolved_reference(
                        reference.clone(),
                    )),
                    expected_dimension,
                    actual_dimension: None,
                };
            }
        };

        let actual_dimension = RenderMaterialTextureDimension::from_image_descriptor(
            &texture.render_image_descriptor(),
        );
        if actual_dimension != expected_dimension {
            return ResolvedTextureReference {
                id: None,
                validation_error: Some(RenderMaterialValidationError::TextureDimensionMismatch {
                    slot: slot.to_string(),
                    reference: reference.clone(),
                    expected: expected_dimension,
                    actual: actual_dimension,
                }),
                fallback_usage: Some(RenderMaterialFallbackUsage {
                    reason: RenderMaterialFallbackReason::Texture {
                        slot: slot.to_string(),
                        reference: reference.clone(),
                    },
                    fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
                }),
                slot_fallback: Some(RenderMaterialTextureSlotFallback::dimension_mismatch(
                    reference.clone(),
                    expected_dimension,
                    actual_dimension,
                )),
                expected_dimension,
                actual_dimension: Some(actual_dimension),
            };
        }

        if !expected_dimension.is_supported_by_default_material_binding() {
            return ResolvedTextureReference {
                id: None,
                validation_error: Some(RenderMaterialValidationError::TextureNotUploadReady {
                    slot: slot.to_string(),
                    reference: reference.clone(),
                    reason: DEFAULT_MATERIAL_TEXTURE_BINDING_REASON.to_string(),
                }),
                fallback_usage: Some(RenderMaterialFallbackUsage {
                    reason: RenderMaterialFallbackReason::Texture {
                        slot: slot.to_string(),
                        reference: reference.clone(),
                    },
                    fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
                }),
                slot_fallback: Some(RenderMaterialTextureSlotFallback::not_upload_ready(
                    reference.clone(),
                    DEFAULT_MATERIAL_TEXTURE_BINDING_REASON,
                )),
                expected_dimension,
                actual_dimension: Some(actual_dimension),
            };
        }

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
                slot_fallback: Some(RenderMaterialTextureSlotFallback::not_upload_ready(
                    reference.clone(),
                    reason,
                )),
                expected_dimension,
                actual_dimension: Some(actual_dimension),
            };
        }

        ResolvedTextureReference {
            id: Some(id),
            validation_error: None,
            fallback_usage: None,
            slot_fallback: None,
            expected_dimension,
            actual_dimension: Some(actual_dimension),
        }
    }
}
