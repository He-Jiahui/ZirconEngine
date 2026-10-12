//! 外部槽位进入档案的写入边界；追加拒绝同名，更新插入则允许整体替换同名槽位。
#[path = "push.rs"]
mod push;
#[path = "upsert.rs"]
mod upsert;

pub(super) use push::push_slot;
pub(super) use upsert::upsert_slot;
