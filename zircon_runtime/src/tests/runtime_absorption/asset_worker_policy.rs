//! 资产工作池的并发与关闭策略需与资产管线的文档门禁一致。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "asset_worker_policy/split_layout.rs"]
mod split_layout;
#[path = "asset_worker_policy/worker_pool.rs"]
mod worker_pool;
