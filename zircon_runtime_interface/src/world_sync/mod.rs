//! Transport-neutral world query, watch, and invalidation contracts.
//!
//! 此模块只定义 Editor 与 Runtime 交换的查询、订阅和失效 DTO，不持有 World 或视图状态。
//! Editor 可通过统一网关或 Runtime 会话提交 WorldQuery；Runtime 返回 WorldQueryResult，
//! 并用 WatchToken 与 InvalidationBatch 推送订阅失效。反射回调替换运行目标时，错误边界
//! 使用 LifecycleTargetChange 表达被替换的目标。
//!
//! # 使用时机与生命周期
//!
//! 查询和事实都属于运行时世界的投影。WatchToken 只在签发它的 Runtime 会话内有效，
//! Editor 侧须把 token 与自己的会话和视图绑定；关闭会话或撤销 watch 后不能继续复用。
//!
//! # 约束
//!
//! DTO 通过 Serde 跨传输边界；本模块不验证实体当前是否存在，也不执行订阅匹配。
//! 查询实现位于 zircon_runtime/src/scene/inspection/snapshot.rs，订阅匹配位于
//! zircon_runtime/src/scene/inspection/subscription.rs；Editor 网关边界见
//! zircon_editor/src/core/gateway/contract.rs。
//!
//! # 参考
//!
//! world_sync_contracts.rs 覆盖该模块的公开重导出和 JSON 契约。

#[path = "invalidation.rs"]
mod invalidation;
#[path = "lifecycle_target_change.rs"]
mod lifecycle_target_change;
#[path = "query.rs"]
mod query;
#[path = "watch.rs"]
mod watch;

pub use invalidation::{AssetReloadFrameApplyReportDto, InvalidationBatch, WorldFact};
pub use lifecycle_target_change::LifecycleTargetChange;
pub use query::{
    ComponentSelector, ComponentWorldQuery, EntityId, EntityRow, QueryFilter, WorldHierarchyQuery,
    WorldHierarchyRow, WorldInspectionFieldRow, WorldInspectionFieldsQuery, WorldQuery,
    WorldQueryResult, WorldTransformSnapshotQuery,
};
pub use watch::{WatchKey, WatchRegistration, WatchToken};
