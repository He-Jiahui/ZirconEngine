//! Borrowed menu data and typed edit preparation; no popup, focus or document owner.

#[path = "adapter.rs"]
mod adapter;
#[path = "types.rs"]
mod types;

pub use adapter::AiBehaviorTreeMenuAdapter;
pub use types::{
    AiBehaviorTreeMenuCommand, AiBehaviorTreeMenuDisabledReason, AiBehaviorTreeMenuError,
    AiBehaviorTreeMenuRequest, AiBehaviorTreeMenuRow, AiBehaviorTreeNodeMenuRow,
};
