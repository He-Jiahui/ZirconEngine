---
related_code:
  - zircon_editor/src/core/tools
  - zircon_editor/src/core/context/tool_scheduler.rs
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/editor_message/message/tool.rs
  - zircon_editor/src/core/editor_message/message/delivery.rs
  - zircon_editor/src/core/editor_message/retention.rs
  - zircon_editor/src/core/editor_message/topics.rs
  - zircon_editor/src/scene/modes
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_apply_command.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_input.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_scene_modes.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_interaction.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_interaction_cancel.rs
  - zircon_editor/src/ui/host/editor_scene_mode_lifecycle.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/workbench/state/editor_state.rs
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/src/ui/host/editor_event_execution/viewport_event.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/platform_input.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport/pointer_dispatch.rs
  - zircon_editor/src/ui/retained_host/viewport/world_space_ui.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/capture.rs
  - zircon_editor/src/ui/retained_host/app/scene_picker_actions.rs
  - zircon_editor/src/ui/retained_host/app/scene_picker_session.rs
  - zircon_editor/src/ui/retained_host/app/command_palette_actions.rs
  - zircon_editor/src/ui/retained_host/app/settings_window_actions.rs
  - zircon_editor/src/ui/retained_host/app/project_close.rs
tests:
  - zircon_editor/src/core/tools/tests.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/scene/modes/tests.rs
  - zircon_editor/src/scene/modes/tests/isolation.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_input.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_scene_modes.rs
  - zircon_editor/src/ui/host/editor_event_execution/viewport_event.rs
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/pointer_bridge.rs
  - zircon_editor/src/tests/host/retained_window/platform_input_translation.rs
  - zircon_editor/src/tests/host/retained_drawer_resize/pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/viewport/tests/controller_submits_shared_ui_overlay_through_render_framework.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/48-editor-message-bus-topic-subscription-inbox-retention-admission-dispatch-request-dirty-projection-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Public/InteractiveToolManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/InteractiveToolManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Public/InputRouter.h
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/InputRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Public/InteractiveToolsContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/InteractiveToolsContext.cpp
  - dev/Fyrox/editor/src/interaction/mod.rs
  - dev/Fyrox/editor/src/lib.rs
  - dev/Fyrox/editor/src/scene/container.rs
  - dev/godot/editor/plugins/editor_plugin.h
  - dev/godot/editor/plugins/editor_plugin.cpp
  - dev/godot/editor/plugins/editor_plugin_list.h
  - dev/godot/editor/plugins/editor_plugin_list.cpp
  - dev/bevy/crates/bevy_picking/src/lib.rs
  - dev/bevy/crates/bevy_picking/src/input.rs
  - dev/bevy/crates/bevy_picking/src/pointer.rs
  - dev/bevy/crates/bevy_input_focus/src/lib.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Utilities/GenericEditorTool.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/LightPlacementTool.cs
refreshes:
  - docs/plans/optimize/zircon_editor/53-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/125-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-current-source-review.md
doc_type: review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 174 · Editor Interactive Tool Scheduler、Resource Lease、Input Capture、Scene Mode、Modal、Extension Lifecycle 当前源码复核

## 1. 结论

Editor53/125 的三个主问题仍然存在，但当前源码已经出现值得保留的 SceneMode 与平台输入工程化进展。`SceneModeActivation` 现在校验 builtin/custom ID，stack 记录 revision 与 activation，replace 会先 enter 新 mode，失败时保留旧 base；extension mode factory、id、enter、exit、input、update、overlay 和 drop 已统一经过 plugin panic boundary，callback 失败会恢复 `SceneModeCtx` checkpoint、quarantine faulted instance，并继续安全 shutdown。native focus loss 和 pointer cancel 也会路由到 viewport `CancelInteraction`，world-space UI 与 drawer resize 能各自释放本地 capture。

