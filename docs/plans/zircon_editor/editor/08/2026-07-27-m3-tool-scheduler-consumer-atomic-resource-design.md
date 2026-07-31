# Editor08 M3.2 ToolScheduler consumers: atomic resource design

## 状态

`m3_2a_source_prepared_static_green_dependency_not_accepted`。本记录冻结 ToolScheduler 进入
Plan05 scene mode 与 Plan15 export wizard 前必须满足的资源语义。Editor08 atomic primitive 已完成
源码与静态检查，但在 M3 union 的 immutable Cargo copy 终态有效前不得被消费或验收；它不是 Cargo、
产品行为、failure fixed 或提交证据。

## 实读入口

- `SceneViewportController::new` 建立默认 `ViewportToolSceneMode`，
  `SceneViewportController::activate_tool` 仅通过 `SceneModeStack::replace_base` 切换基础模式。
  一个 viewport mode 的生命周期不得因模式切换而释放并重新排队它的 scheduler 身份。
- retained host 的 `dispatch_build_export_surface_action` 解析 `DesktopExportWizard/Start` 后，
  立即进入 `DesktopExportWizardSessions::dispatch_profile_action`；当前路径未向
  `EditorContext::tools()` 申请资源，也没有在未获得输入时阻止 `ProcessCommandRunner` 启动。
- 当前 `ToolScheduler::acquire` 只针对一个 `ExclusiveResource`。连续申请
  `ModalSurface` 和 `ViewportInput` 会产生部分成功，且两个独立 FIFO 队列不能给出原子顺序。

## 必须新增的共享契约

`core/tools` 增加不可变、非空且规范排序的 `ToolResourceSet`，由唯一的
`ToolScheduler::acquire_set(tool, resources)` 和 `release_set(tool, resources)` 操作。它不是调用方
循环调用单资源 `acquire` 的便利封装。

- 同一个请求要么一次性持有集合中全部资源，要么持有零个资源并以整个集合进入 scheduler 的
  request FIFO；不得让等待中的 export 预先占有 `ModalSurface`。
- scheduler 为集合请求维护单一严格 FIFO。队头请求仅在其全部资源空闲时激活；后续请求不得越过
  队头，即使它只依赖当前空闲的子集。这样避免多资源死锁和不可复现的跨队列插队。
- 释放、撤销或释放全部资源后，scheduler 在同一锁持有期内运行队头 promotion；产生的完整事件序列
  在解锁后以现有 lossless `editor.tool` topic 依次发布。
- typed lifecycle 事件需能表达整个集合（请求、激活、撤销、拒绝），不能把一个集合请求伪装成两个
  无关联的单资源事件。既有单资源 `acquire/release` 保留其本身语义，但不作为新 consumer 的实现路径。

## Consumer 协议

### Plan05 scene mode

- 使用固定 `ToolId("scene.viewport.mode")`。controller 创建时原子取得
  `{ViewportInput, SceneModeSlot}`；切换 Move/Rotate/Scale 等 base mode 仅替换内部
  `SceneModeStack`，不释放这一逻辑工具。
- export 对 `ViewportInput` 排队后，scene controller 在 UI 线程消费已发布的集合请求，执行一次
  `suspend_viewport_input_for_modal`：只释放 `ViewportInput`，继续持有 `SceneModeSlot`，不退出或
  重建当前 base mode。release 使队头 export 原子取得其完整集合。
- export 的 terminal release 后，scene controller 以相同 ToolId 重新申请 `ViewportInput`；在其获得前
  viewport 不分发编辑输入。不得通过直接改写 `SceneModeStack` 或忽略 queue 位置绕开 scheduler。

### Plan15 export wizard

- 使用 profile-stable `ToolId("workbench.build_export.<profile>")`，启动时原子申请
  `{ModalSurface, ViewportInput}`。如果结果为 `Queued`，只投影等待状态，绝不调用
  `regenerate_profile_plan` 后的 `handle_start_request_with_runner`，也不创建进程。
