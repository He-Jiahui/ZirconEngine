---
title: Runtime Scene ECS / World / Archetype / Query / Schedule / Generation 当前工作树复审
category: zircon_runtime
report_id: Runtime187
review_date: 2026-08-30
baseline_head: working-tree
baseline_epoch: 2026-08-30
verification_head: working-tree
verification_epoch: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99i-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99k-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-current-source-review.md
related_owner_reports:
  - docs/plans/optimize/zircon_runtime/186-runtime-physics-backend-shape-query-event-lifecycle-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/175-runtime-gameplay-ability-effect-attribute-tag-cue-prediction-current-working-tree-authority-artifact-execution-review.md
  - docs/plans/optimize/zircon_editor/246-editor-physics-authoring-preview-debug-current-working-tree-review.md
related_failure:
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-22-level-system-runtime-state-frame-snapshot.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-world-fixed-component-storage-and-stable-query-index.md
related_code:
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/identity.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/generation.rs
  - zircon_runtime/src/scene/world/transaction.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/inspection/artifact/cache.rs
  - zircon_runtime/src/scene/inspection/artifact/data.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/scene/ecs/archetype/index.rs
  - zircon_runtime/src/scene/ecs/archetype/table/table.rs
  - zircon_runtime/src/scene/ecs/component/registry.rs
  - zircon_runtime/src/scene/ecs/entity/registry.rs
  - zircon_runtime/src/scene/ecs/query/query_state
  - zircon_runtime/src/scene/ecs/commands/command_queue.rs
  - zircon_runtime/src/scene/ecs/commands/worker_command_buffer.rs
  - zircon_runtime/src/scene/ecs/change_detection/component_mutation.rs
  - zircon_runtime/src/scene/ecs/scene_system_registry.rs
  - zircon_runtime/src/scene/ecs/schedule.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/World.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h
  - dev/bevy/crates/bevy_ecs/src/world/mod.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/mod.rs
  - dev/bevy/crates/bevy_ecs/src/change_detection/mod.rs
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/graph/mod.rs
  - dev/Fyrox/fyrox-core/src/pool/mod.rs
  - dev/godot/scene/main/node.cpp
  - dev/godot/scene/main/scene_tree.cpp
  - dev/godot/core/object/object.cpp
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime187 · Scene ECS / World Authority 复审

## 1. 结论

Runtime108/109/110 已分别记录 ECS、World 和 hierarchy 的早期差异。本轮按当前工作树重新读取 `World`、archetype/table/sparse storage、entity/component registry、query state、commands/events/change detection、schedule runner、derived state、inspection artifact、dynamic component 和 transaction。当前代码已经具有真正 ECS 的局部机制，但产品边界仍是多套平行投影拼接：

- `World` 同时维护 `entities + entity_dense_rows`、`kinds`、`EntityRegistry + StableQueryOrderIndex`、`ArchetypeIndex + ComponentStorage`、`dynamic_components`、`node_cache` 和 inspection artifact。每套结构有自己的重建和 generation 规则，尚未有一个不可绕过的 entity/component authority。
- `World::clone` 先复制数个 persistent component snapshot，再重建 entity registry、archetype index 和 component storage；`WorldPersistentState` 又把组件拆成二十多个 `HashMap<EntityId, T>`。这让 Play、preflight、serialization 和 render/physics extraction 都依赖完整投影，而不是零拷贝 world snapshot。
- `node_record`/`node_records` 会从多个 component store 重新拼装 owned `NodeRecord`；inspection artifact、world sync、render、physics 和 Editor hierarchy 各自再缓存一份。增量 dirty journal 只覆盖少数路径，组件新增、动态 JSON、resource 和 registry 变化没有统一 extraction revision。
- `WorldGeneration`、`ChangeTick`、`LifecycleVisibilityRevision`、`SceneBindingGenerations`、archetype generation、hierarchy topology generation 和 component schema generation 并存；部分 generation 可饱和，持久化 equality 又刻意忽略 runtime revision，无法作为跨世界/跨帧的单一一致性 token。
- Schedule 虽有 conflict graph、build receipt 和 worker executor，但 native worldful system 仍被主线程序列化，runtime system 每次强制 flush；worker result 使用 `Arc<Mutex<Option<_>>>`，batch/command/timing allocation 和 panic recovery 仍是每帧成本。schedule registry 可在 taken systems 存在时变化，未形成原子 plan activation。
- command/event/message/observer/removal 都有局部队列和诊断，却缺少统一 world generation、sequence/ack、capacity disposition、retirement fence 和 replay contract。失败路径大量依赖 `expect` 或 poison recovery，不能把“内部不变量失败”和“用户输入错误”区分为可诊断的 terminal state。

