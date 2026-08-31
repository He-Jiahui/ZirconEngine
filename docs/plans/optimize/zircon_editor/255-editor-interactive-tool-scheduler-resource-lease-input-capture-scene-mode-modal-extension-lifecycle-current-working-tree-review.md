---
related_code:
  - zircon_editor/src/core/tools
  - zircon_editor/src/core/context/tool_scheduler.rs
  - zircon_editor/src/core/context/tool_scheduler/error.rs
  - zircon_editor/src/core/context/tool_scheduler/observation.rs
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/editor_message/message/tool.rs
  - zircon_editor/src/scene/modes
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_scene_modes.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_input.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_scene_mode_lifecycle.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_construction.rs
  - zircon_editor/src/ui/workbench/state/editor_state.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/runtime_shutdown.rs
  - zircon_editor/src/ui/retained_host/app/project_close.rs
  - zircon_editor/src/ui/retained_host/app/scene_picker_actions.rs
  - zircon_editor/src/ui/retained_host/app/scene_picker_session.rs
  - zircon_editor/src/ui/retained_host/app/command_palette_actions.rs
  - zircon_editor/src/ui/retained_host/app/settings_window_actions.rs
  - zircon_editor/src/ui/retained_host/viewport/world_space_ui.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport/pointer_dispatch.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/session.rs
tests:
  - zircon_editor/src/core/tools/tests.rs
  - zircon_editor/src/core/tools/tests/fairness.rs
  - zircon_editor/src/core/tools/tests/snapshot.rs
  - zircon_editor/src/core/tools/input_capture/tests.rs
  - zircon_editor/src/core/context/tool_scheduler/tests.rs
  - zircon_editor/src/scene/modes/tests.rs
  - zircon_editor/src/scene/modes/tests/isolation.rs
  - zircon_editor/src/scene/modes/tests/lifecycle.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_scene_modes.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests
  - zircon_editor/src/ui/retained_host/app/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/53-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/125-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-current-source-review.md
  - docs/plans/optimize/zircon_editor/174-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-current-source-review.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/05/failure-2026-07-31-scene-mode-input-ownership-hardcut.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/08/failure-2026-08-01-ticketed-command-routing-revoke-missing.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Public/InteractiveToolManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/InteractiveToolManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Public/InputRouter.h
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/InputRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Public/InteractiveToolsContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/InteractiveToolsContext.cpp
  - dev/Fyrox/editor/src/interaction/mod.rs
  - dev/Fyrox/editor/src/scene/container.rs
  - dev/godot/editor/plugins/editor_plugin.h
  - dev/godot/editor/plugins/editor_plugin.cpp
  - dev/godot/editor/plugins/editor_plugin_list.h
  - dev/godot/editor/plugins/editor_plugin_list.cpp
  - dev/bevy/crates/bevy_picking/src/input.rs
  - dev/bevy/crates/bevy_picking/src/pointer.rs
  - dev/bevy/crates/bevy_input_focus/src/lib.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Utilities/GenericEditorTool.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/LightPlacementTool.cs
refreshes:
  - docs/plans/optimize/zircon_editor/53-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/125-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-current-source-review.md
  - docs/plans/optimize/zircon_editor/174-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-current-source-review.md
doc_type: review-and-refactor-plan
review_status: current_working_tree_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 255 · Editor Interactive Tool Scheduler、Resource Lease、Input Capture、SceneMode、Modal 与 Extension 生命周期复核

## 1. 结论与当前状态

当前工作树已经不再是 Editor174 所描述的“scheduler 只有 DTO 和 tests”。`ToolDefinitionId`、`ToolInstanceId`、`ToolRequestId`、`ToolLeaseId` 已分开；每个 instance 只允许一个 active 或 queued claim；原子 resource set、lease release、queued promotion、revisioned transition batch、outbox/journal、snapshot/resync、delivery health、authority shutdown state 和 input capture priority/steal 都已有真实源码。Scene viewport 和 Export Wizard 也确实开始申请共享 `ToolSchedulerService`。

这些进展仍没有形成 Unreal/Fyrox/Godot/Bevy/Unity 意义上的完整交互工具系统。Scene viewport 只在非 Select SceneMode 激活时持有 `{ViewportInput, SceneModeSlot}`，Select、相机导航、pointer route、world-space UI 和 drawer resize 仍由各自局部状态直接处理；Export Wizard 只在 job start 到 terminal 期间持有 `{ViewportInput, ModalSurface}`，打开面板、scene picker、command palette、settings window 没有 modal lease/focus owner。`ToolInputCaptureAuthority` 没有任何 production caller，viewport bridge 也没有把 pointer route 映射到它。

