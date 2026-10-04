---
related_code:
  - zircon_runtime/src/graphics/scene
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
tests:
  - current graphics scene Rust source census 859 of 859 files reviewed, 119899 lines
  - scene_renderer, resources, gpu_scene and root product-test owners reconcile exactly
  - performance task ledger PERF-MVP-1 through PERF-MVP-406 continuous
  - current-source Cargo, F2, scale counters, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene全目录静态审查收口（2026-07-18）

## 当前源守恒

`zircon_runtime/src/graphics/scene/**`当前859/859个Rust文件、119,899行已逐文件静态阅读。owner守恒为：`scene_renderer/**`771文件/104,632行、`resources/**`63文件/8,034行、`gpu_scene/**`15文件/3,487行，以及root/product-test树10文件/3,746行，总和859文件/119,899行。

详细发现分别保存在`2026-07-18-graphics-scene-renderer-complete-static-review.md`、`2026-07-18-graphics-scene-resources-static-review.md`、`2026-07-18-graphics-gpu-scene-static-review.md`和`2026-07-18-graphics-scene-root-product-tests-static-review.md`。性能任务账本连续为`PERF-MVP-1..406`；本页只做模块级文件集合与责任计划汇总，不重复展开子目录热点。

并发会话在收口期间新增`scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/remainder.rs`。该文件和父级调用链已补读，当前全cache-hit路径不再持有`Vec<Option<PendingMeshDraw>>`；`All`枚举索引和`Residual`显式索引维持下游indirect arrays的原始对应关系。它属于既有mesh command-cache优化，不新增重复任务。

## 验收状态

静态审查、局部源码合同、scoped rustfmt及相关diff检查已按子证据执行。当前验证器在启动Cargo前解析协调器JSON失败，F2产品路径、plugin reload、规模counter、GPU timestamp和DX12 RenderDoc也没有形成此current-source模块的完整证据。因此`scene/**`只在`pending.md`标记静态已读，不进入`review.md`；后续动态验收优先core graph/deferred/mesh/UI和resource/GPUScene稳定帧预算，再验optional advanced/environment路径。
