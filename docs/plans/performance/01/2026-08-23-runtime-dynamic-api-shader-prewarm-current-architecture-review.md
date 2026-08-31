---
related_code:
  - zircon_runtime/src/dynamic_api/shader_prewarm
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm/budget.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/worker.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/prewarm_pipeline_validation.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/01/2026-08-15-renderer-material-shader-streaming-current-architecture-review.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Private/PipelineStateCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PSOPrecache.cpp
tests:
  - current shader_prewarm 4 of 4 Rust files and 7 tests reviewed
  - current-source Cargo and CLI scale traces blocked
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Runtime dynamic API shader prewarm当前架构复审（2026-08-23）

## 范围、可达性与当前性

已逐行复读`zircon_runtime/src/dynamic_api/shader_prewarm/**`当前 **4/4** 个Rust文件、**851行、31,720 B、7 tests**，manifest SHA256为`0f8b95577f75a18bfe5b99e156357948b8ea44d14ad02d03e2423f53a56001f0`；目录当前干净，本轮未接管生产源码。并沿调用链读到framework budget、graphics prewarm worker、mesh WGPU pipeline validation和`zircon_shader_prewarm` CLI。

产品检索确认该API当前只由独立`zircon_shader_prewarm`命令调用，常规runtime/editor session启动不调用它。因此这是构建、离线预热和首启资产准备吞吐问题，不是普通帧MVP热路径；优先级继续归PERF-MVP-357/448，不覆盖viewport、extract、session锁等P0。

## 当前算法判定

1. `ShaderVariantPrewarmExecutionBudget::validate`明确拒绝`max_in_flight_variants != 1`，worker对manifest variants单线程串行执行source lookup、Naga validation、可选WGPU validation、zstd和两次atomic file write。source-only Naga结果与module validation结果已按source id在batch内缓存，这是正确的去重，不能继续引用旧报告中“每variant复制WGSL/每variant Naga”的已过时描述。
2. module validation为每个新source创建一次`ShaderModule`，随后立即`pollster::block_on(error_scope.pop())`并丢弃module。pipeline validation又为每个variant从同一WGSL创建module、创建PSO并再次同步等待error scope。双验证模式因此至少是`S`次module validation加`V`次module+pipeline creation，而不是共享一次可复用module artifact。
3. dynamic wrapper在建立offscreen backend前做budget/source-residency preflight；graphics worker进入后再次`budget.validate()`并再次计算`source_table_resident_bytes()`。这是正常成功路径的重复source-table遍历，但它是跨API重复，正确修复应让preflight产物随execution ticket传递，而不是删除建立WGPU device前的拒绝门。
4. report只有requested/written/failure、validation与resident/peak budget计数，没有cache hit/miss、各阶段wall/CPU时间、queue age、worker utilization、WGSL bytes、zstd/I/O bytes或device/PSO compile latency。没有这些数据时，`max_in_flight_variants=1`不能被证明足够，也不能安全地直接放宽。
5. disk worker对既有cache不先lookup；每次CLI执行仍validation并重写WGSL/meta。对于显式`--validate-wgpu-*`这可能是设备正确性要求，对于纯disk prewarm则可能是冗余。必须先冻结“验证当前device”与“只补齐content cache”两种模式，不能用一个隐式cache-hit shortcut改变CLI语义。

## Unreal源码依据

`PipelineStateCache.cpp:154-185`把async pipeline compile和async cache consolidation作为显式模式；`:376-639`提供可配置数量/硬件线程比例、min/max和below-normal priority的专用PSO precompile thread pool，并以priority threshold控制调度；`:641-648`在precompile完成后显式关闭pool。`PSOPrecache.cpp:260-329`提交一组PSO请求并返回async completion events，`:336-475`按需求提升priority并以hash缓存active/completed request，避免同PSO重复提交。

Zircon应采用“内容/PSO key single-flight、异步completion、优先级、显式pool生命周期和有界resident/in-flight bytes”，不照搬UE线程API。WGPU device操作是否允许多lane必须由backend capability和实测确定；CPU Naga/hash/zstd/I/O可以先进入Runtime11共享job service，RHI-affine module/PSO creation由单独有界lane执行。

## 复用计划与实施顺序

- PERF-MVP-357继续负责source-table artifact、canonical key、single-flight和bounded worker graph；PERF-MVP-448负责CLI inventory/DAG重复扫描。本轮不新增重复编号。
- M1先补stage counters：manifest/source/variant数与bytes、preflight/Naga/module/pipeline/zstd/meta/write时间、cache hit/miss/corrupt、device init、in-flight/queue age、peak RSS和CPU利用率。纯disk、module、pipeline、both四种模式分别量测。
- M2拆成CPU artifact lane、RHI-affine validation lane、I/O publish lane；每个lane有count/bytes/deadline/cancel预算，结果按variant ordinal确定性汇总。相同source module只创建一次并由同device generation共享；相同PSO key只有一个active ticket。
- M3让warm cache以canonical artifact identity验证source/template/Naga/WGPU/backend/device兼容信息。纯disk模式可在可信hit时跳过validate/rewrite；显式device validation模式保留PSO验证但不重复Naga/source module工作。
- 删除门包括serial-only budget错误、逐variant同步`pollster::block_on`、CLI owner直接zstd/file I/O以及双验证模式的重复module创建；在替代lane和行为测试存在前不得先删。

## 验收状态

按sources 1/100/10k、variants 1/1k/100k、WGSL 4 KiB/1 MiB、cold/warm/corrupt、workers 1/2/4/8和四种validation模式记录上述阶段指标；要求相同source Naga/module work <= 1/device generation、相同PSO active ticket <= 1、warm纯disk validation/write bytes=0、in-flight count/bytes有界、report顺序与错误归属确定。current Cargo、CLI scale、WPR/CPU/energy和真实DX12 validation未执行，本模块继续留在`pending.md`，不进入`review.md`。