- 只有收到该 tool 的全集合 `Activated` 事件后才允许 start runner。Cancel、启动失败和所有 terminal
  状态都调用 `release_all`；若仍在队列则只 withdraw，不产生 runner。
- retained host 必须经拥有的 `EditorContext::tools()` 或一个窄的 host bridge 接入服务；不得复制
  `ToolScheduler`，不得在 UI 层维护第二个 mutex/队列。

## 最小验收矩阵

1. scene 启动后拥有 `{ViewportInput, SceneModeSlot}`；连续切换 base mode 不产生 release/queue 事件。
2. 活动 scene 下 start export 先产生 export 集合 `Queued`，无 runner；scene 让渡 input 后 export 才以
   一个原子激活取得两个资源并启动。
3. export 取消、runner error 与正常终态都释放两个资源；scene 仅重新取得 `ViewportInput`，仍保留
   `SceneModeSlot` 和原 base mode。
4. 队头集合缺任一资源时后续集合不得越过；撤销队头后下一个合格请求可激活；无 partial holder、无
   duplicate event、无进程在等待态创建。
5. bus 断言精确覆盖 queued -> scene input deactivated -> export set activated -> export set deactivated
   -> scene input activated 的顺序，并保留当前单资源 scheduler 回归。

## 实施切片与所有权

| 切片 | 责任计划 | 预期路径边界 | 前置条件 |
| --- | --- | --- | --- |
| M3.2a atomic lease-set primitive + typed event | Editor08 | `core/tools/*`、`core/context/tool_scheduler.rs`、`core/editor_message/*` 及单元测试 | M3 union 的 immutable Cargo copy 终态有效后才可消费或验收 |
| M3.2b scene mode consumer | Editor05 | `scene/viewport/controller/*`、scene mode tests、EditorState context wiring | M3.2a accepted API；Plan05 exact leases |
| M3.2c export wizard consumer | Editor15 | `ui/retained_host/app/build_export_wizard_session/*`、host bridge、retained tests | M3.2a accepted API；Plan15 exact leases |

禁止跨切片修改对方路径；当前 Editor08 union 仅冻结 scheduler core 及 M3.1 代码，不能据此写入
Plan05/Plan15 consumer 实现。

## 禁止临时方案

- 不得将集合请求实现为两个连续的单资源 acquire，或在第二次失败后补偿 release。
- 不得让 export 在 queued 时创建 `ProcessCommandRunner`、子进程、临时 modal 或本地 queue。
- 不得在模式切换期间 release/reacquire `SceneModeSlot`，或通过重建 controller 取得资源。
- 不得恢复旧工具管理器、兼容别名或 host-local scheduler。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与证据 |
| --- | --- | --- | --- |
| 2026-07-27 | M3.2 consumer architecture | architecture_defined_source_unimplemented | 实读 scene controller、`SceneModeStack`、retained export Start 路由与现有单资源 scheduler；明确 atomic lease-set、scene input 让渡/恢复、export runner 启动门槛、Plan05/Plan15 所有权与五项验收矩阵。未运行 Cargo，未声明代码完成。 |
| 2026-07-27 | M3.2a Editor08 atomic lease-set primitive | source_prepared_static_green_dependency_not_accepted | 在 `core/tools` 实现不可变、非空、去重且规范排序的 `ToolResourceSet`，全局集合 FIFO、active-set 账本、原子 acquire/release/withdraw/release-all 与完整集合生命周期事件；服务层在解锁后逐条发布 lossless `editor.tool` 事件，retention 计量覆盖集合事件。新增单元测试覆盖集合规范化、原子激活、队头不可越过、撤销提升、单资源 API 不能部分释放集合，以及 bus 事件顺序。`rustfmt --check`、scoped `git diff --check` 与既有 ToolScheduler 静态契约测试 6/6 已通过；受管 Cargo 验证仍待 Coordinator01 admission repair，故 Plan05/Plan15 不得消费该 API，且其路径未修改。 |
