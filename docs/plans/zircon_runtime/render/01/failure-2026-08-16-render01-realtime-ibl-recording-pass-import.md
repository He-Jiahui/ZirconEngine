---
handoff_kind: failure
status: open
created_at: 2026-08-16
summary_slug: render01-realtime-ibl-recording-pass-import
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/zircon_runtime/render/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder.rs
tests:
  - ".\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -RepoRoot E:\\Git\\ZirconEngine -Package zircon_app -Bin zircon_editor -NoDefaultFeatures -Features target-editor-host -SkipTest"
---

# Render 01: realtime IBL recorder pass import blocks the Editor product build

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M6 current-source Editor build and native WGPU visual acceptance
- 修复责任计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 交接原因：失败位于 Render01 正在修改的 realtime IBL recording-pass 切片，不属于 UI12 的 device-pixel AA ownership。

## 失败现象与复现证据

受管 `zircon_app --bin zircon_editor --no-default-features --features target-editor-host` 构建在 UI12 已有 Editor 诊断归零后，于 `realtime_ibl_wgpu_recorder.rs:57` 报 E0425：新参数 `recording_passes: &[RealtimeIblGraphPass]` 已加入，但同模块导入仅包含 `RealtimeIblGraphPassKind` 与 `RealtimeIblGraphPlan`。

## 最低共享层根因

Render01 的 recording-pass 参数切换遗漏了同一 owner 模块中的类型导入。无需改动 graph 计划、记录顺序或 UI 层。

## 架构修复验收

- 从 `realtime_ibl_graph_plan` 显式导入 `RealtimeIblGraphPass`。
- scoped rustfmt 与 diff check 通过。
- 当前源码的受管 Editor 产品构建中该 E0425 归零。

## 禁止临时方案

- 不得改变 realtime IBL graph 计划、recording-pass 顺序或 UI feature graph 来绕过缺失导入。
- 不得使用通配导入、test-only cfg 或在 UI12 层复制 Render01 类型。

## 修复结果与回传

Open state: `UI12 在无有效文件租约后仅补齐缺失导入；待当前源码 Editor 构建验证后回传 Render01`。
