---
related_code:
  - zircon_editor/src/core/tools
  - zircon_editor/src/core/context/tool_scheduler.rs
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/editor_message/message/tool.rs
  - zircon_editor/src/core/editor_message/message/delivery.rs
  - zircon_editor/src/core/editor_message/retention.rs
  - zircon_editor/src/core/editor_message/topics.rs
  - zircon_editor/src/scene/modes/scene_mode_activation.rs
  - zircon_editor/src/scene/modes/scene_mode_stack.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_apply_command.rs
  - zircon_editor/src/ui/host/editor_scene_mode_lifecycle.rs
tests:
  - zircon_editor/src/core/tools/tests.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_input.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_scene_modes.rs
  - zircon_editor/src/ui/host/editor_event_execution/viewport_event.rs
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
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
doc_type: review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 125 · Editor Interactive Tool Scheduler、Resource Lease、Input Capture、Scene Mode、Modal、Extension Lifecycle 当前源码刷新审查

## 1. 结论

当前 `core/tools` 不是空接口：`ToolResourceSet` 有非空、去重、排序和反序列化复核；`ToolScheduler` 有单资源 FIFO、集合请求的无部分占用、撤队、`release_all`、typed outcome 和 lifecycle event；`SceneModeStack` 也有 base/overlay、enter/exit、pass-through checkpoint、overlay build 与 shutdown。问题在于这些底座仍停留在测试/构造层，没有成为 Editor 的真实交互 authority。

全量生产反查只发现 `EditorContextBuilder` 构造 `ToolSchedulerService`；没有生产代码调用 `context.tools()`、`acquire*`、`release*`、`withdraw*` 或订阅 `editor.tool`。Viewport、SceneMode、Gizmo drag、Modal、Export 和 extension unload 继续直接修改各自状态，因此 `ViewportInput`、`ModalSurface`、`SceneModeSlot` 三个资源只是装饰性枚举。

实现本体还存在会破坏租约正确性的硬错误：同一 `ToolId` 持有集合 A 后申请集合 B 会覆盖 `active_sets` 中的 A，但旧资源 holder 仍存在；之后可以通过单资源 release 拆掉集合的一部分。Service 在释放 scheduler mutex 后逐条发布 event，另一个线程可插入反向变更，观察者收到与最终 authority 相反的顺序，且 bus dispatch failure 被静默丢弃。

本报告登记 **3 项 P0、48 项 P1、12 项 P2 与 36 个资格门**。它刷新旧编号 53 的结论，Editor03 继续拥有 Scene/Prefab/Selection/Gizmo 语义，Editor09 继续拥有后台 job/export，Editor48 继续拥有消息传输，Editor50 继续拥有 extension lifecycle；本文只收敛交互工具的 definition、instance、lease、capture、transition 和真实接入。

## 2. 当前语料与参考对照

旧编号 53 的 44 个证据根全部重新展开，按物理文件去重得到：

| 类别 | 文件 | 总行 | 非空行 | bytes | `#[test]` | `#[ignore]` | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Zircon source/test | 20 | 4,726 | 4,274 | 164,111 | 41 | 0 | `4c906626fcb86e78045061fa7b49e59b3c4b9375b961c8c68e7b2563bb5fd21e` |
| Unreal/Fyrox/Godot/Bevy/Unity Graphics | 19 | 9,893 | 8,586 | 369,905 | 7 | 0 | `1aeb239eee2e721c561b2f9b00f199306274f5ad8403660443778236249c3871` |
| plan/docs | 7 | 2,984 | 2,357 | 383,710 | 0 | 0 | `497a29b4bf1386ec0f525699e89a9fece80c411b87d85ad26aab72f5cac97f84` |
| 去重 union | 46 | 17,603 | 15,217 | 917,726 | 48 | 0 | `61464222e646b2e66a0b81d82be4d7aa2b963b297cadc7fb6919331dda747116` |

