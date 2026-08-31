---
status: source_complete_validation_pending
created_at: 2026-08-30
implementation_status: builtin-topic-and-scene-export-consumers-source-complete-static-verified
managed_validation_status: blocked_unmanaged_artifacts_detected
related_code:
  - zircon_editor/src/core/tools/scheduler.rs
  - zircon_editor/src/core/context/tool_scheduler.rs
  - zircon_editor/src/core/editor_message/topic.rs
  - zircon_editor/src/scene/modes/scene_mode_stack.rs
  - zircon_editor/src/scene/modes/scene_mode_ctx.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_scene_modes.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_construction.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller.rs
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorModeManager.h
  - dev/UnrealEngine/Engine/Source/Editor/Blutility/Private/EditorUtilitySubsystem.cpp
---

# Editor08 ToolScheduler Consumer Architecture Review

## 目标

复核 `ToolScheduler`、scene mode stack 与 export wizard 的 current-source ownership，确定生产接入所需的原子生命周期边界。该记录只批准结构性方向，不把未接线的 scheduler core 或测试夹具描述为产品能力。

## Current-source 结论

- `ToolScheduler` 已拥有 canonical `ToolResourceSet`、单资源/集合原子租约、FIFO set queue、有界拒绝、withdraw/release-all 与 `ToolScheduleReport` 生命周期事件。
- `ToolSchedulerService` 是 `EditorContext` 的唯一线程安全服务 owner，并在锁外把 report 事件发布到 `editor.tool`；service 不应被复制到 scene 或 export 子域。
- `SceneModeStack` 已拥有 mode enter/exit、base/overlay 顺序、ticket retirement 与输入分派。`SceneModeCtx` 只承接 selection、viewport settings、input effect 与 overlay invalidation；把 scheduler 注入每次 mode callback 会扩大插件接口并制造第二个生命周期 owner。
- `SceneViewportController` 由 `EditorState` 构造，当前只接收 settings mutation owner。生产接入应在 controller 的 mode transition 边界调用共享 scheduler service，而不是在每帧 `handle_input` 中重复 acquire。
- export wizard 的长期工作由 `ExportWizardJobController`/job ticket/cancellation owner 管理。租约必须覆盖一次 wizard session 或 active job 的生命周期，并由 completion/cancel/drop 统一释放；不能只包裹单个 pipeline stage。

## 批准的接线顺序

1. 为 scene controller 构造链注入共享 `ToolSchedulerService`，建立稳定的 scene tool identity 与 `[ViewportInput, SceneModeSlot]` 资源集合。
2. scene base/overlay transition 在调用 mode enter 前取得 set lease；排队或拒绝时不修改 mode stack。成功替换后释放旧 identity；失败回滚必须保留旧 lease 和旧 stack。
3. scene shutdown 与 ticket retirement 使用同一 release owner，禁止只删除 mode 对象却遗留 scheduler holder/queue。
4. export wizard 在 session/job start 前申请 `[ModalSurface]` 或最终计划确认的集合；`Queued/Denied` 作为 typed UI state，不创建 job。
5. completion、cancel、view close、plugin revoke 和 controller drop 汇聚到一个 idempotent release-all owner，再补 scene/export 竞争集成测试与 lifecycle message 断言。

## 拒绝的方案

- 不把 scheduler 放入 `SceneModeCtx`，避免插件 mode callback 获得全局调度权限及每帧 acquire。
- 不按当前 active mode id 临时解析并反复申请；tool identity 必须在 lifecycle owner 中稳定保存。
- 不在 export pipeline stage 内 acquire/release，避免阶段间释放 modal 权限后由其它工具插队。
- 不以 `holder()` 读后写替代 `acquire_set()`，避免 check-then-act 竞争和部分资源持有。

## 本切片实现

内建 `editor.tool` topic 增加与 document/transaction/log/i18n 相同的 typed constructor；`ToolSchedulerService` 删除 `EditorTopic::parse(...).expect(...)` 生产崩溃路径。该修改符合 `engine-code-review-findings-2026-06.md` 的健壮性要求，但不是 scheduler consumer 接入完成。