当前唯一 P0-02 已经关闭：同一 instance 的不同 resource set 不会再覆盖旧 holder，因为 `instances` 只记录一个 claim，lease release 只接受 `ToolLeaseId` 并按 lease 的完整 set 清理。P0-01 从 Open 降为 Partial：已有两个 production adapter，但仍未覆盖全产品输入和模态路径。P0-03 从 Open 降为 Partial：authority revision 与 outbox batch 已原子提交、dispatch health 可观测，但没有 production subscriber 消费 snapshot/cursor，也没有端到端 operation receipt。

| 等级 | Open | Partial | Closed | 合计 |
|---|---:|---:|---:|---:|
| P0 | 0 | 2 | 1 | 3 |
| P1 | 17 | 28 | 3 | 48 |
| P2 | 8 | 3 | 1 | 12 |
| 资格门 | 11 Fail | 21 Partial | 4 Pass | 36 |

本报告只做当前工作树 review 与重构计划。没有修改生产实现，没有运行 Cargo、GUI、并发、失焦、插件 reload 或规模性能验证；Tooling 按用户要求排除。

## 2. 审查边界、currentness 与证据

冻结点为 HEAD `cc5cadbd597c3707954ebd6109fad0fd5643a152`，按当前磁盘而非干净提交审查。重点选择集覆盖：

1. `core/tools` 全部实现与 tests：identity、claim、resource set、scheduler、authority state、transition、snapshot、limits、input capture。
2. `SceneModeStack`、builtin/custom mode、isolated callback boundary、viewport controller 的真实 acquire/release 与 mode retirement。
3. command palette、scene picker、settings、project close、world-space UI、drawer resize、retained viewport pointer bridge 的 input/modal 生命周期。
4. Export Wizard 的 tool lease 与 job terminal；Editor host/runtime shutdown；extension contribution revoke。

证据等级为 E2/E3：源码结构、生产反查、已有测试和仓内参考源码均已核对。测试属性只能证明局部 model/boundary 行为，不能证明 multi-window、consumer gap recovery、plugin reload、terminal ordering 或性能资格。

## 3. 当前产品可达性矩阵

| 子系统 | 当前事实源 | 是否使用 scheduler | 工程判定 |
|---|---|---|---|
| SceneMode 非 Select base/overlay | `SceneModeStack` + registry + controller | 是，controller 持一个 `{ViewportInput, SceneModeSlot}` lease | 有共享租约，但 lease 属于整个 controller，不属于 mode instance/slot。 |
| Select / 普通 scene input | builtin mode effect + controller | 否，Select 不触发 `requires_exclusive_tool()` | 两个 viewport 可同时直接处理 selection/camera，authority 无法解释 owner。 |
| Handle / primary drag | `ViewportDragSession`、HandleToolRegistry、transaction session | 否，无 input capture lease | 有局部 rollback/transaction，但 pointer 生命周期绕过 scheduler。 |
| retained viewport pointer | `UiPointerDispatcher` + `SharedViewportPointerBridge` | 否 | Down capture 只属于 UI surface；事件降成 `EditorViewportEvent` 后没有 pointer/window/surface/sequence generation。 |
| world-space UI | `world_space_ui_pointer_capture: Option<Submission>` | 否 | Down/Move/Up/Cancel 有局部 capture，但与 viewport、keyboard、device 不共享 owner。 |
| drawer resize | `ShellPointerBridge` 的 resize surface capture | 否 | 有异常取消，但只是 shell 内部事实源。 |
| command palette / scene picker / settings | retained host window bridge + session/optional revision | command palette/scene picker/settings 未持 `ModalSurface` | 真实 session、stale catalog 检查和 close 清理存在，但无 nested modal/focus arbitration。 |
| Export Wizard | `ExportWizardPanelSession` + `ExportWizardJobController` | start 时持 `{ViewportInput, ModalSurface}` | job 可达，但面板可见期与后台 JobTicket 混为一个 tool lease 窗口。 |
| extension SceneMode/provider/consumer | `ContributionStore` ticket + prepared retirement | 不持 owner generation/tool lease | revoke 已能回退 active mode、provider、runtime consumer 和 command projection；view/layout/document toolkit 仍非同一 terminal receipt。 |
| authority shutdown | `ToolSchedulerService::quiesce/close` | 只在 tests/显式 caller 可用 | Retained host Drop/runtime shutdown/project close 没有调用 tool close。 |

生产反查结果：`EditorContextBuilder` 构造一次 `ToolSchedulerService`；`EditorState` 和 retained host 将 clone 传入 viewport/Export Wizard；除此之外没有 `context.tools()` 的 scheduler observation consumer，没有 `begin_input_capture`/`end_input_capture` production caller，也没有 `read_transitions` production subscriber。`TOPIC_TOOL` 目前实际消费仍由 tests 证明。

