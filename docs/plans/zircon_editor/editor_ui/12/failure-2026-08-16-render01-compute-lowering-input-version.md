---
handoff_kind: failure
status: open
created_at: 2026-08-16
summary_slug: render01-compute-lowering-input-version
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/zircon_runtime/render/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/feature/compute_pass_descriptor/lowering.rs
tests:
  - ".\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -RepoRoot E:\\Git\\ZirconEngine -Package zircon_app -Bin zircon_editor -NoDefaultFeatures -Features target-editor-host -SkipTest"
---

# Render 01: compute lowering misses the resource input-version contract

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M6 current-source Editor build and native WGPU visual acceptance
- 修复责任计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 交接原因：失败来自 Render01 新增的 render-feature resource version contract，不属于 UI12 的 UI/render primitive ownership。

## 失败现象与复现证据

受管 Editor 产品构建在前一组 shared-source 诊断归零后，于 `compute_pass_descriptor/lowering.rs:147` 报 E0063：`RenderFeatureResourceDescriptor` 新增 `input_version`，旧 compute lowering 的通用 resource 构造器没有初始化该字段。

## 最低共享层根因

Compute lowering 只接收资源名、类型、访问与写入模式，不携带显式 producer pass identity。与其他无 producer 身份的兼容构造路径一致，其 version binding 应为 `None`，由后续 pipeline lowering/validation 处理；不能伪造 producer version。

## 架构修复验收

- 通用 compute resource descriptor 显式设置 `input_version: None`。
- 不改变 compute binding、external binding 或 write-mode 行为。
- scoped rustfmt 与 diff check 通过。
- 当前源码受管 Editor 产品构建中该 E0063 归零。

## 修复结果与回传

Open state: `UI12 在无有效文件租约后补齐兼容构造器字段；待当前源码 Editor 构建验证后回传 Render01`。
