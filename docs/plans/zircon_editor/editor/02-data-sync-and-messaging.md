---
related_code:
  - zircon_editor/src/core/editor_message/bus.rs
  - zircon_editor/src/ui/binding_dispatch/mod.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/scene/inspection/hierarchy.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/mod.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime_interface/src/buffer.rs
reference_sources:
  - dev/bevy/crates/bevy_remote/src/lib.rs
  - dev/bevy/crates/bevy_remote/src/http.rs
  - dev/Fyrox/editor/src/plugin.rs
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md
status: in_progress
---

# 02 信息与数据同步（WorldSyncProtocol）

本计划落地 00 §5 帧数据流的「runtime → 编辑器」半程：查询/订阅/失效协议，把既有 `ViewDirtySet` 通道喂活。「编辑器 → runtime」半程归 03（事务）。

## 参照证据（dev/）

**bevy BRP**（`dev/bevy/crates/bevy_remote/src/lib.rs:8-72`）：JSON-RPC 2.0，请求 `{method, id, params}`；核心方法 `world.query`（with/without 组件过滤）、`world.get_components`（实体 id + 全限定组件类型名）、`world.insert_components`；HTTP 传输层独立于协议本体（`http.rs`）。模板要点：**结构化世界访问协议与传输解耦**——zircon 同一协议要同时服务进程内直连（零拷贝）与 ABI 序列化过界。

**Fyrox 拉模式对照**（`dev/Fyrox/editor/src/plugin.rs:46-132`）：`EditorPlugin::on_sync_to_model()` 每帧从引擎模型全量同步 UI。取其「同步点集中在一个钩子」的纪律，弃其全量拉（规模上限低，zircon hierarchy 需支撑 5k+ 节点）。

## 现状与证据（zircon，2026-07-05 实读）

**编辑器侧失效通道存在但无数据源**：`EditorMessageBus.dirty: ViewDirtySet` + `mark_view_dirty/drain_dirty`（`bus.rs:115-129`）——面板失效机制齐备，但 runtime 世界变化不会到达它。

**拉侧快照已有稳定锚、缺世代与指纹**（v2 记载「行无稳定 id」**失实，此处修正**）：

```rust
// scene/inspection/hierarchy.rs:5-15（实读全文）
pub struct WorldInspectionHierarchyRow {
    pub entity: EntityId, pub parent: Option<EntityId>,   // ← 稳定锚已在
    pub depth: u32, pub display_name: String, pub kind: String,
    pub focused: bool, pub active_in_hierarchy: bool, pub has_children: bool,
}
// scene/inspection/snapshot.rs:12-36
pub struct WorldInspection {
    pub focused_entity: Option<EntityId>,
    pub hierarchy_rows: Vec<WorldInspectionHierarchyRow>,
    pub fields: Vec<WorldInspectionField>,
}
impl WorldInspection { pub fn from_world(world: &World, focused: Option<EntityId>) -> Self { /* 全量重建 */ } }
impl World { pub fn inspect_world(&self, focused: Option<EntityId>) -> WorldInspection }
```

真实缺口：(a) 无 `generation`——编辑器无法判定快照新旧、无法做 `NotModified` 短路；(b) 无 `subtree_hash`——有锚但无「这棵子树没变」的判据，diff 仍需全树对比；(c) `from_world` 把 hierarchy 与 focused 实体的 fields 耦合在一次全量重建里——inspector 换选中会连带重建全 hierarchy 行；(d) `focused` 入参耦合选中态，01 迁出 `selected_node` 后由编辑器 SelectionModel 供值。

**推送/事件侧**：`AssetReloadFrameApplyReport { applied, failed, stale, pending_count }`（`dynamic_scene/scene/reports.rs:70-73`）按帧产出；`dynamic_api/session/events.rs` 已有 14+ 事件 ABI 转码——均不进 bus。`World` 存储按组件类型分 map（`world.rs:42-143`：`entities: Vec<EntityId>` + `hierarchy/local_transforms/world_matrices: HashMap` + `entity_registry`），对「按类型 watch」打点友好。