## 4. P0 正确性与可达性

### ED255-P0-01 · Partial · Scheduler 只接入 Scene viewport 与 Export Wizard，输入/模态产品仍绕过

证据：`scene_viewport_controller_scene_modes.rs:41-71` 只为 controller 的非 Select mode acquire 一个 atomic set；`export_build/wizard/session.rs:440-473,598-632` 只在启动 job 时 acquire/release。`handle_input.rs:42` 在 SceneMode pass-through 后直接调用 camera input，`pointer_dispatch.rs:20-36` 直接构造 `EditorViewportEvent`，world-space UI 与 drawer resize 各自维护 capture。

影响：共享 scheduler 现在能阻止第二个 Scene controller 或 Export job，但不能阻止同一窗口的 viewport drag 与 modal surface、world-space UI 与 viewport pointer、drawer resize 与 keyboard/pointer 之间的冲突；也不能为 capture owner 提供统一 terminal receipt。

必须重构：先把 SceneMode、viewport pointer、Gizmo drag、modal session、extension retirement 接入 `InteractiveToolAuthority` adapter，再决定哪些后台 Job 仅持 `JobTicket`；未接入的路径必须显式报告“未受 authority 保护”，不能把两个 adapter 当作全产品接线完成。

### ED255-P0-02 · Closed · 同一 instance 的不同 set 覆盖与 orphan holder 已被 claim map 消除

证据：`core/tools/scheduler.rs:27-37` 同时维护 `requests`、`leases`、`instances`；`178-180` 发现同 instance 已有 claim 就进入 `report_existing_claim`；`334-400` 对不同资源返回 `AlreadyHeld`/`AlreadyQueued` denial；`215-250` release 使用 lease 完整 set 清除 holder、capture 与 instance。`core/tools/tests.rs:72-128,201-223` 覆盖 active/queued replacement 和 stale lease。

残余风险：`ToolLeaseHandle` 可 clone，`release()` 接受裸 `ToolLeaseId`；serde 可反序列化 ID。跨 owner/session 的能力证明仍不完整，应在后续 P1-03/P1-32 中改为不可伪造 capability handle + generation。

### ED255-P0-03 · Partial · revisioned outbox 已建立，但没有 production consumer 与 operation receipt

证据：`core/context/tool_scheduler.rs:88-124` 在同一 authority lock 内 commit revision、batch、outbox、journal；`403-448` transition 先产生 report 再 commit；`478-509` 通过 dispatcher mutex 顺序 publish 并累计 delivered/dropped/backpressure/error health；`observation.rs:91-153` 提供 immutable snapshot 和 cursor；`148-190` 提供 journal gap/resync。

缺口：outbox batch 只携 `ToolLifecycleEvent`，不携 owner generation、scope、cause、operation/request receipt；没有 production subscriber 按 cursor/resync；dispatch report 虽进入 health，但上层没有把 delivery failure 连接到 UI/recovery。`dispatch_outbox` 先 pop outbox 再 publish，进程中断或 bus panic 的恢复语义也没有 durable outbox receipt。

必须重构：为每次 authority mutation 建立 `ToolOperationReceipt`，将 revision、request/instance/lease、scope、owner generation、terminal disposition 和 delivery state 绑定；consumer 必须从 snapshot/cursor 恢复，而不是依赖 `TOPIC_TOOL` 的逐条消息。

## 5. P1 差距与重构要求

### 5.1 Identity、owner、resource 与 claim

