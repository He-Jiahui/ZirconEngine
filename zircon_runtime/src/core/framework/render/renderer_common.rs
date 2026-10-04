use serde::{Deserialize, Serialize};

use crate::core::resource::{MaterialMarker, ResourceHandle};

use super::{RenderLayerSet, RenderQueueValue};

/// 渲染器在主视图和阴影阶段的可见性策略；ShadowsOnly 仍可投影但不进主视图。
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

/// 按材质槽排序并去重的覆盖集；构造、插入和反序列化维持同一查找契约。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct MaterialOverrideSet {
    slots: Vec<(u32, ResourceHandle<MaterialMarker>)>,
}

impl MaterialOverrideSet {
    pub fn from_slots(
        slots: impl IntoIterator<Item = (u32, ResourceHandle<MaterialMarker>)>,
    ) -> Self {
        let mut slots = slots.into_iter().collect::<Vec<_>>();
        slots.sort_by_key(|(slot, _)| *slot);
        slots.dedup_by(|current, previous| {
            if current.0 != previous.0 {
                return false;
            }
            previous.1 = current.1;
            true
        });
        Self { slots }
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn slots(&self) -> &[(u32, ResourceHandle<MaterialMarker>)] {
        &self.slots
    }

    /// Declared bytes of the owned slot buffer, using its actual retained capacity.
    /// ResourceHandle is a Copy identifier; referenced material assets belong to
    /// their resource owner and are not retained heap allocations of this buffer.
    pub fn retained_payload_bytes(&self) -> Option<usize> {
        self.slots
            .capacity()
            .checked_mul(std::mem::size_of::<(u32, ResourceHandle<MaterialMarker>)>())
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

/// 场景提取传给各渲染路径的共享渲染器策略；材质专属限制可在建 draw 前收紧。
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
#[path = "tests/renderer_common.rs"]
mod tests;
