---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/reports.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/resource_lookup.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - GPU execution context root reports and resource lookup three of three Rust files reviewed, 959 lines
  - compiled resource-access guard regression traced
  - no separate source edit; detailed dispatch ownership and String resource tables route to PERF-MVP-343 and PERF-MVP-366
  - current-source Cargo scale trace and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics GPU execution context root逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读GPU context root、`reports.rs`和`resource_lookup.rs`当前3/3个Rust文件、959行。context的大部分renderer/device/queue/frame/resource成员均为borrowed reference，builder式feature接线只复制Option/handle，没有frame级深clone。剩余成本分别属于PERF-MVP-343的详细compute record所有权和PERF-MVP-366的String-keyed执行资源/view生命周期，本切片不新增编号。

## 热点与已有基础

每个pass创建`compute_dispatches: Vec`，compute executor再为pass/executor/pipeline/storage-write names构造owned record，pass结束后整Vec move进execution record。`take_compute_dispatches`本身无clone，但常见单dispatch pass仍需要首次Vec分配，且详细String DTO在diagnostics关闭时也存在。PERF-MVP-343的dense always-on summary/detail-on-demand方案应让context直接写summary sink；详细dispatch rows使用generation workspace或small inline storage，而不是每pass新Vec。

resource lookup先用compiled resolver验证pass declaration/access，再进入`RenderGraphExecutionResources`按String找physical resource；错误只在失败时format。compiled access索引已有测试保护，当前不应退回Vec扫描。PERF-MVP-366改为handle-indexed execution storage后，require/optional texture/buffer/desc和mip view都应消费dense handle，name只服务错误文本。

## 验收

按passes 16/64/256、compute passes 0/1/64、dispatches 0/1/8 per pass、diagnostics off/on记录context Vec alloc、dispatch String/Vec bytes、resource name lookups与CPU p95。off时detail Vec/String build=0，always-on counters一次增量写；on时workspace warm growth=0、每dispatch detail≤1。resource lookup保持compiled declaration/access语义且最终成功路径String lookup=0。Cargo、F2与RenderDoc完成前留在`pending.md`。