当前源码 fingerprint 以 normalized path + raw bytes 计算，实施前必须重算。`#[test]` 主要覆盖 scheduler model，不证明真实 viewport、modal、plugin 或焦点链路；本轮没有运行 Cargo、GUI、并发、reload、失焦或性能动态验证。

## 3. 参考源码给出的不可省略结构

| 参考 | 可迁移约束 |
|---|---|
| Unreal Interactive Tools / InputRouter | definition -> real instance -> setup/activate -> capture owner -> Accept/Cancel/Completed/Abort -> Shutdown；capture 支持 priority、steal、force end 和 focus loss。 |
| Fyrox interaction | 每个 scene 有真实 interaction mode container，mouse/key/hotkey/UI/update/drop 都进入当前 mode instance，插件 mode 可增删。 |
| Godot EditorPluginList | 插件输入/绘制返回 PASS/CUSTOM/STOP，add/remove/visible/edit/clear 生命周期可观察。 |
| Bevy Picking/Input Focus | PointerId、PointerMap、输入阶段排序、hover/capture/focus owner 与 despawn 清焦点是独立合同。 |
| Unity Graphics 工具 | `OnActivated`/`OnWillBeDeactivated`、状态保存恢复、Escape 取消和前一工具返回不能省略。 |

这些引擎的共同点不是类数量，而是真实实例拥有输入和状态，激活与捕获同代，owner 消失必然终止。Zircon 可用更紧凑的 Rust 数据布局，但不能用固定 `ToolId` 和测试 event 冒充完整工具系统。

## 4. P0：当前正确性和可达性阻断

### **P0-01** Scheduler 被构造但未接入任何生产 consumer

`ToolSchedulerService` 只在 context builder 创建；生产源码没有 acquire/release/withdraw 或 `editor.tool` subscriber。SceneMode、Viewport、Gizmo、Modal、Export 和 extension unload 都绕过租约，导致理论互斥路径可以并发运行而 scheduler 测试仍然通过。

重构为 `InteractiveToolAuthority`，先接入 SceneMode 和 Viewport，再接入 Gizmo、Modal、Export 与 extension teardown。未接入的功能必须显示 Unavailable，不能以“服务存在”作为已保护证据。

### **P0-02** 同一 ToolId 覆盖 active set，允许部分释放集合占用

`acquire_set` 只把同一 tool + 同一 set 视为 AlreadyHeld。tool 持有集合 A 后申请集合 B，`activate_set` 会保留 A 的 resource holder，却用 B 覆盖 `active_sets`；随后 `release_set(tool, A)` 返回 NotHeld，`release(tool, A_resource)` 又可能单独释放 A 的资源，直接破坏集合原子性。

分离 `ToolDefinitionId`、`ToolInstanceId`、`ToolRequestId` 和不可伪造的 `ToolLeaseId`。一个实例要么只能有一个 active lease，要么以显式多 lease 集合管理；任何 transition 后都检查 holder、active set、queue 的不变量。

### **P0-03** Authority commit、解锁与 lifecycle event publish 没有共同 revision

Service 先在 mutex 内修改 authority，解锁后逐条 `bus.publish`；线程 A 的 Activated 可能在解锁后被线程 B 的 Deactivated 插入，最终 event 顺序与 holder 不一致。`publish_events` 忽略 dispatch result、drop 和 backpressure，`#[must_use]` 无法实现注释声称的“publish before exposing new state”。

用单调 `ToolTransitionRevision` 生成 authority snapshot + outbox batch，按 revision 原子提交和发送；query 带 revision，consumer 支持 gap/resync，delivery failure 进入 typed health/fault，而非静默丢失。