这些改进没有形成共同工具 authority。全 production Rust 反查仍只找到 `EditorContextBuilder` 构造 `ToolSchedulerService`；除了定义、消息编码和 tests，没有代码调用 `context.tools()`、scheduler acquire/release/withdraw，三个 `ExclusiveResource` 也没有真实 holder。SceneMode、gizmo、viewport pointer、world-space UI、drawer resize、scene picker/modal 和 extension install 各自维护状态，彼此没有共同 `ToolInstanceId`、lease、capture generation 或 terminal receipt。

Scheduler 的集合正确性只做了局部修补：single-resource `release()` 会拒绝拆当前 `active_sets` 记录中的集合，但同一 `ToolId` 仍能用第二个不同资源集合覆盖该记录。若 A 先持 `{ViewportInput}`，再成功申请 `{ModalSurface}`，`active_sets[A]` 会被后者覆盖，而 Viewport holder 仍是 A；此后 single-resource release 看不到旧集合，可释放 orphan holder。P0-02 因此只能降为 Partial，不能关闭。

本轮不新增 canonical finding，继续由 Editor53 作为 owner，并刷新 Editor125。当前状态：

| 等级 | Open | Partial | Closed | 合计 |
|---|---:|---:|---:|---:|
| P0 | 2 | 1 | 0 | 3 |
| P1 | 34 | 13 | 1 | 48 |
| P2 | 12 | 0 | 0 | 12 |
| 资格门 | 22 Fail | 13 Partial | 1 Pass | 36 |

Tooling 按用户要求排除。本轮只写 review 和计划文档，没有改生产实现，也没有查询、轮询、等待或实时跟踪协调器。

## 2. 当前语料冻结

冻结点为 HEAD `ea35974cdf64068f6789010451d20bbf69e0a29d`、2026-08-27T17:02:35+08:00，共享工作树冻结时有 8,227 个 status 条目。Editor125 原 20-file 核心选择集已扩展到 SceneMode 全目录、extension 安装、平台 cancel、三类真实 capture/picker 及相应 tests，避免只扫描 scheduler 后误判产品现状。

| 类别 | 文件 | 总行 | 非空行 | bytes | `#[test]` | `#[ignore]` | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Zircon source/test | 57 | 11,070 | 10,045 | 383,593 | 110 | 2 | `91689263113c51ddc374014c2bfa441e1421f6b1b54e6da45c13d92b44ccf7fc` |
| Unreal/Fyrox/Godot/Bevy/Unity Graphics | 19 | 9,893 | 8,586 | 369,905 | 7 | 0 | `1aeb239eee2e721c561b2f9b00f199306274f5ad8403660443778236249c3871` |
| plan/docs | 7 | 2,984 | 2,357 | 383,710 | 0 | 0 | `497a29b4bf1386ec0f525699e89a9fece80c411b87d85ad26aab72f5cac97f84` |
| 去重 union | 83 | 23,947 | 20,988 | 1,137,208 | 117 | 2 | `10a0e62418d281e827b89a22f32a3c8a4a79e6564ecc437130ef10c6f5d6c5a8` |

指纹使用 normalized relative path + NUL + raw bytes + NUL。共享工作树中 scheduler、context、SceneMode、viewport 与 retained host 相关文件有大量在途修改，本轮全部按当前磁盘审查且不回退。没有运行 Cargo、GUI、并发、失焦、reload、fault、规模或性能动态验证；110 个 test attribute 只能证明局部 model/boundary 行为。

## 3. 产品可达性矩阵

