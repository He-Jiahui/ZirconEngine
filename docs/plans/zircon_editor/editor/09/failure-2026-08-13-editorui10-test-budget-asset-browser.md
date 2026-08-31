---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-asset-browser
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor/09
related_code:
  - zircon_editor/src/ui/layouts/views/asset_browser/tests
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib asset_browser --locked
---

# Editor09: asset-browser test owner exceeds the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 交接原因：asset-browser browse/query/selection 覆盖属于 Editor09 asset management，不能与 UI asset template 或 reference 管理混合。

## 失败现象与复现证据

准确结构审计在 0 个 test-budget exemption 下报告
`zircon_editor/src/ui/layouts/views/asset_browser/tests.rs` 为 997 行。该 asset-browser 行为 owner 必须拆分，
否则全局 zero-tolerance structure gate 持续 RED。

## 最低共享层根因

asset-browser 的 browse/query/selection/presentation 场景累计到同一个 flat `tests.rs`，没有按 asset-management
行为或唯一 fixture owner 建立 folder-backed 测试边界。

## 架构修复验收

- 按 asset-browser 行为拆为 folder-backed 测试模块，薄 `mod.rs` 挂载，所有 Rust 测试文件不超过 800 行。
- 保留资产浏览、选择、查询和布局 view 的原有断言；共享 fixture 唯一归属。
- 不得保留旧 flat test、`#[path]` compatibility mount、重复 tree 或 test-budget exemption。
- 重审计不再报告该路径；全部 owner 清零后受管 structure gate 才可 GREEN。

## 禁止临时方案

- 不得提高预算、删除 asset-browser 覆盖，或把 UI asset template/asset reference 行为混入此切片。

## 修复结果与回传

Open state: `implementation-complete / upstream-validation-blocked`。已删除 flat
`tests.rs` 并按 virtualization、chrome/regions、list、thumbnail、reference 与 shared
fixture 拆为 folder-backed 测试模块；未删除或弱化任何断言，未引入 `#[path]` 挂载、豁免或第二测试树。

结构审计已不再报告 asset-browser owner，超限测试文件总数从 31 降至 30。聚焦 Cargo
命令在编译 `zr_rhi_wgpu` 时被 14 个既有错误截断，尚未到达 `zircon_editor` 测试，因此本 artifact
仍为 `open`，不得作为 fixed evidence 或回传 origin；待 RHI 依赖恢复后必须重跑声明的聚焦测试并完成
failure return。独立源码复核发现的 virtualization private-helper import 已修复并复核通过，无其余 findings。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-25 | M3 asset-browser test-budget handoff | `implementation-complete / upstream-validation-blocked` | 删除 1065 行的 flat `tests.rs`；新增薄 `tests/mod.rs` 与 6 个行为/fixture 子模块，所有文件均不超过 411 行；结构审计不再列出该 owner（31 -> 30）。主树 20 个测试与既有 reference-list 2 个测试均保留。独立 review 修复并复核了 virtualization private-helper import。`cargo test -p zircon_editor --lib --locked asset_browser` 在 `zr_rhi_wgpu` 的 14 个既有编译错误处截断。 | RHI 修复后重跑聚焦 Cargo 测试，并通过 coordinator failure return 回传 origin。 |
