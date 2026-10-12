//! Asset pipeline service wiring, request types, and worker execution.

#[path = "manager/mod.rs"]
pub mod manager;
#[path = "types.rs"]
pub mod types;
#[path = "worker_pool.rs"]
pub mod worker_pool;