| 子系统 | 当前真实 authority | Scheduler 接入 | 工程判定 |
|---|---|---|---|
| SceneMode base/overlay | `SceneModeStack` + registry + controller，production tick/update/drop 可达。 | 无 `SceneModeSlot` acquire/release。 | 真实 mode 实例系统，但不是共享工具 authority。 |
| Gizmo drag | `ViewportDragSession` + editor transaction begin/preview/finish/cancel。 | 无 `ViewportInput` lease。 | 有事务终态样板，不能阻止 modal/mode/capture 冲突。 |
| Viewport pointer | retained pointer route capture + controller direct dispatch。 | 无 capture owner snapshot 或 scheduler query。 | 输入仍绕过工具调度。 |
| World-space UI | 独立 `world_space_ui_pointer_capture`，Up/Cancel 时 `take()`。 | 无统一 pointer/device/window generation。 | 局部 capture 正确，和 viewport/drawer 并行。 |
| Drawer resize | `shell_pointer_bridge` 维护 resize capture 并显式 cancel。 | 无 `ToolInstance`/lease。 | 独立 shell authority。 |
| Scene picker / command palette / settings | retained App 持有 session，surface 切换/提交/project close 时清理。 | `ModalSurface` 无生产 holder。 | 有 real modal-like session，但没有 nested modal lease/focus policy。 |
| Extension SceneMode | Host candidate-build registry，owner ID，callback boundary，ContributionStore ticket。 | 无 owner generation lease、unregister/revoke consumer。 | install 可达，unload/reload 仍断裂。 |
| Export/build | 后台 job/command 由 Editor09 管理。 | 无交互 request/lease 关联。 | `ToolId` 仍只出现在 tests。 |

全生产反查结果：`ToolSchedulerService::new` 只有 builder 构造；`EditorContext::tools()` 无 production caller；`TOPIC_TOOL` 无 production subscriber；`ExclusiveResource::{ViewportInput, ModalSurface, SceneModeSlot}` 只在 scheduler 定义和 context tests 中使用。P0-01 仍为 Open。

## 4. 当前真实进展

### 4.1 SceneMode activation 与 instance boundary

1. `SceneModeRegistration` 已包含 descriptor、factory 和 owner ID；builtin 与 extension 都通过同一 registry 创建实例。
2. registry 使用 candidate clone 安装 extension modes，并在 commit 前创建每个 mode 以校验 factory 结果 ID；错误保持 typed `SceneModeRegistryError`。
3. `SceneModeStack` 绑定 `SceneModeActivation`，禁止 builtin mode 冒充 overlay，记录 stack revision；replace-base 在新 mode enter 成功后才 exit 旧 base。
4. `IsolatedSceneMode` 将 factory/id/enter/exit/input/update/overlay/drop 纳入 plugin boundary。callback panic 后恢复 selection/input effect/overlay checkpoint；faulted mode 后续 input/update/overlay 被隔离，exit/drop 仍受保护。
5. `EditorState::Drop` 会 shutdown SceneMode stack；production tick 会 update mode。这些是真实 instance lifecycle，而非只有 descriptor。

因此 P1-07/08/09/23/24/28/29 是 Partial，P1-30 可以 Closed。但 owner 仍只有字符串，没有 catalog/build generation、unregister 或 active-instance revoke；stack 也不持 `SceneModeSlot` lease。

### 4.2 Input cancel 与本地 capture

native `PointerLeft` 可生成 pointer cancel，`Focused(false)` 会走 viewport cancellation callback；retained pointer bridge 把 `UiPointerEventKind::Cancel` 转成 `EditorViewportEvent::CancelInteraction`。world-space UI 的 cancel 会 take capture，drawer resize 的异常/终止路径会 cancel resize。P1-21 因而从“完全没有终止”变为 Partial。

但这些结构没有一个共同 `PointerId`、window/device/session generation，也没有 capture request priority、steal/handoff、keyboard/IME/text focus 分离或 owner-despawn cleanup。不同局部 capture 同时存在时，scheduler 无法解释谁是最终输入 owner。

### 4.3 Scheduler 局部算法改进

`ToolResourceSet` 使用 Vec sort/dedup 构造规范集合；release/withdraw 后在 set queue 清空时会晋升其他 single-resource waiter；single release 会对当前 active set 返回 `SetHeld`；service 预解析并复用 `editor.tool` topic。这些是局部 correctness/performance 改善。

未修根因：

```text
acquire_set(A, {ViewportInput}) -> active_sets[A] = {ViewportInput}
acquire_set(A, {ModalSurface})  -> active_sets[A] = {ModalSurface}
                                  ViewportInput holder 仍为 A
release(A, ViewportInput)       -> 旧 set 不可见，single release 成功
```

