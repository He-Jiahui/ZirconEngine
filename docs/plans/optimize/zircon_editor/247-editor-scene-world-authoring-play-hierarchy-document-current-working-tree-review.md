---
title: Editor Scene World / Authoring / Play / Hierarchy / Document 当前工作树复审
category: zircon_editor
report_id: Editor247
review_date: 2026-08-30
baseline_head: working-tree
baseline_epoch: 2026-08-30
verification_head: working-tree
verification_epoch: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/61-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document-product-integration-current-source-review.md
related_owner_reports:
  - docs/plans/optimize/zircon_runtime/187-runtime-scene-ecs-world-archetype-query-schedule-generation-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/186-runtime-physics-backend-shape-query-event-lifecycle-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/246-editor-physics-authoring-preview-debug-current-working-tree-review.md
related_failure:
  - docs/plans/zircon_editor/editor/01/failure-2026-07-31-authoring-world-test-concrete-level-manager.md
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-22-level-system-runtime-state-frame-snapshot.md
related_code:
  - zircon_editor/src/core/project/scene_document.rs
  - zircon_editor/src/core/project/scene_load_job.rs
  - zircon_editor/src/core/editing/authoring_world.rs
  - zircon_editor/src/core/editing/command/play_transform.rs
  - zircon_editor/src/core/gateway/session/world_sync.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/play/controller/runtime_ownership.rs
  - zircon_editor/src/core/play/snapshot/store.rs
  - zircon_editor/src/core/play/pending_edits/queue.rs
  - zircon_editor/src/ui/workbench/state/editor_state_play_mode.rs
  - zircon_editor/src/ui/workbench/state/editor_state_keep_play_changes.rs
  - zircon_editor/src/ui/workbench/state/scene_document_binding.rs
  - zircon_editor/src/ui/host/editor_world_sync.rs
  - zircon_editor/src/ui/host/editor_scene_document_submission.rs
  - zircon_editor/src/ui/host/editor_scene_mode_lifecycle.rs
  - zircon_editor/src/ui/host/play_hierarchy_projection.rs
  - zircon_editor/src/ui/host/play_inspector_projection.rs
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
  - zircon_editor/src/ui/host/play_gizmo.rs
  - zircon_editor/src/ui/host/play_hierarchy_projection.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_world_watch.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_rename.rs
  - zircon_editor/src/ui/retained_host/app/play_viewport_pick.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/World.cpp
  - dev/bevy/crates/bevy_ecs/src/world/mod.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/mod.rs
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

# Editor247 · Scene World / Authoring / Play Authority 复审

## 1. 结论

Editor60/61 记录了 hierarchy 和 Scene Document 的早期问题。本轮重新扫描 ProjectSceneDocument/load job、EditorAuthoringWorld gateway、Play controller/snapshot/pending edits、Workbench state、world sync pump、scene inspection publication、Play hierarchy/inspector/gizmo 和 retained hierarchy routes。当前 Editor 的错误不是“没有 UI”，而是同一 Scene 被多个生命周期和缓存层重复解释：

