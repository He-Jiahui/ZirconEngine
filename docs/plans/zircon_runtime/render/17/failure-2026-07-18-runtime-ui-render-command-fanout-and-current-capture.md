---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-render-command-fanout-and-current-capture
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/render
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/render_pass.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/ElementBatcher.cpp
tests:
  - repeated guide tick row batch-count scale test
  - owner-specialized overdraw pixel test
  - current-source F4 RenderDoc capture
---

# Runtime UI render command fanout与当前源码GPU capture缺口

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：surface render 32/32
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 联动责任：EditorUI06限制逻辑primitive，EditorUI08发布command ranges。
- 交接原因：GPU batch/vertex/upload/overdraw指标与RenderDoc验收属于Render17。

## 失败现象与复现证据

PERF-MVP-288/291：多command node缺stable identity，tree guides/slider ticks/all rows可制造大量原子command，每条复制style/String；owner与specialized surface可能重复。当前只有RenderDoc v1.44工具历史探测，尚无当前源码F4 capture。

## 最低共享层根因

CPU逻辑command没有compact brush/style handle、instance range与可观测budget，且当前产品graphics backend还未产出可比较capture。

## 架构修复验收

- repeated guides/ticks/rows按compatible brush/clip/z/state合并或实例化，command/vertex/draw增长近visible primitives。
- 记录CPU command bytes、batch merge、vertices/indices、uploads、draw/pass与overdraw。
- 同配置冷帧+稳定帧当前源码RenderDoc capture，标明backend/adapter/resolution/build hash；像素一致。
- owner/specialized重复surface有明确证据后只保留一份authority。

## 禁止临时方案

- 不得用2026-07-17旧capture或advanced scene代替当前F4产品路径。
- 不得通过降低UI内容、隐藏rows或丢失clip/z语义伪造draw下降。

## 修复结果与回传

Open state: `runtime WGPU presenter已落generation compiled batching、稳定资源复用及CPU/GPU submission counters，editor F4 profile已投影command visibility scan/cache hit、image prepare visit/cache hit、render pass、draw、vertex upload及retained-copy counters，且有界ZR_RENDERDOC_CAPTURE_FRAME_COUNT=2可在同一viewport连续触发cold/stable capture；逻辑层compact brush/style handle与instance range、current-source F4 PNG/RDC及GPU counter对拍仍待完成后回传`。