当前 tests 只覆盖“当前 active set 不可单资源拆除”，没有覆盖同一 tool 的不同 active/queued set、single+set 同时状态或全不变量 reference model。P0-02 只能记 Partial。

## 5. 仍然断裂的 authority 与事件因果

`ToolId` 继续同时表示 definition、instance、request、lease 和 release capability；任何知道字符串的调用者都能 release/withdraw。资源只有三个全局 enum，没有 project/document/window/viewport/pointer/device/channel scope。Scheduler 无 Setup/Activate callback，也没有 Accept/Cancel/Completed/Aborted/OwnerLost、deadline、cancel handle、shutdown state、bytes/global/owner budget。

`ToolSchedulerService` 在 mutex 内修改 authority，解锁后逐事件调用 message bus。bus 已返回 delivered/coalesced/dropped/backpressured/error report，但 service 完全忽略它。多个 service clone/thread 可在 A 解锁和发布间插入 B transition，event stream 既没有 scheduler revision/epoch，也没有 ordered batch、gap/resync 或 immutable snapshot。`#[must_use]` 只约束 caller 使用 schedule report，不能保证 authority 与消息发布同一因果序。P0-03 保持 Open。

Poisoned scheduler mutex 仍以 `into_inner()` 继续服务；SceneMode callback boundary 的进展不能修复 scheduler invariant fault。Extension ContributionStore 虽可 revoke scene-mode contribution record，但 production controller registry 没有 remove/reconcile，active mode 也没有 owner generation fence。

## 6. 本地参考约束

| 参考 | 当前源码证据 | 对 Zircon 的约束 |
|---|---|---|
| Unreal InteractiveToolManager | CanActivate -> Create/Setup -> PostSetup -> InputRouter register；Deactivate 先 deregister capture source，再用 Accept/Cancel/Completed shutdown；Context shutdown force-terminates capture。 | Acquired 必须晚于真实 setup，终止必须先撤输入 owner，再完成 transaction/dispose。 |
| Unreal InputRouter | 按稳定排序收集 capture request，区分 keyboard/left/right/hover，支持 ForceTerminateSource/All，应用失焦终止全部 capture。 | 不能用单一全局 `ViewportInput` holder 代替多通道 capture。 |
| Fyrox InteractionMode | 每个 scene 有 mode container，activate/deactivate，mouse/key/hotkey/UI/update 全进入当前实例。 | scope 必须至少绑定 scene/document/viewport，instance 生命周期必须真实可达。 |
| Godot EditorPluginList | plugin add/remove 可见；viewport input 返回 PASS/CUSTOM/STOP。 | 插件移除与 routing disposition 必须属于同一 owner 生命周期。 |
| Bevy Picking/Input Focus | `PointerId`/PointerMap、Cancel、hover system ordering 与独立 `InputFocus`。 | pointer、hover、focus、text/keyboard owner 不能混成一个资源。 |
| Unity Graphics tools | OnActivated/OnWillBeDeactivated，保存/恢复前状态，Escape 返回 previous tool。 | 临时 tool stack 与 cancel/resume token 是正式合同。 |

对照只提取本地源码可观察结构，不复制 ABI，也不外推闭源实现。

## 7. P0 状态

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P0-01 | **Open** | scheduler 仍只有 builder 构造、DTO 和 tests；0 production acquire/release/subscriber。 | 建立 `InteractiveToolAuthority`，先接 SceneMode/Viewport/Gizmo，再接 modal/export/extension teardown。 |
| P0-02 | **Partial** | single release 已保护当前 active set；同 ToolId 不同 set 仍覆盖 `active_sets` 并遗留 orphan holder。 | 分离 definition/instance/request/lease ID，建立唯一状态机和 transition 后不变量检查。 |
| P0-03 | **Open** | state commit 后解锁逐条 publish；无 revision/batch，dispatch report 被丢弃。 | authority snapshot + revisioned outbox 原子提交，consumer gap/resync，delivery failure 可见。 |