**LevelSystem 咽喉已核**（`level_system.rs:111-134`）：`snapshot() -> World`、`replace(World)`、`replace_world_and_reset_runtime_state(World)`、`with_world<R>(FnOnce(&World)->R)`、`with_world_mut<R>`、`tick(&CoreHandle, RuntimeTimeAdvance)`——01 gateway 的映射对象与本计划冲刷点。

**绑定消费面**：`ui/binding_dispatch/mod.rs` 是 8 个域级自由函数（`dispatch_{animation,asset,docking,draft,inspector,selection,viewport,welcome}_binding`）+ `apply_*` 回写族——事件进方向成型；数据出方向（世界→面板投影）无脏驱动，靠调用方每次重取。

## 目标

1. **`WorldSyncProtocol`**：查询 + 订阅 + 失效三段协议，DTO 全 serde 可过 ABI；InProcess 零拷贝、Session 序列化，同一契约测试双跑。
2. **世代号与子树指纹**：`World.world_generation: u64`（spawn/despawn/reparent 递增）；`WorldInspectionHierarchyRow` 增 `subtree_hash: u64`（锚已有，只补指纹）；`WorldInspection` 增 `generation` 头并把 fields 重建与 hierarchy 重建解耦（`inspect_hierarchy` / `inspect_fields(entity)` 拆分，`inspect_world` 保留为组合门面）。
3. **世界事实入总线**：`WorldFact` 族（spawn/despawn/reparent/scene load-unload/热重载报告）经泵进 bus 并驱动 `mark_view_dirty`——喂活既有 `ViewDirtySet`，成为 `editor_layout/09` 刷新总线的数据源。
4. **绑定接 watch**：绑定源声明 `WatchKey` 依赖，投影重算由 `drain_dirty` 驱动，删除无条件重取路径。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-editor-data-sync-and-messaging",
  "goal": "完成传输中立的世界查询、订阅、失效、增量投影与 ABI 过界架构",
  "milestones": [
    {"id": "M1", "title": "协议契约与世代号", "depends_on": []},
    {"id": "M2", "title": "订阅表与编辑器泵", "depends_on": ["M1"]},
    {"id": "M3", "title": "绑定接线与 ABI 过界", "depends_on": ["M2"]}
  ]
}
```

## 非目标

- 不做协作编辑/合并；不实现网络传输（协议可序列化即止）；不改面板刷新机制本身（editor_layout/09）；组件值级 change-tick 不在本期——结构级世代 + 子树指纹先行，值级由 03 事务提交事件补足。

## 架构设计

### 协议 DTO（`zircon_runtime_interface/src/world_sync/`，新建四文件）

```rust
// query.rs —— 对齐 BRP world.query / get_components
pub struct WorldQuery {
    pub filter: QueryFilter,               // with/without 组件类型名（全限定字符串）
    pub select: Vec<ComponentSelector>,
    pub generation_hint: Option<u64>,      // 相同则服务端可回 NotModified
}
pub enum WorldQueryResult { Rows(Vec<EntityRow>), NotModified { generation: u64 } }

// watch.rs
pub enum WatchKey {
    Subtree { root: EntityId },            // hierarchy 面板
    ComponentType { type_name: String },   // inspector 按类型
    Asset { resource_id: ResourceId },     // 资产依赖视图/热重载
    WorldStructure,                        // 任何 spawn/despawn/reparent
}
pub struct WatchRegistration { pub key: WatchKey }
pub struct WatchToken(pub u64);            // runtime 侧发号，unwatch/会话回收凭据

