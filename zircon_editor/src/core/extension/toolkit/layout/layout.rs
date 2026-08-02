use std::collections::BTreeSet;
use std::sync::Arc;

use super::{ToolkitArea, ToolkitAreaSlot, ToolkitLayoutError};

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
        let mut slots = BTreeSet::new();
        for area in &areas {
            if !slots.insert(area.slot()) {
                return Err(ToolkitLayoutError::DuplicateAreaSlot { slot: area.slot() });
            }
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
