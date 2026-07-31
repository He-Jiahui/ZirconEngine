---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: editor-state-context-constructor-hardcut
origin_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
fixing_plan: docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
origin_child_dir: docs/plans/zircon_editor/editor/03
fixing_child_dir: docs/plans/zircon_editor/editor/16
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_construction.rs
tests:
  - python tools/tests/test_editor03_scene_transaction_hardcut_contract.py
  - cargo test -p zircon_app --features target-editor-host --locked
---

# Editor 16：CLI operation 仍调用隐式 EditorState Context 构造入口

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 来源执行者：`editor16-editor-state-context-constructor-hardcut-20260718`
- 来源执行切片：Editor03 M2 单一事务内核的 Editor16 CLI operation consumer
- 修复责任计划：`docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
- 交接原因：生产 CLI operation 入口位于 `zircon_app`，应由 Editor16 注入当前 `EditorManager` 的唯一 Context；Editor03 不恢复无 Context 的生产兼容构造器。

## 失败现象与复现证据

Editor03 M2 将场景 undo/redo 硬切到 `EditorContext::transactions()` 后，生产 `EditorState` 必须持有宿主提供的同一个 `Arc<EditorContext>`。`zircon_app/src/entry/entry_runner/editor.rs::run_editor_operation` 仍调用无 Context 参数的 `EditorState::with_default_selection`；该入口只允许测试夹具使用，若恢复为生产 convenience constructor 会创建第二套 transaction/jobs/command service owner，直接违反 Editor03 单一事务内核与 Editor01 Context 唯一事实源。

该调用点属于 Editor16 的 CLI operation 进程宿主，故失败落在本子计划处理；Editor03 不在 `zircon_app` 内添加反向兼容层。

## 最低共享层根因

CLI operation 的宿主启动顺序先构造 `EditorState`、后解析 `EditorManager`，导致生产状态无法取得 manager 已持有的唯一 `Arc<EditorContext>`，从而继续依赖只应存在于测试侧的隐式 Context 构造路径。

## 架构修复验收

- CLI operation 在构造状态前解析 `EditorManager`，以 `editor_manager.context().clone()` 注入 `EditorState`。
- Editor03 只提供显式 `with_default_selection_with_context(world, viewport_size, Arc<EditorContext>)`；无 Context 参数的入口保持 `#[cfg(test)]`。
- `EditorHostEventController` 与 `EditorState` 共享同一个 manager/context，不得创建第二套 transaction engine。
- 不改变 CLI 参数、operation routing、runtime gateway 或进程退出码语义。

## 禁止临时方案

- 禁止恢复生产可见的 `EditorState::with_default_selection` 或新增等价 convenience facade。
- 禁止为 CLI operation 单独构造 `EditorContext`、transaction engine、jobs 或 command service owner。
- 禁止跳过 `EditorManager` 解析、以全局静态 Context 或兼容别名掩盖 owner 分裂。

## 修复结果与回传

Open state: `源码修复、静态合同门与独立 source 复审完成；target-editor-host Cargo、fixed return 与 owner commit 待完成。联合 immutable gate 被 Coordinator01 非零空输出 failure 截断。`

当前 `run_editor_operation` 已先解析 `EditorManager`，再将 `manager.context().clone()` 注入 `EditorState::with_default_selection_with_context`；未恢复旧构造入口。2026-07-19 当前源复跑静态守卫 13/13 GREEN，r7 相关 Rust rustfmt 通过；scoped diff-check 待 exact47 收口。Coordinator01 immutable validation-copy 的终态证据 failure 尚未 fixed，因此本记录不声明 Cargo 行为门或最终关闭。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-07-18 16:46 +08:00 | `实现完成-静态门通过-Cargo待协调器解阻` | `run_editor_operation` 已先解析 `EditorManager` 并注入唯一 Context；旧生产构造调用零残留，未增加兼容 facade。 | `python tools/tests/test_editor03_scene_transaction_hardcut_contract.py` 4/4；`rustfmt` 与 scoped `git diff --check` 待本切片收口；受管 Cargo 因 Coordinator01 immutable compile-input snapshot failure 暂停，不声明行为门。 |
| 2026-07-19 05:06 +08:00 | `源码完成 / 静态门 5/5 GREEN` | 复跑 `python -m unittest tools.tests.test_editor03_scene_transaction_hardcut_contract -v`，5 项单一 transaction/context hard-cut 守卫全部通过；按 `rustfmt +1.94.1 --edition 2024 --config skip_children=true` 收敛 `editor.rs` 导入排序，随后 `--check` 与 scoped `git diff --check` 通过。 | 待 Coordinator01 failure fixed 后，以不可变 current-source manifest 运行 `cargo test -p zircon_app --features target-editor-host --locked`；再做独立复审、failure return 与受管提交。 |
| 2026-07-19 10:33 +08:00 | `exact31 静态门 GREEN / 受管证据 RED` | successor 补齐 `editor/tests` 4 个拆分模块；snapshot `608`、M2 manifest `4d56cc596a1545c8ade20e56775683c7`、31 项 overlay 零漂移，Python 6/6、rustfmt 与 diff guard 通过。 | 联合 inner run `82329a4e961e4ce3ad3894768f9be29c` exit 101 且 stdout/stderr 为空，copy removed；已将 `validation-copy-nonzero-cargo-output-missing` 交接 Coordinator01。未运行 `target-editor-host`，不返回 fixed。 |
| 2026-07-19 11:09 +08:00 | `exact33 源码 / 静态门 9/9 GREEN` | 原子 successor r3 纳入两个 viewport 入口并完成 Editor03 独立初审整改；CLI manager Context 注入与 4 个拆分测试模块未发生语义变化。 | 等待新 snapshot/manifest、独立复审与 Coordinator01 fixed return；未运行 `target-editor-host`，不返回 fixed。 |
| 2026-07-19 11:57 +08:00 | `exact46 源码 / 静态门 12/12 GREEN` | 原子 successor r6 完成 Editor03 生命周期/性能/结构整改；CLI manager Context 注入与 4 个拆分测试模块未发生语义变化。 | 等待新 snapshot/manifest、独立终审与 Coordinator01 fixed return；未运行 `target-editor-host`，不返回 fixed。 |
| 2026-07-19 12:35 +08:00 | `exact47 源码 / 静态门 13/13 GREEN` | exact46 终审 0/3/1 后，原子 successor r7 完成 exclusive world/play transition、preview interlock 与 finally cleanup；CLI manager Context 注入与 4 个拆分测试模块未发生语义变化。 | 等待 exact47 snapshot/manifest、独立复审与 Coordinator01 fixed return；未运行 `target-editor-host`，不返回 fixed。 |
| 2026-07-19 12:49 +08:00 | `independent source review 0/0/0 / Cargo pending` | r7 增量复审确认上轮 0/3/1 全部关闭；CLI Context 注入及其与 exclusive transition 的类型/借用边界无 source finding。 | snapshot `623` / exact47 manifest `8bb61cd3...`；未运行 `target-editor-host`，不返回 fixed。 |