// invalidation.rs
pub struct InvalidationBatch {
    pub generation: u64,
    pub dirty: Vec<WatchToken>,            // 本帧命中的订阅（token 而非 key：编辑器侧 token→view 映射）
    pub facts: Vec<WorldFact>,
}
pub enum WorldFact {
    Spawned(EntityId), Despawned(EntityId),
    Reparented { entity: EntityId, new_parent: Option<EntityId> },
    SceneLoaded { scene: ResourceId }, SceneUnloaded { scene: ResourceId },
    AssetReloadApplied(AssetReloadFrameApplyReportDto),   // 包装既有 reports.rs 结构
}
```

命中语义定稿：`Subtree` 命中=子树内任意结构 fact 的祖先链包含 root；`ComponentType` 命中=该类型 map 的 `&mut` 访问打点（允许假阳性）；`WorldStructure` 命中=任意结构 fact。`dirty` 用 token 不用 key——同 key 多订阅者各收各的 token，编辑器侧维护 `token → ViewInstanceId` 映射，runtime 保持不认识 view 概念。

### runtime 侧（`scene/inspection/` 扩展 + 三咽喉打点）

```rust
// scene/inspection/subscription.rs（新）
pub struct SubscriptionTable {
    next_token: u64,
    by_key: BTreeMap<WatchKeyCanon, BTreeSet<WatchToken>>,   // key 规范化后索引
    pending_facts: Vec<WorldFact>,
    pending_dirty: BTreeSet<WatchToken>,
}
impl SubscriptionTable {
    pub fn watch(&mut self, reg: WatchRegistration) -> WatchToken;
    pub fn unwatch(&mut self, token: WatchToken);
    pub fn record_fact(&mut self, world: &World, fact: WorldFact);   // 折算命中入 pending
    pub fn flush(&mut self, generation: u64) -> Option<InvalidationBatch>;  // 帧末冲刷，空则 None
}
```

挂载点：`RuntimeDynamicSession` 旁（session 级字段，session 销毁即回收——ABI 崩溃清理免费获得）；进程内路径由 `InProcessGateway` 持同一张表。打点三咽喉：

1. world 结构变更 API（spawn/despawn/reparent 实现处）：递增 `world_generation` + `record_fact`；
2. `LevelSystem::tick` 末尾：`flush` 产 `InvalidationBatch` 暂存至 session 出口队列；
3. dynamic_scene 热重载应用处：`AssetReloadFrameApplyReport` 包装为 fact。

不建平行脏标记体系；`ComponentType` 打点借 `World` 分 map 结构在 `&mut` 访问口做，值级精度不足由 fact/03 事务事件兜底。

### 编辑器侧泵（gateway 扩展 + 每帧管线）

```rust
// 01 trait 本期追加
fn query(&self, q: WorldQuery) -> Result<WorldQueryResult, GatewayError>;
fn watch(&self, reg: WatchRegistration) -> Result<WatchToken, GatewayError>;
fn unwatch(&self, token: WatchToken);
fn drain_invalidations(&self) -> Vec<InvalidationBatch>;
```

主循环每帧固定一次：

```
let batches = ctx.gateway().drain_invalidations();
for batch in batches {
    for fact in batch.facts { bus.publish(TOPIC_WORLD_FACT, fact.into_message()); }  // Focus/Custom 族
    for token in batch.dirty {
        if let Some(view) = watch_map.view_for(token) { bus.mark_view_dirty(view, mask_for(token)); }
    }
    sync_state.generation = batch.generation;
}
```

新管线**只到 bus 为止**；bus 以下（`drain_dirty` → 面板重取）复用现物。`watch_map`（token→view）随视图注册/关闭登记/注销，落 `core/sync/watch_map.rs`。

### 端到端时序（改名一例，验收剧本同款）

```
用户改名 → 03 事务 apply → with_world_mut 改 display_name
→ 咽喉打点：record_fact(Reparented? 否——名称属结构 hash 输入，Subtree 命中)
→ tick 末 flush → InvalidationBatch{dirty:[hierarchy 面板 token]}
→ 泵 → mark_view_dirty(hierarchy_view) → 面板 drain_dirty 命中
→ 面板 query(Subtree, generation_hint) → 行级 diff：仅 subtree_hash 变化的子树重建
```

### 迁移映射表

| 现物 | 去向 |
| --- | --- |
| `WorldInspection::from_world` 全量重建 | 保留为组合门面；新增 `inspect_hierarchy(&World)`/`inspect_fields(&World, EntityId)` 拆分入口，hierarchy 视图与 inspector 各取所需 |
| `inspect_world(focused)` 的 focused 入参 | 调用方改由编辑器 SelectionModel 供值（01 M2 之后） |
| binding 投影无条件重取 | `depends_on: Vec<WatchKey>` 声明 + dirty 驱动；删除点执行时 Grep 定稿记状态节 |
| `session/events.rs` 既有 14+ 事件转码 | 保留 ABI 事件面；世界事实走本协议，不与其合并（事件面=输入方向，本协议=状态方向） |

### 深度测试

新增一种视图数据源（夹具：资产依赖面板）只需注册 `WatchKey::Asset` + 查询；`SubscriptionTable`/DTO/泵零改动。

## 里程碑

### M1 协议契约与世代号

- [x] **M1.1 协议 DTO 与 NotModified 契约.** `world_sync/` 四文件 + serde 往返测试 + `NotModified` 语义单测。
- [x] **M1.2 世界世代与拆分 inspection 契约.** `World.world_generation` 三咽喉打点；`WorldInspectionHierarchyRow.subtree_hash`；`inspect_hierarchy/inspect_fields` 拆分（`from_world` 改为组合调用）；既有 inspection 消费方迁移。
- [x] **M1.3 深层 hierarchy 与 malformed-edge 投影硬化.** hierarchy 构建改为显式后序栈，覆盖 5k 深链、cycle/visited edge identity、确定性 hash 与 `E0282/E0277` compile-sync；以 inspection 源码、测试、模块文档和编号产出记录组成精确清单，不把父 M1 其余 pending 范围并入本切片。
- 测试阶段：`cargo test -p zircon_runtime_interface --locked`、`cargo test -p zircon_runtime --lib scene:: --locked`（世代号单调性/结构变更必递增/拆分入口与组合门面等价断言）；authoring 边界守卫不回归——watch DTO 属会话查询面不入场景序列化，守卫矩阵显式声明。更新 `docs/zircon_runtime/scene/inspection.md`。M1.3 还须以其六文件 exact manifest 复放 native slice lifecycle，且不得据此把整个 M1 标为 completed。

### M2 订阅表与编辑器泵

- [ ] **M2.1 订阅表、失效冲刷与 gateway 实现.** `subscription.rs` + 三咽喉冲刷；gateway 四方法 InProcess 实现；`core/sync/watch_map.rs`。
- [ ] **M2.2 编辑器泵与 hierarchy 增量投影.** 主循环泵接线；hierarchy 视图迁 diff 更新（entity 锚 + subtree_hash 判重建），删除全量重建消费路径。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`；验收：5k 节点夹具单节点改名 → hierarchy 重建行数=该子树行数（计数器断言）；watch 注册/注销/session 销毁回收生命周期矩阵；泵每帧至多一次 drain 断言。

