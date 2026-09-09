---
related_code:
  - zircon_runtime/src/scene/world/mod.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/records.rs
implementation_files:
  - zircon_runtime/src/scene/world/world.rs
plan_sources:
  - user: 2026-09-09 ECS 世界说明
tests:
  - zircon_runtime/src/scene/world
  - zircon_runtime/src/scene/tests
doc_type: module-detail
---

# ECS World、实体与层级

## 用途与模型

`World` 是 `Scene` 的类型别名，集中保存实体注册表、组件存储、资源存储、事件、观察者、schedule、层级索引和 generation。实体 ID 类型为 `u64`（`EntityId`/`NodeId`）。节点的持久化投影是 `NodeRecord`，包含名称、`NodeKind`、父级、`Transform`、激活状态、渲染层、mobility 以及可选组件。

## 创建与结构变更

`World::empty()` 仅注册内置反射类型；`World::new()` 额外生成默认相机、方向光和立方体。`spawn_node` 使用分配器生成 ID 并插入默认 `NodeRecord`；`spawn_mesh_node` 同时安装模型和材质句柄。`insert_node_records`/`insert_owned_node_records` 批量校验重复 ID、变换有限性、mobility 和父级后一次性提交。

层级操作使用 `set_parent_checked`，会拒绝自父、缺失父级、环和静态节点重挂载。`remove_entity` 删除实体组件并将直接子节点变为孤儿；需要递归删除时使用动态场景 transaction 的 detached batch API。

## 类型化组件与查询

`insert::<T>`、`get::<T>`、`get_mut::<T>`、`remove::<T>` 操作实现 `Component` 的 Rust 类型；`insert_resource`/`get_resource` 管理全局 `Resource`。`query` 和 `query_filtered` 返回 `QueryState<D,F>`，可组合 `With`、`Without`、`Changed`、`Added` 等过滤器。结构命令可通过 `Commands` 延迟到 `apply_deferred`。

```rust
use zircon_runtime::scene::{NodeKind, World};
use zircon_runtime::scene::components::{LocalTransform, Name};

let mut world = World::empty();
let root = world.spawn_node(NodeKind::Empty)?;
let child = world.spawn_node(NodeKind::Mesh)?;
world.set_parent_checked(child, Some(root))?;
world.insert(child, Name("Hero".into()))?;
if let Some(local) = world.get_mut::<LocalTransform>(child) {
    local.transform.translation.x = 2.0;
}
let record = world.node_record(child).expect("node exists");
```

## 错误与限制

`SceneError` 覆盖缺失实体、重复实体、层级环、静态变换/重挂载、无效浮点变换和未注册动态组件类型。静态 mobility 节点不能更新 `LocalTransform`；动态父节点不能拥有静态子节点。组件生命周期事件、change tick 和 world generation 会在每次成功变更时更新。

## 实现状态

已实现完整类型化 ECS、稳定查询顺序、层级索引、资源存储、延迟命令和 NodeRecord 序列化投影；动态反射组件需先注册 `ComponentTypeDescriptor`。

## 源码与测试

[world/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/world/mod.rs)、[bootstrap.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/world/bootstrap.rs)、[typed_api.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/world/typed_api.rs)、[hierarchy.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/world/hierarchy.rs)。