## 8. P1 状态（01-16）

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P1-01 | **Open** | `ToolId` 混合五类 identity。 | qualified definition/instance/request/lease IDs。 |
| P1-02 | **Open** | Scheduler 无 owner/catalog generation。 | 绑定 extension/build/project/session generation。 |
| P1-03 | **Open** | 可复制字符串仍是 release capability。 | release 只接受不可伪造 lease handle。 |
| P1-04 | **Open** | 三资源均为 editor-global。 | typed project/document/window/viewport scope。 |
| P1-05 | **Open** | 同 tool 仍可跨 single/set/queue 形成多重状态。 | 单一合法 transition graph。 |
| P1-06 | **Open** | 固定 enum 无 plugin/pointer/device/multi-viewport key。 | compiled `ResourceKey { kind, scope, channel }`。 |
| P1-07 | **Partial** | SceneMode 已有 descriptor/factory/owner；通用 tool definition catalog 缺失。 | 编译统一 `ToolDefinition`。 |
| P1-08 | **Partial** | SceneMode create/enter 失败可回退；scheduler Acquired 仍只是 holder 写入。 | Prepare/Create/Setup/Activate/Commit 分阶段补偿。 |
| P1-09 | **Partial** | SceneMode command eval 带 selection/mode revision；scheduler admission 无 target snapshot。 | admission 绑定 target/mode/permission/capability。 |
| P1-10 | **Open** | Gizmo 有局部 finish/cancel，通用 tool event 无 terminal disposition。 | Accept/Cancel/Completed/Aborted/OwnerLost 绑定 transaction。 |
| P1-11 | **Open** | 无 previous tool/overlay/resume token。 | typed tool stack 与恢复证明。 |
| P1-12 | **Open** | SceneMode Drop 会 shutdown；scheduler `release_all` 不绑定 owner close。 | scope teardown terminal receipt。 |
| P1-13 | **Open** | set queue head 仍全局阻止不冲突 single resource。 | 冲突图、reservation 与公平 policy。 |
| P1-14 | **Open** | 晋升条件仍由多处分支隐式决定。 | 统一 aging/fairness/progress 规则。 |
| P1-15 | **Open** | 一次 transition 最多 promote 一个 set，无 fixpoint。 | bounded fixpoint promotion。 |
| P1-16 | **Open** | 单一 queue length 同时充当全部预算。 | per-key/global/owner count+bytes ceiling。 |

## 9. P1 状态（17-32）

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P1-17 | **Open** | `ViewportInput` 仍是全局 enum。 | 加 viewport/window/pointer/device/channel。 |
| P1-18 | **Open** | 无 capture request priority/arbitration。 | 稳定收集与选择 owner。 |
| P1-19 | **Open** | 无 steal/handoff/force-end receipt。 | typed preemption。 |
| P1-20 | **Open** | 多个本地 capture/focus 仍是平行 authority。 | 分离 pointer/keyboard/hover/text/IME owner。 |
| P1-21 | **Partial** | focus loss、PointerLeft/Cancel 能取消 viewport/局部 capture；owner despawn/generation 未覆盖。 | 平台失效统一终止 current capture generation。 |
| P1-22 | **Partial** | SceneMode 有 Consumed/PassThrough + bounded input effect，gizmo 有 transaction。 | routing result 绑定统一 lease/transaction receipt。 |
| P1-23 | **Partial** | replace base 先 enter 新 mode，失败保留旧 mode；无 `SceneModeSlot` lease。 | 共享 authority activation transaction。 |
| P1-24 | **Partial** | overlay 有 typed activation、stack revision、enter/pop/exit；无 resource/owner generation。 | overlay definition 声明 lease 与 stack policy。 |
| P1-25 | **Open** | viewport event 仍直接进入 controller。 | 只按 published capture snapshot routing。 |
| P1-26 | **Partial** | scene picker/command palette/settings 有真实 session 与 close 清理；`ModalSurface` 未使用。 | window-scoped nested modal lease。 |
| P1-27 | **Open** | export 与 tool ID 仍只有 test 关联。 | lease 与 job ticket 分离并交叉寻址。 |
| P1-28 | **Partial** | extension mode 有 owner ID、candidate install、ContributionStore ticket；controller 无 production remove/revoke。 | owner generation + quiesce/revoke/unload fence。 |
| P1-29 | **Partial** | enter panic/ID mismatch 拒绝 stack mutation并运行 cleanup。 | 将 receipt 扩展到 lease/layout/transaction。 |
| P1-30 | **Closed** | factory/id/enter/exit/input/update/overlay/drop 均进入 plugin panic boundary，faulted instance 被隔离，ctx/overlay 有 checkpoint 恢复测试。 | 保持 boundary；后续纳入统一 health/quarantine projection。 |
| P1-31 | **Partial** | `EditorState::Drop` shutdown modes，project close 清理 picker；无 scheduler/scope drain receipt。 | scene/window/project/session 统一 drain leases。 |
| P1-32 | **Open** | lease 无 rebind/handoff/generation。 | 显式 migration receipt。 |