因此本报告不新增 P0，继承 Runtime108/109/110 及 Runtime186 的 composition、world replacement、provider 和 frame authority P0；新增 **30 项 P1、12 项 P2、30 个资格门**。P1 为 **27 Open、3 Partial、0 Closed**；P2 为 **12 Open、0 Partial、0 Closed**；资格门为 **28 Fail、2 Partial、0 Pass**。没有相同 world 数据、同一 schedule、同一 hardware 和 correctness corpus 的实测，不能声称 ECS 或 world pipeline 的性能优于 Unreal/Bevy。

## 2. 审查边界与方法

本轮扫描 `zircon_runtime/src/scene/world` 全部 owner、`scene/ecs` 的 entity/component/archetype/table/sparse/query/change/command/event/resource/schedule 代码，以及 inspection artifact 和 world query 的消费者。测试只用来确认 generation、swap-remove、preflight 和 NotModified 合同，不把测试数量当作完成度；tooling 按用户要求排除。

调用链按以下顺序核对：

```text
scene load / spawn / transaction
  -> stable entity + component registry + archetype/table/sparse storage
  -> typed/dynamic query + change ticks + deferred commands
  -> schedule build/conflict/worker execution + apply-deferred
  -> hierarchy/active/world-matrix/node-record/inspection artifacts
  -> render/physics/animation extraction + world sync + Editor authoring/play
```

参考侧实际存在的源码包括 Unreal `UWorld`/TaskGraph、Bevy `World`/Schedule/change ticks/Entity、Fyrox generation Pool/Graph，以及 Godot Node/SceneTree/Object notification and process ownership。

## 3. 当前真实调用链

| 链路 | 当前事实 | 工程判定 |
|---|---|---|
| Identity | Vec+HashMap、EntityRegistry、stable query order、allocator 同时存在 | P1-001..004 |
| Typed storage | ArchetypeIndex/table columns 与 ComponentStorage sparse set 共同持有位置和 ticks | P1-005..008 |
| Node projection | `node_record` 逐组件读取，`node_cache` 再保存 `Vec<SceneNode>` | P1-009..011 |
| Dynamic schema | JSON map、ComponentTypeRegistry、TypeRegistry、dynamic presence row 分离 | P1-012..014 |
| Derived state | hierarchy topology/active/world matrix/node cache/inspection 各自 dirty/generation | P1-015..018 |
| Snapshot/IO | clone 重建 projection；persistent state 为分散 typed maps，runtime state 被清空或复制 | P1-019..022 |
| Query/change | query state/archetype plans 有缓存，但 world query components 仍构造全量 records/fields | P1-023..025 |
| Schedule | conflict graph/build receipt 存在；worldful native 与 runtime steps 受 flush/serial 约束 | P1-026..027 |
| Commands/events | deferred queue、worker buffers、events、messages、observers、removed events 各有自己的 retention/lock | P1-028..030 |

## 4. 继承边界

本轮不重复计数既有 Runtime108/109/110 的 ECS/World/hierarchy P0，以及 Runtime186 的 Physics provider、single fixed clock、backend truth 和 world replacement P0。ECS 报告中的 generation、snapshot、query 和 extraction gate 必须先关闭这些 owner 的错误 authority，才能被 Physics/Render/Editor 作为事实源。

## 5. P1 差异与重构要求