| ID | 状态 | 当前证据 | 重构要求 |
|---|---|---|---|
| P1-01 | Closed | `identity.rs:41-213` 分开 definition/instance；`claim.rs:5-74` 分开 request/lease。 | 保持四层 ID，禁止重新退化为 string ToolId。 |
| P1-02 | Partial | `ToolOwnerGeneration` 只存在 input capture；ToolLease 没有 owner/catalog/build/session generation。 | 将 extension/build/project/window/document/session generation 纳入 definition、request、lease。 |
| P1-03 | Partial | release 接受 `ToolLeaseId`，handle 仍可 clone，裸 ID 可作为调用参数。 | release/withdraw 仅接受带 authority secret 的 capability handle，并拒绝跨 generation。 |
| P1-04 | Open | `ExclusiveResource` 仍只有三个 editor-global enum。 | 引入 `ResourceKey { kind, scope, channel }`，至少覆盖 project/document/window/viewport/pointer/device。 |
| P1-05 | Closed | `instances: BTreeMap<ToolInstanceId, ClaimStateRef>` 禁止一个 instance 同时 active/queued。 | 增加 property/reference-model 测试，保证 lease/request/resource 三表每次 transition 一致。 |
| P1-06 | Open | resource set 是固定 `ALL: [ViewportInput, ModalSurface, SceneModeSlot]`。 | 编译 resource catalog 和冲突图，插件 resource 必须有 schema/version/diagnostic label。 |
| P1-07 | Partial | `SceneModeRegistration` 有 descriptor/factory/owner；没有通用 ToolDefinition catalog、capability、factory setup contract。 | 将 SceneMode、Gizmo、Modal、Export UI 统一投影为 typed ToolDefinition。 |
| P1-08 | Partial | Scene controller 先 acquire 再 create/enter；失败能在 `acquired_now` 时 release。 | 分离 Prepare/Create/Setup/Activate/Commit，失败必须有补偿 receipt，不能让 holder 早于实例可用。 |
| P1-09 | Partial | Scene command eval 带 selection/mode revision；scheduler admission 无 target/document/mode snapshot。 | request 绑定 target snapshot、permission、mode revision、document generation。 |
| P1-10 | Partial | capture 有 Completed/Accepted/Cancelled/Stolen/OwnerLost/FocusLost/Shutdown；通用 ToolLifecycle 没有 Accept/Cancel/Completed/Aborted/OwnerLost。 | 统一 tool terminal disposition 与 transaction/rollback。 |
| P1-11 | Open | SceneModeStack 只保存 base/overlay，没有 previous-tool/resume token。 | 建 typed tool stack，支持替换、Escape 返回 previous、overlay suspend/resume。 |
| P1-12 | Partial | `EditorState::Drop` shutdown SceneMode，controller Drop release lease；owner close 不调用 scheduler close。 | owner scope teardown 必须 quiesce、force-end capture、drain lease/request 并返回 terminal receipt。 |
| P1-13 | Open | `set_queue` head 阻止后续 set，即使后续 set 可用；单资源只绕过不重叠 head。 | 用冲突图 + reservation + 可解释公平策略，避免无界 head-of-line blocking。 |
| P1-14 | Open | promotion 由 `promote_available_sets`/`promote_waiting_singles` 隐式分支决定，无 aging。 | 固定 aging、priority、progress 和 starvation upper bound。 |
| P1-15 | Partial | set promotion 有 loop，single promotion 按资源遍历；没有统一跨类别 fixpoint/progress proof。 | 一次 transition 执行 bounded fixpoint，并输出 promotion reason。 |
| P1-16 | Open | limits 只有 single/set queue entry count。 | 加 per-key、owner、global count/bytes、deadline、cancel ceiling。 |

### 5.2 Input capture、SceneMode、Modal、Extension

| ID | 状态 | 当前证据 | 重构要求 |
|---|---|---|---|
| P1-17 | Partial | `input_capture.rs:94-108` 有 pointer/keyboard/device、pointer id、window/surface scope；缺 viewport/document/session channel。 | `ResourceKey` 和 InputSource 增加 viewport/document/device generation、hover/text/IME 通道。 |
| P1-18 | Partial | `ToolInputCapturePriority` 支持同 source arbitration；scheduler request 没有 priority。 | 统一 capture request priority、stable tie-break、source/owner admission。 |
| P1-19 | Partial | `input_capture.rs:376-430` 支持 higher-priority steal，事件 Ended(Stolen) 在 Started 前发布；没有 handoff/force-end receipt。 | typed preemption、handoff acknowledgement、force-end source/all。 |
| P1-20 | Open | world-space、viewport UI、drawer resize、SceneMode drag 各自有 capture/focus 状态。 | 唯一 InputRouter，分离 pointer/keyboard/hover/text/IME focus，禁止平行 authority。 |
| P1-21 | Partial | owner generation 能阻止 stale capture end，lease release 会 OwnerLost；没有统一 owner despawn/reload generation sweep。 | generation 失效必须自动 force-end 全部相关 capture 并可查询原因。 |
| P1-22 | Partial | SceneMode `Consumed/PassThrough` 与 capture disposition 各自存在，Gizmo 有 transaction；没有统一 routing/lease receipt。 | routing result 绑定 request/lease/transaction/revision。 |
| P1-23 | Partial | controller acquire `{ViewportInput, SceneModeSlot}`，但 Select 不 acquire，mode stack 没有 per-slot lease。 | SceneMode slot、viewport input 和 document scope 必须由同一 activation transaction 取得。 |
| P1-24 | Partial | SceneMode stack 有 revision、overlay push/pop、contribution owner；overlay 不声明 resource/owner generation。 | overlay descriptor 声明 lease、focus channel、stack policy、terminal behavior。 |
| P1-25 | Open | `handle_input.rs:30-44,61-120` 直接在 mode pass-through 后执行 camera，pointer dispatch 直接转 controller。 | controller 只能消费 published InputRouter snapshot，禁止裸 enum 入口。 |
| P1-26 | Open | scene picker/command palette/settings 会互相关闭或清理 session，但 `ModalSurface` 无生产 holder。 | window-scoped nested modal lease、focus stack、Escape/click-outside policy。 |
| P1-27 | Partial | Export Wizard 有独立 tool id/lease 和 job controller；tool lease 覆盖 job 执行期，未与 JobTicket 交叉诊断。 | UI tool lease 与后台 JobTicket 分离，receipt 可关联但 identity 不混用。 |
| P1-28 | Partial | extension unregister 会 revoke Store、重建 command projection、retire views/runtime consumers、回退 SceneMode/provider。 | owner generation、quiesce、capture force-end、layout/session/document-toolkit 一次性撤销。 |
| P1-29 | Partial | `IsolatedSceneMode` 对 callback panic 做 checkpoint restore/quarantine；lease/input/transaction 没有同一 fault receipt。 | callback fault 要触发 authority terminal path，而非只隔离 mode object。 |
| P1-30 | Closed | factory/id/enter/exit/input/update/overlay/drop 均在 isolated boundary，faulted mode 后续调用被隔离。 | 保持 boundary，并把 health/quarantine 纳入统一 snapshot。 |
| P1-31 | Partial | project close 清理 scene picker、play/runtime/session/focus；没有 scheduler quiesce/close/drain receipt。 | 项目、窗口、session、runtime 关闭采用固定 drain 顺序并验证零 active lease/capture。 |
| P1-32 | Open | lease 没有 rebind/handoff/migration generation；controller clone 只能共享 service 而非迁移 lease。 | 定义 typed rebind/handoff receipt，旧 generation 必须终止或明确转移。 |