## 5. P1：Definition、Identity、Lease、公平性（01-16）

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-01 | `ToolId`同时表示 definition、instance、request、lease。 | 分离 qualified IDs 与不可复制 lease handle。 |
| P1-02 | 没有 package/plugin owner generation。 | 绑定 owner、extension generation、project/session scope。 |
| P1-03 | 字符串 ID 可被任意调用者复制后 release。 | release/withdraw 只接受 lease/request capability。 |
| P1-04 | 没有 project/document/window/viewport scope。 | 引入 typed `ToolScope` 和跨 scope 拒绝。 |
| P1-05 | 一个 instance 可同时占有 single queue、set queue 和 holder。 | 建立唯一状态机，非法 transition fail-close。 |
| P1-06 | 三个固定 enum 无法表达插件资源、pointer、device、多个 viewport。 | `ResourceKey { kind, scope, channel }` 注册表化。 |
| P1-07 | definition 没有 builder/factory/capability metadata。 | 编译 `ToolDefinition` catalog，创建真实 instance。 |
| P1-08 | Acquired 只写 holder，不代表 Setup/Activate 成功。 | Prepare/Create/Setup/Activate 分阶段并补偿。 |
| P1-09 | 没有 selection/target/context eligibility。 | admission snapshot 携 target、mode、permission、capability。 |
| P1-10 | Deactivated 没有 Accept/Cancel/Completed/OwnerLost。 | 定义 terminal disposition 与 commit/rollback。 |
| P1-11 | 没有前一工具、临时 overlay、resume token。 | 引入 tool stack 和可验证恢复。 |
| P1-12 | release_all 没有和 scene/window/plugin/shutdown owner 绑定。 | teardown 走 quiesce/cancel/release receipt。 |
| P1-13 | set queue 的 head 会阻塞所有空闲 single resource。 | 只阻止冲突 key，建立冲突图与公平策略。 |
| P1-14 | set head 等待时 single release 的晋升顺序不透明。 | 统一 reservation/aging/fairness 规则。 |
| P1-15 | 一次 release 最多晋升一个 set，其他可永久滞留。 | 有界 fixpoint promotion 和 progress receipt。 |
| P1-16 | `max_queue_per_resource` 同时限制不同预算域。 | 拆 per-key/global/owner entries 与 bytes ceiling。 |

## 6. P1：Input Capture、SceneMode、Modal、Extension（17-32）

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-17 | `ViewportInput` 只有全局 holder。 | key 中加入 viewport/window/PointerId/device/channel。 |
| P1-18 | 没有 capture request priority 或稳定仲裁。 | InputRouter 收集 request，按稳定规则选 owner。 |
| P1-19 | 没有 capture stealing/handoff/force-end。 | 定义 preemption policy 和旧 owner receipt。 |
| P1-20 | pointer、keyboard、hover、IME/text focus 混为输入。 | 分离 capture、focus、hover、text owner。 |
| P1-21 | window focus loss/pointer cancel/owner despawn 不终止 capture。 | 平台失效事件同步 cancel 并清 generation。 |
| P1-22 | scheduler 不知道 Consumed/PassThrough 和 effect commit。 | routing result 绑定 mode/transaction receipt。 |
| P1-23 | `SceneModeStack::replace_base` 直接切换旧 mode。 | adapter 先持 `SceneModeSlot`，失败保留旧 mode。 |
| P1-24 | overlay push/pop 不通过共享 authority。 | overlay definition 声明资源和 stack policy。 |
| P1-25 | viewport event 直接进入 controller。 | 按发布的 routing snapshot 校验 capture owner。 |
| P1-26 | `ModalSurface` 没有生产 owner。 | dialog/picker 建立 window-scoped nested modal lease。 |
| P1-27 | Export/build tool ID 只有测试 fixture。 | 交互 lease 与后台 job ticket 分离但可关联。 |
| P1-28 | extension mode 没有真实 add/remove/unload consumer。 | owner generation、quiesce、revoke、unload fence。 |
| P1-29 | mode enter 失败只在局部调用 exit。 | activation transaction 保留旧 snapshot 和补偿 receipt。 |
| P1-30 | mode update/overlay/input 没有异常隔离。 | callback fault domain、panic quarantine、继续/终止策略。 |
| P1-31 | scene close/window close 不调用 release_all。 | scope owner teardown 统一 drain 所有 leases。 |
| P1-32 | 锁定的 resource set 不可迁移到新 viewport/session。 | lease rebind 需要显式 handoff 和 generation 校验。 |

