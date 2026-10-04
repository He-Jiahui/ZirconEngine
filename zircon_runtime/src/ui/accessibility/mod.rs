//! 把共享无障碍快照与动作接入 Runtime UI 表面；外部桥接只消费接口层语义，不直接改树。
//! 动作先以当前快照做可见性/能力门控，再由焦点、属性或文本事务 owner 执行。

pub(crate) use action::dispatch_accessibility_action;
pub(crate) use budget::{AccessibilityBuildBudget, AccessibilitySnapshotBudgetError};
pub(crate) use extract::{accessibility_snapshot, accessibility_snapshot_bounded};

mod action;
mod budget;
mod diagnostics;
mod extract;
mod name;
mod semantic_text;

#[cfg(feature = "accessibility-accesskit")]
pub(crate) mod accesskit;