### 5.3 Event、snapshot、并发、契约与性能

| ID | 状态 | 当前证据 | 重构要求 |
|---|---|---|---|
| P1-33 | Partial | `ToolTransitionRevision` 与 `ToolTransitionBatch` 已存在。 | event envelope 还需 operation/request/instance/lease/owner generation/scope。 |
| P1-34 | Open | event 没有 cause、scope、owner generation、terminal reason、schema version。 | 使用 versioned audit schema，未知字段有明确 policy。 |
| P1-35 | Partial | dispatcher mutex 保证 outbox batch 顺序，authority commit 与 batch 同 revision；consumer 未接入。 | producer/consumer 均以 revision cursor 工作，不能把 bus 顺序当产品状态。 |
| P1-36 | Partial | `ToolSchedulerDeliveryHealth` 记录 unobserved/dropped/backpressured/error。 | health 要有 owner、last failure、retry/resync action，并接到 UI/recovery。 |
| P1-37 | Partial | `snapshot()` 返回资源、active leases、queued requests、captures；cursor journal 支持 stale resync。 | 加 bounded filter/cursor、scope projection 和 consumer acknowledgement。 |
| P1-38 | Partial | `AcquireOutcome` 返回 queue position，但 request handle 不带 revision，position 会随 promotion 变化。 | queue query 只能从带 cursor 的 snapshot 读取。 |
| P1-39 | Open | authority 全部由一个 mutex 保护，input capture、lease、event 共享锁。 | 先完成语义，再按 scope shard；不得为性能提前复制第二 authority。 |
| P1-40 | Partial | authority poison 进入 Faulted，dispatcher poison 会累计 error 并 clear poison；cleanup 仍可能继续服务。 | Faulted 必须 fail-stop，仅允许受控 drain/rebuild，禁止 `into_inner()` 后正常 admission。 |
| P1-41 | Open | 无 deadline、cancel handle、awaitable wake 或 request timeout。 | monotonic deadline、cancel receipt、wake source 与 queue expiration。 |
| P1-42 | Partial | Arc identity、canonical resource Vec、typed batch 已减少复制/排序成本。 | 以 workload 数据决定 immutable snapshot/arena，而不是把局部 micro-optimization 当作完整性能。 |
| P1-43 | Open | SceneMode 有局部 checkpoint counter，scheduler 没有 wait/hold/depth/leak/fairness metrics。 | 统一 scheduler/input trace 和 p50/p95/p99 指标。 |
| P1-44 | Partial | `Open/Quiescing/Draining/Faulted/Closed` 已实现。 | 将 host/runtime/plugin close 固定接到状态机并返回 terminal receipt。 |
| P1-45 | Open | serde schema 对 resource/event 没有 version/migration/unknown policy。 | 增加 schema version、migration、forward-compatible unknown handling。 |
| P1-46 | Open | acquire/release report、SceneMode result、Export job result 各自独立，没有共同 operation receipt。 | 从 request 到 setup、routing、terminal、delivery、recovery 贯穿单一 receipt。 |
| P1-47 | Partial | scheduler、SceneMode、capture、snapshot 各有单元测试；无 reference model、property/concurrency、host E2E、reload/fault matrix。 | 建模型测试、并发屏障、bus fault sink、multi-window/plugin E2E。 |
| P1-48 | Open | 没有 1K/10K request、多 viewport、capture churn、plugin reload 的受管 benchmark。 | 在 BuildSet 绑定源码后测 wait/CPU/alloc/RSS/wake/fairness。 |

