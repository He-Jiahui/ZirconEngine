use serde::{Deserialize, Serialize};

use crate::core::resource::{MaterialMarker, ResourceHandle};

use super::{RenderLayerSet, RenderQueueValue};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CastShadowsMode {
    Off,
    On,
    TwoSided,
    ShadowsOnly,
}

impl CastShadowsMode {
    pub const fn casts_shadows(self) -> bool {
        !matches!(self, Self::Off)
    }

    pub const fn renders_in_main_view(self) -> bool {
        !matches!(self, Self::ShadowsOnly)
    }
}

impl Default for CastShadowsMode {
    fn default() -> Self {
        Self::On
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotionVectorMode {
    #[default]
    Auto,
    ForceOn,
    ForceOff,
}

impl MotionVectorMode {
    pub const fn resolves_enabled(self, is_dynamic: bool, transform_changed: bool) -> bool {
        match self {
            Self::Auto => is_dynamic || transform_changed,
            Self::ForceOn => true,
            Self::ForceOff => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LodGroupId(pub u64);

impl LodGroupId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct MaterialOverrideSet {
    slots: Vec<(u32, ResourceHandle<MaterialMarker>)>,
}

impl MaterialOverrideSet {
    pub fn from_slots(
        slots: impl IntoIterator<Item = (u32, ResourceHandle<MaterialMarker>)>,
    ) -> Self {
        let mut overrides = Self::default();
        for (slot, material) in slots {
            overrides.insert(slot, material);
        }
        overrides
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn slots(&self) -> &[(u32, ResourceHandle<MaterialMarker>)] {
        &self.slots
    }

    pub fn get(&self, slot: u32) -> Option<ResourceHandle<MaterialMarker>> {
        self.slots
            .binary_search_by_key(&slot, |(index, _)| *index)
            .ok()
            .map(|index| self.slots[index].1)
    }

    pub fn insert(
        &mut self,
        slot: u32,
        material: ResourceHandle<MaterialMarker>,
    ) -> Option<ResourceHandle<MaterialMarker>> {
        match self.slots.binary_search_by_key(&slot, |(index, _)| *index) {
            Ok(index) => Some(std::mem::replace(&mut self.slots[index].1, material)),
            Err(index) => {
                self.slots.insert(index, (slot, material));
                None
            }
        }
    }
}

#[derive(Deserialize)]
struct MaterialOverrideSetWire {
    #[serde(default)]
    slots: Vec<(u32, ResourceHandle<MaterialMarker>)>,
}

impl<'de> Deserialize<'de> for MaterialOverrideSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = MaterialOverrideSetWire::deserialize(deserializer)?;
        Ok(Self::from_slots(wire.slots))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RendererCommon {
    pub enabled: bool,
    pub layer_mask: RenderLayerSet,
    pub queue_override: Option<RenderQueueValue>,
    pub cast_shadows: CastShadowsMode,
    pub receive_shadows: bool,
    pub motion_vectors: MotionVectorMode,
    pub material_overrides: MaterialOverrideSet,
    pub is_static: bool,
    pub lod_group: Option<LodGroupId>,
}

impl Default for RendererCommon {
    fn default() -> Self {
        Self {
            enabled: true,
            layer_mask: RenderLayerSet::default(),
            queue_override: None,
            cast_shadows: CastShadowsMode::On,
            receive_shadows: true,
            motion_vectors: MotionVectorMode::Auto,
            material_overrides: MaterialOverrideSet::default(),
            is_static: false,
            lod_group: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::resource::{MaterialMarker, ResourceHandle, ResourceId};

    use super::{
        CastShadowsMode, LodGroupId, MaterialOverrideSet, MotionVectorMode, RendererCommon,
    };

    #[test]
    fn render_renderer_common_default_preserves_visible_mesh_semantics() {
        let common = RendererCommon::default();

        assert!(common.enabled);
        assert_eq!(common.layer_mask, super::RenderLayerSet::default());
        assert_eq!(common.queue_override, None);
        assert_eq!(common.cast_shadows, CastShadowsMode::On);
        assert!(common.receive_shadows);
        assert_eq!(common.motion_vectors, MotionVectorMode::Auto);
        assert!(common.material_overrides.is_empty());
        assert!(!common.is_static);
        assert_eq!(common.lod_group, None);
    }

    #[test]
    fn render_material_override_set_keeps_one_handle_per_sorted_slot() {
        let first_slot_seven = material_handle("res://materials/slot-seven-a.zmaterial");
        let replacement_slot_seven = material_handle("res://materials/slot-seven-b.zmaterial");
        let slot_two = material_handle("res://materials/slot-two.zmaterial");
        let mut overrides = MaterialOverrideSet::default();

        assert_eq!(overrides.insert(7, first_slot_seven), None);
        assert_eq!(overrides.insert(2, slot_two), None);
        assert_eq!(
            overrides.insert(7, replacement_slot_seven),
            Some(first_slot_seven)
        );

        assert_eq!(
            overrides.slots(),
            &[(2, slot_two), (7, replacement_slot_seven)]
        );
        assert_eq!(overrides.get(2), Some(slot_two));
        assert_eq!(overrides.get(7), Some(replacement_slot_seven));
        assert_eq!(overrides.get(9), None);
    }

    #[test]
    fn render_material_override_set_deserialization_normalizes_unsorted_duplicate_slots() {
        let first_slot_seven = material_handle("res://materials/serde-slot-seven-a.zmaterial");
        let replacement_slot_seven =
            material_handle("res://materials/serde-slot-seven-b.zmaterial");
        let slot_two = material_handle("res://materials/serde-slot-two.zmaterial");
        let encoded = serde_json::json!({
            "slots": [
                [7, first_slot_seven],
                [2, slot_two],
                [7, replacement_slot_seven]
            ]
        });

        let overrides: MaterialOverrideSet =
            serde_json::from_value(encoded).expect("material override set should deserialize");

        assert_eq!(
            overrides.slots(),
            &[(2, slot_two), (7, replacement_slot_seven)]
        );
    }

    #[test]
    fn render_renderer_common_modes_resolve_shadow_and_velocity_contracts() {
        assert!(!CastShadowsMode::Off.casts_shadows());
        assert!(CastShadowsMode::On.casts_shadows());
        assert!(CastShadowsMode::TwoSided.casts_shadows());
        assert!(CastShadowsMode::ShadowsOnly.casts_shadows());
        assert!(!CastShadowsMode::ShadowsOnly.renders_in_main_view());
        assert!(CastShadowsMode::Off.renders_in_main_view());
        assert!(CastShadowsMode::On.renders_in_main_view());
        assert!(CastShadowsMode::TwoSided.renders_in_main_view());

        assert!(!MotionVectorMode::Auto.resolves_enabled(false, false));
        assert!(MotionVectorMode::Auto.resolves_enabled(true, false));
        assert!(MotionVectorMode::Auto.resolves_enabled(false, true));
        assert!(MotionVectorMode::ForceOn.resolves_enabled(false, false));
        assert!(!MotionVectorMode::ForceOff.resolves_enabled(true, true));

        assert_eq!(LodGroupId::new(17).raw(), 17);
    }

    fn material_handle(label: &str) -> ResourceHandle<MaterialMarker> {
        ResourceHandle::new(ResourceId::from_stable_label(label))
    }
}