- Project document 持有一个 `Scene`，authoring gateway 又通过 serialized callback 暴露 world，Play 通过 materialized JSON snapshot/另一个 gateway 运行；Editor state 只保存 session/selection，缺一个公开的 source-document -> runtime-world -> projection lineage。
- `EditorAuthoringWorld::snapshot` 直接 `with_world(Clone::clone)`；Runtime187 已证明 clone 会复制 snapshot/maps/queues 并重建 storage。进入 Play、失败恢复、Keep Play Changes 和 inspection 都可能产生完整 world clone，且 clone policy 对 resources/events/schedule/derived cache 不是 editor 可见的契约。
- Scene document open/create 有 staging URI 和 project authority，但 load job 只是一项 `Job`，缺取消、去重、依赖 asset/schema revision、progress phase、document lease 和 multi-document conflict resolution。save/submit route 与 runtime world generation 的绑定分散在 host state。
- Play hierarchy projection 做 identity/generation/NotModified 检查是可保留的底座，但 rows 仍为完整 `Arc<[...]>`，`row()` 线性查找；Play inspector 固定 100ms cadence，并把 reflected fields 转成新的 plugin snapshots。大场景、快速 world replacement 或多 view 时会重复复制和筛选。
- `Keep Play Changes` 只允许单一 play entity，按字符串 component type path 过滤 Hierarchy，然后把字段转成 EditorCommand；它没有 schema-aware diff、entity mapping、component add/remove、array/object patch、conflict resolution 或 transaction preview。
- world sync、scene inspection publication、hierarchy watcher、play hierarchy and inspector each maintain their own generation/identity/selection caches；lock poison 统一 `into_inner`，stale/replacement 主要通过调用方清理，缺总的 document/world retirement fence。

因此本报告不新增 P0，继承 Editor60/61、Runtime187 和 Runtime186 的 world authority/provider/snapshot P0；新增 **28 项 P1、10 项 P2、25 个资格门**。P1 为 **25 Open、3 Partial、0 Closed**；P2 为 **10 Open、0 Partial、0 Closed**；资格门为 **23 Fail、2 Partial、0 Pass**。没有带 document revision、world generation、selection identity 和 frame timing 的大场景实测，不能声称 Editor hierarchy/play pipeline 达到 Unreal/Unity 的工程性能。

## 2. 审查边界与方法

本轮扫描 Scene Document/ProjectAuthority、scene load job、authoring world gateway、Play backend/controller/snapshot/pending edits、Workbench state、WorldSyncPump integration、Editor host submit/lifecycle/world replacement、hierarchy/inspector/inspection/gizmo projections 和 retained hierarchy input routes。Tooling 按用户要求排除。

按以下链路核对每个 authority 和失败状态：

```text
project manifest/catalog -> SceneOpen/Create/Load job
  -> ProjectSceneDocument + AuthoringWorld gateway
  -> Editor transaction/journal/save/recovery
  -> Play snapshot/materialized world + gateway identity
  -> WorldSync watches/invalidation/replacement
  -> hierarchy/inspector/gizmo/selection projections
  -> Keep Play Changes / world replacement / close / retirement
```

参考 Unreal UWorld lifecycle/TaskGraph、Bevy World/Schedule、Fyrox generational Pool/Graph、Godot Node/SceneTree/Object notifications，重点核对 source document、stable identity、undo/redo、runtime clone、incremental projection 和 teardown。

## 3. 当前真实调用链

| surface | 当前事实 | 工程判定 |
|---|---|---|
| Document | `ProjectSceneDocument` 以 `Scene` owned world 保存；create 用 staging URI 后 publish | P1-001..004 |
| Load | `ProjectSceneLoadJob` 调 ProjectAuthority open，只有单一 job ticket | P1-005..006 |
| Authoring | gateway callback 序列化访问，snapshot 是 `Scene::clone`，poison lock 取 inner | P1-007..009 |
| Play | materialized snapshot 写 `.zircon/play/<instance>`，controller 管 gateway/terminal backend | P1-010..013 |
| Sync | Edit/Play 各一把 `Mutex<WorldSyncPump>`，watch/pump/replacement 独立 | P1-014..016 |
| Hierarchy | Play projection 保 identity/generation，但完整 rows/changed rows 复制，row lookup O(n) | P1-017..020 |
| Inspector | 固定 100ms query，fields -> plugin component snapshots，editor fallback 自动 field editor | P1-021..023 |
| Keep Changes | 单一 play entity + inspection fields -> EditorCommand，排除 Hierarchy | P1-024..025 |
| Host/UI | scene inspection publication、retained watcher、gizmo、selection route 各自缓存/发布 | P1-026..028 |

## 4. 继承边界