## 10. P1 状态（33-48）

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P1-33 | **Open** | lifecycle event 无 authority revision/epoch。 | envelope 携 request/instance/lease/revision。 |
| P1-34 | **Open** | 无 owner generation/scope/cause/disposition。 | versioned audit schema。 |
| P1-35 | **Open** | set 多 event 可被其他线程穿插。 | 一 transition 一 ordered batch。 |
| P1-36 | **Open** | dispatch report 仍完全忽略。 | drop/backpressure/error 进入 health/resync。 |
| P1-37 | **Open** | query 只有单资源 holder/queue iterator。 | bounded immutable snapshot/cursor/gap。 |
| P1-38 | **Open** | queue position 无 revision。 | 只从 current snapshot 查询。 |
| P1-39 | **Open** | 所有 scope 仍共用一个 mutex。 | 正确后再按 scope shard。 |
| P1-40 | **Open** | poison 继续 `into_inner()`。 | faulted/rebuild/fail-stop。 |
| P1-41 | **Open** | 无 deadline/cancel/awaitable wake。 | monotonic deadline request handle。 |
| P1-42 | **Partial** | ToolId 用 Arc，resource set 规范化，topic 在 service 创建时预解析；event 仍逐条 clone/publish。 | revisioned transition batch，并以基准驱动布局。 |
| P1-43 | **Open** | SceneMode 有输入 profiling；scheduler 无 depth/wait/hold/leak 指标。 | bounded scheduler metrics。 |
| P1-44 | **Open** | 无 Open/Quiescing/Draining/Closed。 | shutdown state machine。 |
| P1-45 | **Open** | resource/event schema 无 version/unknown policy。 | versioned codec/migration。 |
| P1-46 | **Open** | outcome/event/query 无共同 operation receipt。 | request receipt 贯穿全链。 |
| P1-47 | **Partial** | SceneMode panic、ID、stack/input/cancel tests 显著增强；scheduler 仍无 property/model/concurrency/host E2E。 | reference model + barrier + fault sink + product E2E。 |
| P1-48 | **Open** | 无 1K/10K、多 viewport、公平性 workload。 | p50/p95/p99 wait/CPU/alloc/RSS。 |

## 11. P2 状态

| ID | 状态 | 能力 | 目标 |
|---|---|---|---|
| P2-01 | **Open** | compiled resource catalog | 稳定 resource 列表与诊断名。 |
| P2-02 | **Open** | typed validation report | resource/owner/scope/schema 上下文。 |
| P2-03 | **Open** | lease mismatch diagnostics | wrong set/generation typed mismatch。 |
| P2-04 | **Open** | conflict graph | catalog 导出，UI 不维护第二表。 |
| P2-05 | **Open** | localized labels | identity 与本地化分离。 |
| P2-06 | **Open** | queued request query | bounded cursor/filter/owner scope。 |
| P2-07 | **Open** | transition history | source-bound ring/support bundle。 |
| P2-08 | **Open** | idempotent acquire | 返回 canonical handle/revision。 |
| P2-09 | **Open** | typed scheduler settings | count/bytes/aging hard ceiling。 |
| P2-10 | **Open** | accessible tool switcher | keyboard/Escape/focus/a11y。 |
| P2-11 | **Open** | recovery UI | orphan lease/stale capture 修复。 |
| P2-12 | **Open** | performance receipt | BuildSet/scene/input trace/hardware。 |