| ID | 状态 | 当前源码证据 | 必须重构 |
|---|---|---|---|
| ECS4-P1-001 | Open | `World` 同时维护 `entities`, `entity_dense_rows`, `EntityRegistry`, `StableQueryOrderIndex` | 以一个 generational `EntityStore` 作为唯一 identity/location authority；其他索引只能是带 generation 的派生视图 |
| ECS4-P1-002 | Open | `kinds: HashMap<EntityId, NodeKind>` 与 component presence/archetype signature 分离，NodeKind 可与实际 components 漂移 | 将 kind 变成注册 schema/tag component 或明确派生字段，spawn/deserialize/transaction 使用同一验证器 |
| ECS4-P1-003 | Open | `entities.swap_remove` 更新 dense row，stable order 另行维护，删除/重排有两个顺序语义 | 引入稳定迭代序列或明确 dense/stable cursor contract；所有 persistence/query/render/editor 使用同一排序 token |
| ECS4-P1-004 | Open | `rebuild_entity_registry_with_stable_order` 会从 Vec 重新生成 registry，多个路径可直接 reset | 禁止生产路径任意 rebuild；只允许 versioned migration/recovery，并发布 identity map generation 和失败 receipt |
| ECS4-P1-005 | Partial | `ArchetypeIndex` 保存 signature->id、component->ids、record/table；`ComponentStorage` 再保存 sparse rows | 统一 archetype/column/sparse ownership，提供 component location API；index 只保存不可变 metadata，不能复制 live row truth |
| ECS4-P1-006 | Open | table `swap_remove` 移动所有 columns，sparse rows 也独立更新；跨组件移动靠多次 transition | 设计 atomic row move transaction，一次生成 old/new location、moved entity、ticks 和 rollback/commit receipt |
| ECS4-P1-007 | Open | `ComponentRegistry`, `ComponentTypeRegistry`, `TypeRegistry` 各自有 id/descriptor/schema 语义 | 建立 canonical `ComponentDescriptor` registry，Rust/dynamic/reflection/resource 通过稳定 type id、layout、storage、schema revision 对齐 |
| ECS4-P1-008 | Open | table column allocation 使用增长策略和 `expect`，没有 world/archetype memory admission | 引入 allocator/arena budget、capacity high-water、OOM/fragmentation disposition；禁止 allocation failure 变 panic |
| ECS4-P1-009 | Open | `node_record()` 对 Name/Hierarchy/Transform/Render/Physics/Animation 逐个 `get` 并 clone；`node_records()` 被多个系统全量调用 | 以 archetype query 或 generation-scoped extraction view 直接写入 SoA/borrowed rows，增量消费者只读取 changed entities |
| ECS4-P1-010 | Open | `node_cache: Vec<SceneNode>` + `node_cache_rows: HashMap` 是完整 owned projection，拓扑变化会清空重建 | 将 cache 改为 immutable artifact/chunk delta，携带 source world/component revision；避免 per-entity SceneNode 复制 |
| ECS4-P1-011 | Partial | inspection artifact 已有 Arc、name overrides 和 subtree hash，但完整重建仍调用 `world.node_records()` 并维护多个 HashMap | 让 inspection、render、physics、Editor 共享 typed extraction journal；按 entity/component chunk 发布 changed/removed/unchanged disposition |
| ECS4-P1-012 | Open | `dynamic_components: HashMap<EntityId, HashMap<String, Value>>` 与 dynamic presence sparse component 分离 | dynamic component 使用 typed column/byte payload + schema revision + stable component id；JSON 只在 import/export 边界出现 |
| ECS4-P1-013 | Open | dynamic schema 同时进入 `component_types`, `type_registry`, VM path sets 和 ComponentRegistry descriptor imports | 合并注册/卸载为 schema transaction，带 plugin lease、active instance count、dependency and migration receipt |
| ECS4-P1-014 | Open | dynamic component generation 按 type 独立递增，world generation 另递增；property write 可能多次标记/失效 | 定义 source/component/entity/field 四级 revision，单一 commit token 原子更新所有 cache/subscription |
| ECS4-P1-015 | Open | `DerivedStateDirty` 以多个 frontier 管理 hierarchy/active/transform/node cache，`should_run` 依赖 schedule plan | 建立依赖图和 per-system input/output revision；每个 derived artifact 只消费明确 source revision，不靠全局 dirty bool |
| ECS4-P1-016 | Open | hierarchy topology 不 current 时构造临时 `HierarchyTraversalIndex`，并在 rebuild 期间 take/restore topology | 用 persistent parent/child adjacency with mutation log；循环、孤儿、排序和 rebuild 作为 typed validation result |
| ECS4-P1-017 | Open | active/world matrix propagation 以 root frontier 遍历，parent lookup 重复 HashMap/get；cycle 只在 read projection 通过 HashSet 拦截 | 编译 hierarchy forest 的 parent index/depth/roots，传播一次生成 depth/active/transform chunks，cycle/invalid parent 进入 error artifact |
| ECS4-P1-018 | Open | node cache/inspection/render dirty 标记分散在 compiled binding、query、derived state 和 world methods | 单一 `WorldChangeJournal` 记录 structural/component/property/derived causes、source tick、entity generation 和 consumer cursor |
| ECS4-P1-019 | Open | `World::clone` 复制 snapshots、maps、queues、events/resources，再重建 registry/archetype/component storage | 实现 copy-on-write/chunk snapshot 或 explicit `WorldSnapshot`；区分 persistent source、runtime state、queues、subscriptions 和 derived artifacts |
| ECS4-P1-020 | Open | clone 会复制 schedule/commands/events/observers，随后重置 sink/deferred maps；语义取决于调用时 world state | snapshot policy 必须显式列出 retained/dropped/rehydrated state，携带 source world id/generation and provider leases |
| ECS4-P1-021 | Open | `WorldPersistentState` 是二十多个 component maps，serde 字段/默认值/旧 rename 分散；动态 schema 不在 state 中 | 使用 versioned component stream/chunk schema，包含 component id/layout/schema hash、migration and unknown-component policy |
| ECS4-P1-022 | Open | deserialize 后 builtin reflection 注册、entity registry rebuild、component registry reset、storage projection rebuild 多阶段完成 | loader 采用 validate -> allocate -> publish 单一 transaction；所有 orphan/duplicate/unsupported/capacity errors 可定位且不暴露半成品 world |
| ECS4-P1-023 | Open | `query_world` Components 路径先 `node_records()`，每行再 `build_inspection_fields`; bounded path 仍逐 entity 计算 | 将 reflected query 编译成 component/archetype plan，按 matching chunks 批量输出，提供 cursor/page/overflow and source revision |
| ECS4-P1-024 | Partial | query state/cache/archetype plan 有 generation counters，但缓存 invalidation 与 component registry/derived changes 不是同一 token | query cache key 固定为 world identity + registry/schema/archetype generation + filter hash，并支持 lease/eviction/diagnostics |
| ECS4-P1-025 | Open | change detection 使用多个 ChangeTick/active tick，`saturating`/retention 路径没有统一 wrap/expiration contract | 采用明确 tick domain、wrap-safe age policy、system last-run cursor 和 check_tick；跨 snapshot/replay 保留 tick mapping |
| ECS4-P1-026 | Open | Schedule registry 保存 descriptors/native boxes/runtime slots；serde 只保存 descriptors，runtime materialization 不在 artifact 中 | schedule asset 只发布 immutable plan descriptor；运行时 provider 通过 activation receipt 注入 systems，plan generation 原子切换 |
| ECS4-P1-027 | Open | systems taken in flight 时 register/unregister 会延迟 refresh；runner 对 worldful native/runtime steps 反复 flush，runtime system 无并行 lane | 采用 immutable plan lease + frame barrier；按 access set 编译 worker/worldful batches，deferred apply 作为显式 dependency node |
| ECS4-P1-028 | Open | worker batch 每次分配 timing/command buffers，`Arc<Mutex<Option<Result>>>` 收集结果；panic 后可能已有局部 world mutation | 使用 per-world frame arena、typed result channel 和 command-only worker isolation；panic/error 只能提交或丢弃完整 batch，并发布 fault receipt |
| ECS4-P1-029 | Open | `CommandQueue`、worker buffers、deferred spawn maps 使用不同 ordinal/sequence；target generation/ack/retention 没有统一协议 | 一个结构化 command log：sequence, source system, target entity generation, apply tick, outcome, capacity and replay id |
| ECS4-P1-030 | Open | EventStore/MessageStore/ObserverStore/RemovedComponentEvents/WorldSync sink 各自锁、cursor、poison recovery；跨 world retirement 无总 fence | 统一 per-world event bus with topic schema, cursor lease, bounded retention, overflow and retirement generation；poison 进入 faulted world，不 `into_inner` 继续成功 |