本轮不重复计数 Editor60/61 的 hierarchy/document P0，以及 Runtime187 的多 authority/entity generation/snapshot P0、Runtime186 的 physics provider P0。Editor 不能通过额外 projection 或固定 feedback 补足 Runtime 的缺失事实；所有 Play/Inspector/Overlay 必须显示 source/document/world/provider generation 与 stale/unsupported disposition。

## 5. P1 差异与重构要求

| ID | 状态 | 当前源码证据 | 必须重构 |
|---|---|---|---|
| ED5-P1-001 | Open | `ProjectSceneDocument` 同时以 URI、source path、owned `Scene` 表示一个 document；没有 immutable DocumentId/source revision | 引入 `DocumentIdentity + SourceRevision + WorldBinding`，所有 view/transaction 用 identity 而非裸 URI/Scene clone |
| ED5-P1-002 | Open | `PreparedSceneCreation` 使用 staging file 与 `expect` document availability；publish/rollback 与 project catalog 分步完成 | Document transaction 应有 prepare/commit/rollback receipt、CAS revision、recovery journal 和 crash cleanup |
| ED5-P1-003 | Open | `open_scene` 直接 `Scene::load_scene_from_uri`，schema/assets/plugin catalog revision 未作为 admission | load 前解析 schema/asset/plugin dependencies，绑定 catalog revision，缺依赖返回 typed unavailable |
| ED5-P1-004 | Open | Scene save/open API 仍以完整 Scene serialization 为边界，无 component chunk/unknown field provenance | versioned scene package、component stream、asset hash、migration and unknown field retention |
| ED5-P1-005 | Open | `ProjectSceneLoadJob` 只有 progress 0/1，未见 cancellation/dedup/priority/lease | load ticket 支持 cancellation/coalescing/priority、phase progress、document lease、terminal error and retry |
| ED5-P1-006 | Open | 多个 open/create/load 请求没有可见的 per-document conflict/duplicate identity policy | ProjectAuthority 建立 single-flight document registry、open generation、duplicate source arbitration and close fence |
| ED5-P1-007 | Partial | `EditorAuthoringWorld` 通过 gateway serialized callback 访问 Scene，`with_world`/`with_world_mut` 以协议错误保护 re-entry | authoring gateway 应携带 DocumentId/world identity/generation/capability，提供 borrowed view/transaction API，禁止 snapshot clone 作为默认读路径 |
| ED5-P1-008 | Open | `EditorAuthoringWorld::snapshot` 直接 `Clone::clone`，clone 会复制/重建多个 Runtime stores | 使用 Runtime `WorldSnapshot` chunk/COW；明确 persistent/runtime/resource/event/schedule/derived retention policy |
| ED5-P1-009 | Open | access failure 在 Mutex poison 后 `into_inner`，`expect_with_world` 以 panic 表示未加载/协议失败 | gateway faulted state + last-good revision；UI 显示 unavailable，不能把 poison/协议故障降级为成功 |
| ED5-P1-010 | Open | Play snapshot materialize 为 JSON 临时文件，instance id 由 pid/nanos/atomic sequence 生成 | Play instance 绑定 source document revision、snapshot hash、world/provider generation；支持 in-memory/COW path 和 crash orphan GC |
| ED5-P1-011 | Open | `MaterializedPlayScene::cleanup`/Drop 忽略 Drop cleanup error；controller/backend/gateway retirement 分散 | Play lifecycle 使用 explicit lease/fence，cleanup terminal receipt、orphan registry、retry and stale consumer revocation |
| ED5-P1-012 | Open | Play controller 管 terminal backend/gateway/mode 多把 lock；mode transition 与 world replacement callback 分步 | 一个 `PlaySession` authority 统一 mode, instance, source revision, world identity, backend and transition transaction |
| ED5-P1-013 | Partial | `PlayInstanceId`/`GatewaySessionIdentity` 可校验，但 edit/play document mapping 仍依赖 entity raw id | entity mapping artifact 记录 source entity generation、runtime entity、instantiation/remap and destroyed/created disposition |
| ED5-P1-014 | Open | Edit/Play 各有独立 `Mutex<WorldSyncPump>`，watch/unwatch/pump/replacement 没有共同 document lifecycle | `WorldProjectionHub` 统一 document/world domain、watch leases、replacement epoch、consumer cursors and retirement barrier |
| ED5-P1-015 | Open | `editor_world_sync.rs` 每次从 domain 取 gateway/pump，lock poison 直接 recover；shutdown 只 play pump | sync pump 应有 identity-qualified state machine，edit/play/remote shutdown 都产生 receipt，poison 进入 faulted/reconnect |
| ED5-P1-016 | Open | foreign world query/invalidation 经 JSON payload 与 fixed output budgets，缺 page cursor/consumer fairness | query/invalidation protocol 使用 typed pages、world frame/source revision、cursor ack、overflow/resync and cancellation |
| ED5-P1-017 | Partial | `PlayHierarchyProjection` 检查 generation/identity/NotModified，但 `rows` 是完整 Arc slice，topology change 仍重流 | 使用 immutable chunk hierarchy + row delta/index artifact；generation 绑定 world/source/provider and bounded reflow budget |
| ED5-P1-018 | Open | `PlayHierarchyProjection::row` 线性 `iter().find`；changed rows 以 zip 全表比较 | 建立 EntityId -> row index/chunk map，使用 changed entity set/anchor journal，禁止每次 view update 全表比较 |
| ED5-P1-019 | Open | hierarchy reflow 根据 `same_topology` 与 `SceneInspectionHierarchyFragment::patch` 决定，selection/expanded state 不在 runtime artifact | Editor retained hierarchy state 分离 topology/selection/expansion/filter revisions，runtime delta 不覆盖 UI-local state |
| ED5-P1-020 | Open | `SceneInspectionPublication` 通过 Arc pointer compare artifact/fields，focused/selection state 与 world generation 分开 | publish envelope 携带 document/world/replacement/selection revisions，支持 consumer cursor、resync reason and stale drop |
| ED5-P1-021 | Open | Play Inspector 固定 `Duration::from_millis(100)`，每次 query 后重建 `InspectorSnapshot`/plugin component vectors | query cadence 由 view budget/dirty field journal 驱动，fields 使用 shared immutable artifact/field diff and backpressure |
| ED5-P1-022 | Open | `play_inspector_projection` 对缺少 FieldEditor 时 `.unwrap_or_else(FieldEditorInstance::automatic)`，未知 field 仍可编辑 | field editor 必须由 schema/capability/permission 决定；unsupported/readonly/stale 显示 typed state，禁止 silent automatic write |
| ED5-P1-023 | Open | Inspector 由字符串 component/type/property path 分组，字段值从 reflected value 再格式化为 strings | canonical reflected field descriptor + typed units/ranges/validation/identity；避免 display string 作为 mutation payload |
| ED5-P1-024 | Open | Keep Play Changes 要求单一 play selection，只复制 inspection fields，并硬编码排除 `Hierarchy` | diff engine 支持 multi-selection/entity mapping/component add/remove/array patch、schema-aware conflict and preview transaction |
| ED5-P1-025 | Open | Keep changes 读 play 后在 authoring `with_world` 构造 commands，再 begin history；play snapshot/source revision 不是 CAS precondition | apply 需 source/play revision CAS、per-field conflict report、undoable transaction、partial/abort policy and savepoint |
| ED5-P1-026 | Open | `editor_scene_document_submission.rs`、scene mode lifecycle、world replacement routes 分别提交/清理/发布，缺统一 operation receipt | 建立 SceneWorldOperation coordinator：open/replace/play/apply/close 的 typed operation id、preflight、commit/rollback and notifications |
| ED5-P1-027 | Open | retained hierarchy watcher/rename/pick/gizmo 各自从 UI state 或 world sync 读取 entity；stale target 依赖上层判断 | 所有 input route 使用 `EntityRef { id, generation, document/world }`，command admission/rejection 可见，pick 使用 displayed frame identity |
| ED5-P1-028 | Open | play hierarchy/inspector/gizmo/scene inspection 各自有 selection or generation cache，clear 时机分散 | 统一 `SceneProjectionStore` 管 topology/fields/transform/selection caches、view leases、replacement resync and memory budgets |

