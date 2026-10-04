---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
tests:
  - complete scene_renderer current Rust source census seven hundred seventy-one of seven hundred seventy-one files reviewed, 104632 lines
  - all twenty-two root and top-level module groups accounted for
  - performance task ledger PERF-MVP-1 through PERF-MVP-403 continuous
  - scoped source guards and diff check passed for direct fixes
  - current-source Cargo, F2 pixels, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer全目录静态审查收口（2026-07-18）

## 当前源守恒

`zircon_runtime/src/graphics/scene/scene_renderer/**`当前771/771个Rust文件、104,632行已逐文件静态阅读。当前源按owner目录守恒为：root 2、advanced_lighting 40、anti_alias 3、core 92、deferred 17、environment 46、graph_execution 47、history 7、hzb 4、lighting 4、mesh 107、overlay 50、particle 19、post_process 175、prepass 3、primitives 51、scene_clear 4、shadow 12、sprite 6、temporal 9、transparent 2、ui 71，总和771，无未列目录。

各目录的逐文件发现、局部RED→GREEN与责任计划已拆分记录在本目录同日`graphics-*.md`证据中，权威性能任务连续为PERF-MVP-1..403。为避免汇总重复膨胀，本页不重述各文件热点；模块级主线为compiled generation artifact、dense graph/executor/resource handles、persistent GPU arena/binding/view owner、按需feature资源、有界异步prepare/readback以及diagnostics gate。

当前工作树中个别早期证据的行数会因后续局部修复或并行会话删除/新增文件而与本次census相差少量；本页的771/104,632是当前源汇总，早期证据仍保留其审查时切片计数和具体结论。新增的`mesh/build_mesh_draws/build/pending_command_cache_extract/remainder.rs`已连同父级调用链逐行复核：它在全cache-hit路径避免`Vec<Option<_>>`残余容器，并以原始枚举索引或显式残余索引保持indirect arrays对应关系，不新增独立性能任务。已确认所有当前`.rs`都归入已读owner组，没有用旧基线数冒充当前完成度。

## 验收状态

本轮直接修复覆盖稳定CPU重复工作、空路径分配、冗余GPU clear/upload、锁粒度、String/Vec临时量及明显atlas slot错误，并已执行对应源码合同、scoped rustfmt与`git diff --check`。但Cargo协调预约本次再次以consumed/null job结束，F2产品像素、规模counter、GPU timestamp和DX12 RenderDoc仍未形成current-source证据，因此整个`scene_renderer`只在`pending.md`标记“静态已读”，不进入`review.md`。后续动态验收按MVP core/core graph/deferred/mesh/UI优先，再到advanced/environment optional feature。
