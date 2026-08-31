---
title: Editor Navigation 当前工作树 Bake、NavMesh/Settings Asset、Operation、Overlay 与 PIE 复审及重构计划
category: zircon_editor
report_id: Editor229
review_date: 2026-08-30
baseline_head: working-tree
baseline_epoch: 2026-08-30
verification_head: working-tree
verification_epoch: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/141-editor-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-preview-current-source-review.md
  - docs/plans/optimize/zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/95-editor-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-preview-product-integration-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/169-runtime-navigation-current-working-tree-bake-artifact-query-crowd-editor-boundary-review.md
related_code:
  - zircon_plugins/navigation/editor
  - zircon_plugins/navigation/runtime
  - zircon_plugins/navigation/native
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/navigation.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/navigation
  - zircon_editor/src/core/editor_extension
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/jobs
  - zircon_editor/src/scene/viewport
  - zircon_plugins/first_party_editor_catalog
  - zircon_plugins/first_party_runtime_catalog
  - zircon_app/src/entry
  - zircon_runtime/src/core/framework/navigation
  - zircon_runtime/src/navigation/operation
plan_sources:
  - docs/plans/optimize/zircon_editor/141-editor-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-preview-current-source-review.md
  - docs/plans/optimize/zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/95-editor-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-preview-product-integration-current-source-review.md
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_plugins/05/failure-2026-07-15-navigation-bake-selection-operation-arguments.md
  - docs/plans/zircon_plugins/05/failure-2026-07-30-navigation-overlay-frame-publication.md
  - docs/plans/zircon_plugins/05/failure-2026-08-02-navigation-editor-operation-status-v2-cutover.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/WorldPartition/WorldPartitionNavigationDataBuilder.cpp
  - dev/UnrealEngine/Engine/Source/Developer/NavigationTestSuite
  - dev/Fyrox/editor/src/interaction/navmesh/mod.rs
  - dev/Fyrox/editor/src/scene/commands/navmesh.rs
  - dev/godot/modules/navigation_3d/editor
  - dev/godot/scene/resources/navigation_mesh.cpp
  - dev/bevy/crates/bevy_app/src/main_schedule.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Editor229 · Navigation 当前工作树复审

## 1. 结论

Navigation Editor 已经比旧报告更接近真实编辑器边界：注册了 NavMesh/Settings asset toolkit、Bake Scene/Selected/Clear operation、Navigation authoring surfaces、retained BakePanel 状态机、V2 progress 校验、快照 restore undo command、PIE mirror、正式 `ViewportOverlayProvider` 和 overlay filter 数据结构；当前工作树还增加了用于 UI asset editor external-effect replay 的容量测试。这些是可保留的架构信号。

但从用户可完成工作流看，它仍是声明与模型投影，而不是可资格化的 Navigation 编辑器。Bake operation 的 runtime owner 对 Scene/Surface 仍拒绝 prepare/apply；BakePanel 没有绑定真实 job、artifact、cancel、shutdown 或 revision；NavMesh/Settings asset view 和 component drawer 大量仍是 `Space`；Overlay provider 固定使用 `NavigationOverlayOptions::default()`，UI filter 没有连接到 provider 状态；PIE mirror 只传一份 frame，缺少 bounded demand、stale/generation 处理和可回放 evidence。操作命令虽能校验 ABI、handle、phase 和 result，但 16 次 `thread::yield_now` 轮询不是 editor job scheduler，也没有 cancel/wake/timeout receipt。

因此旧 Editor141 的“没有操作/没有 overlay”描述需要改为“局部 wiring 已存在，但纵向产品仍不可执行”。当前最严重的问题是 UI 暴露了可点击的 Bake 操作，却在运行时 operation `prepare` 阶段必然失败；这会让用户误以为项目有 bake、undo 和 artifact，而真实系统没有。

## 2. 复审范围与物理统计