## 6. P2 长期完整性

| ID | 状态 | 能力差距 | 目标 |
|---|---|---|---|
| P2-01 | Open | compiled resource catalog | 稳定资源列表、scope、冲突图和诊断名。 |
| P2-02 | Partial | typed validation report | denials 同时带资源、owner、scope、generation、revision。 |
| P2-03 | Partial | lease/capture mismatch diagnostics | wrong lease/request/set/generation 返回可恢复诊断。 |
| P2-04 | Open | conflict graph | authority 导出唯一冲突图，UI 不维护第二份。 |
| P2-05 | Open | localized tool labels | identity、description、本地化和可访问性分离。 |
| P2-06 | Partial | queued request query | 已有 snapshot queue，但缺 owner/scope/filter/bounded cursor UI。 |
| P2-07 | Open | transition history | source-bound ring、operation receipt、support bundle。 |
| P2-08 | Closed | idempotent acquire | 同 claim 返回 canonical lease/request handle，已有测试覆盖。 |
| P2-09 | Open | typed scheduler settings | count/bytes/owner/global/aging/deadline ceiling。 |
| P2-10 | Open | accessible tool switcher | keyboard、Escape、focus、a11y、previous tool stack。 |
| P2-11 | Open | recovery UI | orphan lease、stale capture、faulted authority 重建。 |
| P2-12 | Open | performance receipt | request/hold/queue/capture trace 绑定源码、硬件、BuildSet。 |

## 7. 参考源码给出的硬约束

| 参考 | 可观察机制 | 对 Zircon 的约束 |
|---|---|---|
| Unreal `InteractiveToolManager` | CanActivate → Build/Create → Setup → PostSetup → InputRouter Register；Deactivate 先 `DeregisterSource`，再按 Accept/Cancel/Completed Shutdown；Context Shutdown force-terminates capture 和 active tools。 | `Acquired` 不能早于真实 setup；终止必须先撤输入 owner，再提交 transaction/dispose；host close 必须调用 authority close。 |
| Unreal `InputRouter` | 分离 keyboard/left/right/hover capture，稳定收集请求，支持 `ForceTerminateSource/All` 和 focus loss。 | 三个 global enum 与一个 viewport capture 不足以解释多窗口、多 pointer、多通道输入。 |
| Fyrox `InteractionMode` / scene container | 每个 scene 有 mode container；activate/deactivate、mouse/key/hotkey/UI/update 全进入当前实例。 | SceneMode 必须绑定 scene/document/viewport scope，不能让 controller 继续拥有平行交互事实源。 |
| Godot `EditorPluginList` | plugin add/remove 与 `forward_3d_gui_input` 的 PASS/CUSTOM/STOP disposition 同一插件生命周期。 | revoke 必须同时撤 routing、capture、mode、view；不能只隐藏 UI row 或 Store descriptor。 |
| Bevy Picking/InputFocus | `PointerId`/`PointerMap`、Cancel、hover ordering 与独立 `InputFocus`。 | pointer、hover、keyboard/text focus 不能塞进一个 `ViewportInput` 或局部 `Option`。 |
| Unity Graphics `GenericEditorTool` / `LightPlacementTool` | OnActivated、OnWillBeDeactivated、previous state restore、Escape `RestorePreviousTool`、Undo/redo hookup。 | tool stack 需要 previous/resume token；camera/view state 和 transaction 的恢复不能靠 variant 消失推断。 |

## 8. 目标架构

```text
CompiledToolDefinition
  identity + owner/build/project/session generation
  resource keys + scope + capability + input channels
  factory + setup + terminal policy
        |
ToolRequest(request_id, scope, target snapshot, priority, deadline)
        |
InteractiveToolAuthority
  conflict graph + fairness + count/bytes budgets
  revisioned state + ordered outbox + immutable snapshot
        |
ToolLease(capability, lease_id, instance_id, generation)
        |
InputRouterSnapshot
  pointer/device/window/viewport capture
  keyboard/hover/text/IME focus
        |
SceneMode / Gizmo / Modal / Picker / Export adapters
        |
Accept | Cancel | Completed | Aborted | OwnerLost | FocusLost
        |
quiesce -> force-end capture -> rollback/commit -> dispose -> terminal receipt
```