### M3 绑定接线与 ABI 过界

- [ ] **M3.1 绑定依赖与 dirty 驱动重算.** 绑定源 `depends_on` + dirty 驱动重算；无条件重取路径删除（清单记状态节）。
- [ ] **M3.2 SessionGateway ABI 过界.** `SessionGateway` 实现 query/watch/drain（经 `buffer.rs` 序列化）；契约测试双实现复跑。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（dirty 驱动重算次数断言：值不变时零重算）+ `cargo test -p zircon_runtime_interface --locked`（ABI 往返）。证据记状态节。

## 风险与开放问题

- `subtree_hash` 输入=子节点 id 序列 + display_name（不含 transform/组件值，避免每帧全树重 hash）；5k 节点重 hash 超帧预算时改父链向上传播脏位图，证据裁决。
- `ComponentType` 打点假阳性（借 `&mut` 未必写）：接受（多刷新不错刷新），03 提交事件提供精确通道后收窄。
- `fields` 拆分后 inspector 与 hierarchy 取数节奏不同步的撕裂窗口：同帧内两次 query 携带同一 `generation_hint`，世代不一致时丢弃重取（泵保证每帧 generation 单调）。

## 产出记录与时间

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

M1.1、M1.2 已完成实现；M1.3 的深层 hierarchy/cycle-edge/compile-sync 硬化也已完成并通过 fresh 定向证据。接口与 core-min 门禁全绿。Shader04/Plugins08 原 Failure 已 fixed 回传后，fresh 默认 scene 门禁为 1700 passed / 3 failed / 6 ignored，全部 Editor02 generation、inspection、5k 深链与 cycle-edge 合同通过；三条新失败已分别路由到 Runtime15、Text05、Plugins08。父 M1 保持 pending；M1.3 先按六文件 exact manifest 独立提交，M1 整体仍等待其余 lifecycle 和 fresh 整门禁；完整证据已迁入编号归档。

