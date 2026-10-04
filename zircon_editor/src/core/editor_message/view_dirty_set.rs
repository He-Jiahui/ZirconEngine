use std::collections::BTreeMap;

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use crate::core::editor_event::ViewInstanceId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
/// 视图需重新计算的工作类别；生产者按真实影响组合标志，消费者选择布局、反射或绘制路径。
pub struct EditorViewInvalidationMask(u16);

impl EditorViewInvalidationMask {
    pub const NONE: Self = Self(0);
    pub const LAYOUT: Self = Self(1 << 0);
    pub const TREE_STRUCTURE: Self = Self(1 << 1);
    pub const PRESENTATION_DATA: Self = Self(1 << 2);
    pub const PAINT_ONLY: Self = Self(1 << 3);
    pub const POINTER_HOVER: Self = Self(1 << 4);
    pub const VIEWPORT_IMAGE: Self = Self(1 << 5);
    pub const HIT_TEST: Self = Self(1 << 6);
    pub const WINDOW_METRICS: Self = Self(1 << 7);
    pub const RENDER: Self = Self(1 << 8);

    const KNOWN_BITS: u16 = Self::LAYOUT.0
        | Self::TREE_STRUCTURE.0
        | Self::PRESENTATION_DATA.0
        | Self::PAINT_ONLY.0
        | Self::POINTER_HOVER.0
        | Self::VIEWPORT_IMAGE.0
        | Self::HIT_TEST.0
        | Self::WINDOW_METRICS.0
        | Self::RENDER.0;

    pub const fn bits(self) -> u16 {
        self.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub const fn requires_host_layout(self) -> bool {
        self.intersects(
            Self::LAYOUT
                .union(Self::TREE_STRUCTURE)
                .union(Self::WINDOW_METRICS),
        )
    }
}

impl<'de> Deserialize<'de> for EditorViewInvalidationMask {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bits = u16::deserialize(deserializer)?;
        if bits & !Self::KNOWN_BITS != 0 {
            return Err(D::Error::custom(
                "view invalidation mask contains unknown bits",
            ));
        }
        Ok(Self(bits))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 按视图实例合并本轮失效请求；同一实例的标志取并集，排空后的新请求属于下一刷新轮。
pub struct ViewDirtySet {
    views: BTreeMap<ViewInstanceId, EditorViewInvalidationMask>,
}

impl ViewDirtySet {
    pub fn mark(&mut self, view: ViewInstanceId, mask: EditorViewInvalidationMask) {
        if mask.is_empty() {
            return;
        }
        self.views
            .entry(view)
            .and_modify(|existing| existing.insert(mask))
            .or_insert(mask);
    }

    pub(crate) fn mark_ref(&mut self, view: &ViewInstanceId, mask: EditorViewInvalidationMask) {
        if mask.is_empty() {
            return;
        }
        if let Some(existing) = self.views.get_mut(view) {
            existing.insert(mask);
        } else {
            self.views.insert(view.clone(), mask);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.views.is_empty()
    }

    pub fn len(&self) -> usize {
        self.views.len()
    }

    pub fn mask_for(&self, view: &ViewInstanceId) -> Option<EditorViewInvalidationMask> {
        self.views.get(view).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ViewInstanceId, EditorViewInvalidationMask)> {
        self.views.iter().map(|(view, mask)| (view, *mask))
    }
}
