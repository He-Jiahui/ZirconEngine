---
handoff_kind: failure
status: open
created_at: 2026-08-16
summary_slug: runtime-core-dependent-collection-type
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/optimize/zircon_runtime/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/runtime/handle/activation.rs
tests:
  - ".\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -RepoRoot E:\\Git\\ZirconEngine -Package zircon_app -Bin zircon_editor -NoDefaultFeatures -Features target-editor-host -SkipTest"
---

# Runtime Core 01: dependent collection inference blocks the Editor product build

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M6 current-source Editor build and native WGPU visual acceptance
- 修复责任计划：`docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md`
- 交接原因：失败位于 Runtime Core lifecycle owner 的 deactivate-module veto 切片，不属于 UI12 的 UI/render ownership。

## 失败现象与复现证据

同一受管 Editor 产品构建于 `activation.rs:172` 报 E0282。`running_dependents` 从 `module_dependents.iter().filter(...).cloned().collect()` 构造，随后只通过 `is_empty()` 与 `CoreError::ModuleUnloadBlocked` 使用，编译器在 tuple 形成处无法确定 `collect()` 目标类型。

## 最低共享层根因

Runtime lifecycle 改动引入了新的 collection 中间值，却未在首次收集处固定其契约类型。错误不要求改变 veto、shutdown 或 lifecycle 行为。

## 架构修复验收

- 将 `running_dependents` 明确为 `Vec<String>`，匹配 `CoreError::ModuleUnloadBlocked.dependents`。
- scoped rustfmt 与 diff check 通过。
- 当前源码的受管 Editor 产品构建中该 E0282 归零。

## 禁止临时方案

- 不得改变 deactivate-module veto、shutdown 顺序或 `ModuleUnloadBlocked` 的 dependent 语义。
- 不得使用无约束集合、test-only cfg 或在 UI12 层复制 Runtime lifecycle 状态来绕过类型推断。

## 修复结果与回传

Open state: `UI12 在无有效文件租约后仅补齐 collection 类型；待当前源码 Editor 构建验证后回传 Runtime Core 01`。
