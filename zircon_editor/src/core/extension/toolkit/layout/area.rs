use std::collections::HashSet;
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
/// 单个工作台槽位的 tab 契约；构造后活动 tab 属于非空且无重复的集合。
pub struct ToolkitArea {
    slot: ToolkitAreaSlot,
    tabs: Arc<[String]>,
    active_tab: String,
}

impl ToolkitArea {
    /// 插件注册布局前完成校验并保留声明顺序，供工作台稳定恢复活动页。
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
        let mut unique_tabs = HashSet::with_capacity(tabs.len());
        let mut first_duplicate = None;
        for tab in &tabs {
            if !unique_tabs.insert(tab.as_str()) {
                first_duplicate = Some(match first_duplicate {
                    Some(previous) if previous < tab.as_str() => previous,
                    _ => tab.as_str(),
                });
            }
        }
        if let Some(tab) = first_duplicate {
            return Err(ToolkitLayoutError::DuplicateTabId {
                slot,
                tab: tab.to_string(),
            });
        }
        let active_tab = active_tab.into();
        if !unique_tabs.contains(active_tab.as_str()) {
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

#[cfg(test)]
#[path = "tests/area_optimization_tests.rs"]
mod optimization_tests;
