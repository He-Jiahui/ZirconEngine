---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: render-graph-compile-analysis-scaling
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/render_graph/builder/compile.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime.rs
---

# RenderGraph compile analysis scaling

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/render_graph`当前源14/14 Rust文件及直接F2调用方
- 修复责任计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 交接原因：依赖编译、culling、transient allocation、compiled cache与realtime graph owner均属于Render01；相关源码正由该计划活跃修改。

## 失败现象与复现证据

当前compile以HashSet clone传播manual reachability，多writer再做pair比较，每个transient read又全扫pass/access验证producer；culling按pass临时collect writes，allocation bucket/slot多处线性find。主pipeline虽有compiled cache，但immutable `graph.stats()`仍在每帧多遍扫描，realtime IBL有工作batch仍重新build/compile。性能切片已只在未冲突builder路径把每次resource access验证从O(resources)降为typed handle O(1)。

## 最低共享层根因

graph authoring/compile没有统一adjacency、resource access index和一次性compiled metadata，正确性检查各自重建局部集合并重复遍历；frame diagnostics与realtime IBL也未完整消费compiled generation事实。

## 架构修复验收

- 依赖、producer、WAW与root culling共享adjacency/resource index；不在nested pass loops clone HashSet或对每个read全图扫描。
- 16/64/256/1024 pass×resource的chain/fan-out/multi-writer/plugin-heavy基准记录compile p50/p95、edge/access visits、alloc bytes与复杂度斜率；目标接近O(P+E+A)，确需closure时使用有界bitset/索引并记录原因。
- `CompiledRenderGraphStats`在compile时生成，steady frame `stats()`为O(1)，pass/resource visited=0。
- realtime IBL按request/operation topology signature复用compiled graph；stable topology compile=0，变化generation只编译一次。
- transient bucket/slot计划避免bucket×resource与allocation×reservation线性find；保持descriptor bucket、readback lifetime、persistent/sparse bypass语义。
- current-source Cargo、RenderDoc graph dump/non-culled marker、pass/resource统计和产品像素对拍通过。

## 禁止临时方案

- 不得通过减少插件pass、关闭validation/culling/stats或只放宽测试图规模伪造改善。
- 不得把每帧compile/stat scan无界投递到worker；先以generation/cache消除稳定帧工作，再评估阈值并行。
- 不得覆盖当前Render01 transient-pool hard cutover或留下第二套graph metadata/cache事实源。

## 修复结果与回传

Open state: `builder typed-handle O(1) validation已落地；待Render01回传indexed compile/culling、precomputed stats、realtime IBL topology cache及规模/RenderDoc证据`。
