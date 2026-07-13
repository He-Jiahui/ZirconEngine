---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: workbench-projection-file-budget-regression
origin_plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
fixing_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/06
fixing_child_dir: docs/plans/zircon_editor/editor_layout/15
related_code:
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection/typed_canvas.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/editor_workbench_window_projection.rs
tests:
  - tests::runtime_absorption::structure_convention::test_file_budget::editor_workbench_window_projection::runtime_15_editor_retained_host_workbench_window_projection_tests_are_child_owner
  - tests::runtime_absorption::structure_convention::
resolved_at: 2026-07-13
---


# Editor Layout 15：Workbench projection 文件预算回归

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md`
- 来源执行切片：Frameworks 06 M1 优先结构 / 完整 structure-convention 复验
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 交接原因：最低共享原因位于 Editor Layout 15 当前拥有的 retained Workbench projection；Frameworks 06 只拥有结构预算守卫，不拥有该 projection 的功能拆分。

## 失败现象与复现证据

2026-07-13 05:02:26 生成的 Windows Runtime 当前二进制先通过 M4 child-owner exact 1/1 与 module-doc unique exact 1/1；完整执行 `tests::runtime_absorption::structure_convention::` 后为 1,303 passed / 1 failed / 0 ignored。唯一失败：

```text
tests::runtime_absorption::structure_convention::test_file_budget::editor_workbench_window_projection::runtime_15_editor_retained_host_workbench_window_projection_tests_are_child_owner
```

守卫报告 `zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs should stay below the 1000-line large-file hotspot threshold after the split`。当前文件为 1,002 行；守卫要求严格 `< 1000`。失败不是旧二进制：guard 与 source 都早于该二进制生成时间。

## 最低共享层根因

该 projection 已有 `typed_canvas.rs` 与 `tests.rs` child owner，但父文件仍同时拥有节点投影、selection/control 样式归一、显示文本、路由、颜色解析和 host-value→TOML 转换等多组职责。最新 typed Preview Timeline 接线后父 owner 增长到 1,002 行，重新跨过既有结构预算。

这不是把阈值从 1,000 提高到 1,010 的理由。当前活动会话 `20260710-0523-editor-layout-atomic-ui` 正在修改并验证该文件，Frameworks 会话不得抢写。

## 架构修复验收

- 将一组完整、可命名的 helper 职责抽到 `workbench_window_projection/` 下 directory-backed child；优先考虑已成组的 host-value/TOML conversion 或 selection-style normalization，而不是按行数随意切片。
- 父文件严格少于 1,000 行，并保留 `typed_canvas.rs`、`tests.rs` 与新增 child 的唯一挂载；禁止两个 owner 复制同一逻辑。
- exact file-budget test 1/1。
- 完整 `tests::runtime_absorption::structure_convention::` 恢复 1,304/1,304，或在测试数并发变化时为零失败。
- `cargo fmt -p zircon_editor -- --check` 与 Editor `--lib --no-run` 通过；typed Preview Timeline / timeline strip focused tests保持通过。

## 禁止临时方案

- 禁止提高 `PARENT_FILE_BUDGET`、增加 allowlist、删除守卫或把 `<` 改成 `<=`。
- 禁止用空行/注释压缩、机械合并语句、删除必要文档来仅减少 3 行。
- 禁止新增 facade、re-export、compat module、重复 helper、旧路径 shim 或 cfg 绕过。

## 修复结果与回传

- 根因：typed Preview Timeline 接线后 workbench_window_projection 父 owner 增长到 1002 行，重新跨过既有严格小于 1000 行预算
- 架构修复：将完整 host-value 到 TOML conversion 职责抽到 workbench_window_projection/host_value_toml.rs，父文件降到 961 行，保持 typed_canvas/tests/host_value_toml 三个 directory-backed child 为唯一 owner
- 验证：旧 current Runtime binary 上 file-budget exact 1/1；workbench_window_projection.rs 与 host_value_toml.rs scoped rustfmt check exit 0；完整 structure 向上复验越过该门后只暴露并发 Render limit guard 漂移
- 回传：Editor Layout15 Workbench projection 已按职责拆分并恢复文件预算，failure 原子回迁 Frameworks06 fixed 归档；完整 structure 继续由最新 Runtime guard binary 收口