## 6. P2 差异

| ID | 当前问题 | 重构方向 |
|---|---|---|
| ED5-P2-001 | Document URI、source path、asset URI、play relative path 的 canonicalization 分散 | 统一 project-relative URI/value object、case/symlink policy and display path |
| ED5-P2-002 | load job progress 只有 0/1，缺 I/O/deserialize/asset resolve/compile phase | phase-weighted progress with ETA, cancellation and diagnostic spans |
| ED5-P2-003 | snapshot instance id 含 pid/nanos，无法作为可复现 replay key | deterministic snapshot id from source revision + session nonce, with human display id separate |
| ED5-P2-004 | WorldSync output budgets 与 view cadence 独立，无法按 view priority 调度 | shared budget scheduler with hierarchy/inspector/viewport priorities |
| ED5-P2-005 | hierarchy filter/expansion/selection 由 retained UI state 管理，缺 persistence per document | document-scoped UI state revision and restore policy |
| ED5-P2-006 | inspector values 格式化为固定 decimal strings，缺 locale/unit/precision provenance | typed value presentation descriptor and locale-safe round trip |
| ED5-P2-007 | Keep Play Changes 只支持单实体/字段，不能处理 spawned/destroyed runtime entities | explicit runtime entity classification and adopt/discard workflow |
| ED5-P2-008 | lock poisoning/reconnect telemetry 没有与 document/world generation 关联 | structured gateway/sync diagnostics with fault chain and recovery count |
| ED5-P2-009 | close/replace 时 retained watcher/pick/gizmo cleanup 依赖调用顺序 | lease-based view retirement and idempotent cleanup tests |
| ED5-P2-010 | 没有 1K/10K/100K hierarchy/inspector/play clone benchmark 与 correctness corpus | benchmark source/package size, projection latency, memory, resync and stale rejection before performance claims |

