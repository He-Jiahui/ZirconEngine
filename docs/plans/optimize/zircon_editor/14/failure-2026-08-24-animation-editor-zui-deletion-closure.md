---
handoff_kind: failure
status: open
created_at: 2026-08-24
summary_slug: animation-editor-zui-deletion-closure
origin_plan: docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
fixing_plan: docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
origin_child_dir: docs/plans/optimize/zircon_editor/09
fixing_child_dir: docs/plans/optimize/zircon_editor/14
plan_link_mode: child_record_only
failure_scope: cross_plan
related_code:
  - zircon_editor/assets/ui/editor/animation_editor.zui
  - zircon_editor/src/tests/ui/animation_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/ui/layouts/views/animation_editor.rs
  - zircon_editor/tests/integration_contracts/workbench_animation_editor_shell.rs
tests:
  - cargo +1.94.1 test -p zircon_editor --locked --lib --release core::jobs -- --include-ignored --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --locked --lib --release editor09_ -- --include-ignored --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --locked --lib --release editor10_ -- --include-ignored --nocapture --test-threads=1
---

# Editor 14: animation_editor.zui 删除迁移未闭合

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md`
- 来源执行切片：Editor 09 后台作业热路径批量验证的 validation-copy closure planning gate；同一缺口也会阻断 Editor 10 及其他 `zircon_editor --lib` 批次。
- 修复责任计划：`docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md`
- 交接原因：缺失资源及其全部引用都属于动画编辑器布局、模板与产品入口迁移边界，最低共享责任不在 Editor 09 作业系统或 Editor 10 通知系统。

## 失败现象与复现证据

协调器票据 `d79991c986e84551837f2ab8642f3d06` 与 `0b8cc9f7a8d949ada04e863c84b26d6e` 均在 Cargo 启动前失败，阶段为 `closure_planning`，错误码为 `validation_copy_compile_time_resource_missing`。缺失路径是 `zircon_editor/assets/ui/editor/animation_editor.zui`，发现来源是 `zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs` 的编译期 `include_str!`。

当前工作树把该受版本控制资源标记为删除，但仍有以下五处编译期或产品引用：

- `zircon_editor/src/tests/ui/animation_editor/bootstrap_assets.rs`
- `zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs`
- `zircon_editor/src/tests/ui/boundary/template_assets.rs`
- `zircon_editor/src/ui/layouts/views/animation_editor.rs`
- `zircon_editor/tests/integration_contracts/workbench_animation_editor_shell.rs`

因此原始 Editor 09 和相关 Editor 10 命令尚未进入 Cargo 编译或测试执行，不能将这两张票据表述为测试失败或测试通过。

## 最低共享层根因

Editor 14 正在进行动画工作台 ZUI 资产迁移，但删除旧 `animation_editor.zui` 的 hard cutover 尚未同时收束 compile-time asset inventory、模板测试、产品布局常量与 integration contract。validation-copy 正确地拒绝构造包含悬空编译期资源的验证快照。

最低已证实边界是“动画编辑器布局资源的 canonical owner 与全部消费者未原子迁移”；未对更深层的目标布局方案作推断。

## 架构修复验收

- 明确并实现唯一 canonical 动画编辑器布局资产 owner；若旧资产确实应删除，全部消费者必须在同一 hard cutover 中迁移到新 owner。
- 仓库中不再存在指向缺失 `animation_editor.zui` 的 `include_str!`、产品路径常量、模板 inventory 或 integration contract。
- 通过协调器重新提交 Editor 14 的 focused asset/template validation，并确认 validation-copy closure planning 与 Cargo 测试均通过。
- 重新运行上列 Editor 09 与 Editor 10 原始批量命令，确认上层性能计划 gate 可以恢复。

## 禁止临时方案

- 不得仅为绕过 validation-copy 而恢复已废弃的旧资产、复制一份同名资产或引入 alias、compatibility shim、silent fallback、test-only bypass、call-site exception。
- 不得削弱 compile-time resource 检查、模板 inventory、integration contract 或计划验收标准来隐藏悬空引用。
- 不得把 Editor 14 的动画资产迁移实现混入 Editor 09/10 性能优化提交。

## 修复结果与回传

Open state: `待修复`; no pass is claimed. Editor 09/10 的独立代码优化可以继续，但所有受影响的 `zircon_editor --lib` 验证在此交接关闭前保持未验收。