2026-07-18：Performance01 回传的 editor-event journal/listener 无界保留失败已完成源码硬切：统一 retention store、durable/frame-local/latest-state 独立预算、共享 Arc payload、per-listener inbox 和 sequence/journal/listener 分锁均已落地；1k/10k、字节预算、逆序 fanout 与 lag/coalesce 测试已加入。当前仅有静态 gate，受 Coordinator01 immutable full-input snapshot barrier 阻塞，尚未执行 source-bound Cargo、独立 review、failure return 或 managed commit，因此 Failure 保持 open，且不据此勾选 M2。

- 迁入记录：[`02/2026-07-14-world-sync-m1-output-records.md`](02/2026-07-14-world-sync-m1-output-records.md)
- 当前源状态收束：[`02/2026-07-17-m1-current-source-status-reconciliation.md`](02/2026-07-17-m1-current-source-status-reconciliation.md)
- editor-event retention/lock split：[`02/2026-07-18-editor-event-retention-and-lock-split.md`](02/2026-07-18-editor-event-retention-and-lock-split.md)

## Failure 生命周期

- fixed 已修复：[mutation-queue-finish-lease-stall](02/fixed-2026-07-14-mutation-queue-finish-lease-stall.md)
- fixed 已修复：[compute-fullscreen-descriptor-compile-regression](02/fixed-2026-07-14-compute-fullscreen-descriptor-compile-regression.md)
- fixed 已修复：[ecs-resource-marker-owner-missing](02/fixed-2026-07-14-ecs-resource-marker-owner-missing.md)
- fixed 已修复并返回（Runtime02 owner；`SystemStage` 唯一 owner 与结构守卫均已硬切至 `core/framework/scene`）：[`system-stage-owner-guard-drift`](../../zircon_runtime/runtime/08/fixed-2026-07-14-system-stage-owner-guard-drift.md)。
- fixed 已修复并返回（Frameworks05 owner；`LevelManager` consumer 与 versioned-handle 测试均已硬切）：[`level-manager-export-cutover-incomplete`](../../zircon_runtime/runtime/02/fixed-2026-07-14-level-manager-export-cutover-incomplete.md)。
- fixed 已修复：[f18-asset-manager-review-guard-owner-drift](02/fixed-2026-07-14-f18-asset-manager-review-guard-owner-drift.md)
- fixed 已修复：[core-runtime-state-plugin-bridge-lifecycle-anchor-drift](02/fixed-2026-07-14-core-runtime-state-plugin-bridge-lifecycle-anchor-drift.md)
- fixed 已修复：[level-manager-name-core-error-import-drift](02/fixed-2026-07-14-level-manager-name-core-error-import-drift.md)
- fixed 已修复：[project-asset-manager-access-test-consumer-drift](02/fixed-2026-07-14-project-asset-manager-access-test-consumer-drift.md)
- fixed 已修复：[editor-retained-host-manager-resolver-consumer-drift](02/fixed-2026-07-14-editor-retained-host-manager-resolver-consumer-drift.md)
- fixed 已修复：[runtime-diagnostics-pane-payload-visibility-drift](02/fixed-2026-07-14-runtime-diagnostics-pane-payload-visibility-drift.md)
- fixed 已修复：[vm-reflection-catalog-test-support-import-drift](02/fixed-2026-07-14-vm-reflection-catalog-test-support-import-drift.md)
- fixed 已修复：[cargo-release-retains-live-child-process-lock](02/fixed-2026-07-14-cargo-release-retains-live-child-process-lock.md)
- fixed 已修复：[standard-pbr-transmission-render-queue-root-export-drift](02/fixed-2026-07-14-standard-pbr-transmission-render-queue-root-export-drift.md)
- fixed 已修复：[advanced-pbr-transparent-selection-uninitialized](02/fixed-2026-07-14-advanced-pbr-transparent-selection-uninitialized.md)
- fixed 已修复：[ui-text-module-split-import-drift](02/fixed-2026-07-14-ui-text-module-split-import-drift.md)
- fixed 已修复：[material-abi-layout-expectation-drift](02/fixed-2026-07-14-material-abi-layout-expectation-drift.md)
- fixed 已修复：[dynamic-reflection-json-projection-regression](02/fixed-2026-07-14-dynamic-reflection-json-projection-regression.md)
- fixed 已修复：[mutation-queue-offline-recurrence](02/fixed-2026-07-14-mutation-queue-offline-recurrence.md)
- fixed 已修复：[depth-prepass-source-guard-owner-drift](02/fixed-2026-07-15-depth-prepass-source-guard-owner-drift.md)
- fixed 已修复：[sdf-font-bake-cjk-loaded-font-count-regression](02/fixed-2026-07-15-sdf-font-bake-cjk-loaded-font-count-regression.md)
- fixed 已修复：[vm-dynamic-property-write-structure-regression](02/fixed-2026-07-14-vm-dynamic-property-write-structure-regression.md)
- fixed 已修复：[support-slice-exact-finalize-plan-output-conflict](02/fixed-2026-07-16-support-slice-exact-finalize-plan-output-conflict.md)
- open / Coordinator01 native slice closeout checker 仍依赖共享暂存区：[native-slice-closeout-checker-staged-index-contract-drift](../../zircon_tooling/session_coordinator/01/failure-2026-07-16-native-slice-closeout-checker-staged-index-contract-drift.md)
- open / editor-event retention 源码已落地但 source-bound Cargo/review/fixed return 仍被 Coordinator01 immutable full-input barrier 阻塞：[editor-event-journal-listener-unbounded-retention](02/failure-2026-07-17-editor-event-journal-listener-unbounded-retention.md)
- open / World sync subscription基础源码已落地但subtree触发按watch重复祖先遍历/visited分配，asset/component/fact burst仍有全扫、分配与无界风险；按variant/root direct index和bounded coalesce归PERF-MVP-468：[world-sync-subscription-invalidation-scaling](02/failure-2026-07-22-world-sync-subscription-invalidation-scaling.md)
- open / 待修复；plugin registration 必须在 extension batch 成功后原子提交 runtime-event consumers：[plugin registration runtime consumer atomicity](02/failure-2026-08-01-plugin-registration-runtime-consumer-atomicity.md)
- 2026-07-22 message/event当前源码复核：inbox已分lossless/bounded/latest且event retention已分级有界/shared payload；本轮补O(1) message depth、ack removed-byte单遍、首末sequence O(1)并删除listener status的records clone+merge+sort。剩余PERF-MVP-019/067要求per-owner inbox/listener queue、immutable route generation、latest key index、锁外fanout，以及payload size不在每事件热路重复serde traversal。
- 2026-07-30 current-source校正：`src/tests/editor_event`现为22/22文件、117 tests；retention已有10k paused-event、1k latest-state、byte/age/lag/Arc共享覆盖，但没有listener-count、锁等待/持有、clone bytes或cursor分页成本门。PERF-MVP-067验收必须增加polling/stalled listener storm，记录payload/delivery/JSON clone bytes、global/per-owner lock wait、queue age与p95；保留现有sequence/lag/ack语义。
- 2026-07-22 runtime event consumer复核：lock-out callback、generation commit和256 events/4ms公平预算已成立，但gateway仍在预算前无上限drain成完整Vec并append无界pending，每delivery还双锁active map。PERF-MVP-069必须把`max_events/max_bytes/deadline`推进到Runtime10 ABI/typed producer，返回remaining/oldest age，editor只拉本帧额度；不得只在ABI尾端截断。
- 2026-07-23 runtime event capability reconcile补充：`tick_runtime_event_consumers`当前每active play tick先clone完整`EditorCapabilitySnapshot`与enabled Vec，再让host全量clone registrations、构建desired BTreeMap/active BTreeSet与delta Vec；stable capability仍持续主线程O(C+R logR)。Editor02按PERF-MVP-565保存Editor12发布的capability generation和自身registration generation，只在begin/change时更新affected subscriptions；稳定tick只执行runtime demand与PERF-MVP-069有界pump，capability/registry/active锁、Map/Set/Vec build和subscribe/unsubscribe均为0。
- 2026-07-22 world watch-map复核：同view多token合并已改borrowed mark，ViewInstanceId只在首次
  dirty insert clone；Editor02静态合同7/7。runtime typed direct index/bounded fact coalesce已有源码，但editor
  每batch仍建seen/duplicate/unknown三套BTreeSet且transport dirty Vec无count/bytes/canonical标志；按更新后
  PERF-MVP-468要求canonical bounded batch正常快路，malformed diagnostics另走慢路，Cargo/100k/F4待验收。