## 12. 资格门

| Gate | 状态 | 当前判定 |
|---|---|---|
| G-01 | **Fail** | SceneMode/Viewport/Gizmo/Modal/Export 均无 scheduler authority consumer。 |
| G-02 | **Fail** | 未接入功能仍没有 truthful protection availability。 |
| G-03 | **Partial** | 当前 active set 防 single release；different-set overwrite 仍破坏 invariant。 |
| G-04 | **Fail** | 无全状态 property/reference model。 |
| G-05 | **Fail** | authority/event 无共同 revision。 |
| G-06 | **Fail** | consumer 无 gap/resync。 |
| G-07 | **Fail** | bus report 静默丢弃。 |
| G-08 | **Fail** | capture 校验不含 pointer/window/device/session generation。 |
| G-09 | **Partial** | focus loss/pointer cancel 已终止局部 capture；owner despawn 未覆盖。 |
| G-10 | **Fail** | 无 capture steal/handoff。 |
| G-11 | **Partial** | enter/input callback 有 checkpoint/cleanup；未绑定 lease/transaction。 |
| G-12 | **Partial** | overlay 与 stack revision 同代；无 slot lease。 |
| G-13 | **Partial** | picker/palette/project close 有 session 清理；无 nested modal lease。 |
| G-14 | **Fail** | extension unload 未撤 active mode/capture。 |
| G-15 | **Partial** | EditorState Drop shutdown modes；无 release-all terminal receipt。 |
| G-16 | **Fail** | scheduler Acquired 不执行 Setup/Activate。 |
| G-17 | **Partial** | gizmo 有 cancel/finish transaction；通用 tool 无 terminal disposition。 |
| G-18 | **Fail** | 无 aging/progress proof。 |
| G-19 | **Fail** | queue 无 bytes/owner/global budget。 |
| G-20 | **Fail** | scheduler 无 shutdown phase。 |
| G-21 | **Partial** | SceneMode callback fault 已隔离；scheduler poison 仍继续服务。 |
| G-22 | **Fail** | lifecycle schema 无 version/migration。 |
| G-23 | **Fail** | request/event/query 无共同 receipt。 |
| G-24 | **Partial** | owner/resource mismatch 有 typed outcome；scope/generation 不存在。 |
| G-25 | **Fail** | reload 后 scheduler resource/owner generation 不可重建。 |
| G-26 | **Fail** | viewport routing 不消费 scheduler snapshot。 |
| G-27 | **Fail** | modal/scene mode/viewport 没有共享 focus/capture policy。 |
| G-28 | **Fail** | host shutdown 与 scheduler shutdown 无固定顺序。 |
| G-29 | **Fail** | duplicate 只返回 enum，不返回 canonical request/lease handle。 |
| G-30 | **Partial** | 单 mutex + canonical set order 避免内部多锁死锁；无跨 request 正式证明。 |
| G-31 | **Fail** | zero subscriber/drop/reject 对 tool domain 不可见。 |
| G-32 | **Partial** | extension SceneMode 的 input/update/overlay/drop 走同一 isolated instance；remove 缺失。 |
| G-33 | **Pass** | 参考对照仅转译本地源码结构，不复制 ABI。 |
| G-34 | **Fail** | 无规模性能报告。 |
| G-35 | **Partial** | Windows focus loss/pointer cancel 有 tests；多窗口/reload E2E 未闭合。 |
| G-36 | **Partial** | 83-file union 指纹已重算；本轮无独立 reviewer。 |

## 13. 目标架构