## 6. P2 差异

| ID | 当前问题 | 重构方向 |
|---|---|---|
| ECS4-P2-001 | entity/component ids、archetype ids、schema ids、world generation 的位宽/溢出策略不统一 | 统一 id domain、generation width、exhaustion error 和 persistence encoding |
| ECS4-P2-002 | HashMap/BTreeMap 混用，stable ordering 依赖额外 sort/collect | 为 hot query/iteration 采用 deterministic dense index；map 仅作 cold lookup |
| ECS4-P2-003 | table column 以 Rust layout/Vec growth 为中心，没有 chunk/page locality contract | 采用 archetype chunk/page size、SoA alignment、prefetch and fragmentation metrics |
| ECS4-P2-004 | sparse set 与 table component 的 tick/borrow/error shape 不一致 | 统一 `ComponentRef/ComponentMut` metadata、ticks、storage kind and access diagnostics |
| ECS4-P2-005 | `NodeKind` 只有 9 个 enum variant，扩展 node/component 需要修改 world snapshot fields | 用 registered scene class/component schema，内置类型只是 catalog entries |
| ECS4-P2-006 | `WorldPersistentState` 缺 asset dependency/hash/unknown field provenance | 加入 asset reference graph、source hash、migration warning and unknown field retention |
| ECS4-P2-007 | resource registry/store 参与 clone/transaction，但 resource schema/reload/borrow 与 component 不同 | 统一 resource descriptor/lease/version and world snapshot policy |
| ECS4-P2-008 | schedule conflict graph 的 conservative access 可能把可并行系统降为 serial，缺 planner quality budget | 发布 conflict reason、batch utilization、critical path and tuning diagnostics |
| ECS4-P2-009 | diagnostics counters 大量 `saturating_add`，无法区分 overflow 与真实极值 | counters 使用 checked/overflow bit，并在 telemetry 中报告 lost counts |
| ECS4-P2-010 | `expect` 描述内部不变量，但没有 fault injection/repair mode | 为 storage move、plan build、cache publication、event bus 增加故障注入和 recovery corpus |
| ECS4-P2-011 | world query bounded budget 只限制 items/bytes/time，缺 cancellation and fairness across consumers | query ticket + scheduler quota + per-consumer budget/priority/partial page protocol |
| ECS4-P2-012 | derived artifacts 与 render/physics snapshots 的 source tick 仍可由调用方任意传入 | 所有 extraction receipt 由 World frame boundary 生成，禁止 consumer 自行伪造 generation |