- 2026-07-23 `zircon_runtime_interface/src/world_sync/**` 4/4性能复核：M1.1 DTO当前仍只有contract-test caller；
  `WorldQuery::result_for_generation`在接收已完整物化的`Vec<EntityRow>`后才判断`generation_hint`，因此
  `NotModified`无法跳过producer hierarchy/component访问、JSON/String/BTreeMap分配与row构造。按
  PERF-MVP-563增加generation-first lazy builder并让runtime producer在投影前短路；完整inspection generation
  与watch/fact budget继续分别归PERF-MVP-456/468，未完成current-source Cargo、规模counter与F4接线前保持pending。
- 2026-07-23 reflection transport补充：schema/fields DTO无generation/NotModified/page/bytes/depth，Editor动态Inspector snapshot按component深clone schema与全部values。Editor02按PERF-MVP-567让runtime query携带catalog/object generation与有界cursor，stable generation直接NotModified；进程内只转发Runtime13共享catalog/field-delta handle，不把owned JSON page缓存成第二authority。与PERF-MVP-456的world inspection generation同源验收。

## 2026-07-30 Performance01 editor_message current-source supplement

- 当前`zircon_editor/src/core/editor_message/**` 29/29源文件已静态复读。旧“latest在混合Vec线性找key”结论不再成立：当前已有lossless/bounded/latest独立lane、`latest_by_key`索引、O(1) depth、shared payload与4096/256/256 entries、16MiB inbox硬界。
- PERF-MVP-019仍由Editor02负责总线/队列根因：一个全局mutex持有subscription、全部inbox、dirty与fanout；lossless预检和enqueue重复构造/计量，零target与不读细节report的caller仍物化delivery/Vec，`drain_deliveries`锁内整箱搬空。计划是immutable route generation、per-owner inbox、prepared shared delivery和count+bytes+deadline page，返回remaining与oldest wall age；不得破坏lossless atomic fanout、request revalidation、ordering与dirty语义。
- PERF-MVP-594由Editor12/Plugins01主责、Editor02提供page drain合同、Runtime11只提供显式affinity的bounded ticket：当前lifecycle bridge把整箱delivery复制到第二pending，并在持pending mutex时于UI tick无预算执行全部active-plugin callback。验收要求单一bounded owner、callback-in-lock=0、每tick entry/bytes/time硬预算、slow/error/reload/unload generation安全与无loss/dup/reorder；不创建私有线程池。
- 2026-07-30 Performance01 topic增量：document与transaction publisher已使用canonical `EditorTopic` constructor，去掉静态topic的重复validation scan；constructor仍逐调用拥有String，scene/tool路径仍parse。Editor02把topic String allocation纳入PERF-MVP-019的1/100/10K publish counter，但不为低频document cadence另建任务；优先级仍是全局bus锁、单次shared delivery和bounded page drain。current 29-file指纹与证据见`../../performance/01/2026-07-30-editor-core-editor-message-current-review.md`。
- `SceneInspection`以一个latest key承载incremental generation delta，Editor02必须把gap检测和full-resync定义为协议验收项，不能以coalesce后“最终generation更新”代替artifact可重建证明。当前managed Cargo、contention/backpressure counter和F4 retained-host WPR仍pending，本补充不勾选M2。

