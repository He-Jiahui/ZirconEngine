use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::editor_event::ViewInstanceId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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
