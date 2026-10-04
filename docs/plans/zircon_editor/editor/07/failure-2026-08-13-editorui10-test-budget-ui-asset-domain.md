---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-ui-asset-domain
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor/07
related_code:
  - zircon_editor/src/tests/editing/ui_asset/inspector
  - zircon_editor/src/tests/editing/ui_asset/tree_and_undo
  - zircon_editor/src/tests/editing/ui_asset_palette_drop
  - zircon_editor/src/tests/editing/ui_asset_preview_binding_authoring
  - zircon_editor/src/tests/editing/ui_asset_replay
  - zircon_editor/src/tests/editing/ui_asset_theme_authoring
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib editing::ui_asset --locked
  - rustfmt --edition 2024 --check zircon_editor/src/tests/editing/ui_asset_replay/mod.rs
  - rustfmt --edition 2024 --check zircon_editor/src/tests/editing/ui_asset_palette_drop/mod.rs zircon_editor/src/tests/editing/ui_asset_preview_binding_authoring/mod.rs zircon_editor/src/tests/editing/ui_asset_theme_authoring/mod.rs
  - python -m unittest tools.tests.test_zui_static_suffix_convergence.ZuiStaticSuffixConvergenceTests.test_editor_ui_asset_editing_tests_use_zui_suffix -v
---

# Editor07: UI asset domain test owners exceed the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 交接原因：这些覆盖 UI asset inspector、tree/undo、palette drop、preview binding、replay 与 theme authoring；
  它们属于 Editor07 domain editor 语义，不属于 EditorUI10 的通用审计。asset reference/promotion 已另交 Editor10。

## 失败现象与复现证据

当前结构审计以 800 行为界、0 个 test-budget exemption，报告以下 6 个 Editor07 owner：

- `tests/editing/ui_asset/inspector.rs`：997 行。
- `tests/editing/ui_asset/tree_and_undo.rs`：829 行。
- `tests/editing/ui_asset_palette_drop.rs`：1029 行。
- `tests/editing/ui_asset_preview_binding_authoring.rs`：1798 行。
- `tests/editing/ui_asset_replay.rs`：1910 行。
- `tests/editing/ui_asset_theme_authoring.rs`：1381 行。

它们混合多个 UI asset 行为，导致通用 zero-tolerance structure gate 保持 RED。

## 最低共享层根因

domain editor 的新增回归集中附加到 flat 测试 owner，尚未将 inspector、tree/undo、palette、preview、replay
与 theme 的 fixture/operation/test suite 按行为域拆成 folder-backed 子模块。

## 架构修复验收

- 每个列举 owner 必须依行为迁移到 folder-backed 目录并留薄 `mod.rs`，每个 Rust 测试文件不超过 800 行。
- 保留 UI asset 文档、mock preview host、undo/replay 和 authoring 的原有测试语义；共享 fixture 只可进入其唯一
  domain helper，不得复制测试树。
- 不得保留旧 flat 文件、`#[path]` shim 或 compatibility re-export。
- 重跑本 domain 的受管回归后，结构审计不得再报告这 6 条；全局 48 项归零后才允许
  `structure_convention` gate GREEN。

## 禁止临时方案

- 不得提高预算、添加目录/glob/blanket exemption、删除测试或弱化 EditorUI10 zero-tolerance gate。
- 不得把 UI asset reference/promotion 责任从 Editor10 吸回，或把 UI component/MUI 覆盖混入本切片。

## 修复结果与回传

Open state: `源码修复完成，受管动态验收待完成`。稳定 Session 为 `failure-roll-01a07160-editor07`；原始历史路径和行数保留在上方复现证据中。

- 2026-09-06 核对现行源码：inspector、tree/undo 已迁入规范目录，最大子文件分别为 719、605 行，本轮复用现有源码。palette drop、preview binding、theme authoring 已有行为子模块，但根文件仍包含夹具与辅助函数，本轮继续完成薄根验收。
- 这三个根模块现为 5、5、4 行声明；21 份 TOML 夹具分别归入各自唯一 `fixtures.rs`，palette 的 3 个辅助函数归入 `support.rs`，子测试显式导入使用的符号。最大子文件分别为 358、562、478 行；现有 58 个测试体经同版 rustfmt 归一后逐项一致，夹具值逐字节一致，辅助函数体一致。精确 17 路径经协调器所有权转移 `a03093e137d14de6b5bbb319c9bc718b` 纳入当前 Session。
- 剩余 1,915 行 replay owner 已按跨文件副作用、样式命令、提升回放、命令日志与脱敏产物、工作区恢复、文档命令拆分；根 `mod.rs` 为 7 行声明，公共 TOML 夹具唯一归属 `fixtures.rs`，最大子文件为 493 行。
- 当前全部 21 个测试体在格式化归一后逐项一致，9 份 TOML 夹具逐字节一致；原 flat 文件已删除，没有 `#[path]`、兼容 re-export 或重复测试挂载。
- replay 与另外三个域的递归 Rust 2024 format 检查通过；最后一次聚焦 `.zui` 后缀检查实际执行 1 项并通过（0.032 秒）。共享 Python 守卫与产品入口文档分别仍由原有 UI06/UI05 Session 维护，本轮仅同步规范路径并要求测试目录非空，不混入 Editor07 validation overlay。
- 按用户指令跳过 `zr_vm`。原始 `editing::ui_asset` 受管 Rust 回归、上行结构验收、独立审查、fixed return 和 coordinator closeout 尚未完成；源码/静态检查不代替动态验收，也未产生提交 SHA 或企微发送结果。
- 源码快照 `2822` 固定 13 个 Editor07 路径（含旧 flat 删除）。受管格式请求 `failure-roll-01a07160-editor07-replay-format-20260906-r1` 已取得票据 `07f7b678011442809abdc65fe5a02070`，准入无 blocker，回执为 queued；该票据只验格式与模块解析。最新结构审计为 13 个超预算测试、3 个生产文件、2 个重复测试树、0 豁免，本记录原始六条已不在超预算列表中。
- 后续任务边界收取 replay 格式票据：`passed`，实际执行、退出码 0，job `5a316a8d4c0c41dfbce685f9b8843bdc`；9 个验证输入仍匹配，manifest hash 为 `31fd1876f7e750dfc3a17710353f5016b0254adafb7d66594ecb04dcebfb4ef6`。
- 扩展源码快照 `2824` 固定当前 30 个 Editor07 路径。其余三个根模块的精确 17 输入格式请求 `failure-roll-01a07160-editor07-domain-roots-format-20260906-r1` 取得票据 `070a36a7392846e5917aa7888e24a426`，回执为 `queued`、无 blocker；不重复 replay 请求，也不将该队列回执计作执行通过。
- 后续任务边界收取该三域票据：`passed`，实际执行、退出码 0，job `757f2e848b884ba6992eeb862dfe8c87`。17 个输入哈希全部匹配当前源码，manifest hash 为 `4edf318066ca6e0756325a4f07ee2011009bc70af37549f43702af666ab36613`；该结果仍仅证明格式与模块解析。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 UI asset domain test-budget handoff | `open` | 从准确的 48/0 审计中隔离 6 个 UI asset domain owner，最大 replay owner 为 1910 行。 | 取得源码 exact lease 后按行为 folder-backed 拆分，受管 domain 测试和结构审计均需复验。 |