## 2026-07-30 Performance01 editor_event current-source supplement

- 当前`zircon_editor/src/core/editor_event/**` 32/32生产文件及`src/tests/editor_event/**` 22/22测试文件已逐文件静态复读。旧“journal/listener无界且fanout逐listener深clone payload”根因已被三类硬预算retention与共享`Arc<SharedEditorEventRecord>`替代；filter预规范化、ack单遍removed-byte与status首尾O(1)也已成立。
- PERF-MVP-067仍由Editor02负责：`SharedEditorEventRecord::new`每event完整serde counting traversal，成功dispatch先clone完整record；LatestState每inbox线性找key并中段remove；全局listener mutex跨全listener filter/prune/coalesce/enqueue。查询先clone三队列Arc并全sort，cursor过滤在后，再转owned delivery与JSON。
- 修复合同是shared encoded owner/构造期一次accounting、immutable route/filter generation与锁外per-owner enqueue、latest key index、cursor-first count+bytes+deadline k-way page及仅ABI边界owned JSON；不得退回无界channel、不得在锁内调用foreign callback、不得用私有线程池掩盖主线程工作。
- 验收矩阵覆盖0/1/1k/10k listeners/events、64B/2MiB/64MiB payload、0/50/100% filter、0/1/99% cursor与1/16 threads；记录serde traversals、record/delivery/JSON clone bytes、coalesce visits/shifts、merge/sort、visited/returned rows、global/per-owner lock、queue bytes/age、p95/RSS，并复跑117 tests及F4 retained-host WPR。
- 当前两个`EditorEventRecord`测试literal静态缺`binding_path`、`transaction_id`、`save_generation`，因此尚无current-source managed Cargo证据；Performance01未改这些foreign dirty文件，也不据此宣称RED或勾选M2。

