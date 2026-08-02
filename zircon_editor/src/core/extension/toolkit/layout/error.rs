use thiserror::Error;

use super::ToolkitAreaSlot;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ToolkitLayoutError {
    #[error("document toolkit layout id cannot be empty")]
    EmptyLayoutId,
    #[error("document toolkit layout must declare at least one area")]
    EmptyAreas,
    #[error("document toolkit tab id cannot be empty")]
    EmptyTabId,
    #[error("document toolkit area {slot:?} must declare at least one tab")]
    EmptyTabs { slot: ToolkitAreaSlot },
    #[error("document toolkit layout declares area {slot:?} more than once")]
    DuplicateAreaSlot { slot: ToolkitAreaSlot },
    #[error("document toolkit area {slot:?} declares tab {tab:?} more than once")]
    DuplicateTabId { slot: ToolkitAreaSlot, tab: String },
    #[error("document toolkit area {slot:?} active tab {active_tab:?} is not declared")]
    ActiveTabNotFound {
        slot: ToolkitAreaSlot,
        active_tab: String,
    },
}
