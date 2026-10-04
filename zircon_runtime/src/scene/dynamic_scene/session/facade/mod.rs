//! Archive 对外操作的薄入口；行为分派给会话域实现，避免公开内部计划与索引布局。

mod capture;
mod construction;
mod mutation;
mod query;
mod restore;
mod retention;
mod store;
mod transfer;
mod validation;
