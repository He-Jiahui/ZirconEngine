use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::resource::{AssetReference, ResourceId};

// Compact inspection data for authored material texture slots after resolution.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialTextureSlotSummary {
    pub total_count: usize,
    pub resolved_count: usize,
    pub fallback_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialTextureSlotState {
    pub slot: String,
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
    NotUploadReady { detail: String },
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
        let texture_ids = slots.values().cloned().collect::<Vec<_>>();
        Self::from_texture_ids(&texture_ids)
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
                texture_id,
                fallback,
            })
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
mod tests {
    use super::*;

    #[test]
    fn material_texture_slot_summary_counts_resolved_and_fallback_slots() {
        let mut slots = BTreeMap::new();
        slots.insert(
            "mask_map".to_string(),
            Some(ResourceId::from_stable_label("texture:mask")),
        );
        slots.insert("detail_map".to_string(), None);

        let summary = RenderMaterialTextureSlotSummary::from_non_standard_slots(&slots);

        assert_eq!(summary.total_count, 2);
        assert_eq!(summary.resolved_count, 1);
        assert_eq!(summary.fallback_count, 1);
    }

    #[test]
    fn material_texture_slot_summary_counts_authored_standard_slot_states() {
        let texture_ids = [
            Some(ResourceId::from_stable_label("texture:base")),
            None,
            Some(ResourceId::from_stable_label("texture:normal")),
        ];

        let summary = RenderMaterialTextureSlotSummary::from_texture_ids(&texture_ids);

        assert_eq!(summary.total_count, 3);
        assert_eq!(summary.resolved_count, 2);
        assert_eq!(summary.fallback_count, 1);
    }

    #[test]
    fn material_texture_slot_state_lists_slot_keys_and_resolution_state() {
        let mut slots = BTreeMap::new();
        let detail_id = ResourceId::from_stable_label("texture:detail");
        slots.insert("mask_map".to_string(), None);
        slots.insert("detail_map".to_string(), Some(detail_id));

        let states = RenderMaterialTextureSlotState::from_non_standard_slots(&slots);

        assert_eq!(
            states,
            vec![
                RenderMaterialTextureSlotState {
                    slot: "detail_map".to_string(),
                    texture_id: Some(detail_id),
                    fallback: None,
                },
                RenderMaterialTextureSlotState {
                    slot: "mask_map".to_string(),
                    texture_id: None,
                    fallback: None,
                },
            ]
        );
        assert!(states[0].is_resolved());
        assert!(!states[1].is_resolved());
        assert!(states[1].uses_fallback());
    }

    #[test]
    fn material_texture_slot_state_keeps_fallback_reference_and_reason() {
        let reference = AssetReference::from_locator(
            crate::core::resource::ResourceLocator::parse("res://textures/container.ktx2")
                .expect("valid texture locator"),
        );

        let states = RenderMaterialTextureSlotState::from_resolved_slots([(
            "base_color",
            None,
            Some(RenderMaterialTextureSlotFallback::not_upload_ready(
                reference.clone(),
                "ktx2 texture format or level index is not upload-ready",
            )),
        )]);

        assert_eq!(states.len(), 1);
        assert_eq!(states[0].slot, "base_color");
        assert_eq!(states[0].texture_id, None);
        assert_eq!(
            states[0].fallback,
            Some(RenderMaterialTextureSlotFallback {
                reference,
                reason: RenderMaterialTextureSlotFallbackReason::NotUploadReady {
                    detail: "ktx2 texture format or level index is not upload-ready".to_string(),
                },
            })
        );
    }
}