## 7. 资格门

| Gate | 验收条件 | 当前 |
|---|---|---|
| ECS4-G01 | 一个 entity identity/location authority 覆盖 spawn/remove/query/serialize | Fail |
| ECS4-G02 | stale entity handle 在所有 typed/dynamic/resource routes 上被 generation 拒绝 | Partial |
| ECS4-G03 | dense/stable iteration order 有公开且持久化一致的 contract | Fail |
| ECS4-G04 | kind/presence/archetype signature 不可互相漂移 | Fail |
| ECS4-G05 | row move 原子更新所有 columns, sparse rows, ticks and location | Fail |
| ECS4-G06 | component descriptors/layout/storage/reflection 只有一个 canonical registry | Fail |
| ECS4-G07 | allocation/capacity/fragmentation 超限返回 typed disposition，不 panic | Fail |
| ECS4-G08 | node/inspection/render/physics extraction 使用同一 change journal | Fail |
| ECS4-G09 | `node_records()` 不再是大多数 consumer 的全量中间层 | Fail |
| ECS4-G10 | hierarchy cycle/orphan/order validation 产生带 source revision 的结果 | Partial |
| ECS4-G11 | active/world transform propagation 按 changed roots/chunks 增量执行 | Fail |
| ECS4-G12 | derived artifact 只接受匹配的 world/component/topology generation | Fail |
| ECS4-G13 | clone/snapshot 明确 persistent/runtime/queue/subscription/derived policy | Fail |
| ECS4-G14 | snapshot/reopen 不丢失 stable ids, schema, assets and migration provenance | Fail |
| ECS4-G15 | load/deserialize 是 validate-then-publish transaction，无半成品可见 | Fail |
| ECS4-G16 | dynamic component JSON 只在 boundary 使用，runtime typed storage 可查询 | Fail |
| ECS4-G17 | dynamic schema unload/reload 受 plugin lease 和 active instance 保护 | Fail |
| ECS4-G18 | world generation/change tick/lifecycle/schema revisions 有映射且不会饱和静默 | Fail |
| ECS4-G19 | component query 按 archetype plan/chunk 批量执行，具备 page/cursor/overflow | Fail |
| ECS4-G20 | query cache key 包含 world identity、schema、archetype 和 filter generation | Fail |
| ECS4-G21 | change detection 通过 wrap/expiration/replay corpus | Fail |
| ECS4-G22 | schedule plan 有 immutable lease 与 atomic activation barrier | Fail |
| ECS4-G23 | worker/worldful system 按 access set 真并行，deferred apply 是显式依赖 | Fail |
| ECS4-G24 | worker error/panic 不产生部分 world mutation，且保留 fault receipt | Fail |
| ECS4-G25 | command log 有 sequence、target generation、ack、capacity、replay outcome | Fail |
| ECS4-G26 | event/message/observer/removal 共享 bounded cursor/overflow/retirement contract | Fail |
| ECS4-G27 | world replacement 在所有 derived/query/event consumers 上有 retirement fence | Fail |
| ECS4-G28 | render/physics/animation/Editor 读取同一 frame source revision | Fail |
| ECS4-G29 | 1K/10K/100K entity 的 memory、query、schedule、extraction 曲线有基准 | Fail |
| ECS4-G30 | correctness corpus 通过后才允许宣称优于 Unreal/Bevy 的性能 | Fail |

