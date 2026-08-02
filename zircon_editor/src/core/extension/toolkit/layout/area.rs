use std::sync::Arc;

use super::ToolkitLayoutError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ToolkitAreaSlot {
    Center,
    Left,
    Right,
    Bottom,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolkitArea {
    slot: ToolkitAreaSlot,
    tabs: Arc<[String]>,
    active_tab: String,
}

impl ToolkitArea {
    pub fn new<Tab, Tabs, Active>(
        slot: ToolkitAreaSlot,
        tabs: Tabs,
        active_tab: Active,
    ) -> Result<Self, ToolkitLayoutError>
    where
        Tab: Into<String>,
        Tabs: IntoIterator<Item = Tab>,
        Active: Into<String>,
    {
        let tabs = tabs.into_iter().map(Into::into).collect::<Vec<_>>();
        if tabs.is_empty() {
            return Err(ToolkitLayoutError::EmptyTabs { slot });
        }
        if tabs.iter().any(|tab| tab.trim().is_empty()) {
            return Err(ToolkitLayoutError::EmptyTabId);
        }
        let mut sorted = tabs.clone();
        sorted.sort();
        for pair in sorted.windows(2) {
            if pair[0] == pair[1] {
                return Err(ToolkitLayoutError::DuplicateTabId {
                    slot,
                    tab: pair[0].clone(),
                });
            }
        }
        let active_tab = active_tab.into();
        if !tabs.contains(&active_tab) {
            return Err(ToolkitLayoutError::ActiveTabNotFound { slot, active_tab });
        }
        Ok(Self {
            slot,
            tabs: tabs.into(),
            active_tab,
        })
    }

    pub const fn slot(&self) -> ToolkitAreaSlot {
        self.slot
    }

    pub fn tabs(&self) -> &[String] {
        &self.tabs
    }

    pub fn active_tab(&self) -> &str {
        &self.active_tab
    }
}
