//! 显式 ID 的槽位管理边界；供 facade、选择器和路径接口共用变更规则及只读摘要。
mod metadata;
mod remove;
mod rename;
mod report;
mod touch;

pub(super) use metadata::{preview_update_slot_metadata, update_slot_metadata};
pub(super) use remove::{preview_remove_slot, remove_slot};
pub(super) use rename::{preview_rename_slot, rename_slot};
pub(super) use touch::{preview_touch_slot, touch_slot};