| 范围 | files | lines | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---|
| Navigation editor + UI session path | 23 | 3,714 | 128,698 | 34 | 1 | `9595eacaf8de3b0acbe27a81f503878bc222be471ec5efc5245fec21e9af8e7e` |
| Navigation runtime + native（editor owner 依赖证据） | 83 | 12,961 | 434,802 | 116 | 6 | `73882c92cdd0d57204c95800c3cd89923c85a45f1c11a351083a0bbccd0961f3` |
| Runtime framework + builtin navigation | 37 | 5,551 | 185,950 | 46 | 4 | `40e8378614d37ad0c221f05d538e50ebc91acfb52a79f711ca2d9cf4a30d3a92` |
| Catalog/App/editor integration | 218 | 30,815 | 1,140,226 | 484 | 1 | `d27c4cd49d65088edafa0214653e878aebf1bcda404a87e4834a6a2c66ca274f` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference selection | 8 | 10,963 | 452,711 | 0 | 0 | n/a |

Editor 统计覆盖 plugin editor、UI session 和其 runtime/catalog owner 证据；reference 统计只计当前实际存在的选定文件。工作树含已修改与未跟踪 Navigation 文件，fingerprint 不是实现后的稳定基线。

## 3. 当前真实编辑器链路

1. `NavigationEditorPlugin` 创建 `NavigationPieMirror`，登记 runtime event consumers，并把 authoring drawer、四个 view、component drawers、asset toolkit、operation command 与 overlay provider 注册到 `EditorExtensionRegistry`。
2. `bake.zui` 已有 Bake Scene、Bake Selected、Clear Selected、surface Table、force-full-rebuild checkbox、diagnostics Table、ProgressBar 和 status Label。surface row 与 selected identity 可向 operation action payload 投影。
3. `NavigationBakePanel` 维护 surface rows、selection、busy request、phase/progress、last report/error；它可拒绝重复提交并验证 progress 的 request id、动作 phase 和单调性，但 backend 仍由调用者注入，没有默认 editor job owner。
4. `NavigationOperationCommand` 通过 runtime gateway submit/poll/harvest，校验 V2 progress 和 V1 result，捕获 before/after generated snapshot，restore command 可用于 undo/redo；poll budget 固定 16 次并在主线程 yield。
5. `NavigationViewportOverlayProvider` 从 `NavigationPieMirror` 提取一份 `SceneGizmoOverlayExtract`，包含 navmesh triangles、off-mesh links、agent path、desired/avoidance velocity；provider registration 要求 Navigation gizmo capability。

## 4. P0：产品真实性阻断

### P0-01：Bake 按钮对应的 runtime operation 必然失败

`zircon_runtime/src/navigation/operation/handler.rs:115-165` 的 Bake Scene/Surface `prepare` 返回“navigation bake requires a pure prepare backend”，`apply` 返回“cannot reach owner apply without a prepared command”。Editor operation factory 和 command 因而只能证明 gateway 协议，不能证明 bake。必须先有纯 prepare（source snapshot、geometry plan、settings fingerprint、artifact target）和 owner apply（job receipt、CAS publish、generated snapshot、report），再把 UI 操作标为可用。

### P0-02：Surface/Settings/NavMesh asset 仍无真实 editor owner

`plugin/registration/assets.rs` 注册 toolkit 和 open-browser event，但 `navmesh_asset.zui` 与 `navigation_settings_asset.zui` 的主体都是 `Space`；`surfaces.zui`、`agents_areas.zui` 也主要由 `Space`/容器组成。没有 asset session、typed field editor、mesh topology view、settings validation、dirty revision、save/reopen 或 artifact linkage。注册 AssetTypeContribution 不能替代可编辑 asset。

### P0-03：组件 drawer 不是 authoring controls

NavMeshSurface、Modifier、Obstacle、OffMeshLink、Agent 的 drawer ZUI 只有一个 `Space` fields node。尤其 OffMeshBridge 没有独立 drawer/operation，geometry、area、carve、capacity、motion、bidirectional、agent profile 等关键字段不能通过可审计控件修改，也没有 selection identity、property command 和 conflict feedback。

### P0-04：Overlay 的 UI 过滤器没有生效路径

`debug_gizmos.zui` 声明 areas/links/path/avoidance checkbox，但 `NavigationViewportOverlayProvider::extract` 总是调用 `NavigationOverlayOptions::default()`；`NavigationOverlayController` 的 `options_mut` 没有注册为 editor state 或 operation。用户切换过滤器不能改变 overlay payload，且 provider 默认消费完整 navmesh triangles 与所有 debug agents，没有 tile/selection demand 或预算。

## 5. P1：Operation、Job、Artifact 与状态

### P1-01：命令仍是同步 gateway 轮询，不是后台 editor job