```text
CompiledToolDefinition
  identity + owner/build generation + capability
  resource keys + scope policy + input behaviors
  factory + setup + terminal policy
        |
ToolRequest(request_id, scope, target snapshot, deadline)
        |
InteractiveToolAuthority
  conflict graph + fairness + count/bytes budgets
  revisioned state + ordered outbox + immutable snapshot
        |
ToolLease(lease_id, instance_id, resource keys, generation)
        |
InputRouterSnapshot
  pointer/device/window capture + keyboard/focus/hover/text owners
        |
Accept / Cancel / Completed / Aborted / OwnerLost
        |
quiesce -> force-end capture -> rollback/commit -> dispose -> terminal receipt
```

必须坚持以下不变量：

1. 一个 instance 的 active/queued/terminal 状态唯一，resource holder、lease table 和 queue 在每次 transition 后互相可证明。
2. Acquire success 晚于 factory/setup/activate；失败不暴露 holder。
3. Input routing 只读取同 revision 的 capture snapshot，owner generation 失效立即 force-end。
4. Authority revision 与 outbox batch 一起 commit；delivery failure 不回滚 authority，但必须可检测 gap 并 resync。
5. SceneMode、Gizmo、viewport、modal、picker、export 和 extension unload 只能通过 adapter 使用这一个 authority。

## 14. 重构顺序

### Phase A：算法止血

1. 禁止同一 identity 同时拥有/排队多个 single/set request；先补 different-set overwrite、single+set、release_all 和 randomized reference-model tests。
2. 分离 Definition/Instance/Request/Lease ID，release/withdraw 改用 capability handle。
3. 给 authority 增加 checked revision、typed invariant fault 和 immutable snapshot；poison 不再继续服务。

### Phase B：revisioned transition/outbox

1. 一次 scheduler mutation 生成一个 ordered event batch 与 operation receipt。
2. dispatch report 汇总进 tool health；consumer 以 cursor 检测 gap 并 snapshot resync。
3. 建立 count/bytes/owner/global budget、deadline、cancel 和 shutdown phase。

### Phase C：输入与 SceneMode 接入

1. `ResourceKey` 加 scene/document/window/viewport/pointer/device/channel scope。
2. 以现有 SceneMode factory/boundary 为 adapter，activation 必须先持 `SceneModeSlot` + input lease。
3. retained pointer、world-space UI、drawer resize 收敛到同一 InputRouter；保持现有 native cancel 行为。

### Phase D：Gizmo、Modal、Extension、Export

1. Gizmo transaction 映射 Accept/Cancel/OwnerLost；capture force-end 必须 rollback preview。
2. scene picker/palette/settings 建立 nested modal lease，window/project close 按 terminal receipt drain。
3. contribution ticket/generation 贯穿 SceneMode instance，revoke 先 quiesce/cancel，再删除 registry/factory。
4. Export 只在需要交互 UI 时持 tool lease，后台执行另持 JobTicket，二者可交叉诊断但不混 identity。

### Phase E：资格与性能

补全 property/concurrency/fault/Windows multi-window/reload E2E；再对 1K/10K request、多 viewport、capture churn、plugin reload 报告 p50/p95/p99、CPU、alloc、RSS、wake 和 fairness。当前 SceneMode profiling 与 Hash/Vec 局部优化不能替代这些产品 workload。

## 15. 复核结论

Zircon 已经拥有一套比旧报告更可信的 SceneMode instance/fault boundary，也已经把 native focus loss 和 pointer cancel 接入局部交互终止。这些进展应直接复用。真正阻止其成为工程级交互工具系统的，是 scheduler 仍不可达、identity/lease 未分离、不同 set 覆盖仍破坏 holder 不变量，以及所有局部 capture 没有共同 revision/generation authority。

后续实现必须先封 P0-02/P0-03，再把 SceneMode/Viewport/Gizmo 迁入唯一 authority；不能把现有 tests、三个 enum 或局部 cancel handler继续当作“工具系统已完成”的证据。实施前需重新冻结共享工作树并重算选择集。