## 7. P1：Event、Snapshot、Concurrency、Contract（33-48）

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-33 | lifecycle event 没有 transition revision/authority epoch。 | envelope 携 epoch、revision、request、instance、lease。 |
| P1-34 | event 没有 owner generation、scope、cause、terminal disposition。 | 建立可审计 transition schema。 |
| P1-35 | set release 多 event 可被其他变更穿插。 | 一次 transition 只发布 ordered batch。 |
| P1-36 | bus dispatch report 被完全忽略。 | failure/backpressure 进入 health、metrics、resync。 |
| P1-37 | 只有单资源 holder query。 | 提供 bounded immutable snapshot、cursor、gap/resync。 |
| P1-38 | 没有 queued request 的位置变更事件。 | position 以 revision snapshot 查询，不能当承诺。 |
| P1-39 | 所有 scope 共用一个 mutex。 | 先保持正确性，再按 scope shard，跨 key ordered commit。 |
| P1-40 | poison 后无条件 `into_inner` 继续服务。 | invariant failure 进入 faulted/rebuild/fail-stop。 |
| P1-41 | 没有 deadline、aging、cancel 或 awaitable wakeup。 | request handle 支持 monotonic deadline/cancel/wait。 |
| P1-42 | 每个事件重复 clone ID/set 并 parse topic。 | 编译 topic、batch transition，先测量再优化布局。 |
| P1-43 | 没有 queue depth、wait/hold、preemption、leak 指标。 | bounded metrics 按 resource/scope/owner 输出。 |
| P1-44 | 没有 Open/Quiescing/Draining/Closed 状态。 | shutdown 拒绝新 acquire，排空旧 lease。 |
| P1-45 | serialized resource/event 没有 schema/version。 | qualified schema、unknown variant、migration policy。 |
| P1-46 | outcome、event、query 没有共同 operation receipt。 | request receipt 贯穿 admission/transition/delivery。 |
| P1-47 | 没有 property/model/concurrency 产品测试。 | reference model + barrier/fault sink + host E2E。 |
| P1-48 | 没有 1K/10K、多 viewport、长时公平性基准。 | workload identity、p50/p95/p99 wait/CPU/alloc/RSS。 |

## 8. P2：长期完整性

| ID | 能力 | 目标 |
|---|---|---|
| P2-01 | compiled resource catalog | 从 catalog 生成稳定 resource 列表和诊断名。 |
| P2-02 | typed validation report | 资源、owner、scope、schema 错误有结构化上下文。 |
| P2-03 | lease mismatch diagnostics | wrong-set/wrong-generation 返回 typed mismatch。 |
| P2-04 | conflict graph | 从 compiled catalog 导出冲突图，UI 不维护第二份表。 |
| P2-05 | localized labels | tool/resource label 用 LocalizationKey，identity 不本地化。 |
| P2-06 | queued request query | bounded cursor 分页、过滤、owner scope。 |
| P2-07 | transition history | source-bound ring buffer 与 support bundle。 |
| P2-08 | idempotent acquire | AlreadyHeld/Queued 返回 canonical handle/revision。 |
| P2-09 | typed scheduler settings | queue capacity、aging、bytes 有 hard ceiling。 |
| P2-10 | accessible tool switcher | keyboard/Escape/cancel 与 focus/a11y 集成。 |
| P2-11 | recovery UI | orphan lease、owner loss、stale capture 的受控修复。 |
| P2-12 | performance receipt | BuildSet、场景、目录、输入 trace、硬件绑定结果。 |

## 9. 资格门