`operation_command/command.rs:45-86` 在 `apply` 中最多 poll 16 次并 `thread::yield_now()`，超时被包装为 `CommandExecutionError::ExternalEffect`。没有 `EditorJobId`、可取消 token、wake/notification、progress subscription、shutdown join、retry、diagnostic attachment 或 stale generation。长 tiled bake 会冻结命令执行语义，且应用/窗口关闭无法保证 worker 停止。

### P1-02：BakePanel 状态与 runtime task 没有共享 identity

Panel 自己生成递增 `request_id`，runtime bake task 使用另一套 `NavMeshBakeTaskHandle`/generation。两者没有稳定映射、owner session、surface artifact key 或 terminal receipt；runtime 完成/失败不能自动驱动 panel 的 `observe_progress`/`complete`。面板的 Queued/Baking/Complete 只能由外部手工喂状态。

### P1-03：Undo 只恢复完整 snapshot，不恢复可验证 artifact

`NavigationOperationCommand` 捕获 `NavigationGeneratedBakeSnapshot`，restore operation 比较 current snapshot 后替换 raw `NavMeshAsset`。这能防止一部分过期回放，但没有 source/settings/compiler fingerprint、CAS artifact id、file publish transaction、dependent scene revision 或 unload/residency ticket；大网格 undo 会复制完整 DTO，并无法证明重开项目后仍能恢复同一产物。

### P1-04：Bake options 与 diagnostics 没有真实输入反馈

`force_full_rebuild` 已进入 panel payload，但 UI 只显示固定说明“Tiles and source geometry are collected...”；没有 source count、agent profile、voxel/region/tile settings、ignored knob error、dirty tile list、memory estimate、artifact path 或 per-tile progress。runtime 仍可能用简单 surface fallback 和默认 Recast settings，editor 不能把 warning 当作可接受产物。

### P1-05：Asset catalog/provider closure 仍待证明

Navigation editor plugin 的本地 registry wiring 存在，但普通 `first_party_editor_catalog` 需逐项确认 Navigation provider closure、asset loader、template resources、capability status、host installation 和 teardown。测试中的 `registration_report` 只能证明 registration 结构，不代表默认 App、headless editor host、PIE 与 reload 都安装了同一 provider。

## 6. P1：Overlay、PIE、Query 与 debug

### P1-06：PIE mirror 是单向 frame 投影

`runtime_mirror.rs` 按 play session/sequence 接收 `NavigationOverlayFrame`，记录 owner generation 与 navmesh snapshot；没有 request/ack、sequence gap recovery、world identity、stale frame rejection beyond local checks、backpressure 或 frame budget。完整 triangles + agent paths 在每个 tick 可能复制到 editor，无法满足大世界调试。

### P1-07：Overlay pick 与 selection 没有 command 语义

`build_navigation_overlay` 只创建 agent sphere 和 line segment；navmesh triangles/off-mesh links 的 pick/owner 不能映射回 surface entity、polygon/tile/link command。用户可看到线，但不能从 overlay 选中 tile、link 或 agent 并进入 inspector/diagnostic。

### P1-08：Debug data 没有统一可重置面板合同

Unity Graphics `DebugManager` 提供集中 panel/widget 注册、reset callback 和 runtime/editor 共用的 debug data ownership。Zircon 只有 capability、provider extract 和固定 debug view；没有 panel state registration、reset、filter persistence、demand subscription、telemetry counters 或 provider retirement 的用户反馈。

### P1-09：Query preview 与测试场景缺失

当前 view 没有真正的 start/end/agent/filter 输入、path status/visited nodes/partial policy、sample/raycast inspector、query receipt 或 golden-scene comparison。runtime conformance、tile seam、settings sensitivity、obstacle dirty update、off-mesh handoff 不能在 editor 中可视化重放。

## 7. 参考引擎差异

* Unreal World Partition navigation builder 将分区 source、dirty tile、build artifact、异步状态和 editor feedback 视为完整工作流，Bake 操作不是一个同步按钮；Zircon 还停在 operation gateway 与 panel model。
* Fyrox editor navmesh mode 对真实 vertices/edges/triangles 提供 selection 与 command undo；Zircon asset view/drawer 没有 topology 或 typed field owner。
* Godot NavigationMesh 资源明确暴露 source geometry mode、agent height/radius/climb/slope、region/edge 参数，并由 source geometry data 与后台 bake API 分离；Zircon UI 没有这些可审计输入，runtime 也没有纯 prepare bake。
* Bevy schedule 文档将准备、固定更新、普通更新与渲染交换分开；Zircon command poll/yield 和 PIE overlay 没有 job/render backpressure 边界。
* Unity Graphics `DebugManager` 通过集中注册的 panel/widget/reset 合同保证 debug data 可控；Zircon 的过滤器只是 ZUI 声明，未改变 provider options。

