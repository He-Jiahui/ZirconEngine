//! Archive 对外操作的薄入口；行为分派给会话域实现，避免公开内部计划与索引布局。

#[path = "capture/mod.rs"]
mod capture;
#[path = "construction/mod.rs"]
mod construction;
#[path = "mutation/mod.rs"]
mod mutation;
#[path = "query/mod.rs"]
mod query;
#[path = "restore/mod.rs"]
mod restore;
#[path = "retention/mod.rs"]
mod retention;
#[path = "store/mod.rs"]
mod store;
#[path = "transfer/mod.rs"]
mod transfer;
#[path = "validation/mod.rs"]
mod validation;