## 8. 参考引擎差异

- Bevy `World` 把 entity allocator、component/resource registry、change tick 和 schedule 作为一个明确 world owner，`Entity` generation 和 change-tick wrap policy 有直接 API；Zircon 需要收敛并非继续增加 map/index。
- Unreal `UWorld`/`World.cpp` 将 level/world lifecycle、component registration、tick groups 和 world teardown 由单一生命周期边界控制，TaskGraph 以依赖图和线程安全 contract 调度；Zircon 当前 runner 的 worker-safe worldless 限制和多锁队列不能承载同等产品语义。
- Fyrox generation Pool/Graph 让 handle validity、node graph ownership 和 reorder/remap 成为基础数据结构；Zircon 的 Vec+HashMap+registry+stable index 组合需要同样的 generational invariant，而不是调用方分别维护。
- Godot Node/SceneTree/Object 以 enter/exit tree、notification、process owner/thread group 和 deferred call 形成可观察 lifecycle；Zircon 的 hierarchy/active/derived systems 与 event/observer stores 还没有一个统一 lifecycle event stream。

## 9. 重构顺序与 owner

1. 先由 Runtime08/110 owner 收敛 EntityStore、ComponentDescriptor registry、archetype row move 和 hierarchy forest；冻结 legacy maps 只读迁移适配层。
2. 建立 `WorldSnapshot`/`WorldChangeJournal`/frame source revision，令 clone、serialization、inspection、render、physics 和 Editor 共用同一 source/artifact boundary。
3. 把 dynamic schema、query plan、change ticks、resources 和 events 接入同一 generation/lease/overflow protocol，并移除 JSON/全量 NodeRecord 作为 hot path 中间层。
4. 重构 schedule 为 immutable plan lease + access-set executor + command-only worker batch；补齐 panic/fault/rollback、capacity、replay 和 1K/10K/100K benchmark。
5. Runtime/Editor/Physics/Render 的所有 current-source 报告关闭对应 gate 后，才可进行跨引擎性能结论。

本报告仅写 review 与重构合同，没有修改 Runtime、Editor、Rust、Cargo 或资产；tooling 迁移按用户要求另立范围。
