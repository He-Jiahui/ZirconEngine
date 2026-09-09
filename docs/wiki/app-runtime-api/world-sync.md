---
related_code:
  - zircon_runtime_interface/src/world_sync/mod.rs
  - zircon_runtime_interface/src/world_sync/query.rs
  - zircon_runtime_interface/src/world_sync/watch.rs
  - zircon_runtime_interface/src/world_sync/invalidation.rs
  - zircon_runtime/src/dynamic_api/session/world_sync.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/runtime_session/foreign_output.rs
implementation_files:
  - zircon_runtime_interface/src/world_sync/query.rs
  - zircon_runtime_interface/src/world_sync/watch.rs
  - zircon_runtime_interface/src/world_sync/invalidation.rs
  - zircon_runtime/src/dynamic_api/session/world_sync.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
plan_sources:
  - user: 2026-09-09 为 ZirconEngine 构建引擎说明书级 Wiki
  - docs/zircon_runtime_interface/world_sync.md
tests:
  - zircon_runtime_interface/src/tests/world_sync_contracts.rs
  - zircon_runtime/src/dynamic_api/tests/session_entry_points.rs
doc_type: module-detail
---

# 世界同步 API

## 用途与边界

世界同步让编辑器、远程工具和运行时宿主读取 runtime-owned world，而不共享 ECS 借用、组件指针或编辑器 view id。接口层提供 transport-neutral DTO；动态 ABI 仅负责 JSON 编解码和 opaque token。

它是只读投影与失效通知协议，不是完整的 ECS 远程控制 API。写操作应通过具有事务、权限和版本语义的 operation/command 通道完成。

## 查询类型

`WorldQuery` 是 tagged JSON enum：

| variant | 功能 | 结果 |
| --- | --- | --- |
| `Components` | 用 `with`/`without` 过滤实体并选择反射组件 | `ComponentRows`，按 entity id 确定性排序 |
| `Hierarchy` | 获取 runtime-owned 层级投影 | `HierarchyRows` |
| `InspectionFields` | 获取单个实体的 Inspector 字段 | `InspectionFields` 或 `EntityMissing` |
| `TransformSnapshot` | 获取开始编辑事务所需的精确局部 Transform | `TransformSnapshot` 或 `EntityMissing` |

前三类支持 `generation_hint`。当提示与当前 generation 相同且 generation 不是 `u64::MAX` 时，运行时返回 `NotModified`，避免重复传输完整数据。`TransformSnapshot` 还返回 `world_replacement_epoch`，调用者可检测 world 被整体替换后旧事务已失效。

## Rust 查询示例

```rust
use zircon_runtime_interface::{
    ComponentSelector, ComponentWorldQuery, QueryFilter, WorldQuery,
};

let query = WorldQuery::Components(ComponentWorldQuery {
    filter: QueryFilter {
        with: vec!["zircon.transform.Transform".into()],
        without: vec!["gameplay.Disabled".into()],
    },
    select: vec![
        ComponentSelector::new("zircon.transform.Transform"),
        ComponentSelector::new("gameplay.Health"),
    ],
    generation_hint: last_generation,
});

let request_json = serde_json::to_vec(&query)?;
// 生产宿主随后调用 RuntimeSession 的 query_world 包装；直接 ABI 调用时，
// request_json 必须保持到 query_world 返回，结果必须通过 release_allocation 释放。
```

结果处理：

```rust
match result {
    zircon_runtime_interface::WorldQueryResult::ComponentRows { generation, rows } => {
        cache.replace(generation, rows);
    }
    zircon_runtime_interface::WorldQueryResult::NotModified { generation } => {
        cache.confirm(generation);
    }
    zircon_runtime_interface::WorldQueryResult::EntityMissing { entity, .. } => {
        selection.remove(entity);
    }
    _ => {}
}
```

## Watch 注册

`WatchKey` 支持：

- `Subtree { root }`：某个实体子树；
- `ComponentType { type_name }`：某类组件的相关变更；
- `Asset { resource_id }`：某个资源；
- `WorldStructure`：全局世界结构。

```rust
use zircon_runtime_interface::{WatchKey, WatchRegistration};

let registration = WatchRegistration::new(WatchKey::WorldStructure);
let json = serde_json::to_vec(&registration)?;
// watch_world 返回非零 WatchToken；token 由 runtime 签发。
```

一个 view 可以持有多个 token，每个 token 可独立撤销。运行时永远不保存 editor view id。`unwatch_world` 的 `out_removed` 为 0/1，表示 token 是否实际存在并被移除；无效零 token 属于 `InvalidArgument`。

## 失效批次

`InvalidationBatch` 包含单调 generation、dirty token 列表和事实列表。`WorldFact` 当前包含实体生成/销毁/改父级、场景加载/卸载、world replacement 和 asset reload apply report。

运行时的 dirty token 应严格递增且唯一。`has_canonical_dirty_tokens()` 提供无分配快路径检查。若外部传输返回非规范次序，数据仍可被观察和诊断，但宿主不应使用依赖有序唯一性的快速投影。

推荐消费循环：

1. 为 UI/工具关注的事实注册 watch 并保存 token 到本地 view 状态。
2. 每帧或 wake 后调用 `drain_world_invalidations`。
3. 使用 `RuntimeForeignOutputState` 按 `WORLD_INVALIDATION_OUTPUT_BUDGET` 解码和释放。
4. 将 dirty token 映射到本地 cache，再用上次 generation 作为 hint 发起精确查询。
5. view 销毁时逐个 `unwatch_world`，不要复用旧 token。

## 预算和错误

query 请求上限为 1 MiB、16,384 items、128 层、25 ms；watch 请求上限为 256 KiB、1,024 items、10 ms。query 输出上限为 1 MiB、16,384 items、25 ms；invalidation 输出采用同样大小和 item 上限，并允许规范空结果。

未知 JSON 字段因 `deny_unknown_fields` 被拒绝。格式错误返回 `InvalidArgument`，合法形状但超过资源预算返回 `LimitExceeded`。输出协议错误会触发宿主 foreign-output fuse。

## 当前状态

- 已实现：四种查询、四种 watch key、generation short-circuit、失效事实、V8 必需槽位。
- 部分实现：可查询组件取决于 runtime 反射注册和插件可用性；`serializable`/`writable` 字段只描述能力，不自动授权写入。
- 规划约束：不把编辑器状态或裸 ECS handle 加入 transport DTO。