| Gate | 通过条件 |
|---|---|
| G-01 | SceneMode、Viewport、Gizmo、Modal、Export 至少各有一个真实 authority consumer。 |
| G-02 | 未接入的功能不会显示 scheduler-protected 或 Available。 |
| G-03 | 同一 ToolInstance 不会覆盖 active lease，也不会部分释放 set。 |
| G-04 | 所有 holder/active set/queue invariant 有 property/model coverage。 |
| G-05 | Authority revision 与 event batch revision 一致。 |
| G-06 | consumer 能检测 gap 并从 bounded snapshot resync。 |
| G-07 | bus drop/backpressure 进入 typed health，不静默成功。 |
| G-08 | pointer/window/device/session generation 参与 capture 校验。 |
| G-09 | focus loss、pointer cancel、owner despawn 会终止 capture。 |
| G-10 | capture steal 有旧 owner force-end 和新 owner receipt。 |
| G-11 | SceneMode enter/exit/update/input 的失败可补偿且不丢旧 mode。 |
| G-12 | overlay push/pop 与 slot lease、stack revision 同代。 |
| G-13 | nested modal、dialog close、window destroy 全部释放 lease。 |
| G-14 | extension reload/unload 先 quiesce、撤销 capture，再销毁 instance。 |
| G-15 | scene/project/session close 有 release-all terminal receipt。 |
| G-16 | Acquired 不会在 Setup/Activate 失败时泄漏 holder。 |
| G-17 | Accept/Cancel/Completed/Aborted/OwnerLost 语义进入 transaction。 |
| G-18 | queue fairness、aging、reservation 在长时模型测试中有进度证明。 |
| G-19 | queue entries 同时受 item、bytes、owner、global budget 限制。 |
| G-20 | shutdown 阶段拒绝新请求且能等待既有 lease 终止。 |
| G-21 | poison、panic、callback fault 不会继续暴露不可信 authority。 |
| G-22 | lifecycle schema 带 version、unknown policy 和 migration。 |
| G-23 | 每个 request/event/query 都可追溯同一 operation receipt。 |
| G-24 | wrong owner、wrong scope、wrong generation 都返回脱敏 typed error。 |
| G-25 | scheduler 资源与插件 owner generation 能在 reload 后重建。 |
| G-26 | Viewport routing 只消费当前 snapshot，不读取过期 holder。 |
| G-27 | modal/scene mode/viewport 共享同一 focus and capture policy。 |
| G-28 | host event loop shutdown 与 scheduler shutdown 顺序固定且可测试。 |
| G-29 | duplicate acquire/release/withdraw 是幂等且返回原 handle。 |
| G-30 | 同时提交多个 resource set 的 deadlock 顺序有明确证明。 |
| G-31 | 无 subscriber、subscriber drop、subscriber reject 均有可见诊断。 |
| G-32 | 实际插件交互输入、绘制、drop、remove 全走同一 instance。 |
| G-33 | Unreal/Fyrox/Godot/Bevy/Unity 对照只转译结构，不复制不适用 ABI。 |
| G-34 | 1K/10K workloads 报告 p50/p95/p99、alloc、RSS、wake 和 fairness。 |
| G-35 | 真实 Windows 多窗口、失焦、窗口销毁、reload E2E 通过。 |
| G-36 | 重新计算 46-file union fingerprint，并由独立 review 检查生产 caller 矩阵。 |

## 10. 建议实施顺序

1. 先修 `active_sets` 覆盖和 transition revision，加入 reference model 与 fault sink；这是 P0-02、P0-03 的共同根。
2. 建立真实 `ToolDefinition -> ToolInstance -> ToolLease -> InputCapture` authority，首批接 SceneMode/Viewport，禁止继续扩展旁路。
3. 将 modal、Gizmo、Export 和 extension unload 接入 owner generation、quiesce、force-end、terminal disposition。
4. 再做 message bus snapshot/resync、shutdown、accessibility、metrics 与大规模基准；固定 UI 和测试 fixture 不得作为 consumer 证据。

## 11. 复核结论

Editor 交互工具已经有局部 scheduler 和 mode 底座，但还没有 Unreal/Fyrox/Godot 意义上的可创作、可抢占、可恢复工具实例系统。最优先的不是增加更多工具类型，而是先建立唯一 lease authority、输入 capture 生命周期和 ordered transition receipt；在这三项完成前，SceneMode、Viewport、Modal、Gizmo 和插件交互仍会互相绕过，性能优化也缺乏可比较的真实 workload。