## Code Review 建议 (2026-07-30)

### 与代码现状不符，需修订

- §现状与证据仍以现在时把「无 `generation` / 无 `subtree_hash` / `from_world` 把 hierarchy 与 fields 耦合」列为「真实缺口 (a)(b)(c)」，但这些在 M1（已勾选 [x]）中均已落地并可核：`zircon_runtime/src/scene/inspection/hierarchy.rs:12` 已有 `subtree_hash: u64`；`snapshot.rs:14` 的 `WorldInspection` 已有 `generation: u64`，且 `from_world`（`snapshot.rs:21-32`）已改为组合调用 `build_hierarchy_rows` + `inspect_fields`，另暴露解耦入口 `World::inspect_hierarchy()`（`snapshot.rs:42-44`，focus 无关）与 `World::inspect_fields(entity)`（`snapshot.rs:47-49`）。建议把 §现状证据的 (a)(b)(c) 标注为「M1 已收敛」，与已勾选的 M1.1/M1.2/M1.3 保持一致，避免读者按「缺口仍在」重复实现。
- §现状证据 (d)「`focused` 入参耦合选中态，01 迁出 `selected_node` 后由编辑器 SelectionModel 供值」：`inspect_hierarchy()` 已不接 focus 参数（`snapshot.rs:42`），`from_world` 的 `focused` 仅用于筛 fields 与 `focused_entity` 标记。结合 01 M2.3 已删除 `RuntimeDynamicSession.selected_node`，此项也应更新为「hierarchy 查询已与 focus 解耦；fields 查询的 focus 由编辑器侧供值」。
- §迁移映射表首行「`WorldInspection::from_world` 全量重建 → 保留为组合门面；新增 `inspect_hierarchy/inspect_fields` 拆分入口」已完成（`snapshot.rs:37-49`），建议标记为已落地，与 §里程碑 M1.2 的 [x] 呼应。

### 验证缺口

- §现状证据把 `subscription.rs` 列为「新建」待办，但 `zircon_runtime/src/scene/inspection/subscription.rs` 与 `subscription/tests.rs` 已存在且 `flush(world.world_generation())` 已被测试覆盖（`subscription/tests.rs:75-123`），另有 `artifact.rs` 的 generation-bound `Arc` 复用测试（`tests.rs:60-74`）。这与 §里程碑 M2.1「[ ] 未勾选」存在张力——订阅表基础源码已落地但 M2.1 仍标未完成。建议在状态节明确 M2.1 的「已落地部分（SubscriptionTable/flush/artifact Arc 复用）」与「仍缺部分（gateway 四方法 InProcess 实现、`core/sync/watch_map.rs` 编辑器泵）」，避免 checkbox 与代码现状读起来矛盾（open failure `world-sync-subscription-invalidation-scaling` 已记录 runtime 侧扩展性债，可交叉引用）。
