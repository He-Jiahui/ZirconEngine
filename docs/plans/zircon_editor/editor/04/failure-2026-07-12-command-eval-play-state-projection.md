---
handoff_kind: failure
status: open
created_at: 2026-07-12
summary_slug: command-eval-play-state-projection
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_editor/editor/04-pie-and-simulation.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_editor/editor/04
related_code:
  - zircon_editor/src/core/commands/when.rs
  - zircon_editor/src/ui/host/command_eval_projection.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/play/mode.rs
tests:
  - cargo test -p zircon_editor --lib --locked command_eval
  - cargo test -p zircon_editor --lib --locked play_mode
---

# Editor 04：CommandEvalCtx 缺少 Building/PlaySessionController 权威投影

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：Plan08 M1.2 类型化 when 谓词与统一求值环境
- 修复责任计划：`docs/plans/zircon_editor/editor/04-pie-and-simulation.md`
- 交接原因：Plan08 负责谓词类型与消费一致性；`Edit/Building/Playing` 状态机和 `PlaySessionController` 的生命周期权威属于 Editor04，不能由命令层制造或猜测。

## 失败现象与复现证据

`WhenClause::PlayMode` 与 `CommandEvalCtx.play_state` 已存在，但 `ui/host/command_eval_projection.rs` 当前只把 `EditorSessionMode::Playing` 映射为 `Playing`，把 `Welcome/Project` 映射为 `Edit`。Chrome 快照没有 `Building` 变体，因此真实构建阶段永远无法使命令谓词命中 `PlayMode(Building)`；现有 Playing 值也来自 UI session mode，而非本计划尚未落地的权威 `PlaySessionController`。

静态复现：读取 `command_eval_ctx_from_chrome` 的 `match chrome.session_mode`，可见没有任何 `PlayStateKind::Building` 写入路径。当前共享里程碑禁止本切片运行 Cargo，因此不声明行为门通过。

## 最低共享层根因

Editor04 尚未落地三态 `PlaySessionController` 及其稳定快照/消息出口，Plan08 的中性 `CommandEvalCtx` 因而只能诚实保留不完整投影。最低修复层是 Play session 权威状态，不是菜单、命令面板或单条命令特判。

## 架构修复验收

- `PlaySessionController` 成为 `Edit/Building/Playing` 唯一权威，并提供宿主可订阅或读取的类型化快照。
- `CommandEvalCtx.play_state` 从该权威快照更新；进入构建、构建失败回 Edit、构建成功进 Playing、退出 Playing 回 Edit 均触发一致更新。
- 菜单、命令面板、UI binding 使用同一共享 snapshot；不得各自读取不同 UI flag。
- 增加三态转换与 `WhenClause::PlayMode` 投影回归，并回跑 Plan08 command eval 测试。

## 禁止临时方案

- 禁止把 Building 折叠为 Edit 或 Playing，禁止用任务文案、进度条、按钮可见性猜测状态。
- 禁止在菜单/命令面板单独特判 play 状态，禁止建立第二份命令求值 context。
- 禁止恢复旧 play-mode backend 兼容别名或把 UI session mode 宣称为最终权威。

## 修复结果与回传

Open state: `源码修复已落地，待受管Cargo与独立复审后回传fixed`。`PlaySessionController` 已成为 host-owned 三态权威，`command_eval_ctx_from_chrome` 现在显式接收 controller `PlayModeKind` 并覆盖 `PlayStateKind::Building`；生产投影不再匹配 `chrome.session_mode`。菜单、startup 与测试已硬切 `PluginBridgeActivation`，旧 backend/bridge 兼容名为0。Python契约 RED（3 fail+1 error）→GREEN 4/4；Rust迁移矩阵已落盘但当前未执行Cargo，故不改 `status: open`、不向 Editor08 宣称 fixed。

## 产出记录与时间

| 日期 | 事项 | 状态 | 证据与后续 |
| --- | --- | --- | --- |
| 2026-07-18 | PlaySessionController → CommandEval 三态投影 | 源码完成 / open待验证 | controller `Edit/Building/Playing`、build结果迁移、menu同源API与CommandEval Building投影已完成；Python静态契约4/4，旧token 0。待 managed focused/broad Cargo、独立review及 lifecycle fixed-return。 |
| 2026-07-27 | CommandEval 测试输入硬切 | resolving_failure | 已复核 production projection、reflection 与 remote/CLI 路径均读取 `PlaySessionController::mode()`，菜单 effect 与 backend terminal 路径均会刷新 command-eval snapshot；旧 backend/bridge token 搜索为 0。测试 helper 已改为显式接收 `PlayModeKind`，不再由 `chrome.session_mode` 推断；`python tools/tests/test_editor04_play_session_controller_contract.py`（4/4）、`rustfmt --check` 与 `git diff --check` 通过。仍待受管 Cargo、独立复审与 lifecycle fixed-return。 |