## 7. 资格门

| Gate | 验收条件 | 当前 |
|---|---|---|
| ED5-G01 | 每个 Scene document 有唯一 DocumentId/source revision/world binding | Fail |
| ED5-G02 | open/create/save 使用 prepare-commit-rollback/CAS/recovery receipt | Fail |
| ED5-G03 | load job 支持 cancellation, dedup, dependency admission and terminal result | Fail |
| ED5-G04 | serialized authoring gateway 暴露 identity/generation/capability and fault state | Partial |
| ED5-G05 | authoring snapshot 不再默认 deep-clone World，snapshot policy 可审计 | Fail |
| ED5-G06 | Play instance 绑定 source revision/snapshot hash/world/provider identity | Fail |
| ED5-G07 | Play cleanup/terminal backend retirement 有 lease/fence/retry receipt | Fail |
| ED5-G08 | Edit/Play/remote sync 共享 replacement epoch、watch cursor、retirement barrier | Fail |
| ED5-G09 | poison/protocol/session loss 进入 faulted/unavailable，不 `into_inner` 继续成功 | Fail |
| ED5-G10 | world query/invalidation 支持 typed page/cursor/overflow/resync/cancel | Fail |
| ED5-G11 | hierarchy projection 有 O(1) entity lookup 和 changed-set patch | Fail |
| ED5-G12 | hierarchy topology、selection、expansion、filter revisions 相互独立 | Partial |
| ED5-G13 | inspection publication envelope 同时携带 document/world/replacement/selection revisions | Fail |
| ED5-G14 | inspector cadence 由 dirty journal/budget 驱动而非固定 polling | Fail |
| ED5-G15 | unknown/unsupported/readonly inspector field 不会自动成为可写 field | Fail |
| ED5-G16 | inspector mutation 使用 typed descriptor/value，不以 display string 为 source of truth | Fail |
| ED5-G17 | Keep Play Changes 支持 schema-aware multi-field diff/conflict preview | Fail |
| ED5-G18 | Keep Play Changes 以 source/play revision CAS 提交 undoable transaction | Fail |
| ED5-G19 | spawned/destroyed/reparented runtime entity 有 adopt/discard/mapping policy | Fail |
| ED5-G20 | scene open/replace/play/apply/close 由统一 operation coordinator 管理 | Fail |
| ED5-G21 | retained hierarchy/rename/pick/gizmo 使用 generation-qualified EntityRef | Fail |
| ED5-G22 | world replacement 后所有 view caches 自动 stale-drop/resync 且无泄漏 | Fail |
| ED5-G23 | 1K/10K/100K hierarchy/inspector projection memory/latency 有 baseline | Fail |
| ED5-G24 | document/world correctness corpus 覆盖 reload, plugin schema, stale, crash/recovery and replay | Fail |
| ED5-G25 | 通过 correctness 后才允许与 Unreal/Unity/Bevy 做同 workload 性能对比 | Fail |