必须保持以下不变量：

1. 一个 instance 的 active/queued/terminal 状态唯一，resource holder、lease、request、capture 和 snapshot 可在每次 transition 后互相证明。
2. Acquire success 晚于 factory/setup/activate；失败不暴露 holder，queued 不得偷偷变成已激活。
3. Input routing 只读取同 revision 的 capture/focus snapshot；owner generation 失效立即 force-end。
4. Authority revision、outbox batch、operation receipt 一起 commit；delivery failure 不回滚 authority，但 consumer 必须能检测 gap 并 snapshot resync。
5. SceneMode、Gizmo、viewport、modal、picker、export、extension unload 和 host shutdown 只能通过 adapter 使用唯一 authority。

## 9. 重构顺序

### Phase A · 算法与 capability 止血

1. 保持单 instance claim invariant，增加 randomized reference-model、different set、single+set、release_all、promotion fixpoint 测试。
2. 将 release/withdraw 从裸 ID 改为不可伪造 handle，加入 owner/session/build generation。
3. Faulted authority 进入 fail-stop；只允许显式 drain/rebuild，不允许 poison 后继续普通 admission。

### Phase B · Revision、receipt 与预算

1. 扩展 `ToolTransitionBatch` 为 versioned envelope，携 request/instance/lease/scope/generation/cause/terminal disposition。
2. 建立 operation receipt、outbox retention、consumer cursor、gap/resync、delivery health/recovery UI。
3. 增加 per-resource/owner/global count+bytes、deadline、cancel、shutdown phase 和 bounded wake。

### Phase C · InputRouter 与 SceneMode

1. 将 `ResourceKey` 扩展到 scene/document/window/viewport/pointer/device/channel。
2. 用 SceneMode factory/boundary 构造 adapter：activation 先准备 mode，再取得 `SceneModeSlot + InputRouter` lease，成功后才 commit stack。
3. retained viewport、world-space UI、drawer resize 收敛到同一 InputRouter，保留现有 native cancel 语义但统一 generation/receipt。

### Phase D · Gizmo、Modal、Export、Extension

1. Gizmo transaction 映射 Accept/Cancel/OwnerLost；force-end capture 必须 rollback preview 并有 terminal receipt。
2. command palette、scene picker、settings 建立 window-scoped nested modal lease、focus stack 和 click-outside/Escape policy。
3. Export UI lease 与后台 JobTicket 分离；只有可见交互需要 Modal/Viewport lease，后台过程不占输入资源。
4. contribution ticket/generation 贯穿 mode/provider/view/runtime consumer；revoke 先 quiesce/cancel/force-end，再删除 registry/factory。

### Phase E · Host shutdown 与资格

1. project close、main/floating window close、runtime shutdown、RetainedEditorHost Drop 按固定顺序 quiesce → drain → close scheduler，并保留 receipt。
2. 补齐 multi-window、focus loss、pointer cancel、plugin reload、fault injection、zero subscriber、late consumer、stale cursor E2E。
3. 在源码绑定 BuildSet 后对 1K/10K request、multi viewport、capture churn、plugin reload 测 wait p50/p95/p99、CPU、alloc、RSS、wake、fairness。

## 10. 36 个资格门当前重判

