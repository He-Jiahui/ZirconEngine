---
related_code:
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests
  - zircon_runtime/src/graphics/scene/render_product_zshader_import_tests.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - complete graphics scene root and product-test tree ten of ten current Rust files reviewed, 3746 lines
  - thirty-five product tests inventoried
  - thirty-five independent offscreen backend constructions inventoried
  - current-source Cargo and real test wall-clock measurement pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene root产品测试逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`graphics/scene`在`scene_renderer/**`、`resources/**`、`gpu_scene/**`之外的当前10/10个Rust文件、3,746行：root module 1/57，material-property产品测试2/942，resource-streamer产品测试6/2,552，zshader import产品测试1/195。共35个`render_product_*`测试，覆盖material uniform/pipeline key、texture slot与dimension fallback、shader redirect/readiness/ABI、cache invalidation和management diagnostics。该切片没有新的产品运行时代码；本记录只声明测试源静态覆盖，不声明测试执行通过。

## 产品性能合同观察

现有测试已锁定两项关键稳定合同：相同material revision二次ensure复用property/standard uniform `Arc`，texture dependency revision或upload readiness变化会使material cache失效。GPUScene与resource streamer本轮直接修复应继续依赖这些合同；诊断/management大测试还揭示同一material可连续请求record set、overview、status/issue views、query/selection和prepared state，当前每个accessor可能重新构建/排序/深clone管理记录，产品侧须由PERF-MVP-404的显式diagnostic generation artifact统一承载，不能把测试中的连续调用当免费操作。

## PERF-MVP-406：RenderProduct测试上下文分层与复用

35个测试各自调用一次`RenderBackend::new_offscreen()`，即35次独立instance/adapter/device/queue初始化，并重复创建相同texture bind-group layout、fallback ResourceStreamer owner和大段asset fixture。大量测试实际上只验证material descriptor投影、readiness rows、fallback分类、sorting/query DTO或shader dependency解析，GPU只用于`ensure_material`内部uniform/fallback构造；每个纯合同测试都申请adapter会放大测试墙钟、driver初始化、并发adapter请求和协调Cargo lane占用。

Runtime15应建立test-only分层：纯material/shader/diagnostic projection通过无GPU prepared-data入口或受控fake uniform/resource sink运行；真正验证WGPU buffer、bind、upload和resident handle的少量测试使用按`RenderBackendConfig + required features/limits`键控的进程级`RenderProductTestContextPool`。共享context须保留每测试独立asset manager/streamer、error scope和资源identity，要求独占device-error/device-loss语义的测试进入serial key，不用全局共享掩盖隔离错误。公共shader/material/texture/layout fixtures下沉到小型测试support owner，现有行为断言不减少。

Render17/测试输出记录adapter/device初始化次数、context key/hit、pure/GPU test数、每executable setup/test wall time和失败隔离。验收按单测/过滤批次/完整scene产品测试，线程1/默认、WGPU backend DX12/Vulkan/GL可用组合采样：当前35 tests/35 backend init；目标兼容feature key内backend init<=1、纯测试backend init=0，stable测试结果与原断言一致，单个测试失败不污染后续error scope，完整批次wall time/p95显著下降。协调验证器修复并完成Cargo前保留在`pending.md`，不进入`review.md`。

本轮`validate-matrix.ps1 -Package zircon_runtime -SkipTest`仍在Cargo启动前因第187行`ConvertFrom-Json`失败，故没有可用的现状测试时长或执行结果；不得用静态计数冒充提速数据。