## 8. 参考引擎差异

- Unreal 的 UWorld/level lifecycle、component registration 和 tick groups 由一个明确 world/teardown boundary 驱动；Editor transaction 与 preview world 有独立但可追踪的 world identity。Zircon 目前把 ProjectSceneDocument、AuthoringWorld、Play snapshot、WorldSyncPump 和 UI projection 分开协调。
- Bevy `World`/`Entity`/change ticks 与 `Schedule` 将资源、组件、生命周期和运行计划放在可验证的 ECS contract 中；Zircon Editor 不应把 `NodeRecord`/JSON inspection 作为 world truth。
- Fyrox generation Pool/Graph 为 node handle validity、reparent/remap、clone 和 hierarchy ownership 提供基础 invariant；Zircon retained hierarchy 需要将 raw `EntityId` 升级为 document/world-generation qualified reference。
- Godot Node/SceneTree/Object 的 enter/exit tree、process owner/thread group、notifications 和 deferred calls 是可观察 lifecycle；Zircon 的 world replacement、watcher、inspection publication 和 Play cleanup 应收敛到同一事件/retirement stream。

## 9. 重构顺序与 owner

1. 先由 Runtime187/Editor61 owner 定义 DocumentId、WorldBinding、WorldSnapshot、change journal 和 generational EntityRef；冻结各 projection 自行解释 raw entity/generation。
2. 建立 ProjectSceneDocument transaction/load registry 与 `SceneWorldOperation` coordinator，补 CAS、recovery、cancel、dependency and close fences。
3. 将 Play controller/snapshot/gateway/sync pump 收敛为一个 PlaySession/WorldProjectionHub，明确 source/play/world/provider identity 和 retirement。
4. 用 chunked hierarchy/field/transform artifacts、changed-set index、dirty cadence 和 typed inspector descriptors 替换全表 compare、100ms polling、display string mutation。
5. 把 Keep Play Changes 做成 schema-aware diff transaction，覆盖 spawned/destroyed/reparented entities、prefab/asset conflicts、undo/redo/replay。
6. 完成 stale/replacement/fault/recovery correctness corpus 及 1K/10K/100K benchmark 后，再评估 Editor 与 Unreal/Unity/Bevy 的性能表现。

本报告仅写 review 与重构合同，没有修改 Editor、Runtime、Rust、Cargo 或 ZUI；tooling 迁移按用户要求另立范围。