| Gate | 状态 | 当前判定 |
|---|---|---|
| G-01 | Partial | SceneMode 与 Export 已有 scheduler adapter，viewport input、modal/picker、drawer、world-space UI 仍无。 |
| G-02 | Partial | 已接入路径有资源保护，未接入路径没有 truthful availability。 |
| G-03 | Pass | 单 instance claim map + 完整 lease set 清理已消除 different-set overwrite。 |
| G-04 | Fail | 没有全状态 reference/property/concurrency model。 |
| G-05 | Pass | authority revision 与 transition batch 原子 commit。 |
| G-06 | Partial | journal cursor/resync API 存在，没有 production consumer。 |
| G-07 | Partial | delivery health 记录 dropped/backpressure/error，但没有上层恢复动作。 |
| G-08 | Partial | capture 有 pointer/window/surface/device/generation 部分字段，缺 viewport/document/session channel。 |
| G-09 | Partial | focus/pointer cancel 与 lease owner loss 有局部路径，未统一所有 capture。 |
| G-10 | Partial | higher-priority steal 已实现，无 handoff/force-end receipt。 |
| G-11 | Partial | SceneMode callback checkpoint/quarantine 存在，未绑定 lease/transaction terminal。 |
| G-12 | Partial | SceneMode stack revision 与 overlay lifecycle 存在，无 slot lease。 |
| G-13 | Fail | picker/palette/settings 没有 nested ModalSurface lease/focus stack。 |
| G-14 | Partial | extension revoke 已退休 mode/provider/consumer/commands，未覆盖全部 host toolkit/session。 |
| G-15 | Partial | EditorState/controller Drop 清理局部 mode/lease，无 scheduler close receipt。 |
| G-16 | Fail | acquire 不执行通用 Setup/Activate callback。 |
| G-17 | Partial | capture 有 terminal disposition，通用 tool 没有统一 terminal transaction。 |
| G-18 | Fail | 无 aging/starvation/progress proof。 |
| G-19 | Fail | 无 bytes/owner/global budget。 |
| G-20 | Partial | authority shutdown state 存在，host/runtime 没有固定接入。 |
| G-21 | Partial | SceneMode fault 隔离已通过，scheduler fault 后仍可进入受限 cleanup。 |
| G-22 | Fail | event/resource schema 无 version/migration。 |
| G-23 | Partial | request/lease/report IDs 存在，但没有共同 operation receipt。 |
| G-24 | Partial | owner/resource mismatch typed outcome 存在，scope/generation 不完整。 |
| G-25 | Fail | reload 后 resource/owner/lease 不可重建。 |
| G-26 | Fail | viewport routing 不消费 scheduler/InputRouter snapshot。 |
| G-27 | Fail | modal/scene/viewport 没有共享 focus/capture policy。 |
| G-28 | Fail | host close/runtime close 与 scheduler shutdown 无固定顺序。 |
| G-29 | Pass | repeated same claim 返回 canonical lease/request handle。 |
| G-30 | Partial | dispatcher mutex + 单 authority 避免局部竞态，无跨 request formally checked proof。 |
| G-31 | Partial | zero subscriber/drop/error 在 health 可见，但没有 production recovery/consumer。 |
| G-32 | Partial | extension SceneMode/provider/consumer 走 ticket retirement，capture/generation 和全部 view/session 未收束。 |
| G-33 | Pass | 参考比较仅依据仓内源码结构。 |
| G-34 | Fail | 没有规模性能 receipt。 |
| G-35 | Partial | Windows focus/pointer cancel 有局部 tests，多窗口/reload/shutdown E2E 缺失。 |
| G-36 | Partial | 当前选择集已重算，本报告无独立 reviewer 与 managed Cargo validation。 |

## 11. 最低测试矩阵

| 层级 | 必须覆盖 |
|---|---|
| Pure scheduler | randomized reference model；不同 set、single+set、重复 claim、stale capability、promotion fixpoint、queue budget、revision exhaustion。 |
| InputRouter | 多 window/viewport/pointer/device；keyboard/hover/text/IME 分离；priority tie、steal、handoff、focus loss、owner generation、destroy/reload、Cancel 顺序。 |
| SceneMode/Gizmo | setup 失败、enter panic、overlay suspend/resume、SceneModeSlot lease、selection/transform rollback、OwnerLost、Accept/Cancel/Completed/Aborted。 |
| Modal/session | palette/picker/settings nested open、Escape、click outside、project/window close、stale catalog/session、ModalSurface release receipt。 |
| Extension | candidate reject zero publication；revoke active base/overlay/provider/consumer/view/layout/toolkit；其他 plugin preservation；late callback quarantine。 |
| Host lifecycle | project close、floating/main close、runtime shutdown、RetainedHost Drop 的固定 drain order，assert zero active lease/request/capture。 |
| Qualification | zero subscriber、bus drop/backpressure/error、late consumer resync、1K/10K、多 viewport、capture churn、plugin reload/fault。 |

## 12. 复核结论

Zircon 当前已有可保留的 scheduler 内核和 SceneMode fault boundary，不应再按“临时字符串 ToolId”描述；P0-02 已真正关闭，revision/outbox、snapshot/resync、capture steal 也不再是空壳。真正未完成的是产品级统一：工具定义没有 factory/setup/terminal contract，InputRouter 没有接入，Select/viewport/modal/world-space/drawer 仍有平行 authority，owner generation 和 operation receipt 不贯穿，host shutdown 不调用 scheduler close。

后续实现必须先完成 Phase A/B 的 capability、receipt、fail-stop 和预算，再按 Phase C/D 迁移 SceneMode、viewport、Gizmo、modal、Export、extension。不得用新增 enum、更多局部 `Option`、单元测试数量或 UI session 清理来宣称 Editor interactive tool system 已达到工程级完成。
