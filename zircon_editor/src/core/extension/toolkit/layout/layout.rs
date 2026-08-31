use std::sync::Arc;

use super::{ToolkitArea, ToolkitAreaSlot, ToolkitLayoutError};

#[cfg(test)]
#[path = "layout/slot_bitset_tests.rs"]
mod slot_bitset_tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolkitLayout {
    id: String,
    areas: Arc<[ToolkitArea]>,
}

impl ToolkitLayout {
    pub fn new(
        id: impl Into<String>,
        areas: impl IntoIterator<Item = ToolkitArea>,
    ) -> Result<Self, ToolkitLayoutError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ToolkitLayoutError::EmptyLayoutId);
        }
        let areas = areas.into_iter().collect::<Vec<_>>();
        if areas.is_empty() {
            return Err(ToolkitLayoutError::EmptyAreas);
        }
        let mut occupied_slots = 0u8;
        for area in &areas {
            let slot_bit = area_slot_bit(area.slot());
            if occupied_slots & slot_bit != 0 {
                return Err(ToolkitLayoutError::DuplicateAreaSlot { slot: area.slot() });
            }
            occupied_slots |= slot_bit;
        }
        Ok(Self {
            id,
            areas: areas.into(),
        })
    }

    pub fn single_tab(
        id: impl Into<String>,
        tab: impl Into<String>,
    ) -> Result<Self, ToolkitLayoutError> {
        let tab = tab.into();
        Self::new(
            id,
            [ToolkitArea::new(
                ToolkitAreaSlot::Center,
                [tab.clone()],
                tab,
            )?],
        )
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn areas(&self) -> &[ToolkitArea] {
        &self.areas
    }
}

const fn area_slot_bit(slot: ToolkitAreaSlot) -> u8 {
    1 << match slot {
        ToolkitAreaSlot::Center => 0,
        ToolkitAreaSlot::Left => 1,
        ToolkitAreaSlot::Right => 2,
        ToolkitAreaSlot::Bottom => 3,
    }
}