## 8. 重构路线

### E0：先让失败真实可见

在 Bake Scene/Surface handler 完成前，UI 必须显示 `Unavailable: prepare backend missing`，禁止伪造 queued/complete。建立统一 `EditorJobId`、`NavigationBakeReceipt`、`NavigationArtifactId`、session/world/surface identity、revision conflict 和 durable failure schema。

### E1：Navigation document 与真实 asset toolkit

建立 NavMesh/Settings document session、typed property schema、scene surface index、agent/area/modifier/link selection、validation diagnostics、dirty/save/reopen。asset toolkit 打开真实 document；NavMesh view 至少支持 tile/polygon/link topology、bounds、area colors 和 artifact metadata，Settings view 支持 agent/area/voxel/region/tile 参数。

### E2：Pure prepare + cancellable bake job

Editor prepare 从 immutable world/source snapshot 构造 geometry/compiler plan，不访问可变 world；apply 只接收 validated artifact receipt 并原子提交 document/generated snapshot。job scheduler 提供 progress、cancel、wake、shutdown、memory admission、per-tile diagnostics、retry 与 stale-generation discard。

### E3：Operation/Undo/Artifact

把 undo 从 raw snapshot 迁移到 artifact identity + source/settings/compiler fingerprint + dependent revision；支持 clear/restore、跨 session reopen、CAS dedupe、failure recovery 和 server/PIE 同一 artifact。

### E4：Overlay/PIE/Query 产品

将 overlay filters 纳入 editor state，按 selected surface/tile/agent demand 提取 bounded payload；pick shape 关联实体、polygon、tile、link；PIE mirror 使用 world/session/sequence/generation、ack/backpressure 和 stale frame rejection。增加 path/sample/raycast preview、golden-scene diff 与 query receipt。

### E5：资格化与移除占位

移除 `Space` fields、固定 bake text、同步 16-poll command、default-only overlay options、无 handler 的 toggle、硬编码 surface rows 和测试 fake-only backend。用真实 editor host、PIE、save/reopen、cancel/shutdown、large-world overlay、fault injection、100K agent scale、p99 job latency 通过资格门。

## 9. 资格门

| Gate | 当前状态 | 必须证明 |
|---|---|---|
| ED-NAV-1 Bake operation 可执行 | Fail | prepare/apply/receipt/artifact 真实完成 |
| ED-NAV-2 Document/asset toolkit | Fail | NavMesh/Settings 可编辑、save/reopen、revision |
| ED-NAV-3 Component authoring | Fail | typed controls、selection、undo、validation、OffMeshBridge |
| ED-NAV-4 Cancellable editor job | Fail | progress、cancel、wake、shutdown、stale discard |
| ED-NAV-5 Artifact-backed undo | Fail | CAS identity、reopen、dependent revision、clear/restore |
| ED-NAV-6 Overlay filter/pick | Fail | filters 改变 payload，tile/link/agent 可选中并定位 |
| ED-NAV-7 PIE/debug observation | Partial | mirror/provider 存在，但 demand/backpressure/ack 缺失 |
| ED-NAV-8 Query preview/conformance | Fail | path/sample/raycast、golden scenes、provider parity |
| ED-NAV-9 Catalog/reload/host closure | Partial | 本地 registry 存在，默认 host/reload 尚未证明 |
| ED-NAV-10 Large-world/editor performance | Fail | 100K agents、tile overlay、memory/p99、fault/soak |

本轮为 review-only；只新增本报告及索引/coverage 记录，没有修改 production Rust、测试、Cargo、ABI 或 ZUI，也没有运行 Cargo、Editor、PIE、真实 bake、save/reopen、fault、scale、soak 或动态 benchmark。Tooling 按用户要求排除，未查询、轮询、等待或实时跟踪协调器。实施前必须重新冻结 source fingerprint。
