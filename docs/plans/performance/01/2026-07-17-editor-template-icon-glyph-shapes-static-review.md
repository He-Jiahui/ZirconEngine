---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_segments.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
tests:
  - icon segment fractional/scaling geometry tests
  - current-source Windows Cargo pending
  - shipped MVP icon resolve/fallback counter pending
  - 1/1000/10000 icon command/draw/raster trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template icon glyph shapes逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_icon_button_glyph_shapes/**` **34/34** 个Rust文件与共享`template_icon_button_glyph_segments.rs` **1/1**，合计 **35/35** 文件、**1,181** 行已逐文件阅读。Segment leaf已在前一独立leaf切片读过，本切片完成全部action/asset/chrome/file/tool/visibility shape与typed dispatch。当前源Cargo、产品fallback counter和规模trace未完成，因此仍留在`pending.md`。

## 已有正确边界

Glyph kind已经是typed enum，dispatch为两层match而非字符串链；每个shape只遍历静态segment slice并append command，没有I/O、锁、队列或无界循环。Fractional grid与非方形缩放有测试。真实SVG asset在上游优先，manual glyph是asset miss fallback。

## 热点与计划

PERF-MVP-179：26类manual glyph各用2–8个quad segment模拟，EyeOff还在Eye的5段之上追加一段。每段都会构造FrameRect、clone clip并形成独立HostPaintCommand，之后参与global stable sort、Softbuffer primitive draw与GPU draw-list转换。PERF-MVP-178只可避免稳定generation重复构建，不能消除这些per-frame draw commands。

先在真实MVP toolbar、activity rail、tree/table action上记录resource resolve与manual fallback。若fallback为0，补asset completeness/source guard即可，避免优化冷错误路径。若fallback>0且确为产品需要，Render13以`glyph kind + raster size + tint + theme/resource generation`生成有界cached mask/atlas，EditorUI08 compiled segment只携带单个resource handle/UV；不得为每consumer建glyph cache，也不得保留多quad draw expansion。

## 动态验收

记录1/1,000/10,000 icon nodes的asset hits/misses、fallback glyphs、segments、Host/RHI commands、sort items、software primitives、raster/upload count与bytes。首选gate是shipped MVP icon fallback=0；否则每glyph command=1、raster/upload≤1/key/generation，stable frame build/raster/upload=0，cache entries/bytes有硬上限。保持26类glyph identity、tint、fractional/rectangular scale、clip/z/opacity与GPU/Softbuffer pixels一致。
