use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::framework::render::{RenderImageDescriptor, RenderImageDimension};
use crate::core::resource::{AssetReference, ResourceId};

// Compact inspection data for authored material texture slots after resolution.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialTextureSlotSummary {
    pub total_count: usize,
    pub resolved_count: usize,
    pub fallback_count: usize,
}

/// 材质纹理槽状态是资源流转诊断 DTO：期望/实际维度与回退原因解释绑定为何未就绪，不改变材质或纹理资源所有权。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialTextureSlotState {
    pub slot: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_dimension: Option<RenderMaterialTextureDimension>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_dimension: Option<RenderMaterialTextureDimension>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_id: Option<ResourceId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<RenderMaterialTextureSlotFallback>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialTextureSlotFallback {
    pub reference: AssetReference,
    pub reason: RenderMaterialTextureSlotFallbackReason,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum RenderMaterialTextureSlotFallbackReason {
    UnresolvedReference,
    NotUploadReady {
        detail: String,
    },
    DimensionMismatch {
        expected: RenderMaterialTextureDimension,
        actual: RenderMaterialTextureDimension,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMaterialTextureDimension {
    D1,
    #[default]
    D2,
    D2Array,
    Cube,
    CubeArray,
    D3,
}

impl RenderMaterialTextureDimension {
    pub fn from_shader_kind(kind: &str) -> Self {
        // RUNTIME131_TEXTURE_DIMENSION_ZERO_ALLOCATION_MATCH_BENCH_V1
        let kind = kind.trim();
        match kind.len() {
            16 if kind.eq_ignore_ascii_case("texture_2d_array") => Self::D2Array,
            14 if kind.eq_ignore_ascii_case("texture2darray") => Self::D2Array,
            8 if kind.eq_ignore_ascii_case("2d_array") => Self::D2Array,
            18 if kind.eq_ignore_ascii_case("texture_cube_array") => Self::CubeArray,
            16 if kind.eq_ignore_ascii_case("texturecubearray") => Self::CubeArray,
            10 if kind.eq_ignore_ascii_case("cube_array") => Self::CubeArray,
            12 if kind.eq_ignore_ascii_case("texture_cube") => Self::Cube,
            11 if kind.eq_ignore_ascii_case("texturecube") => Self::Cube,
            7 if kind.eq_ignore_ascii_case("cubemap") => Self::Cube,
            4 if kind.eq_ignore_ascii_case("cube") => Self::Cube,
            10 if kind.eq_ignore_ascii_case("texture_3d") => Self::D3,
            9 if kind.eq_ignore_ascii_case("texture3d") => Self::D3,
            2 if kind.eq_ignore_ascii_case("3d") => Self::D3,
            10 if kind.eq_ignore_ascii_case("texture_1d") => Self::D1,
            9 if kind.eq_ignore_ascii_case("texture1d") => Self::D1,
            2 if kind.eq_ignore_ascii_case("1d") => Self::D1,
            _ => Self::D2,
        }
    }

    pub const fn wgsl_sampled_texture_type(self) -> &'static str {
        match self {
            Self::D1 => "texture_1d<f32>",
            Self::D2 => "texture_2d<f32>",
            Self::D2Array => "texture_2d_array<f32>",
            Self::Cube => "texture_cube<f32>",
            Self::CubeArray => "texture_cube_array<f32>",
            Self::D3 => "texture_3d<f32>",
        }
    }

    /// The MVP material set layout exposes only filterable 2D sampled textures.
    pub const fn is_supported_by_default_material_binding(self) -> bool {
        matches!(self, Self::D2)
    }

    pub fn from_image_descriptor(descriptor: &RenderImageDescriptor) -> Self {
        match descriptor.dimension {
            RenderImageDimension::D1 => Self::D1,
            RenderImageDimension::D2 if descriptor.depth_or_array_layers > 1 => Self::D2Array,
            RenderImageDimension::D2 => Self::D2,
            RenderImageDimension::D3 => Self::D3,
            RenderImageDimension::Cube if descriptor.depth_or_array_layers > 6 => Self::CubeArray,
            RenderImageDimension::Cube => Self::Cube,
        }
    }
}

impl RenderMaterialTextureSlotFallback {
    pub fn unresolved_reference(reference: AssetReference) -> Self {
        Self {
            reference,
            reason: RenderMaterialTextureSlotFallbackReason::UnresolvedReference,
        }
    }

    pub fn not_upload_ready(reference: AssetReference, detail: impl Into<String>) -> Self {
        Self {
            reference,
            reason: RenderMaterialTextureSlotFallbackReason::NotUploadReady {
                detail: detail.into(),
            },
        }
    }

    pub fn dimension_mismatch(
        reference: AssetReference,
        expected: RenderMaterialTextureDimension,
        actual: RenderMaterialTextureDimension,
    ) -> Self {
        Self {
            reference,
            reason: RenderMaterialTextureSlotFallbackReason::DimensionMismatch { expected, actual },
        }
    }
}

impl RenderMaterialTextureSlotSummary {
    pub fn from_texture_ids(texture_ids: &[Option<ResourceId>]) -> Self {
        let resolved_count = texture_ids.iter().filter(|id| id.is_some()).count();
        Self {
            total_count: texture_ids.len(),
            resolved_count,
            fallback_count: texture_ids.len().saturating_sub(resolved_count),
        }
    }

    pub fn from_non_standard_slots(slots: &BTreeMap<String, Option<ResourceId>>) -> Self {
        let resolved_count = slots.values().filter(|id| id.is_some()).count();
        Self {
            total_count: slots.len(),
            resolved_count,
            fallback_count: slots.len().saturating_sub(resolved_count),
        }
    }
}

impl RenderMaterialTextureSlotState {
    pub fn is_resolved(&self) -> bool {
        self.texture_id.is_some()
    }

    pub fn uses_fallback(&self) -> bool {
        self.texture_id.is_none()
    }

    pub fn from_named_texture_ids<I, S>(texture_ids: I) -> Vec<Self>
    where
        I: IntoIterator<Item = (S, Option<ResourceId>)>,
        S: Into<String>,
    {
        Self::from_resolved_slots(
            texture_ids
                .into_iter()
                .map(|(slot, texture_id)| (slot, texture_id, None)),
        )
    }

    pub fn from_resolved_slots<I, S>(texture_ids: I) -> Vec<Self>
    where
        I: IntoIterator<
            Item = (
                S,
                Option<ResourceId>,
                Option<RenderMaterialTextureSlotFallback>,
            ),
        >,
        S: Into<String>,
    {
        texture_ids
            .into_iter()
            .map(|(slot, texture_id, fallback)| Self {
                slot: slot.into(),
                expected_dimension: None,
                actual_dimension: None,
                texture_id,
                fallback,
            })
            .collect()
    }

    pub fn from_dimensioned_slots<I, S>(texture_ids: I) -> Vec<Self>
    where
        I: IntoIterator<
            Item = (
                S,
                Option<ResourceId>,
                Option<RenderMaterialTextureDimension>,
                Option<RenderMaterialTextureDimension>,
                Option<RenderMaterialTextureSlotFallback>,
            ),
        >,
        S: Into<String>,
    {
        texture_ids
            .into_iter()
            .map(
                |(slot, texture_id, expected_dimension, actual_dimension, fallback)| Self {
                    slot: slot.into(),
                    expected_dimension,
                    actual_dimension,
                    texture_id,
                    fallback,
                },
            )
            .collect()
    }

    pub fn from_non_standard_slots(slots: &BTreeMap<String, Option<ResourceId>>) -> Vec<Self> {
        Self::from_named_texture_ids(
            slots
                .iter()
                .map(|(slot, texture_id)| (slot.clone(), *texture_id)),
        )
    }
}

#[cfg(test)]
#[path = "tests/texture_slot_summary.rs"]
mod tests;

#[cfg(test)]
#[path = "texture_slot_summary/tests/dimension_kind_tests.rs"]
mod dimension_kind_tests;
