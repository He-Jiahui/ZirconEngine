---
handoff_kind: failure
status: open
created_at: 2026-07-12
summary_slug: command-eval-scene-mode-selection-projection
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_editor/editor/05
related_code:
  - zircon_editor/src/core/commands/when.rs
  - zircon_editor/src/ui/host/command_eval_projection.rs
  - zircon_editor/src/scene/viewport
tests:
  - cargo test -p zircon_editor --lib --locked command_eval
  - cargo test -p zircon_editor --lib --locked scene_mode
  - cargo test -p zircon_editor --lib --locked selection
---

# Editor 05：CommandEvalCtx 缺少 SceneModeId/SelectionModel 权威投影

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：Plan08 M1.2 类型化 when 谓词与统一求值环境
- 修复责任计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 交接原因：Plan08 只定义 `SceneModeActive/SelectionNonEmpty` 的中性消费合同；模式栈和选中集权威模型属于 Editor05。

## 失败现象与复现证据

`CommandEvalCtx` 已有 `scene_mode: Option<SceneModeId>` 与 `selection_count`，但 `command_eval_ctx_from_chrome` 从未调用 `with_scene_mode`，所以任何 `WhenClause::SceneModeActive` 在生产交互快照中恒为 false。当前 selection 仅用 `chrome.inspector.is_some()` 投影为 0/1，可暂时支撑布尔谓词，却不能表达本计划 `SelectionModel` 的多选、主选中与双域权威。

静态复现：扫描生产 `with_scene_mode(` 调用为零；`with_selection_count` 的生产调用只有 inspector presence 映射。当前共享里程碑禁止本切片运行 Cargo，因此不声明行为门通过。

## 最低共享层根因

Editor05 尚未把 `SceneViewportTool`/硬编码 handle 切换收敛成权威 `SceneModeStack`，也尚未落地统一 `SelectionModel`。没有 owner 快照时，命令层不能从视图可见性或 inspector 存在性伪造具体 `SceneModeId`。

## 架构修复验收

- `SceneModeStack` 暴露当前活跃 `SceneModeId`，模式 enter/exit/push/pop 后原子更新共享 `CommandEvalCtx`。
- `SelectionModel` 成为 viewport/hierarchy/inspector 同源权威；`selection_count` 来自模型实际集合长度，Edit/PIE 域切换不串值。
- `SceneModeActive` 与 `SelectionNonEmpty` 在菜单、命令面板、UI binding 三入口一致。
- 增加模式切换、多选清空、Edit/PIE 域切换的 command eval 回归并回跑 Plan08 测试。

## 禁止临时方案

- 禁止从当前工具按钮、gizmo 可见性、view id 或 inspector 是否显示猜测具体 `SceneModeId`。
- 禁止保留 `SceneViewportTool` 与新模式栈双权威，禁止为菜单建立独立 selection/mode gate。
- 禁止把 inspector 0/1 映射宣称为最终 `SelectionModel` 接线。

## 修复结果与回传

Open state: `待修复`; no pass is claimed. 完成后由 Editor05 在本文件记录验证证据，并向 `../08-tool-orchestration-and-commands.md` 回传可关闭结论。