同时，`SceneViewportController` 现在持有共享 `ToolSchedulerService`，并在第一次需要独占场景工具的模式 transition 前，以固定
`editor.scene.viewport` 身份一次性申请 `[ViewportInput, SceneModeSlot]` 原子租约。所有基础模式
切换与自定义 overlay 加入都在修改 `SceneModeStack` 前确认租约；队列或拒绝直接返回 typed
controller error，既不创建/安装新模式，也不改变现有栈。模式 factory 失败或 stack replacement
回滚时，只有本次新取得的租约会释放，避免失败路径遗留 holder。`shutdown_scene_modes` 与
controller `Drop` 汇聚到幂等 `release_all`，允许 scheduler 晋升等待的完整 set。

EditorState 构造链把 `EditorContext.tools()` 传入 viewport controller；测试构造器仍创建独立
内存 scheduler，因此不会隐式共享测试状态。此接线不把 scheduler 放进 `SceneModeCtx`，也不在
每帧输入回调中 acquire。

导出向导的生产 retained-host 构造链同样传入 `EditorContext.tools()`。`ExportWizardPanelSession`
只在实际提交 job 前申请固定 `editor.export.wizard.<job>` 的 `[ViewportInput, ModalSurface]` set lease，job submit
失败、终态 poll、同步 finish 和 session drop 都调用同一幂等释放路径；队列/拒绝转为 typed
`ExportWizardPanelSessionError`，不创建 job。测试构造器保留独立内存 scheduler，避免隐式共享。

## 验证门

- 本切片需通过 scoped rustfmt、diff check 与 topic/scheduler focused Cargo 后，才可标记 source validated。
- scene/export 生产接入完成前，Editor08 M3.2 仍为 open。
- 该工作不是性能优化；没有 profiler、p50/p95、功耗或规模结论。若后续优化 transition/input 路径，必须先采集 current-source profile 并写入对应 optimize 报告。

## 产出记录与时间

| 日期 | 里程碑/切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-30 | M3.2 scene/export scheduler consumer topology | `architecture-review-complete / implementation-pending` | 完整复核 scheduler core/service、scene mode stack/context/controller 构造链与 export wizard job lifecycle；批准 transition/session 级 set-lease 接线，拒绝 callback 注入、每帧 acquire、stage 级租约和 read-then-write。 |
| 2026-08-30 | Built-in tool topic error-path cleanup | `source-complete / static-verified / managed-validation-pending` | 新增 `EditorTopic::tool()` typed constructor，`ToolSchedulerService` 不再通过生产 `expect` 解析内建常量；scoped rustfmt 与 `git diff --check` 通过。Editor focused managed request 已提交但受 Runtime22 FIFO 占用，未声明 Cargo、性能、milestone、commit 或企微完成。 |
| 2026-08-30 | Scene viewport scheduler consumer lifecycle | `source-complete / static-verified / managed-validation-pending` | `EditorState -> SceneViewportController` 接入共享 scheduler；控制器持有固定 scene tool 的 `[ViewportInput, SceneModeSlot]` set lease，模式切换/overlay 安装先做原子 admission，失败保持原 stack，shutdown/drop 使用 `release_all`。新增 controller 测试覆盖 set holder/release 与 denied admission 不改 mode。rustfmt、`git diff --check` 通过；Cargo、独立 review、milestone、commit、企微仍未声明。 |
| 2026-08-30 | Export wizard modal scheduler consumer lifecycle | `source-complete / static-verified / managed-validation-blocked` | retained host 将共享 `ToolSchedulerService` 注入按 profile 的 `ExportWizardPanelSession`；job start 申请 `[ViewportInput, ModalSurface]` set lease，submit failure、poll/finish terminal、session drop 统一 release；queue/deny 返回 typed session error 且不创建 job。新增 modal lease 生命周期回归。scoped rustfmt/diff-check 通过；受管验证因协调器未登记 D 盘清理保留项阻断，未声明 Cargo、review、milestone、commit、企微。 |
| 2026-08-30 | Export wizard regenerated-plan tool identity hardening | `source-complete / static-verified / managed-validation-blocked` | `replace_plan` 现在与 job id 一起重建 `editor.export.wizard.<job>` scheduler identity，避免 inactive session 重新生成计划后沿用旧 job 的 lease owner；新增重生成后启动并确认新 holder 的回归测试。scoped rustfmt 与 `git diff --check` 通过；受管验证仍受未登记 D 盘清理保留项阻断。 |
