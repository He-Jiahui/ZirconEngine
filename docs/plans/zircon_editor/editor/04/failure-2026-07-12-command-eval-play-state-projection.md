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
  - zircon_editor/src/core/play/bridge.rs
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

Open state: `待修复`; no pass is claimed. 完成后由 Editor04 在本文件记录验证证据，并向 `../08-tool-orchestration-and-commands.md` 回传可关闭结论。
